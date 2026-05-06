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

/// TTL del cache de JWKS si la respuesta no incluye `Cache-Control: max-age=...`.
/// 1 hora es conservador: las claves de Google rotan cada varias horas.
const FALLBACK_TTL: Duration = Duration::from_secs(60 * 60);

/// Tolerancia de reloj para `exp`, `iat`, `auth_time`. 60 segundos es el
/// estandar de la industria: corrige drift NTP sin abrir ventana de replay.
const CLOCK_SKEW_SECS: u64 = 60;

/// Timeout duro para la descarga del JWKS. Si Google tarda mas, fallamos
/// rapido en vez de bloquear handlers eternamente.
const JWKS_FETCH_TIMEOUT: Duration = Duration::from_secs(10);

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
    /// Issuer — debe ser `https://securetoken.google.com/<project_id>`.
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
    /// `true` si el token corresponde a un usuario anonimo de Firebase.
    pub fn is_anonymous(&self) -> bool {
        self.firebase.sign_in_provider == "anonymous"
    }

    /// `true` si el token corresponde a un login con Google.
    pub fn is_google(&self) -> bool {
        self.firebase.sign_in_provider == "google.com"
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
    /// URL del JWKS. Configurable para tests con `new_with_url`.
    jwks_url: Arc<String>,
    /// Cliente HTTP reutilizable. `reqwest::Client` ya envuelve un Arc internamente.
    http: reqwest::Client,
    /// Cache de claves publicas con TTL.
    cache: Arc<RwLock<JwksCache>>,
    /// Mutex para serializar refrescos del cache (evita thundering herd).
    refresh_lock: Arc<Mutex<()>>,
}

impl FirebaseVerifier {
    /// Crea un verificador apuntando al JWKS oficial de Google.
    ///
    /// # Panics
    /// Si reqwest no puede construir su cliente (extremadamente raro, suele
    /// indicar un sistema sin TLS roots). Aceptable porque ocurre solo al
    /// arrancar el servidor — fallo rapido es preferible a degradacion.
    pub fn new(project_id: String) -> Self {
        Self::new_with_url(project_id, JWKS_URL_DEFAULT.to_string())
    }

    /// Constructor parametrizable. Usado en tests con `wiremock` para
    /// servir un JWKS mockeado en lugar del de Google.
    pub fn new_with_url(project_id: String, jwks_url: String) -> Self {
        let issuer = format!("https://securetoken.google.com/{project_id}");
        let http = reqwest::Client::builder()
            .timeout(JWKS_FETCH_TIMEOUT)
            .build()
            .expect("reqwest::Client::builder() fallo al construir cliente HTTP");

        Self {
            project_id: Arc::new(project_id),
            issuer: Arc::new(issuer),
            jwks_url: Arc::new(jwks_url),
            http,
            cache: Arc::new(RwLock::new(JwksCache {
                set: None,
                // `Instant::now()` ya esta caducado: el primer `verify` forzara
                // un fetch del JWKS. Es el comportamiento deseado.
                expires_at: Instant::now(),
            })),
            refresh_lock: Arc::new(Mutex::new(())),
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

        // 2. Buscar la JWK en cache. Si no esta y el cache esta poblado,
        //    forzar UN unico refresh y reintentar (las claves rotaron).
        let jwk = match self.find_jwk(&kid).await? {
            Some(j) => j,
            None => {
                tracing::info!(kid = %kid, "kid no en cache JWKS, forzando refresh");
                self.refresh_jwks().await?;
                self.find_jwk(&kid).await?.ok_or_else(|| {
                    tracing::warn!(kid = %kid, "kid no encontrado en JWKS de Google tras refresh");
                    AppError::Unauthorized
                })?
            }
        };

        // 3. Construir DecodingKey desde la JWK.
        //    Por seguridad solo aceptamos RSA — Firebase usa RS256, nada mas.
        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .map_err(|e| AppError::Internal(format!("JWK RSA malformada: {e}")))?,
            other => {
                tracing::warn!(?other, "Tipo de JWK no soportado, esperaba RSA");
                return Err(AppError::Unauthorized);
            }
        };

        // 4. Configurar la validacion estricta:
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

        // 5. Decodificar y validar firma + claims.
        let data = decode::<FirebaseClaims>(id_token, &decoding_key, &validation).map_err(|e| {
            tracing::warn!(error = %e, "Validacion Firebase ID Token fallo");
            AppError::Unauthorized
        })?;
        let claims = data.claims;

        // 6. Comprobaciones manuales que `jsonwebtoken` no hace:
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

        Ok(claims)
    }

    /// Busca una `Jwk` por `kid` en el cache, refrescando si esta caducado.
    async fn find_jwk(&self, kid: &str) -> Result<Option<Jwk>, AppError> {
        if self.is_expired().await {
            self.refresh_jwks().await?;
        }
        let cache = self.cache.read().await;
        Ok(cache.set.as_ref().and_then(|set| set.find(kid).cloned()))
    }

    /// `true` si el cache esta vacio o ha caducado.
    async fn is_expired(&self) -> bool {
        let cache = self.cache.read().await;
        cache.set.is_none() || Instant::now() >= cache.expires_at
    }

    /// Refresca el cache de JWKS desde Google.
    ///
    /// # Double-checked locking
    /// Si llegan 50 requests simultaneas con el cache caducado, sin esta
    /// proteccion las 50 dispararian fetches paralelos al JWKS (thundering
    /// herd). Con el `refresh_lock`, solo la primera entra; las demas esperan
    /// el guard, y al entrar comprueban de nuevo `is_expired()`: si el primero
    /// ya refresco, no vuelven a fetchear.
    async fn refresh_jwks(&self) -> Result<(), AppError> {
        let _guard = self.refresh_lock.lock().await;

        // Double-check: el cache puede haberse refrescado mientras esperabamos.
        if !self.is_expired().await {
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
    use super::parse_max_age;

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
}
