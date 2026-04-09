use serde::{Deserialize, Serialize};


// OrigenContenido
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "origen_contenido", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrigenContenido {
    Oficial,
    Comunidad,
}

// EstadoModeración
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "estado_moderacion", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EstadoModeracion {
    Pendiente,
    Aprobado,
    Rechazado,
    EnRevision,
}

// TipoContenido
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tipo_contenido", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TipoContenido {
    Oferta,
    Consejo,
    Curso,
}

// MotivoReporte
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "motivo_reporte", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MotivoReporte {
    Spam,
    Inapropiado,
    Desactualizado,
    Incorrecto,
    Duplicado,
    Otro,
}

// EstadoReporte
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "estado_reporte", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EstadoReporte {
    Pendiente,
    Aceptado,
    Rechazado,
}