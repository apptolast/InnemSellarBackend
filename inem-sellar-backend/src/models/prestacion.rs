// src/models/prestacion.rs
//
// Tabla: prestaciones (RAI, SED, etc. — datos nacionales del SEPE)
// CONCEPTO NUEVO: TEXT[] (array de PostgreSQL) → Vec<String>

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Prestacion SEPE nacional (RAI, SED, subsidio, etc.).
///
/// # Por que `Vec<String>` para TEXT[]
/// PostgreSQL soporta columnas de tipo array (TEXT[]). En Rust,
/// el equivalente es `Vec<String>` — un vector (lista dinamica) de strings.
/// SQLx sabe convertir entre TEXT[] y Vec<String> automaticamente
/// gracias al feature "postgres" que ya tenemos.
///
/// En Dart seria `List<String>`. La diferencia es que Vec en Rust
/// es dueno de sus datos (no es una referencia) y crece/encoge
/// en el heap igual que List en Dart.
///
/// Es `Option<Vec<String>>` porque la columna es nullable.
/// Si tiene valor, sera `Some(vec!["Tener 45 o mas anos", "Estar inscrito..."])`.
/// Si es NULL en la BD, sera `None`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Prestacion {
    /// SERIAL PK
    pub id: i32,

    pub titulo: Option<String>,
    pub descripcion: Option<String>,

    /// TEXT[] en PostgreSQL → Option<Vec<String>> en Rust
    /// Lista de requisitos para acceder a la prestacion
    pub requisitos: Option<Vec<String>>,

    /// Enlace a mas informacion sobre la prestacion
    pub url: Option<String>,

    pub activo: Option<bool>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
