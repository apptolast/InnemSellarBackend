// src/models/comunidad_autonoma.rs
//
// Tabla: comunidades_autonomas (19 registros: 17 CCAA + Ceuta + Melilla)
//
// PRIMER MODELO DEL PROYECTO — aqui se explican los conceptos base
// que se repiten en todos los demas modelos.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Representa una Comunidad Autonoma de Espana.
///
/// # Por que `#[derive(FromRow)]`
/// `FromRow` viene de SQLx. Cuando haces una query como:
/// ```ignore
/// sqlx::query_as::<_, ComunidadAutonoma>("SELECT * FROM comunidades_autonomas")
/// ```
/// SQLx toma cada fila del resultado y la convierte en un `ComunidadAutonoma`.
/// Para ello, busca columnas con el MISMO NOMBRE que los campos del struct.
/// Si un nombre no coincide, obtienes error en COMPILACION, no en runtime.
///
/// # Por que `Serialize` + `Deserialize`
/// - `Serialize`: convierte el struct a JSON para enviarlo al frontend Flutter.
/// - `Deserialize`: convierte JSON del cliente en este struct.
///   Para una tabla de referencia como esta, `Deserialize` se usaria en un
///   endpoint de admin que permita editar datos de una comunidad.
///
/// # Por que `Debug` y `Clone`
/// - `Debug`: permite imprimir el struct en logs con `{:?}`.
/// - `Clone`: permite duplicar el struct. Rust usa ownership — sin `Clone`,
///   al pasar el struct a una funcion, pierdes acceso al original.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComunidadAutonoma {
    /// SERIAL PRIMARY KEY → i32 (siempre tiene valor, nunca Option)
    pub id: i32,

    /// TEXT UNIQUE — nullable en schema (no tiene NOT NULL)
    pub nombre: Option<String>,

    /// Nombre del servicio de empleo autonomico (ej: "SAE", "SOC", "Lanbide")
    pub nombre_servicio_empleo: Option<String>,

    /// Web del servicio de empleo autonomico
    pub web_servicio_empleo: Option<String>,

    /// Enlace para sellado/renovacion de demanda de empleo
    pub url_sellado: Option<String>,

    /// TIMESTAMPTZ DEFAULT now() → Option<DateTime<Utc>>
    /// Tiene DEFAULT en PostgreSQL, pero sigue siendo nullable.
    /// DateTime<Utc> es un timestamp con zona horaria UTC.
    pub creado_en: Option<DateTime<Utc>>,

    /// Se actualiza automaticamente via trigger trg_comunidades_actualizado
    pub actualizado_en: Option<DateTime<Utc>>,
}
