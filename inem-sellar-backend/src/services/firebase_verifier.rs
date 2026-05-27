// src/services/firebase_verifier.rs
//
// Verificador de Firebase ID Tokens — fase: login con Google y anonimo.
//
// El cliente Flutter completa el login con FirebaseAuth en su lado y obtiene
// un ID Token JWT firmado con RS256 por Google. El backend NO confia en ese
// token sin verificar: descarga la clave publica del JWKS de Google, valida
// la firma y los claims (issuer, audience, expiracion) y solo entonces
// considera valido al usuario.
//
// NO accede a la BD — eso lo hace el repositorio en `proveedor_autenticacion_repo`.
// NO maneja HTTP de la API — eso lo hace el handler en `handlers/auth.rs`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use jsonwebtoken::jwk::{AlgorithmParameters, Jwk, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};

use crate::errors::AppError;

/// URL publica del JWKS de Google que firma los Firebase ID Tokens.
///
/// # Por que esta URL y no `/x509/...`
/// Google publica las claves publicas en dos formatos:
///   - JWKS (JSON Web Key Set) — formato JSON con `kty`, `n`, `e`, `kid`. ESTE.
///   - x509 — certificados PEM en JSON.
///
/// La URL JWKS es la canonica y la que mejor encaja con `jsonwebtoken` 9.
/// La eligio el equipo de Firebase como fuente oficial.
const JWKS_URL_DEFAULT: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

/// URL publica de certificados X.509 del firmante legacy de Identity Toolkit.
///
/// El admin web usa `accounts:signInWithPassword` via REST. En este proyecto
/// esa API esta emitiendo tokens legacy con `iss=https://identitytoolkit.google.com/`
/// y `kid` corto (ej: `3XdImQ`). Google documenta este endpoint para verificar
/// esos tokens legacy.
const LEGACY_PUBLIC_KEYS_URL_DEFAULT: &str = "https://identitytoolkit.googleapis.com/v1/publicKeys";

/// TTL del cache de JWKS si la respuesta no incluye `Cache-Control: max-age=...`.
/// 1 hora es conservador: las claves de Google rotan cada varias horas.
const FALLBACK_TTL: Duration = Duration::from_secs(60 * 60);

/// Tolerancia de reloj para `exp`, `iat`, `auth_time`. 60 segundos es el
/// estandar de la industria: corrige drift NTP sin abrir ventana de replay.
const CLOCK_SKEW_SECS: u64 = 60;

/// Timeout duro para la descarga del JWKS. Si Google tarda mas, fallamos
/// rapido en vez de bloquear handlers eternamente.
const JWKS_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

// ─── Provider whitelist tipada ────────────────────────────────────────────

/// Conjunto de Firebase `sign_in_provider` que el backend acepta.
///
/// # Por que un enum en vez de comparaciones de string sueltas
/// El `match` exhaustivo en el handler dejara de COMPILAR el dia que se
/// anada `Apple` y se olvide cubrir esa rama. Centraliza los literales
/// en un solo sitio y evita que un provider nuevo pase silenciosamente
/// por el camino equivocado.
///
/// # Por que `Copy`
/// El enum no tiene datos asociados (3 variantes unitarias), asi que
/// copiarlo es trivial y permite pasarlo por valor sin ceremonia
/// (`fn(provider: SignInProvider)` en vez de `&SignInProvider`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignInProvider {
    /// Firebase OAuth con Google. `sign_in_provider == "google.com"`.
    GoogleCom,
    /// Firebase OAuth con Apple. `sign_in_provider == "apple.com"`.
    ///
    /// En iOS via AuthenticationServices nativo (Sign in with Apple del
    /// sistema), en Android via Firebase OAuth web (Chrome Custom Tab
    /// abriendo `appleid.apple.com`). En ambos casos Firebase emite el
    /// mismo ID Token RS256 que validamos contra el JWKS de Google.
    AppleCom,
    /// Firebase Email/Password. `sign_in_provider == "password"`.
    Password,
    /// Firebase Anonymous Auth. `sign_in_provider == "anonymous"`.
    Anonymous,
}

