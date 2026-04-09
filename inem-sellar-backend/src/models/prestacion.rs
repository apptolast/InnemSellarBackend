use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Prestaciones SEPE nacionales.
/// El campo `requisitos` es TEXT[] en PostgreSQL.
/// SeaORM lo soporta con la feature `postgres-array`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "prestaciones")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    /// TEXT[] en PostgreSQL → Vec<String> en Rust (via postgres-array feature)
    pub requisitos: Option<Vec<String>>,
    pub url: Option<String>,
    pub activo: Option<bool>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
