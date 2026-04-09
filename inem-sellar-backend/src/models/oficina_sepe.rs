// src/models/oficina_sepe.rs
//
// Tabla: oficinas_sepe (52 registros, una por provincia)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Datos de contacto SEPE por provincia.
///
/// Relacion 1:1 con provincias (id_provincia es UNIQUE).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OficinaSepe {
    /// SERIAL PRIMARY KEY
    pub id: i32,

    /// FK a provincias — NOT NULL + UNIQUE (una oficina por provincia)
    pub id_provincia: i32,

    /// Telefono de la oficina SEPE provincial
    pub telefono: Option<String>,

    /// Web de la oficina SEPE provincial
    pub web: Option<String>,

    /// Enlace al catalogo de cursos de la provincia
    pub url_cursos: Option<String>,

    /// Enlace a orientacion laboral de la provincia
    pub url_orientacion: Option<String>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