impl SignInProvider {
    /// Devuelve el literal Firebase que se guarda en
    /// `proveedores_autenticacion.proveedor` y se serializa en
    /// `usuario.proveedor` de la respuesta JSON al cliente.
    ///
    /// El round-trip `try_from(p.as_str()) == Ok(p)` se verifica en tests.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GoogleCom => "google.com",
            Self::AppleCom => "apple.com",
            Self::Password => "password",
            Self::Anonymous => "anonymous",
        }
    }
}

impl TryFrom<&str> for SignInProvider {
    type Error = String;

    /// Convierte el `sign_in_provider` recibido de Firebase a un
    /// `SignInProvider` tipado.
    ///
    /// # Por que `Err(String)` y no un enum de error
    /// El caller (handler `login_firebase`) usa el literal desconocido
    /// para construir un mensaje 400 informativo del estilo
    /// `"proveedor `phone` no soportado todavia"`. Devolver el string
    /// original ahorra una capa de mapeo y mantiene el caller delgado.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "google.com" => Ok(Self::GoogleCom),
            "apple.com" => Ok(Self::AppleCom),
            "password" => Ok(Self::Password),
            "anonymous" => Ok(Self::Anonymous),
            other => Err(other.to_string()),
        }
    }
}

// ─── Claims del Firebase ID Token ─────────────────────────────────────────

/// Datos del proveedor concreto que firmo el token (claim `firebase`).
///
/// Firebase Auth usa este bloque para distinguir entre los distintos
/// providers (google.com, password, anonymous, apple.com...) que pueden
/// haber autenticado al usuario.
///
/// # Por que `#[serde(default)]` en `identities`
/// Para tokens de usuarios anonimos, `identities` esta presente como objeto
/// vacio `{}`. Para tokens de Google viene rellenado con
/// `{ "google.com": ["1234567890"], "email": ["alice@gmail.com"] }`.
/// `default` evita errores si Firebase omite el campo en algun futuro.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FirebaseProviderInfo {
    /// Identifica el proveedor: `"google.com"`, `"password"`, `"anonymous"`,
    /// `"apple.com"`, etc.
    pub sign_in_provider: String,

    /// Identidades por proveedor, ej: `{ "google.com": ["uid1"], "email": [...] }`.
    /// Util para Account Linking (un usuario con multiples identidades).
    #[serde(default)]
    pub identities: HashMap<String, serde_json::Value>,

    /// Solo presente si Firebase Auth se uso con multitenancy.
    /// En InemSellar no se usa, asi que normalmente sera `None`.
    #[serde(default)]
    pub tenant: Option<String>,
}

/// Claims completos de un Firebase ID Token tras la validacion criptografica.
///
/// # Por que tantos `Option<...>`
/// Solo los claims estandar JWT (iss, aud, sub, iat, exp, auth_time) estan
/// SIEMPRE presentes — los validamos como `Required` en `Validation`.
/// Los demas dependen del proveedor: un usuario anonimo no tiene `email`,
/// un usuario que se acaba de registrar no tiene `picture`, etc.
/// `Option` modela esto correctamente sin asumir nada.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FirebaseClaims {
    /// Issuer del token aceptado.
    ///
    /// Normalmente es `https://securetoken.google.com/<project_id>`. Para
    /// tokens legacy de Identity Toolkit REST puede ser
    /// `https://identitytoolkit.google.com/`.
    pub iss: String,
    /// Audience — debe ser `<project_id>` exactamente.
    pub aud: String,
    /// Subject — el `firebase_uid` del usuario. Identificador estable.
    pub sub: String,
    /// Issued At (Unix epoch en segundos).
    pub iat: i64,
    /// Expiration (Unix epoch en segundos).
    pub exp: i64,
    /// Cuando el usuario se autentico (puede ser anterior a `iat` si refresco).
    pub auth_time: i64,

    /// Email del usuario, si el proveedor lo proporciona.
    /// Vacio para anonimos y para usuarios de phone-only.
    #[serde(default)]
    pub email: Option<String>,
    /// `true` si el proveedor (Google, etc.) ha verificado el email.
    #[serde(default)]
    pub email_verified: Option<bool>,
    /// Nombre completo segun el proveedor.
    #[serde(default)]
    pub name: Option<String>,
    /// URL del avatar segun el proveedor.
    #[serde(default)]
    pub picture: Option<String>,

    /// Bloque con el provider y las identidades.
    pub firebase: FirebaseProviderInfo,
}

