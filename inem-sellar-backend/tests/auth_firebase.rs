//! Tests de integracion del handshake `POST /api/v1/auth/firebase`.
//!
//! # Como funcionan
//! Cada test:
//! 1. Levanta un servidor wiremock que sirve un JWKS con una clave RSA
//!    generada al vuelo (una sola vez via `LazyLock` para no pagar el
//!    coste de generacion en cada test).
//! 2. Pre-programa una `MockDatabase` de SeaORM con la secuencia de
//!    respuestas SELECT/INSERT/UPDATE en el ORDEN FIFO en que el handler
//!    las pedira (cada operacion SeaORM consume una entrada del mock).
//! 3. Construye un Salvo `Service` con `AuthService`, `FirebaseVerifier`,
//!    `SeaAuthRepo` y `SeaProveedorAutenticacionRepo` inyectados.
//! 4. Forja un Firebase ID Token RS256 con la clave privada del paso 1
//!    y los claims relevantes (`iss`, `aud`, `sub`, `firebase.sign_in_provider`...).
//! 5. Envia POST `/api/v1/auth/firebase` via `TestClient` (sin TCP real).
//! 6. Asserta el `status_code` y el shape del JSON de respuesta.
//!
//! # Por que MockDatabase y no testcontainers
//! `MockDatabase` permite probar el flujo del handler sin Postgres, lo que
//! mantiene los tests rapidos (~1s) y sin dependencia de Docker. El precio
//! es la fragilidad: cualquier reordenacion de queries en el codigo rompe
//! los mocks. Aceptable para una superficie pequena como `/auth/firebase`.

use std::sync::{Arc, LazyLock};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use rsa::pkcs8::EncodePrivateKey;
use rsa::traits::PublicKeyParts;
use rsa::{RsaPrivateKey, pkcs8::LineEnding};
use salvo::affix_state;
use salvo::prelude::*;
use salvo::test::{ResponseExt, TestClient};
use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use inem_sellar_backend::handlers::auth;
use inem_sellar_backend::models::{proveedor_autenticacion, token_refresco, usuario};
use inem_sellar_backend::repositories::{SeaAuthRepo, SeaProveedorAutenticacionRepo};
use inem_sellar_backend::services::{AuthService, FirebaseVerifier};

// ─── Constantes globales de los tests ────────────────────────────────────

const KID: &str = "test-kid-1";
const PROJECT_ID: &str = "test-project";
const JWT_SECRET: &str = "test-jwt-secret-no-usado-en-prod";

/// RSA keypair generado UNA sola vez (2048 bits, ~1s).
///
/// Compartido entre todos los tests via `LazyLock`. La clave privada firma
/// los tokens forjados; la publica se publica en el JWKS mockeado para que
/// el `FirebaseVerifier` la valide. Generar una clave nueva por test
/// multiplicaria el tiempo de la suite por 8.
static PRIVATE_KEY: LazyLock<RsaPrivateKey> = LazyLock::new(|| {
    let mut rng = rand::thread_rng();
    RsaPrivateKey::new(&mut rng, 2048).expect("RSA key generation fallo")
});

/// Segunda RSA keypair usada SOLO en el test de firma invalida: forjamos
/// el token con esta y el JWKS sirve la primera; el verifier debe rechazar.
static PRIVATE_KEY_INTRUSO: LazyLock<RsaPrivateKey> = LazyLock::new(|| {
    let mut rng = rand::thread_rng();
    RsaPrivateKey::new(&mut rng, 2048).expect("RSA key intruso fallo")
});

// ─── Helpers de cripto y JWKS ────────────────────────────────────────────

fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Construye el JSON Web Key Set (JWKS) que sirve el mock de Google.
fn build_jwks_json() -> serde_json::Value {
    let public = PRIVATE_KEY.to_public_key();
    let n = b64url(&public.n().to_bytes_be());
    let e = b64url(&public.e().to_bytes_be());
    json!({
        "keys": [{
            "kty": "RSA",
            "use": "sig",
            "alg": "RS256",
            "kid": KID,
            "n": n,
            "e": e,
        }]
    })
}

