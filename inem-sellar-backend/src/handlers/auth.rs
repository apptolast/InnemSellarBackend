//! Handlers de autenticacion: firebase (handshake unico), refrescar, logout.
//!
//! El cliente delega TODO el login en FirebaseAuth (Google, email/password,
//! anonimo) y entrega el Firebase ID Token en `/auth/firebase`. El backend
//! lo valida contra los JWKS de Google y emite sus propios tokens (JWT
//! HS256 + refresh opaco), que son los que usan el resto de endpoints.
//!
//! Cada handler coordina servicio + repositorio, pero NO contiene
//! logica de criptografia ni acceso a BD directamente.
//!
//! Los handlers usan `#[endpoint]` (en vez de `#[handler]`) para que
//! Salvo pueda generar la documentacion OpenAPI automaticamente.

use chrono::{Duration, Utc};
use salvo::oapi::extract::JsonBody;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{proveedor_autenticacion, usuario};
use crate::repositories::{
    AuthRepo, ProveedorAutenticacionRepo, SeaAuthRepo, SeaProveedorAutenticacionRepo,
};
use crate::services::firebase_verifier::{FirebaseClaims, SignInProvider};
use crate::services::{AuthService, FirebaseVerifier};

// ─── DTOs (Data Transfer Objects) ────────────────────────────────
// Structs que definen la forma del JSON de entrada y salida.
// Son diferentes a las entidades de BD — solo exponen lo necesario.

/// Body del POST /api/v1/auth/refrescar
#[derive(Deserialize, ToSchema)]
pub struct RefrescarRequest {
    /// Refresh token emitido en registro o login. Se revoca tras usarlo (rotacion).
    pub refresh_token: String,
}

/// Body del POST /api/v1/auth/logout
#[derive(Deserialize, ToSchema)]
pub struct LogoutRequest {
    /// Refresh token a revocar. Tras el logout el token queda invalido.
    pub refresh_token: String,
}

/// Body del POST /api/v1/auth/firebase
///
/// La app cliente completa el login con FirebaseAuth (Google Sign-In,
/// password de Firebase, etc.) y obtiene un ID Token JWT firmado por
/// Google con RS256. Lo envia tal cual en este campo.
#[derive(Deserialize, ToSchema)]
pub struct FirebaseLoginRequest {
    /// ID Token JWT obtenido en el cliente con
    /// `FirebaseUser.getIdToken(forceRefresh = false)` (KMP / Flutter).
    /// El backend lo verifica criptograficamente contra los JWKS de Google.
    pub id_token: String,
}

/// Respuesta del handshake `/auth/firebase` — contiene tokens propios
/// (HS256) + datos basicos del usuario.
///
/// # Por que `Serialize` y `ToSchema`
/// `Serialize` (de serde) convierte el struct a JSON para la respuesta HTTP.
/// `ToSchema` permite que OpenAPI documente la estructura de la respuesta.
/// Juntos, forman el contrato completo: implementacion + documentacion.
#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    /// Token JWT de acceso. Corta duracion (15 min por defecto).
    /// Incluirlo en `Authorization: Bearer <token>` en cada peticion protegida.
    pub access_token: String,
    /// Token de refresco. Larga duracion (30 dias). Guardar de forma segura.
    pub refresh_token: String,
    /// Datos basicos del usuario recien autenticado.
    pub usuario: UsuarioResponse,
}

/// Datos publicos del usuario (sin hash_contrasena).
///
/// # Por que campos nullable
/// Esta struct la consumen los 3 sub-flujos del handshake Firebase:
///   - `google.com`: tiene email, nombre y avatar de Google.
///   - `password` (Firebase Email/Password): tiene email; name/picture
///     vienen del Firebase profile si el cliente los puso, opcionales.
///   - `anonymous`: no tiene email ni nombre.
///
/// Todos los campos identitarios son `Option` para que el cliente reciba
/// `null` en JSON y pueda tomar decisiones de UI sin romperse.
#[derive(Serialize, ToSchema)]
pub struct UsuarioResponse {
    /// UUID del usuario en la base de datos.
    pub id: Uuid,
    /// Email del usuario. `None` para anonimos y para providers sin email.
    pub email: Option<String>,
    /// Nombre visible publicamente. `None` si no se proporciono.
    pub nombre_visible: Option<String>,
    /// URL del avatar. Procede del proveedor (ej: foto de Google) o del
    /// perfil que el propio usuario subio. `None` si no hay foto.
    pub url_avatar: Option<String>,
    /// `sign_in_provider` literal de Firebase para ESTE login:
    /// `"google.com"` | `"password"` | `"anonymous"`. El cliente lo usa
    /// para decidir que pantalla de re-login mostrar si hace falta.
    pub proveedor: Option<String>,
    /// `true` si el usuario es anonimo. El cliente lo usa para mostrar el
    /// flujo "completar registro".
    pub anonimo: bool,
    /// `true` si el proveedor (Google, etc.) verifico el email. `None` si no
    /// aplica (anonimos, email/password sin verificacion).
    pub email_verificado: Option<bool>,
}