impl FirebaseClaims {
    /// Convierte el campo `sign_in_provider` del claim `firebase` a un
    /// `SignInProvider` tipado.
    ///
    /// Devuelve `Err(literal)` si el provider no esta en la whitelist
    /// (Apple, phone, etc., todavia no soportados). El caller
    /// (`login_firebase`) usa ese literal para construir un 400 con un
    /// mensaje accionable para el cliente.
    pub fn provider(&self) -> Result<SignInProvider, String> {
        SignInProvider::try_from(self.firebase.sign_in_provider.as_str())
    }
}

// ─── Cache de JWKS ────────────────────────────────────────────────────────

/// Estado interno del cache de claves publicas.
struct JwksCache {
    /// `None` mientras no se haya descargado nunca; `Some(set)` despues.
    set: Option<JwkSet>,
    /// Momento a partir del cual el cache esta caducado. Derivado de
    /// `Cache-Control: max-age` de la respuesta de Google, con `FALLBACK_TTL`
    /// si la cabecera no se puede parsear.
    expires_at: Instant,
}

/// Cache de certificados X.509 legacy publicados por Identity Toolkit.
struct LegacyPublicKeysCache {
    /// `kid -> certificado PEM`.
    keys: Option<HashMap<String, String>>,
    expires_at: Instant,
}

/// Claims de los tokens legacy de Identity Toolkit REST.
#[derive(Debug, Clone, Deserialize)]
struct LegacyIdentityToolkitClaims {
    pub iss: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    pub user_id: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub sign_in_provider: Option<String>,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub display_name: Option<String>,
}

// ─── FirebaseVerifier ─────────────────────────────────────────────────────

/// Verificador de Firebase ID Tokens.
///
/// # Por que `Clone`
/// Salvo necesita que los servicios inyectados en `Depot` sean `Clone`.
/// Internamente todo es `Arc<...>`, asi que clonar es barato (un AtomicUsize++).
///
/// # Por que `RwLock` y no `Mutex` para el cache
/// El 99% de las llamadas son LECTURAS del cache (verificar tokens). Solo
/// cuando el cache caduca o no encontramos un `kid` necesitamos ESCRIBIR
/// (refrescar). `RwLock` permite multiples lectores concurrentes y solo
/// bloquea cuando hay un escritor — exactamente nuestro patron de acceso.
///
/// # Por que un `Mutex` adicional para `refresh_lock`
/// Sin el, varias peticiones simultaneas que vean el cache caducado
/// dispararian fetches en paralelo al JWKS (thundering herd). Con
/// `refresh_lock`, solo una hace el fetch; las demas esperan y reusan
/// el cache recien actualizado (double-checked locking).
#[derive(Clone)]
pub struct FirebaseVerifier {
    /// `project_id` de Firebase. Validado contra el claim `aud`.
    project_id: Arc<String>,
    /// Issuer esperado: `https://securetoken.google.com/<project_id>`.
    issuer: Arc<String>,
    /// Issuer de los tokens legacy de Identity Toolkit REST.
    legacy_issuer: Arc<String>,
    /// URL del JWKS. Configurable para tests con `new_with_url`.
    jwks_url: Arc<String>,
    /// URL de certificados legacy. Configurable para tests con `new_with_urls`.
    legacy_public_keys_url: Arc<String>,
    /// Cliente HTTP reutilizable. `reqwest::Client` ya envuelve un Arc internamente.
    http: reqwest::Client,
    /// Cache de claves publicas con TTL.
    cache: Arc<RwLock<JwksCache>>,
    /// Cache de certificados legacy con TTL.
    legacy_cache: Arc<RwLock<LegacyPublicKeysCache>>,
    /// Mutex para serializar refrescos del cache (evita thundering herd).
    refresh_lock: Arc<Mutex<()>>,
    /// Mutex para serializar refrescos de certificados legacy.
    legacy_refresh_lock: Arc<Mutex<()>>,
}