/// Levanta un wiremock con el JWKS en `/jwks`. Devuelve el server
/// (su `Drop` lo limpia al final del test) y la URL completa.
async fn start_jwks_server() -> (MockServer, String) {
    let server = MockServer::start().await;
    let jwks = build_jwks_json();
    Mock::given(method("GET"))
        .and(path("/jwks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(jwks))
        .mount(&server)
        .await;
    let url = format!("{}/jwks", server.uri());
    (server, url)
}

/// Forja un Firebase ID Token RS256 firmado con la clave indicada.
fn forge_token_with(claims: &serde_json::Value, kid: &str, key: &RsaPrivateKey) -> String {
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    let pem = key.to_pkcs8_pem(LineEnding::LF).expect("encode pkcs8");
    let enc_key = EncodingKey::from_rsa_pem(pem.as_bytes()).expect("from_rsa_pem");
    encode(&header, claims, &enc_key).expect("jwt encode")
}

/// Forja un token con la clave correcta de la suite.
fn forge_token(claims: serde_json::Value, kid: &str) -> String {
    forge_token_with(&claims, kid, &PRIVATE_KEY)
}

/// Construye el JSON de claims base con los campos obligatorios y el
/// provider especificado. `iat`/`exp` se calculan al momento de la llamada.
fn base_claims(provider: &str, sub: &str) -> serde_json::Value {
    let now = Utc::now().timestamp();
    json!({
        "iss": format!("https://securetoken.google.com/{PROJECT_ID}"),
        "aud": PROJECT_ID,
        "sub": sub,
        "iat": now,
        "exp": now + 3600,
        "auth_time": now,
        "firebase": {
            "sign_in_provider": provider,
            "identities": {}
        }
    })
}

/// Variante con email y email_verified — para password / google.com.
fn claims_con_email(
    provider: &str,
    sub: &str,
    email: &str,
    email_verified: bool,
) -> serde_json::Value {
    let mut c = base_claims(provider, sub);
    c["email"] = json!(email);
    c["email_verified"] = json!(email_verified);
    c
}

/// Variante con name + picture — para google.com.
fn claims_google(sub: &str, email: &str, name: &str, picture: &str) -> serde_json::Value {
    let mut c = claims_con_email("google.com", sub, email, true);
    c["name"] = json!(name);
    c["picture"] = json!(picture);
    c
}

// ─── Helpers de modelos para MockDatabase ────────────────────────────────

fn mock_usuario(id: Uuid, email: Option<&str>, anonimo: bool) -> usuario::Model {
    let now = Some(Utc::now().fixed_offset());
    usuario::Model {
        id,
        email: email.map(String::from),
        hash_contrasena: None,
        nombre_visible: if anonimo {
            None
        } else {
            Some("Test User".into())
        },
        url_avatar: None,
        url_linkedin: None,
        url_curriculum: None,
        activo: Some(true),
        id_provincia: None,
        ultimo_login: now,
        creado_en: now,
        actualizado_en: now,
    }
}

fn mock_usuario_legacy(id: Uuid, email: &str) -> usuario::Model {
    let mut u = mock_usuario(id, Some(email), false);
    u.hash_contrasena = Some("$argon2id$legacy-dead-data".into());
    u
}

fn mock_proveedor(
    id: Uuid,
    id_usuario: Uuid,
    proveedor: &str,
    sub: &str,
) -> proveedor_autenticacion::Model {
    let now = Some(Utc::now().fixed_offset());
    proveedor_autenticacion::Model {
        id,
        id_usuario,
        proveedor: Some(proveedor.into()),
        identificador_proveedor: Some(sub.into()),
        email_proveedor: None,
        datos_proveedor: None,
        creado_en: now,
        actualizado_en: now,
    }
}

fn mock_refresh(id: Uuid, id_usuario: Uuid) -> token_refresco::Model {
    let now = Some(Utc::now().fixed_offset());
    token_refresco::Model {
        id,
        id_usuario,
        hash_token: Some("test-hash".into()),
        informacion_dispositivo: None,
        expira_en: now,
        revocado: Some(false),
        creado_en: now,
    }
}

// ─── Builder del Service Salvo con todas las inyecciones ─────────────────

fn build_app(db: Arc<DatabaseConnection>, jwks_url: String) -> Service {
    let auth_service = AuthService::new(JWT_SECRET.to_string(), 15);
    let firebase = FirebaseVerifier::new_with_url(PROJECT_ID.to_string(), jwks_url);
    let auth_repo = SeaAuthRepo::new(Arc::clone(&db));
    let proveedor_repo = SeaProveedorAutenticacionRepo::new(db);

    let router = Router::new()
        .hoop(affix_state::inject(auth_service))
        .hoop(affix_state::inject(firebase))
        .hoop(affix_state::inject(auth_repo))
        .hoop(affix_state::inject(proveedor_repo))
        .push(Router::with_path("api/v1/auth/firebase").post(auth::login_firebase));
    Service::new(router)
}

/// Helper: vec vacio tipado de un modelo concreto. Util para mockar
/// queries que devuelven `None` (`.one()` con cero filas).
fn empty<M: sea_orm::ModelTrait>() -> Vec<M> {
    Vec::new()
}

// ═════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════

/// Provider `anonymous` nuevo: crea usuario + identidad, devuelve
/// `usuario.proveedor="anonymous"`, `anonimo=true`, `email=null`.
#[tokio::test]
async fn firebase_login_anonymous_crea_usuario_y_proveedor() {
    let (_server, jwks_url) = start_jwks_server().await;

    let user_id = Uuid::new_v4();
    let prov_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let sub = "firebase-anonymous-uid";

    let user = mock_usuario(user_id, None, true);
    let prov = mock_proveedor(prov_id, user_id, "anonymous", sub);
    let tok = mock_refresh(token_id, user_id);

    let db: Arc<DatabaseConnection> = Arc::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            // 1. find prov by (provider='anonymous', sub) -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 2. find prov by uid (lookup defensivo) -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 3. crear_usuario_anonimo (INSERT...RETURNING)
            .append_query_results(vec![vec![user.clone()]])
            // 4. crear_proveedor_identidad (INSERT...RETURNING)
            .append_query_results(vec![vec![prov.clone()]])
            // 5. actualizar_ultimo_login: find_by_id
            .append_query_results(vec![vec![user.clone()]])
            // 6. actualizar_ultimo_login: ActiveModel::update (UPDATE...RETURNING)
            .append_query_results(vec![vec![user.clone()]])
            // 7. (anonimo => salta enriquecer_perfil_si_vacio)
            // 8. guardar_refresh_token (INSERT...RETURNING)
            .append_query_results(vec![vec![tok.clone()]])
            // 9. buscar_usuario_por_id (reload final)
            .append_query_results(vec![vec![user.clone()]])
            .into_connection(),
    );

    let service = build_app(db, jwks_url);
    let token = forge_token(base_claims("anonymous", sub), KID);

    let mut res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    let body: serde_json::Value = res.take_json().await.unwrap();
    assert_eq!(body["usuario"]["proveedor"], "anonymous");
    assert_eq!(body["usuario"]["anonimo"], true);
    assert!(body["usuario"]["email"].is_null());
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
}

/// Provider `password` sin usuario previo: crea usuario OAuth + identidad.
#[tokio::test]
async fn firebase_login_password_crea_usuario_si_no_existe() {
    let (_server, jwks_url) = start_jwks_server().await;

    let user_id = Uuid::new_v4();
    let prov_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let sub = "firebase-password-uid";
    let email = "alice@example.com";

    let user = mock_usuario(user_id, Some(email), false);
    let prov = mock_proveedor(prov_id, user_id, "password", sub);
    let tok = mock_refresh(token_id, user_id);

    let db: Arc<DatabaseConnection> = Arc::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            // 1. find prov (password, sub) -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 2. find prov by uid -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 3. (no anonymous) -> buscar_por_email -> None
            .append_query_results::<usuario::Model, _, _>(vec![empty()])
            // 4. crear_usuario_oauth
            .append_query_results(vec![vec![user.clone()]])
            // 5. crear_proveedor_identidad
            .append_query_results(vec![vec![prov.clone()]])
            // 6. actualizar_ultimo_login: find_by_id
            .append_query_results(vec![vec![user.clone()]])
            // 7. actualizar_ultimo_login: update
            .append_query_results(vec![vec![user.clone()]])
            // 8. enriquecer_perfil_si_vacio: find_by_id
            //    (no actualiza porque mock_usuario ya tiene nombre_visible)
            .append_query_results(vec![vec![user.clone()]])
            // 9. guardar_refresh_token
            .append_query_results(vec![vec![tok.clone()]])
            // 10. buscar_usuario_por_id (reload final)
            .append_query_results(vec![vec![user.clone()]])
            .into_connection(),
    );

    let service = build_app(db, jwks_url);
    let token = forge_token(claims_con_email("password", sub, email, true), KID);

    let mut res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    let body: serde_json::Value = res.take_json().await.unwrap();
    assert_eq!(body["usuario"]["proveedor"], "password");
    assert_eq!(body["usuario"]["anonimo"], false);
    assert_eq!(body["usuario"]["email"], email);
    assert_eq!(body["usuario"]["email_verificado"], true);
}