/// Respuesta de refrescar — solo tokens nuevos.
#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    /// Nuevo access token JWT.
    pub access_token: String,
    /// Nuevo refresh token (el anterior queda revocado).
    pub refresh_token: String,
}

/// Respuesta generica con mensaje de confirmacion.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Mensaje descriptivo de la operacion realizada.
    pub mensaje: String,
}

// ─── Helpers privados ────────────────────────────────────────────

/// Construye un `UsuarioResponse` a partir del modelo de BD y el contexto
/// del login en curso (proveedor, flag anonimo, email verificado).
///
/// Centraliza la transformacion para que cualquier nuevo flujo de login
/// devuelva un response consistente con los demas. Los handlers solo
/// pasan el contexto que conocen (proveedor del login actual, etc.).
fn construir_usuario_response(
    user: &usuario::Model,
    proveedor: &str,
    anonimo: bool,
    email_verificado: Option<bool>,
) -> UsuarioResponse {
    UsuarioResponse {
        id: user.id,
        email: user.email.clone(),
        nombre_visible: user.nombre_visible.clone(),
        url_avatar: user.url_avatar.clone(),
        proveedor: Some(proveedor.to_string()),
        anonimo,
        email_verificado,
    }
}

/// Construye el `datos_proveedor` JSONB para una fila Firebase a partir
/// de los claims verificados del ID Token.
///
/// # Que se guarda
/// Solo claims publicos (no la firma) que tengan utilidad operativa:
///   - `sign_in_provider`: distingue google.com / apple.com / etc.
///   - `email_verified`: para auditar account-linking en disputas.
///   - `name`, `picture`: snapshot del perfil al momento del login.
///   - `identities`: util si en el futuro un mismo usuario vincula
///     varios providers (Google + Apple) — el `firebase_uid` no cambia
///     pero las identidades si.
///   - `auth_time`: cuando el usuario realmente autentico (puede ser
///     anterior al `iat` si refresco con un session cookie).
///   - `tenant`: para multitenancy (no usado en InemSellar pero lo
///     guardamos por si acaso).
///
/// # Que NO se guarda
/// El `id_token` original NUNCA. Es un secreto de corta vida que no
/// debe persistir. Si se filtrara la BD, no se podria suplantar al
/// usuario con estos datos solos.
fn construir_datos_proveedor_firebase(claims: &FirebaseClaims) -> serde_json::Value {
    serde_json::json!({
        "sign_in_provider": claims.firebase.sign_in_provider,
        "email_verified": claims.email_verified,
        "name": claims.name,
        "picture": claims.picture,
        "identities": claims.firebase.identities,
        "auth_time": claims.auth_time,
        "tenant": claims.firebase.tenant,
    })
}

/// Lee de forma defensiva el header `User-Agent` de la peticion para
/// usarlo como `informacion_dispositivo` al guardar el refresh token.
///
/// Devuelve `None` si el header esta ausente o no es ASCII parseable.
fn extraer_user_agent(req: &Request) -> Option<String> {
    req.header::<String>("user-agent")
        .filter(|s| !s.trim().is_empty())
}

// ─── Handlers ────────────────────────────────────────────────────