impl FirebaseVerifier {
    /// Crea un verificador apuntando al JWKS oficial de Google.
    ///
    /// # Panics
    /// Si reqwest no puede construir su cliente (extremadamente raro, suele
    /// indicar un sistema sin TLS roots). Aceptable porque ocurre solo al
    /// arrancar el servidor — fallo rapido es preferible a degradacion.
    pub fn new(project_id: String) -> Self {
        Self::new_with_urls(
            project_id,
            JWKS_URL_DEFAULT.to_string(),
            LEGACY_PUBLIC_KEYS_URL_DEFAULT.to_string(),
        )
    }

    /// Constructor parametrizable. Usado en tests con `wiremock` para
    /// servir un JWKS mockeado en lugar del de Google.
    pub fn new_with_url(project_id: String, jwks_url: String) -> Self {
        Self::new_with_urls(
            project_id,
            jwks_url,
            LEGACY_PUBLIC_KEYS_URL_DEFAULT.to_string(),
        )
    }

    /// Constructor parametrizable de ambas fuentes de claves.
    pub fn new_with_urls(
        project_id: String,
        jwks_url: String,
        legacy_public_keys_url: String,
    ) -> Self {
        let issuer = format!("https://securetoken.google.com/{project_id}");
        let http = reqwest::Client::builder()
            .timeout(JWKS_FETCH_TIMEOUT)
            .build()
            .expect("reqwest::Client::builder() fallo al construir cliente HTTP");

        Self {
            project_id: Arc::new(project_id),
            issuer: Arc::new(issuer),
            legacy_issuer: Arc::new("https://identitytoolkit.google.com/".to_string()),
            jwks_url: Arc::new(jwks_url),
            legacy_public_keys_url: Arc::new(legacy_public_keys_url),
            http,
            cache: Arc::new(RwLock::new(JwksCache {
                set: None,
                // `Instant::now()` ya esta caducado: el primer `verify` forzara
                // un fetch del JWKS. Es el comportamiento deseado.
                expires_at: Instant::now(),
            })),
            legacy_cache: Arc::new(RwLock::new(LegacyPublicKeysCache {
                keys: None,
                expires_at: Instant::now(),
            })),
            refresh_lock: Arc::new(Mutex::new(())),
            legacy_refresh_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Verifica un Firebase ID Token y devuelve los claims si es valido.
    ///
    /// # Por que `AppError::Unauthorized` para todos los fallos de validacion
    /// Desde el punto de vista del cliente, cualquier fallo de verificacion
    /// (firma, expiracion, audience, issuer, kid desconocido, etc.) significa
    /// lo mismo: "tu token no sirve". Distinguir entre subcategorias daria
    /// pistas a un atacante sobre QUE parte de su token forjado fallo.
    /// Por eso todos colapsan a 401 — y se loguea el motivo real para el operador.
    ///
    /// # Por que algunos errores son `Internal`
    /// Si falla la red al descargar el JWKS, no es culpa del cliente.
    /// Devolver 500 indica al cliente que reintente; devolver 401 le diria
    /// que su token es invalido (mentira).
    pub async fn verify(&self, id_token: &str) -> Result<FirebaseClaims, AppError> {
        // 1. Decodificar la cabecera SIN validar firma para sacar el `kid`.
        //    Necesitamos el `kid` para saber que clave publica buscar.
        let header = decode_header(id_token).map_err(|e| {
            tracing::warn!(error = %e, "Header de Firebase ID Token invalido");
            AppError::Unauthorized
        })?;
        let kid = header.kid.ok_or_else(|| {
            tracing::warn!("Firebase ID Token sin claim `kid` en el header");
            AppError::Unauthorized
        })?;

        // 2. Buscar primero el formato moderno de Firebase Secure Token.
        if let Some(jwk) = self.find_jwk(&kid).await? {
            return self.verify_secure_token(id_token, &jwk).await;
        }

        // 3. Si no existe en Secure Token, probar el firmante legacy de
        //    Identity Toolkit. El admin web REST de este proyecto llega por
        //    este camino (`kid` corto, issuer `identitytoolkit.google.com`).
        if let Some(cert_pem) = self.find_legacy_cert(&kid).await? {
            return self.verify_legacy_identity_toolkit_token(id_token, &cert_pem);
        }

        // 4. Las claves pueden haber rotado antes de caducar la cache: forzar
        //    un refresh de ambas fuentes y reintentar una vez.
        tracing::info!(kid = %kid, "kid no en caches de Google, forzando refresh");
        self.refresh_jwks(true).await?;
        if let Some(jwk) = self.find_jwk(&kid).await? {
            return self.verify_secure_token(id_token, &jwk).await;
        }
        self.refresh_legacy_public_keys(true).await?;
        if let Some(cert_pem) = self.find_legacy_cert(&kid).await? {
            return self.verify_legacy_identity_toolkit_token(id_token, &cert_pem);
        }

        tracing::warn!(kid = %kid, "kid no encontrado en Secure Token ni Identity Toolkit");
        Err(AppError::Unauthorized)
    }

    async fn verify_secure_token(
        &self,
        id_token: &str,
        jwk: &Jwk,
    ) -> Result<FirebaseClaims, AppError> {
        // Construir DecodingKey desde la JWK.
        //    Por seguridad solo aceptamos RSA — Firebase usa RS256, nada mas.
        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .map_err(|e| AppError::Internal(format!("JWK RSA malformada: {e}")))?,
            other => {
                tracing::warn!(?other, "Tipo de JWK no soportado, esperaba RSA");
                return Err(AppError::Unauthorized);
            }
        };

        // Configurar la validacion estricta:
        //    - Algoritmo PINNED a RS256 (no HS256, no none — protege contra alg confusion).
        //    - audience = project_id.
        //    - issuer = securetoken.google.com/<project_id>.
        //    - claims minimos requeridos.
        //    - leeway de 60s para clock skew.
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.project_id.as_str()]);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_required_spec_claims(&["exp", "iat", "aud", "iss", "sub"]);
        validation.leeway = CLOCK_SKEW_SECS;

