use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// PK de tipo String (TEXT en PostgreSQL)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "configuracion_aplicacion")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub clave: String,
    pub valor: Option<String>,
    pub descripcion: Option<String>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
