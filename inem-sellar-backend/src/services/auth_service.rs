// src/services/auth_service.rs
//
// Logica de negocio de autenticacion: hash de contrasenas, JWT, refresh tokens.
// NO accede a la BD — eso lo hace el repositorio.
// NO maneja HTTP — eso lo hace el handler.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// UUID del usuario como string
    pub sub: String,
    /// Timestamp de expiracion (segundos desde Unix epoch)
    pub exp: usize,
    /// Timestamp de emision
    pub iat: usize,
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
}

impl AuthService {
    /// Crea un nuevo servicio con la configuracion JWT dada.
    pub fn new(jwt_secret: String, jwt_expiracion_minutos: u64) -> Self {
        Self {
            jwt_secret,
            jwt_expiracion_minutos,
        }
    }

    /// Hashea una contrasena con Argon2id.
    ///
    /// # Por que Argon2id y no bcrypt
    /// Argon2id es el ganador de la Password Hashing Competition (PHC).
    /// Es resistente tanto a ataques GPU como a ataques side-channel.
    /// bcrypt tiene un limite de 72 bytes en la contrasena; Argon2 no.
    ///
    /// # Por que generar un salt aleatorio cada vez
    /// El salt previene ataques de rainbow tables. Cada contrasena tiene
    /// su propio salt unico, incluido en el hash resultante. Al verificar,
    /// Argon2 extrae el salt del hash almacenado.
    pub fn hashear_contrasena(&self, contrasena: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(contrasena.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Error hasheando contrasena: {e}")))?;
        Ok(hash.to_string())
    }

    /// Verifica una contrasena contra un hash Argon2id almacenado.
    pub fn verificar_contrasena(&self, contrasena: &str, hash: &str) -> Result<bool, AppError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(format!("Hash invalido: {e}")))?;
        Ok(Argon2::default()
            .verify_password(contrasena.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Genera un access token JWT firmado con HS256.
    ///
    /// # Flujo
    /// 1. Crea los claims (sub=uuid, exp=ahora+15min, iat=ahora)
    /// 2. Firma con HS256 usando jwt_secret
    /// 3. Devuelve el token como string (header.payload.signature)
    pub fn generar_access_token(&self, id_usuario: Uuid) -> Result<String, AppError> {
        let ahora = Utc::now();
        let expiracion = ahora + Duration::minutes(self.jwt_expiracion_minutos as i64);

        let claims = Claims {
            sub: id_usuario.to_string(),
            exp: expiracion.timestamp() as usize,
            iat: ahora.timestamp() as usize,
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
