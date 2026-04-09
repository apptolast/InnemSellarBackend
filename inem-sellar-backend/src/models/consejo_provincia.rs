// src/models/consejo_provincia.rs
//
// Tabla: consejos_provincias (relacion N:M)
// Sin filas = el consejo es nacional (aplica a toda Espana)

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Relacion N:M entre consejos y provincias.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConsejoProvincia {
    /// FK a consejos — NOT NULL
    pub id_consejo: Uuid,

    /// FK a provincias — NOT NULL
    pub id_provincia: i32,
}