/// Provider `google.com` con name + picture: crea usuario y enriquece perfil.
#[tokio::test]
async fn firebase_login_google_crea_usuario_y_enriquece_perfil() {
    let (_server, jwks_url) = start_jwks_server().await;

    let user_id = Uuid::new_v4();
    let prov_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let sub = "firebase-google-uid";
    let email = "bob@gmail.com";

    // Empezamos con nombre/avatar vacios para que enriquecer los rellene.
    let mut user_inicial = mock_usuario(user_id, Some(email), false);
    user_inicial.nombre_visible = None;
    user_inicial.url_avatar = None;
    let mut user_enriquecido = user_inicial.clone();
    user_enriquecido.nombre_visible = Some("Bob Smith".into());
    user_enriquecido.url_avatar = Some("https://lh3.googleusercontent.com/avatar".into());
    let prov = mock_proveedor(prov_id, user_id, "google.com", sub);
    let tok = mock_refresh(token_id, user_id);

    let db: Arc<DatabaseConnection> = Arc::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            .append_query_results::<usuario::Model, _, _>(vec![empty()])
            .append_query_results(vec![vec![user_inicial.clone()]])
            .append_query_results(vec![vec![prov.clone()]])
            // actualizar_ultimo_login
            .append_query_results(vec![vec![user_inicial.clone()]])
            .append_query_results(vec![vec![user_inicial.clone()]])
            // enriquecer_perfil_si_vacio: find_by_id (campos vacios)
            .append_query_results(vec![vec![user_inicial.clone()]])
            // enriquecer: update aplicado (nombre_visible y url_avatar None -> Set)
            .append_query_results(vec![vec![user_enriquecido.clone()]])
            // guardar_refresh_token
            .append_query_results(vec![vec![tok.clone()]])
            // buscar_usuario_por_id (reload) -> devuelve el enriquecido
            .append_query_results(vec![vec![user_enriquecido.clone()]])
            .into_connection(),
    );

    let service = build_app(db, jwks_url);
    let token = forge_token(
        claims_google(
            sub,
            email,
            "Bob Smith",
            "https://lh3.googleusercontent.com/avatar",
        ),
        KID,
    );

    let mut res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    let body: serde_json::Value = res.take_json().await.unwrap();
    assert_eq!(body["usuario"]["proveedor"], "google.com");
    assert_eq!(body["usuario"]["nombre_visible"], "Bob Smith");
    assert_eq!(
        body["usuario"]["url_avatar"],
        "https://lh3.googleusercontent.com/avatar"
    );
}

