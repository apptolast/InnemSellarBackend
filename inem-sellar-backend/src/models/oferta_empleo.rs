// src/models/oferta_empleo.rs
//
// Tabla: ofertas_empleo
// CONCEPTO NUEVO: usar enums propios (EstadoModeracion) como tipo de campo

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Importamos el enum del modulo hermano `enums`.
// `super` significa "el modulo padre" (models/mod.rs).
// Desde ahi accedemos a `enums::EstadoModeracion`.
// Gracias al `pub use enums::*` en mod.rs, tambien podriamos
// escribir `use super::EstadoModeracion` directamente.
use super::enums::EstadoModeracion;

/// Oferta de empleo publicada por un usuario.
///
/// # Contadores de votos
/// `cantidad_upvotes` y `cantidad_downvotes` se actualizan automaticamente
/// por el trigger `trg_votos_contadores` en PostgreSQL cuando alguien vota.
/// El backend no necesita calcularlos — lee los valores directamente.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OfertaEmpleo {
    pub id: Uuid,

    /// FK a usuarios — NOT NULL (toda oferta tiene autor)
    pub id_autor: Uuid,

    pub titulo_puesto: Option<String>,
    pub empresa: Option<String>,
    pub ubicacion: Option<String>,
    pub descripcion: Option<String>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub web_contacto: Option<String>,

    /// Oferta visible o archivada
    pub activo: Option<bool>,

    /// Fecha de caducidad — las ofertas pueden expirar automaticamente
    pub caduca_en: Option<DateTime<Utc>>,

    /// Contadores actualizados por trigger de PostgreSQL
    pub cantidad_upvotes: Option<i32>,
    pub cantidad_downvotes: Option<i32>,
    pub cantidad_reportes: Option<i32>,

    /// Tipo enum de PostgreSQL → enum de Rust (EstadoModeracion).
    /// SQLx sabe convertirlo gracias al derive(sqlx::Type) en EstadoModeracion.
    /// Es Option porque la columna es nullable en el schema (tiene DEFAULT
    /// pero no NOT NULL).
    pub estado_moderacion: Option<EstadoModeracion>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
