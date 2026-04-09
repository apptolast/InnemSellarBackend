// src/models/curso.rs
//
// Tabla: cursos (oficial por admin + comunitario por usuarios)
// CONCEPTO NUEVO: NaiveDate para columnas DATE (sin zona horaria)
// CONCEPTO NUEVO: FK nullable (id_autor puede ser NULL por ON DELETE SET NULL)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::{EstadoModeracion, OrigenContenido};

/// Curso formativo — puede ser oficial (admin) o comunitario (usuario).
///
/// # Por que `id_autor` es `Option<Uuid>`
/// A diferencia de ofertas y consejos donde id_autor es NOT NULL,
/// aqui la FK es nullable: `REFERENCES usuarios(id) ON DELETE SET NULL`.
/// Si el usuario que creo el curso se elimina, PostgreSQL pone id_autor = NULL
/// en vez de borrar el curso. Esto permite mantener cursos oficiales
/// aunque el admin que los creo ya no exista.
///
/// # Por que `NaiveDate` (no `DateTime<Utc>`)
/// Las columnas `fecha_inicio` y `fecha_fin` son tipo DATE en PostgreSQL
/// (solo fecha, sin hora ni zona horaria). En Rust, `NaiveDate` de chrono
/// representa exactamente eso: una fecha sin timezone (ej: 2026-04-09).
/// `DateTime<Utc>` seria para TIMESTAMPTZ (fecha + hora + zona horaria).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Curso {
    pub id: Uuid,

    /// FK nullable — ON DELETE SET NULL (puede quedar huerfano)
    pub id_autor: Option<Uuid>,

    pub titulo: Option<String>,

    /// Descripcion breve del curso
    pub descripcion: Option<String>,

    /// Contenido detallado / temario
    pub contenido: Option<String>,

    /// Enlace al curso
    pub web: Option<String>,

    pub imagen_url: Option<String>,

    /// Numero de horas del curso
    pub duracion_horas: Option<i32>,

    /// DATE en PostgreSQL → NaiveDate en Rust (solo fecha, sin hora)
    pub fecha_inicio: Option<NaiveDate>,
    pub fecha_fin: Option<NaiveDate>,

    /// true = curso oficial homologado
    pub curso_homologado: Option<bool>,

    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,

    /// Enum: 'oficial' o 'comunidad'
    pub origen: Option<OrigenContenido>,

    pub activo: Option<bool>,

    pub cantidad_upvotes: Option<i32>,
    pub cantidad_downvotes: Option<i32>,
    pub cantidad_reportes: Option<i32>,

    pub estado_moderacion: Option<EstadoModeracion>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