        // Decodificar y validar firma + claims.
        let data = decode::<FirebaseClaims>(id_token, &decoding_key, &validation).map_err(|e| {
            tracing::warn!(error = %e, "Validacion Firebase ID Token fallo");
            AppError::Unauthorized
        })?;
        let claims = data.claims;

        self.validar_claims_comunes(&claims)?;
        Ok(claims)
    }

    fn verify_legacy_identity_toolkit_token(
        &self,
        id_token: &str,
        cert_pem: &str,
    ) -> Result<FirebaseClaims, AppError> {
        let decoding_key = DecodingKey::from_rsa_pem(cert_pem.as_bytes()).map_err(|e| {
            tracing::warn!(error = %e, "Certificado legacy Identity Toolkit malformado");
            AppError::Unauthorized
        })?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[self.project_id.as_str()]);
        validation.set_issuer(&[self.legacy_issuer.as_str()]);
        validation.set_required_spec_claims(&["exp", "iat", "aud", "iss"]);
        validation.leeway = CLOCK_SKEW_SECS;

        let data = decode::<LegacyIdentityToolkitClaims>(id_token, &decoding_key, &validation)
            .map_err(|e| {
                tracing::warn!(error = %e, "Validacion legacy Identity Toolkit token fallo");
                AppError::Unauthorized
            })?;

        let legacy = data.claims;
        if legacy.user_id.trim().is_empty() {
            tracing::warn!("Legacy Identity Toolkit token con `user_id` vacio");
            return Err(AppError::Unauthorized);
        }

        let provider = legacy.sign_in_provider.ok_or_else(|| {
            tracing::warn!("Legacy Identity Toolkit token sin `sign_in_provider`");
            AppError::Unauthorized
        })?;

        let mut identities = HashMap::new();
        if let Some(email) = legacy.email.as_ref() {
            identities.insert("email".to_string(), serde_json::json!([email]));
        }

        let claims = FirebaseClaims {
            iss: legacy.iss,
            aud: legacy.aud,
            sub: legacy.user_id,
            iat: legacy.iat,
            exp: legacy.exp,
            auth_time: legacy.iat,
            email: legacy.email,
            email_verified: legacy.verified,
            name: legacy.display_name,
            picture: None,
            firebase: FirebaseProviderInfo {
                sign_in_provider: provider,
                identities,
                tenant: None,
            },
        };

        self.validar_claims_comunes(&claims)?;
        Ok(claims)
    }

    fn validar_claims_comunes(&self, claims: &FirebaseClaims) -> Result<(), AppError> {
        // Comprobaciones manuales que `jsonwebtoken` no hace:
        //    - `sub` no vacio (es el firebase_uid).
        //    - `auth_time` no en el futuro (con tolerancia de reloj).
        if claims.sub.trim().is_empty() {
            tracing::warn!("Firebase ID Token con `sub` vacio");
            return Err(AppError::Unauthorized);
        }
        let now = chrono::Utc::now().timestamp();
        if claims.auth_time > now + CLOCK_SKEW_SECS as i64 {
            tracing::warn!(auth_time = claims.auth_time, now, "auth_time en el futuro");
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }

    /// Busca una `Jwk` por `kid` en el cache, refrescando si esta caducado.
    async fn find_jwk(&self, kid: &str) -> Result<Option<Jwk>, AppError> {
        if self.is_expired().await {
            self.refresh_jwks(false).await?;
        }
        let cache = self.cache.read().await;
        Ok(cache.set.as_ref().and_then(|set| set.find(kid).cloned()))
    }

    /// Busca un certificado legacy por `kid` en cache, refrescando si caduca.
    async fn find_legacy_cert(&self, kid: &str) -> Result<Option<String>, AppError> {
        if self.is_legacy_expired().await {
            self.refresh_legacy_public_keys(false).await?;
        }
        let cache = self.legacy_cache.read().await;
        Ok(cache.keys.as_ref().and_then(|keys| keys.get(kid).cloned()))
    }

    /// `true` si el cache esta vacio o ha caducado.
    async fn is_expired(&self) -> bool {
        let cache = self.cache.read().await;
        cache.set.is_none() || Instant::now() >= cache.expires_at
    }

    async fn is_legacy_expired(&self) -> bool {
        let cache = self.legacy_cache.read().await;
        cache.keys.is_none() || Instant::now() >= cache.expires_at
    }

    /// Refresca el cache de JWKS desde Google.
    ///
    /// # Double-checked locking
    /// Si llegan 50 requests simultaneas con el cache caducado, sin esta
    /// proteccion las 50 dispararian fetches paralelos al JWKS (thundering
    /// herd). Con el `refresh_lock`, solo la primera entra; las demas esperan
    /// el guard, y al entrar comprueban de nuevo `is_expired()`: si el primero
    /// ya refresco, no vuelven a fetchear.
    async fn refresh_jwks(&self, force: bool) -> Result<(), AppError> {
        let _guard = self.refresh_lock.lock().await;

        // Double-check: el cache puede haberse refrescado mientras esperabamos.
        if !force && !self.is_expired().await {
            return Ok(());
        }

        let resp = self
            .http
            .get(self.jwks_url.as_str())
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("error de red al descargar JWKS: {e}")))?
            .error_for_status()
            .map_err(|e| AppError::Internal(format!("HTTP no-OK al descargar JWKS: {e}")))?;

        // Respetar `Cache-Control: max-age=N` de Google.
        // Si la cabecera no se puede parsear, usar FALLBACK_TTL.
        let ttl = resp
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_max_age)
            .map(Duration::from_secs)
            .unwrap_or(FALLBACK_TTL);

        let body = resp
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("error leyendo cuerpo JWKS: {e}")))?;
        let set: JwkSet = serde_json::from_slice(&body)
            .map_err(|e| AppError::Internal(format!("JWKS malformado: {e}")))?;

        let mut cache = self.cache.write().await;
        cache.set = Some(set);
        cache.expires_at = Instant::now() + ttl;
        tracing::info!(ttl_secs = ttl.as_secs(), "JWKS de Firebase refrescado");
        Ok(())
    }

    async fn refresh_legacy_public_keys(&self, force: bool) -> Result<(), AppError> {
        let _guard = self.legacy_refresh_lock.lock().await;

        if !force && !self.is_legacy_expired().await {
            return Ok(());
        }

        let resp = self
            .http
            .get(self.legacy_public_keys_url.as_str())
            .send()
            .await
            .map_err(|e| {
                AppError::Internal(format!(
                    "error de red al descargar certificados Identity Toolkit: {e}"
                ))
            })?
            .error_for_status()
            .map_err(|e| {
                AppError::Internal(format!(
                    "HTTP no-OK al descargar certificados Identity Toolkit: {e}"
                ))
            })?;

        let ttl = resp
            .headers()
            .get(reqwest::header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_max_age)
            .map(Duration::from_secs)
            .unwrap_or(FALLBACK_TTL);

        let body = resp.bytes().await.map_err(|e| {
            AppError::Internal(format!(
                "error leyendo cuerpo de certificados Identity Toolkit: {e}"
            ))
        })?;
        let keys: HashMap<String, String> = serde_json::from_slice(&body).map_err(|e| {
            AppError::Internal(format!("certificados Identity Toolkit malformados: {e}"))
        })?;

        let mut cache = self.legacy_cache.write().await;
        cache.keys = Some(keys);
        cache.expires_at = Instant::now() + ttl;
        tracing::info!(
            ttl_secs = ttl.as_secs(),
            "Certificados Identity Toolkit refrescados"
        );
        Ok(())
    }
}

