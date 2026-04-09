use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::EstadoModeracion;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "consejos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub id_autor: Uuid,
    pub titulo: Option<String>,
    pub cuerpo: Option<String>,
    pub web: Option<String>,
    pub imagen_url: Option<String>,
    pub activo: Option<bool>,
    pub cantidad_upvotes: Option<i32>,
    pub cantidad_downvotes: Option<i32>,
    pub cantidad_reportes: Option<i32>,
    pub estado_moderacion: Option<EstadoModeracion>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::IdAutor",
        to = "super::usuario::Column::Id"
    )]
    Autor,
}

impl Related<super::usuario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Autor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
