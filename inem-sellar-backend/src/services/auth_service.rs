// src/services/auth_service.rs
//
// Logica de negocio de autenticacion: JWT propios + refresh tokens.
// La verificacion de Firebase ID Tokens vive en `firebase_verifier.rs`.
// NO accede a la BD — eso lo hace el repositorio.
// NO maneja HTTP — eso lo hace el handler.

use std::collections::HashSet;

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::AppError;

/// Claims del JWT — la informacion que va DENTRO del token.
///
/// # Por que `sub` y no `id_usuario`
/// `sub` (subject) es el claim estandar de JWT (RFC 7519) para identificar
/// al sujeto del token. Usar nombres estandar facilita la interoperabilidad
/// con librerias y servicios externos.
///
/// # Por que `anonimo: bool` con `#[serde(default)]`
/// Distingue entre usuarios "completos" (Firebase google.com / password)
/// y usuarios "anonimos" (Firebase Anonymous). El cliente lo usa para
/// decidir si mostrar el flujo "completar registro".
///
/// `#[serde(default)]` garantiza retrocompatibilidad: los tokens emitidos
/// ANTES de anadir este campo no llevan `anonimo` en el JSON; al
/// deserializar usaran `false` automaticamente.
///
/// `admin` sigue la misma regla: tokens antiguos no llevan el campo y se
/// tratan como no-admin. Esto evita romper sesiones moviles existentes.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// UUID del usuario como string
    pub sub: String,
    /// Timestamp de expiracion (segundos desde Unix epoch)
    pub exp: usize,
    /// Timestamp de emision
    pub iat: usize,
    /// `true` si el token corresponde a un usuario anonimo.
    /// El default `false` se aplica si el campo no esta presente (tokens
    /// emitidos antes de la integracion con Firebase).
    #[serde(default)]
    pub anonimo: bool,
    /// `true` si el token corresponde a un email verificado incluido en
    /// `ADMIN_EMAIL_ALLOWLIST`.
    #[serde(default)]
    pub admin: bool,
}

/// Servicio de autenticacion — encapsula toda la criptografia y logica de tokens.
///
/// # Por que un struct y no funciones sueltas
/// El servicio necesita configuracion (jwt_secret, expiracion). En vez de pasar
/// estos valores como parametros en cada llamada, los almacenamos en el struct.
/// Es el patron "Service Object" — equivalente a un servicio inyectado con
/// Provider/GetIt en Flutter.
///
/// # Por que `Clone`
/// Salvo necesita que los tipos inyectados en Depot sean `Clone`.
/// `String` es Clone, `u64` es Copy (aun mas barato que Clone).
#[derive(Clone)]
pub struct AuthService {
    jwt_secret: String,
    jwt_expiracion_minutos: u64,
    admin_email_allowlist: HashSet<String>,
}

impl AuthService {
    /// Crea un nuevo servicio con la configuracion JWT dada.
    pub fn new(jwt_secret: String, jwt_expiracion_minutos: u64) -> Self {
        Self::new_with_admin_email_allowlist(jwt_secret, jwt_expiracion_minutos, "")
    }

    /// Crea un servicio configurando tambien la allowlist admin.
    ///
    /// `admin_email_allowlist` usa el formato de entorno:
    /// `admin1@example.com,admin2@example.com`. La comparacion es
    /// case-insensitive y se ignoran espacios/entradas vacias.
    pub fn new_with_admin_email_allowlist(
        jwt_secret: String,
        jwt_expiracion_minutos: u64,
        admin_email_allowlist: impl AsRef<str>,
    ) -> Self {
        Self {
            jwt_secret,
            jwt_expiracion_minutos,
            admin_email_allowlist: parse_admin_email_allowlist(admin_email_allowlist.as_ref()),
        }
    }

    /// Devuelve `true` si el email pertenece a la allowlist admin.
    pub fn email_en_admin_allowlist(&self, email: &str) -> bool {
        self.admin_email_allowlist
            .contains(&normalizar_email(email))
    }

    /// Devuelve `true` solo si el email esta verificado y en allowlist.
    pub fn es_email_admin_verificado(
        &self,
        email: Option<&str>,
        email_verificado: Option<bool>,
    ) -> bool {
        email_verificado == Some(true)
            && email
                .map(|email| self.email_en_admin_allowlist(email))
                .unwrap_or(false)
    }