/// POST /api/v1/auth/refrescar — Obtener nuevos tokens con un refresh token.
///
/// # Rotacion de tokens
/// Al refrescar, el refresh token anterior se revoca y se emite uno nuevo.
/// Esto limita el dano si un refresh token es comprometido: el atacante
/// solo tiene una ventana corta antes de que el token se invalide.
#[endpoint(tags("Auth"))]
pub async fn refrescar(
    req: &mut Request,
    body: JsonBody<RefrescarRequest>,
    depot: &mut Depot,
) -> Result<Json<TokenResponse>, AppError> {
    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();
    let proveedor_repo = depot
        .obtain::<SeaProveedorAutenticacionRepo>()
        .map_err(|_| AppError::Internal("ProveedorAutenticacionRepo no disponible".into()))?
        .clone();

    // Hashear el refresh token recibido y buscarlo en BD
    let hash = auth_service.hashear_refresh_token(&body.refresh_token);
    let token_db = auth_repo
        .buscar_refresh_token_por_hash(&hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verificar que no ha expirado
    if let Some(expira) = token_db.expira_en
        && expira < Utc::now().fixed_offset()
    {
        return Err(AppError::Unauthorized);
    }

    // Revocar el token anterior (rotacion)
    auth_repo.revocar_refresh_token(token_db.id).await?;

    // Preservar el flag `anonimo` al rotar: consultamos si el usuario tiene
    // una identidad `proveedor='anonymous'`. Asi un anonimo que refresca
    // sigue siendo anonimo en el nuevo access_token.
    let es_anonimo = proveedor_repo.es_anonimo(token_db.id_usuario).await?;

    // Generar nuevo par de tokens manteniendo el flag.
    let access_token =
        auth_service.generar_access_token_con_flag(token_db.id_usuario, es_anonimo)?;
    let new_refresh_raw = auth_service.generar_refresh_token();
    let new_refresh_hash = auth_service.hashear_refresh_token(&new_refresh_raw);

    let expira = (Utc::now() + Duration::days(30)).fixed_offset();
    let user_agent = extraer_user_agent(req);
    auth_repo
        .guardar_refresh_token(
            token_db.id_usuario,
            &new_refresh_hash,
            user_agent.as_deref(),
            expira,
        )
        .await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: new_refresh_raw,
    }))
}

