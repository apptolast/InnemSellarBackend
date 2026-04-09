use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "tokens_refresco")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub id_usuario: Uuid,
    #[serde(skip_serializing)]
    pub hash_token: Option<String>,
    pub informacion_dispositivo: Option<String>,
    pub expira_en: Option<DateTimeWithTimeZone>,
    pub revocado: Option<bool>,
    /// Esta tabla NO tiene actualizado_en — solo creado_en
    pub creado_en: Option<DateTimeWithTimeZone>,
}

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
