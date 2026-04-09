// src/models/token_refresco.rs
//
// Tabla: tokens_refresco
// NOTA: esta tabla NO tiene campo actualizado_en (a diferencia de las demas)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Sesion JWT — almacena el hash del refresh token.
///
/// # Flujo de autenticacion JWT
/// 1. Login: el backend genera un access token (corta vida, ~15min) y un
///    refresh token (larga vida, ~30 dias)
/// 2. El refresh token se hashea y se guarda en esta tabla
/// 3. Cuando el access token expira, el frontend envia el refresh token
/// 4. El backend verifica el hash contra esta tabla y emite un nuevo access token
/// 5. Logout: se marca `revocado = true`
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TokenRefresco {
    /// UUID PK
    pub id: Uuid,

    /// FK a usuarios — NOT NULL
    pub id_usuario: Uuid,

    /// Hash del refresh token — NUNCA enviar al cliente.
    /// El cliente tiene el token en claro; la BD solo guarda el hash.
    #[serde(skip_serializing)]
    pub hash_token: Option<String>,

    /// Info del dispositivo que inicio sesion (ej: "iPhone 15, iOS 18")
    pub informacion_dispositivo: Option<String>,

    /// Cuando expira este refresh token
    pub expira_en: Option<DateTime<Utc>>,

    /// true = el usuario hizo logout o se invalido la sesion
    pub revocado: Option<bool>,

    /// Solo creado_en — esta tabla NO tiene actualizado_en en el schema
    pub creado_en: Option<DateTime<Utc>>,
}