/// Auto-link: usuario legacy con `hash_contrasena` no nulo entra via Firebase
/// password con `email_verified=true`, mismo email -> reutiliza el id_usuario.
#[tokio::test]
async fn firebase_login_password_auto_link_a_usuario_legacy() {
    let (_server, jwks_url) = start_jwks_server().await;

    let user_id = Uuid::new_v4();
    let prov_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let sub = "firebase-password-uid-legacy";
    let email = "carol@example.com";

    let user_legacy = mock_usuario_legacy(user_id, email);
    let prov = mock_proveedor(prov_id, user_id, "password", sub);
    let tok = mock_refresh(token_id, user_id);

    let db: Arc<DatabaseConnection> = Arc::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            // 1. find prov (password, sub) -> None (es la primera vez)
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 2. find prov by uid -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 3. buscar_por_email -> Some(usuario legacy)
            .append_query_results(vec![vec![user_legacy.clone()]])
            // 4. crear_proveedor_identidad para enlazar
            .append_query_results(vec![vec![prov.clone()]])
            // 5. actualizar_ultimo_login: find_by_id
            .append_query_results(vec![vec![user_legacy.clone()]])
            // 6. actualizar_ultimo_login: update
            .append_query_results(vec![vec![user_legacy.clone()]])
            // 7. enriquecer_perfil_si_vacio: find_by_id (ya tiene nombre, no actualiza)
            .append_query_results(vec![vec![user_legacy.clone()]])
            // 8. guardar_refresh_token
            .append_query_results(vec![vec![tok.clone()]])
            // 9. reload final
            .append_query_results(vec![vec![user_legacy.clone()]])
            .into_connection(),
    );

    let service = build_app(db, jwks_url);
    let token = forge_token(claims_con_email("password", sub, email, true), KID);

    let mut res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    let body: serde_json::Value = res.take_json().await.unwrap();
    assert_eq!(body["usuario"]["id"], user_id.to_string());
    assert_eq!(body["usuario"]["email"], email);
    assert_eq!(body["usuario"]["proveedor"], "password");
}

