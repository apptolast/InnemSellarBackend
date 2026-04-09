// src/models/oferta_provincia.rs
//
// Tabla: ofertas_provincias (relacion N:M entre ofertas_empleo y provincias)
// CONCEPTO NUEVO: tabla de union sin id propio (PK compuesta)
//
// Este tipo de tabla se llama "tabla de union" o "tabla pivote".
// No tiene campos propios — solo conecta dos entidades.
// Una oferta puede estar en multiples provincias, y una provincia
// puede tener multiples ofertas.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Relacion N:M entre ofertas de empleo y provincias.
///
/// No tiene `id` propio — la PK es la combinacion (id_oferta, id_provincia).
/// No tiene timestamps porque la relacion no se "actualiza", solo se crea o elimina.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfertaProvincia {
    /// FK a ofertas_empleo — NOT NULL
    pub id_oferta: Uuid,

    /// FK a provincias — NOT NULL
    pub id_provincia: i32,
}
