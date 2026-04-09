use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "usuarios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: Option<String>,
    /// Hash de la contrasena — nunca enviar al cliente
    #[serde(skip_serializing)]
    pub hash_contrasena: Option<String>,
    pub nombre_visible: Option<String>,
    pub url_avatar: Option<String>,
    pub url_linkedin: Option<String>,
    pub url_curriculum: Option<String>,
    pub activo: Option<bool>,
    pub id_provincia: Option<i32>,
    pub ultimo_login: Option<DateTimeWithTimeZone>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::provincia::Entity",
        from = "Column::IdProvincia",
        to = "super::provincia::Column::Id"
    )]
    Provincia,
}

impl Related<super::provincia::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provincia.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
