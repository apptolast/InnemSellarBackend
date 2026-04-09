use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "oficinas_sepe")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub id_provincia: i32,
    pub telefono: Option<String>,
    pub web: Option<String>,
    pub url_cursos: Option<String>,
    pub url_orientacion: Option<String>,
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