/// Parsea el valor `max-age=N` de un header `Cache-Control`.
///
/// Acepta formatos como:
///   - `public, max-age=21600`
///   - `max-age=300, must-revalidate`
///   - `max-age=0` (devuelve 0)
///
/// Devuelve `None` si no encuentra `max-age=` o si el numero no es valido.
fn parse_max_age(header: &str) -> Option<u64> {
    header
        .split(',')
        .map(str::trim)
        .find_map(|p| p.strip_prefix("max-age=").and_then(|n| n.parse().ok()))
}

#[cfg(test)]
mod tests {
    use super::{SignInProvider, parse_max_age};

    #[test]
    fn parsea_max_age_simple() {
        assert_eq!(parse_max_age("max-age=300"), Some(300));
    }

    #[test]
    fn parsea_max_age_con_directivas() {
        assert_eq!(parse_max_age("public, max-age=21600"), Some(21600));
        assert_eq!(parse_max_age("max-age=60, must-revalidate"), Some(60));
    }

    #[test]
    fn devuelve_none_si_no_hay_max_age() {
        assert_eq!(parse_max_age("public, no-cache"), None);
    }

    #[test]
    fn devuelve_none_si_max_age_no_es_numero() {
        assert_eq!(parse_max_age("max-age=abc"), None);
    }