/// `email_verified=false` con email coincidente -> 409 Conflict.
#[tokio::test]
async fn firebase_login_email_no_verificado_devuelve_409() {
    let (_server, jwks_url) = start_jwks_server().await;

    let user_id = Uuid::new_v4();
    let sub = "firebase-password-uid-noverif";
    let email = "dave@example.com";
    let user_existente = mock_usuario(user_id, Some(email), false);

    let db: Arc<DatabaseConnection> = Arc::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            // 1. find prov -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 2. find prov by uid -> None
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 3. buscar_por_email -> Some, pero email_verified=false -> 409
            .append_query_results(vec![vec![user_existente]])
            .into_connection(),
    );

    let service = build_app(db, jwks_url);
    let token = forge_token(claims_con_email("password", sub, email, false), KID);

    let res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::CONFLICT));
}

/// `sign_in_provider` desconocido (apple.com) -> 400 con mensaje informativo.
#[tokio::test]
async fn firebase_login_provider_desconocido_devuelve_400() {
    let (_server, jwks_url) = start_jwks_server().await;
    // No deberia tocar la BD, pero MockDatabase requiere algo aunque sea vacio.
    let db: Arc<DatabaseConnection> =
        Arc::new(MockDatabase::new(DatabaseBackend::Postgres).into_connection());

    let service = build_app(db, jwks_url);
    let token = forge_token(base_claims("apple.com", "firebase-apple-uid"), KID);

    let mut res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::BAD_REQUEST));
    let body = res.take_string().await.unwrap();
    assert!(
        body.contains("apple.com"),
        "esperaba mensaje con `apple.com`, recibido: {body}"
    );
}

