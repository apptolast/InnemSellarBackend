// src/models/provincia.rs
//
// Tabla: provincias (52 registros, codigos INE 1-52)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Representa una provincia espanola.
///
/// # Nota sobre `id`
/// A diferencia de otras tablas que usan SERIAL (auto-incremento),
/// aqui el `id` es INTEGER con valores fijos del INE (1-52).
/// Se insertan manualmente en el seeding, no los genera PostgreSQL.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Provincia {
    /// INTEGER PRIMARY KEY — codigo INE (1-52), NO auto-generado
    pub id: i32,

    /// Nombre de la provincia (ej: "Madrid", "Barcelona")
    pub nombre: Option<String>,

    /// FK a comunidades_autonomas — tiene NOT NULL en schema,
    /// por eso es i32 directo (no Option<i32>).
    /// Cuando un campo tiene NOT NULL, la BD garantiza que siempre
    /// tendra valor, asi que no necesitamos Option.
    pub id_comunidad: i32,

    /// Ruta al asset del logo provincial en el frontend Flutter
    pub logo_asset: Option<String>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