    // ─── SignInProvider ───────────────────────────────────────────────────

    #[test]
    fn provider_google_com_se_parsea() {
        assert_eq!(
            SignInProvider::try_from("google.com"),
            Ok(SignInProvider::GoogleCom)
        );
    }

    #[test]
    fn provider_password_se_parsea() {
        assert_eq!(
            SignInProvider::try_from("password"),
            Ok(SignInProvider::Password)
        );
    }

    #[test]
    fn provider_anonymous_se_parsea() {
        assert_eq!(
            SignInProvider::try_from("anonymous"),
            Ok(SignInProvider::Anonymous)
        );
    }

    #[test]
    fn provider_apple_com_se_parsea() {
        assert_eq!(
            SignInProvider::try_from("apple.com"),
            Ok(SignInProvider::AppleCom)
        );
    }

    #[test]
    fn provider_desconocido_devuelve_literal_en_err() {
        assert_eq!(SignInProvider::try_from("phone"), Err("phone".to_string()));
        assert_eq!(SignInProvider::try_from(""), Err(String::new()));
    }

    #[test]
    fn provider_as_str_es_consistente_con_try_from() {
        for p in [
            SignInProvider::GoogleCom,
            SignInProvider::AppleCom,
            SignInProvider::Password,
            SignInProvider::Anonymous,
        ] {
            assert_eq!(SignInProvider::try_from(p.as_str()), Ok(p));
        }
    }
}
