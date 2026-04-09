// src/models/enums.rs
//
// Enums de PostgreSQL adaptados a SeaORM.
//
// CAMBIO vs version anterior:
//   Antes: #[derive(sqlx::Type)] + #[sqlx(type_name = "...")]
//   Ahora: #[derive(DeriveActiveEnum)] + #[sea_orm(rs_type, db_type, enum_name)]
//
// # Por que `DeriveActiveEnum`
// Es la macro de SeaORM que genera el codigo para convertir entre
// un ENUM de PostgreSQL y un enum de Rust. Es el equivalente de
// `sqlx::Type` pero integrado con el ecosistema del ORM.
//
// # Por que `rs_type = "String"` y `db_type = "Enum"`
// Le dice a SeaORM:
//   - En Rust, cada variante se representa como un String internamente
//   - En PostgreSQL, es un tipo ENUM (CREATE TYPE ... AS ENUM)
//   - `enum_name` es el nombre exacto del TYPE en PostgreSQL

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "origen_contenido")]
pub enum OrigenContenido {
    #[sea_orm(string_value = "oficial")]
    Oficial,
    #[sea_orm(string_value = "comunidad")]
    Comunidad,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "estado_moderacion")]
pub enum EstadoModeracion {
    #[sea_orm(string_value = "pendiente")]
    Pendiente,
    #[sea_orm(string_value = "aprobado")]
    Aprobado,
    #[sea_orm(string_value = "rechazado")]
    Rechazado,
    #[sea_orm(string_value = "en_revision")]
    EnRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tipo_contenido")]
pub enum TipoContenido {
    #[sea_orm(string_value = "oferta")]
    Oferta,
    #[sea_orm(string_value = "consejo")]
    Consejo,
    #[sea_orm(string_value = "curso")]
    Curso,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "motivo_reporte")]
pub enum MotivoReporte {
    #[sea_orm(string_value = "spam")]
    Spam,
    #[sea_orm(string_value = "inapropiado")]
    Inapropiado,
    #[sea_orm(string_value = "desactualizado")]
    Desactualizado,
    #[sea_orm(string_value = "incorrecto")]
    Incorrecto,
    #[sea_orm(string_value = "duplicado")]
    Duplicado,
    #[sea_orm(string_value = "otro")]
    Otro,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "estado_reporte")]
pub enum EstadoReporte {
    #[sea_orm(string_value = "pendiente")]
    Pendiente,
    #[sea_orm(string_value = "aceptado")]
    Aceptado,
    #[sea_orm(string_value = "rechazado")]
    Rechazado,
}
