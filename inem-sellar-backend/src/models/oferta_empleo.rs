use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::EstadoModeracion;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ofertas_empleo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub id_autor: Uuid,
    pub titulo_puesto: Option<String>,
    pub empresa: Option<String>,
    pub ubicacion: Option<String>,
    pub descripcion: Option<String>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub web_contacto: Option<String>,
    pub activo: Option<bool>,
    pub caduca_en: Option<DateTimeWithTimeZone>,
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
