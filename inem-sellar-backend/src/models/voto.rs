use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::TipoContenido;

/// Tabla polimorfica con PK compuesta triple.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "votos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_usuario: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tipo_contenido: TipoContenido,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_contenido: Uuid,
    /// 1 = upvote, -1 = downvote, 0 = sin voto
    pub tipo_voto: Option<i32>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
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