/// POST /api/v1/auth/logout — Cerrar sesion (revocar refresh token).
/// Requiere autenticacion (access token en header Authorization).
///
/// # Por que `security(("bearer_auth" = []))`
/// Esta anotacion le dice a OpenAPI que este endpoint requiere el esquema
/// de seguridad `bearer_auth` definido en main.rs (JWT Bearer token).
/// Swagger UI mostrara un candado y permitira al usuario introducir su token
/// para probar el endpoint directamente desde la documentacion.
#[endpoint(tags("Auth"), security(("bearer_auth" = [])))]
pub async fn logout(
    body: JsonBody<LogoutRequest>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();

    // Buscar el refresh token en BD y revocarlo
    let hash = auth_service.hashear_refresh_token(&body.refresh_token);
    let token_db = auth_repo
        .buscar_refresh_token_por_hash(&hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    auth_repo.revocar_refresh_token(token_db.id).await?;

    Ok(Json(MensajeResponse {
        mensaje: "Sesion cerrada".into(),
    }))
}

// ─── Login con Firebase (handshake unico para todos los providers) ─

/// POST /api/v1/auth/firebase — Handshake unico con Firebase ID Token.
///
/// La app cliente delega TODO el login en FirebaseAuth (Google,
/// email/password, anonimo) y entrega aqui el ID Token JWT firmado por
/// Google con RS256. El backend lo verifica contra los JWKS de Google,
/// tipa el `sign_in_provider` con la whitelist `SignInProvider`, y
/// resuelve un usuario segun el caso:
///
///   - **`google.com` / `password`**: si la identidad ya existe (mismo
///     `firebase_uid` y mismo provider) la reusa y refresca su
///     snapshot JSONB. Si no, intenta auto-link por email cuando
///     `email_verified=true` (asi un usuario legacy con `hash_contrasena`
///     se enlaza automaticamente). Si el email coincide pero NO esta
///     verificado, devuelve 409 (anti-takeover). Si nada coincide, crea
///     un usuario OAuth nuevo.
///   - **`anonymous`**: crea un usuario sin email/hash y guarda el
///     `firebase_uid` como `identificador_proveedor`. El JWT propio
///     emitido lleva `anonimo=true` para que el cliente sepa que falta
///     completar el registro.
///   - **Lookup defensivo (`linkWithCredential` en cliente)**: si el
///     `firebase_uid` ya existe en OTRO provider, reusa el mismo
///     `id_usuario` y solo anade la nueva fila de identidad. Evita
///     duplicar usuarios cuando un anonimo se actualiza a Google/password
///     conservando su uid.
///
/// # Codigos de error
/// - 400: `id_token` vacio o `sign_in_provider` no soportado todavia
///   (Apple, phone, etc.).
/// - 401: token invalido (firma, expirado, audience, issuer, kid desconocido).
/// - 409: account linking bloqueado por `email_verified=false`.
/// - 500: fallo al descargar JWKS de Google o error de BD.
#[endpoint(tags("Auth"))]
pub async fn login_firebase(
    req: &mut Request,
    body: JsonBody<FirebaseLoginRequest>,
    depot: &mut Depot,
) -> Result<Json<AuthResponse>, AppError> {
    // ── 1. Extraer servicios y repos del Depot ──
    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let firebase = depot
        .obtain::<FirebaseVerifier>()
        .map_err(|_| AppError::Internal("FirebaseVerifier no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();
    let proveedor_repo = depot
        .obtain::<SeaProveedorAutenticacionRepo>()
        .map_err(|_| AppError::Internal("ProveedorAutenticacionRepo no disponible".into()))?
        .clone();

    // ── 2. Validar input minimo ──
    if body.id_token.trim().is_empty() {
        return Err(AppError::BadRequest("id_token es obligatorio".into()));
    }

    // ── 3. Verificar el ID Token de Firebase (criptografia + claims) ──
    let claims = firebase.verify(&body.id_token).await?;

    // ── 4. Tipar el provider via whitelist (rechaza Apple/phone/etc. con 400) ──
    let provider = claims.provider().map_err(|p| {
        tracing::warn!(provider = %p, "sign_in_provider de Firebase no soportado todavia");
        AppError::BadRequest(format!("proveedor `{p}` no soportado todavia"))
    })?;
    let anonimo = matches!(provider, SignInProvider::Anonymous);

    // ── 5. Resolver el usuario (upsert + account-linking + lookup defensivo) ──
    let usuario = upsert_usuario_firebase(&auth_repo, &proveedor_repo, &claims, provider).await?;

    // ── 6. Ultimo login + enriquecer perfil con name/picture (solo no-anonimos) ──
    auth_repo.actualizar_ultimo_login(usuario.id).await?;
    if !anonimo {
        auth_repo
            .enriquecer_perfil_si_vacio(
                usuario.id,
                claims.name.as_deref(),
                claims.picture.as_deref(),
            )
            .await?;
    }

    // ── 7. Emitir tokens propios (HS256, flag anonimo derivado del provider) ──
    let access_token = auth_service.generar_access_token_con_flag(usuario.id, anonimo)?;
    let refresh_raw = auth_service.generar_refresh_token();
    let refresh_hash = auth_service.hashear_refresh_token(&refresh_raw);
    let expira = (Utc::now() + Duration::days(30)).fixed_offset();
    let user_agent = extraer_user_agent(req);
    auth_repo
        .guardar_refresh_token(usuario.id, &refresh_hash, user_agent.as_deref(), expira)
        .await?;

    // ── 8. Recargar el usuario (puede haber cambiado nombre_visible / url_avatar) ──
    let usuario_final = auth_repo
        .buscar_usuario_por_id(usuario.id)
        .await?
        .unwrap_or(usuario);

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: refresh_raw,
        usuario: construir_usuario_response(
            &usuario_final,
            provider.as_str(),
            anonimo,
            claims.email_verified,
        ),
    }))
}

