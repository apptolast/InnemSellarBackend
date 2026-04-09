// src/models/reporte.rs
//
// Tabla: reportes (polimorfica — igual que votos)
// Usa tres enums: TipoContenido, MotivoReporte, EstadoReporte

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::enums::{EstadoReporte, MotivoReporte, TipoContenido};

/// Reporte de contenido inapropiado por un usuario.
///
/// # Flujo de moderacion
/// 1. Un usuario reporta un contenido → se crea un reporte con estado 'pendiente'
/// 2. Un moderador/admin revisa el reporte
/// 3. Lo marca como 'aceptado' (y toma accion) o 'rechazado'
/// 4. Se registra quien lo proceso (id_procesador) y cuando (procesado_en)
///
/// # Constraint UNIQUE
/// La combinacion (tipo_contenido, id_contenido, id_reportero) es UNIQUE.
/// Un usuario solo puede reportar un contenido una vez.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reporte {
    /// UUID PK — a diferencia de votos, reportes SI tiene id propio
    pub id: Uuid,

    /// Indica a que tabla apunta id_contenido (oferta/consejo/curso)
    pub tipo_contenido: Option<TipoContenido>,

    /// UUID del contenido reportado (sin FK por polimorfismo)
    pub id_contenido: Option<Uuid>,

    /// FK a usuarios — NOT NULL (quien reporta)
    pub id_reportero: Uuid,

    /// Enum: spam, inapropiado, desactualizado, incorrecto, duplicado, otro
    pub motivo: Option<MotivoReporte>,

    /// Texto libre con mas detalle sobre el reporte
    pub detalle_motivo: Option<String>,

    /// Enum: pendiente, aceptado, rechazado
    pub estado: Option<EstadoReporte>,

    /// FK a usuarios — el moderador que proceso el reporte (nullable)
    pub id_procesador: Option<Uuid>,

    /// Cuando se proceso el reporte
    pub procesado_en: Option<DateTime<Utc>>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
