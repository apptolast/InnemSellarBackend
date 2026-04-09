// src/models/usuario.rs
//
// Tabla: usuarios
// CONCEPTO NUEVO: #[serde(skip_serializing)] para campos sensibles
// CONCEPTO NUEVO: Uuid como tipo de PK (en vez de i32/SERIAL)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Representa un usuario registrado en InemSellar.
///
/// # Seguridad: `#[serde(skip_serializing)]`
/// El campo `hash_contrasena` lleva este atributo para que NUNCA
/// se incluya en respuestas JSON al cliente. Aunque el campo sea `pub`
/// (accesible en Rust), serde lo ignora al serializar. Es una capa
/// de seguridad extra: incluso si un handler serializa el struct entero
/// por error, el hash no se envia.
///
/// # Por que `Uuid` en vez de `i32`
/// Las tablas de usuario usan UUID como PK (gen_random_uuid() en PostgreSQL).
/// UUIDs son mejores que IDs secuenciales para APIs publicas porque:
/// - No revelan cuantos usuarios hay (privacidad)
/// - No son predecibles (seguridad)
/// - Permiten generar IDs sin consultar la BD (distribucion)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Usuario {
    /// UUID PRIMARY KEY — generado por PostgreSQL con gen_random_uuid()
    pub id: Uuid,

    /// Email unico del usuario (UNIQUE pero nullable)
    pub email: Option<String>,

    /// Hash bcrypt de la contrasena.
    /// `skip_serializing` = nunca aparece en JSON de respuesta.
    /// Si ademas no quieres aceptarlo desde el cliente (ej: el hash
    /// lo genera el backend, no el frontend), puedes anadir tambien
    /// `skip_deserializing`.
    #[serde(skip_serializing)]
    pub hash_contrasena: Option<String>,

    /// Nombre que se muestra en la app
    pub nombre_visible: Option<String>,

    /// URL del avatar (imagen de perfil)
    pub url_avatar: Option<String>,

    /// Enlace al perfil de LinkedIn
    pub url_linkedin: Option<String>,

    /// Enlace al CV (PDF en storage)
    pub url_curriculum: Option<String>,

    /// Cuenta activa o deshabilitada
    pub activo: Option<bool>,

    /// FK a provincias — nullable (el usuario puede no haber elegido provincia)
    /// Es Option<i32> porque la FK NO tiene NOT NULL en el schema.
    pub id_provincia: Option<i32>,

    /// Ultimo momento en que el usuario inicio sesion
    pub ultimo_login: Option<DateTime<Utc>>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
