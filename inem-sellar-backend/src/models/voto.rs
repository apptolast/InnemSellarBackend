// src/models/voto.rs
//
// Tabla: votos (polimorfica — un voto puede ser para oferta, consejo o curso)
// CONCEPTO NUEVO: PK compuesta triple (id_usuario + tipo_contenido + id_contenido)
// CONCEPTO NUEVO: tabla polimorfica (un campo indica a que otra tabla apunta)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::TipoContenido;

/// Voto de un usuario sobre un contenido (oferta, consejo o curso).
///
/// # Polimorfismo en base de datos
/// Esta tabla es "polimorfica": `tipo_contenido` indica a que tabla pertenece
/// `id_contenido`:
/// - `TipoContenido::Oferta`  → id_contenido apunta a ofertas_empleo
/// - `TipoContenido::Consejo` → id_contenido apunta a consejos
/// - `TipoContenido::Curso`   → id_contenido apunta a cursos
///
/// No hay FK directa en id_contenido porque PostgreSQL no soporta FK
/// condicionales a multiples tablas. La integridad se garantiza desde
/// el codigo Rust (validacion en el service/handler).
///
/// # Valores de tipo_voto
/// - `1`  = upvote
/// - `-1` = downvote
/// - `0`  = voto retirado (el usuario quito su voto)
///
/// # PK compuesta
/// No tiene campo `id`. La PK es (id_usuario, tipo_contenido, id_contenido).
/// Esto garantiza en la BD que un usuario solo puede tener un voto por contenido.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Voto {
    /// FK a usuarios — NOT NULL (parte de la PK)
    pub id_usuario: Uuid,

    /// Enum que indica a que tabla apunta id_contenido — NOT NULL (parte de la PK)
    pub tipo_contenido: TipoContenido,

    /// UUID del contenido votado — NOT NULL (parte de la PK)
    /// No tiene FK en PostgreSQL por ser polimorfico.
    pub id_contenido: Uuid,

    /// 1 = upvote, -1 = downvote, 0 = sin voto
    pub tipo_voto: Option<i32>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
