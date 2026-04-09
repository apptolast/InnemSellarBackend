// Modelo preparado para OAuth (Google, Apple) — fase 2.
// Se permite dead_code porque la entidad existe en la BD
// y sera usada cuando se implemente autenticacion con proveedores externos.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "proveedores_autenticacion")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub id_usuario: Uuid,
    pub proveedor: Option<String>,
    pub identificador_proveedor: Option<String>,
    pub email_proveedor: Option<String>,
    /// JSONB en PostgreSQL → serde_json::Value en Rust
    #[sea_orm(column_type = "JsonBinary")]
    pub datos_proveedor: Option<serde_json::Value>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::IdUsuario",
        to = "super::usuario::Column::Id"
    )]
    Usuario,
}

impl Related<super::usuario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Usuario.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