/// Token firmado con clave RSA distinta a la del JWKS -> 401.
#[tokio::test]
async fn firebase_login_token_invalido_devuelve_401() {
    let (_server, jwks_url) = start_jwks_server().await;
    let db: Arc<DatabaseConnection> =
        Arc::new(MockDatabase::new(DatabaseBackend::Postgres).into_connection());

    let service = build_app(db, jwks_url);
    let claims = base_claims("anonymous", "firebase-uid-intruso");
    // Forjamos con la SEGUNDA clave (no expuesta en el JWKS) -> firma invalida.
    let token = forge_token_with(&claims, KID, &PRIVATE_KEY_INTRUSO);

    let res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::UNAUTHORIZED));
}

/// Lookup defensivo: usuario anonimo se "actualiza" via linkWithCredential,
/// el `firebase_uid` ya existe en otro provider, no se duplica el usuario.
#[tokio::test]
async fn firebase_login_anonimo_que_se_upgradea_no_duplica_usuario() {
    let (_server, jwks_url) = start_jwks_server().await;

    let user_id = Uuid::new_v4();
    let prov_anon_id = Uuid::new_v4();
    let prov_google_id = Uuid::new_v4();
    let token_id = Uuid::new_v4();
    let sub = "firebase-uid-shared";
    let email = "eve@gmail.com";

    let user = mock_usuario(user_id, None, true);
    let prov_anonimo = mock_proveedor(prov_anon_id, user_id, "anonymous", sub);
    let prov_google = mock_proveedor(prov_google_id, user_id, "google.com", sub);
    let tok = mock_refresh(token_id, user_id);

    let db: Arc<DatabaseConnection> = Arc::new(
        MockDatabase::new(DatabaseBackend::Postgres)
            // 1. find prov (google.com, sub) -> None (NO existe esta combinacion exacta)
            .append_query_results::<proveedor_autenticacion::Model, _, _>(vec![empty()])
            // 2. find prov by uid -> Some(la fila anonymous existente)
            .append_query_results(vec![vec![prov_anonimo.clone()]])
            // 3. crear_proveedor_identidad para google.com (mismo id_usuario)
            .append_query_results(vec![vec![prov_google.clone()]])
            // 4. cargar_usuario_obligatorio: buscar_usuario_por_id(prov.id_usuario)
            .append_query_results(vec![vec![user.clone()]])
            // 5. actualizar_ultimo_login: find_by_id
            .append_query_results(vec![vec![user.clone()]])
            // 6. actualizar_ultimo_login: update
            .append_query_results(vec![vec![user.clone()]])
            // 7. enriquecer_perfil_si_vacio: find_by_id (nombre_visible y url_avatar None)
            .append_query_results(vec![vec![user.clone()]])
            // 8. enriquecer: update (claims aportan name+picture, ambos campos vacios -> SET)
            .append_query_results(vec![vec![user.clone()]])
            // 9. guardar_refresh_token
            .append_query_results(vec![vec![tok.clone()]])
            // 10. reload final
            .append_query_results(vec![vec![user.clone()]])
            .into_connection(),
    );

    let service = build_app(db, jwks_url);
    let token = forge_token(
        claims_google(sub, email, "Eve Anon", "https://x/eve.png"),
        KID,
    );

    let mut res = TestClient::post("http://127.0.0.1/api/v1/auth/firebase")
        .json(&json!({ "id_token": token }))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
    let body: serde_json::Value = res.take_json().await.unwrap();
    // El id_usuario es el mismo que tenia la identidad anonima; google.com
    // se ha enlazado al MISMO usuario sin duplicar.
    assert_eq!(body["usuario"]["id"], user_id.to_string());
    assert_eq!(body["usuario"]["proveedor"], "google.com");
}