    /// Genera un access token JWT firmado con HS256 marcando si el usuario
    /// es anonimo.
    ///
    /// Usado por:
    ///   - `/auth/firebase` con el flag derivado del `sign_in_provider`
    ///     (`true` solo para Firebase Anonymous).
    ///   - `/auth/refrescar` preservando el flag del usuario consultando
    ///     `proveedor_repo.es_anonimo()`.
    ///
    /// # Por que `anonimo` va en el JWT y no se consulta la BD en cada request
    /// El JWT se valida sin tocar BD (criptografia pura). Si tuvieramos que
    /// consultar la BD para saber si el usuario es anonimo, anadiriamos una
    /// query a CADA request protegida — innecesario porque el flag no cambia
    /// hasta que el usuario "se complete" (futuro endpoint `/auth/upgrade`),
    /// momento en el que se emite un nuevo JWT con `anonimo=false`.
    pub fn generar_access_token_con_flag(
        &self,
        id_usuario: Uuid,
        anonimo: bool,
    ) -> Result<String, AppError> {
        self.generar_access_token_con_flags(id_usuario, anonimo, false)
    }

    /// Genera un access token JWT firmado con HS256 marcando los flags
    /// actuales del usuario.
    pub fn generar_access_token_con_flags(
        &self,
        id_usuario: Uuid,
        anonimo: bool,
        admin: bool,
    ) -> Result<String, AppError> {
        let ahora = Utc::now();
        let expiracion = ahora + Duration::minutes(self.jwt_expiracion_minutos as i64);

        let claims = Claims {
            sub: id_usuario.to_string(),
            exp: expiracion.timestamp() as usize,
            iat: ahora.timestamp() as usize,
            anonimo,
            admin,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Error generando JWT: {e}")))
    }

    /// Verifica y decodifica un access token JWT.
    /// Devuelve los Claims si el token es valido y no ha expirado.
    pub fn verificar_access_token(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized)?;

        Ok(token_data.claims)
    }

    /// Genera un refresh token aleatorio (32 bytes, codificado en hex = 64 chars).
    ///
    /// # Por que hex y no base64
    /// Hex es mas simple, no tiene caracteres especiales (+, /, =) que
    /// podrian causar problemas en URLs o JSON. 64 chars hex = 256 bits
    /// de entropia, mas que suficiente.
    pub fn generar_refresh_token(&self) -> String {
        let bytes: [u8; 32] = rand::thread_rng().r#gen();
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Hashea un refresh token con SHA-256 para almacenarlo en la BD.
    ///
    /// # Por que SHA-256 y no Argon2 para refresh tokens
    /// Los refresh tokens ya tienen alta entropia (256 bits aleatorios).
    /// No necesitan un hash lento (Argon2) porque un atacante no puede
    /// hacer ataques de diccionario contra bytes aleatorios. SHA-256 es
    /// suficiente y mucho mas rapido.
    pub fn hashear_refresh_token(&self, token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

fn parse_admin_email_allowlist(raw: &str) -> HashSet<String> {
    raw.split(',')
        .map(normalizar_email)
        .filter(|email| !email.is_empty())
        .collect()
}

fn normalizar_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_allowlist_ignora_espacios_case_y_vacios() {
        let service = AuthService::new_with_admin_email_allowlist(
            "secret".into(),
            15,
            " Admin@Example.com, ,owner@example.com ",
        );

        assert!(service.email_en_admin_allowlist("admin@example.com"));
        assert!(service.email_en_admin_allowlist("OWNER@EXAMPLE.COM"));
        assert!(!service.email_en_admin_allowlist("user@example.com"));
    }

    #[test]
    fn admin_requiere_email_verificado() {
        let service =
            AuthService::new_with_admin_email_allowlist("secret".into(), 15, "admin@example.com");

        assert!(service.es_email_admin_verificado(Some("admin@example.com"), Some(true)));
        assert!(!service.es_email_admin_verificado(Some("admin@example.com"), Some(false)));
        assert!(!service.es_email_admin_verificado(Some("admin@example.com"), None));
        assert!(!service.es_email_admin_verificado(None, Some(true)));
    }
}
