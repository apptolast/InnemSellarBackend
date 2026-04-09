// src/models/consejo.rs
//
// Tabla: consejos (tips/consejos escritos por usuarios)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::EstadoModeracion;

/// Consejo o tip publicado por un usuario de la comunidad.
///
/// Si no tiene filas en `consejos_provincias`, el consejo es nacional
/// (aplica a toda Espana). Si tiene filas, aplica solo a esas provincias.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Consejo {
    pub id: Uuid,

    /// FK a usuarios — NOT NULL
    pub id_autor: Uuid,

    pub titulo: Option<String>,

    /// Texto completo del consejo
    pub cuerpo: Option<String>,

    /// Enlace adicional opcional
    pub web: Option<String>,

    /// URL de imagen ilustrativa
    pub imagen_url: Option<String>,

    pub activo: Option<bool>,

    /// Contadores actualizados por trigger
    pub cantidad_upvotes: Option<i32>,
    pub cantidad_downvotes: Option<i32>,
    pub cantidad_reportes: Option<i32>,

    pub estado_moderacion: Option<EstadoModeracion>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