/// Resuelve a que usuario corresponde un login Firebase, creando o
/// vinculando segun haga falta.
///
/// Extraida del handler para mantener el flujo principal legible.
/// Cubre 5 casos en orden:
///   1. Identidad EXACTA `(provider, firebase_uid)` ya existente -> reusa.
///   2. `firebase_uid` existe en OTRO provider (`linkWithCredential`) -> reusa
///      `id_usuario` y solo anade la nueva fila de identidad.
///   3. Anonymous nuevo -> crea usuario sin email/hash + identidad.
///   4. password/google.com con email coincidente y verificado -> auto-link.
///      Si el email NO esta verificado -> 409 (anti-takeover).
///   5. password/google.com sin match -> crea usuario OAuth nuevo + identidad.
async fn upsert_usuario_firebase(
    auth_repo: &SeaAuthRepo,
    proveedor_repo: &SeaProveedorAutenticacionRepo,
    claims: &FirebaseClaims,
    provider: SignInProvider,
) -> Result<usuario::Model, AppError> {
    let provider_str = provider.as_str();
    let datos = construir_datos_proveedor_firebase(claims);

    // 1. Identidad EXACTA (provider, sub) ya existe -> refresca y devuelve.
    if let Some(prov) = proveedor_repo
        .buscar_por_proveedor_e_identificador(provider_str, &claims.sub)
        .await?
    {
        proveedor_repo
            .actualizar_datos(prov.id, claims.email.as_deref(), Some(datos))
            .await?;
        return cargar_usuario_obligatorio(auth_repo, prov.id_usuario).await;
    }

    // 2. Lookup defensivo: el mismo `sub` ya existe en OTRO provider.
    //    Caso: usuario anonimo que hace `linkWithCredential` en el cliente y
    //    conserva su firebase_uid pero cambia de provider. Reusamos el
    //    `id_usuario` y solo anadimos la nueva fila de identidad para no
    //    crear un usuario duplicado.
    if let Some(prov) = proveedor_repo
        .buscar_por_firebase_uid_cualquier_provider(&claims.sub)
        .await?
    {
        tracing::info!(
            firebase_uid = %claims.sub,
            provider_existente = %prov.proveedor.as_deref().unwrap_or("(null)"),
            provider_nuevo = %provider_str,
            "firebase_uid existente en otro provider, anadiendo nueva identidad al mismo usuario"
        );
        crear_proveedor_identidad(proveedor_repo, prov.id_usuario, provider_str, claims, datos)
            .await?;
        return cargar_usuario_obligatorio(auth_repo, prov.id_usuario).await;
    }

    // 3. Anonymous nuevo: crea usuario sin email/hash, fila con identificador=firebase_uid.
    if matches!(provider, SignInProvider::Anonymous) {
        let nuevo = auth_repo.crear_usuario_anonimo().await?;
        crear_proveedor_identidad(proveedor_repo, nuevo.id, provider_str, claims, datos).await?;
        return Ok(nuevo);
    }

    // 4. password/google.com con email coincidente -> auto-link si email_verified=true.
    //    Asi un usuario legacy con `hash_contrasena` se enlaza automaticamente
    //    a su nueva identidad Firebase password sin intervencion manual.
    if let Some(email) = claims.email.as_deref()
        && let Some(usuario_existente) = auth_repo.buscar_por_email(email).await?
    {
        // Politica: sin verified=true un atacante podria registrar un
        // proveedor OAuth con un email ajeno y apoderarse de la cuenta local.
        if claims.email_verified != Some(true) {
            return Err(AppError::Conflict(
                "ya existe una cuenta con este email; verifica el email en tu \
                 proveedor y reintenta para vincular automaticamente"
                    .into(),
            ));
        }
        crear_proveedor_identidad(
            proveedor_repo,
            usuario_existente.id,
            provider_str,
            claims,
            datos,
        )
        .await?;
        return Ok(usuario_existente);
    }

    // 5. password/google.com sin match previo -> usuario OAuth nuevo + identidad.
    let nuevo = auth_repo
        .crear_usuario_oauth(
            claims.email.as_deref(),
            claims.name.as_deref(),
            claims.picture.as_deref(),
        )
        .await?;
    crear_proveedor_identidad(proveedor_repo, nuevo.id, provider_str, claims, datos).await?;
    Ok(nuevo)
}

/// Inserta una fila en `proveedores_autenticacion` que representa la
/// identidad Firebase de un usuario en un provider concreto.
///
/// `provider_str` es el literal `sign_in_provider` (ya validado contra
/// la whitelist tipada `SignInProvider`); ej: `"google.com"`, `"password"`,
/// `"anonymous"`. Se guarda en la columna `proveedor`, mientras que
/// `claims.sub` (el `firebase_uid`) va a `identificador_proveedor`.
async fn crear_proveedor_identidad(
    proveedor_repo: &SeaProveedorAutenticacionRepo,
    id_usuario: Uuid,
    provider_str: &str,
    claims: &FirebaseClaims,
    datos: serde_json::Value,
) -> Result<proveedor_autenticacion::Model, AppError> {
    proveedor_repo
        .crear(
            id_usuario,
            provider_str,
            Some(&claims.sub),
            claims.email.as_deref(),
            Some(datos),
        )
        .await
}

/// Carga un usuario por id devolviendo `Internal` si no existe — protege
/// contra inconsistencias: si una fila de `proveedores_autenticacion`
/// apunta a un usuario que ya no existe, es un bug, no un 401.
async fn cargar_usuario_obligatorio(
    auth_repo: &SeaAuthRepo,
    id: Uuid,
) -> Result<usuario::Model, AppError> {
    auth_repo
        .buscar_usuario_por_id(id)
        .await?
        .ok_or_else(|| AppError::Internal(format!("usuario {id} referenciado pero inexistente")))
}
