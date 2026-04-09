// src/models/curso_provincia.rs
//
// Tabla: cursos_provincias (relacion N:M)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Relacion N:M entre cursos y provincias.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CursoProvincia {
    /// FK a cursos — NOT NULL
    pub id_curso: Uuid,

    /// FK a provincias — NOT NULL
    pub id_provincia: i32,
}
