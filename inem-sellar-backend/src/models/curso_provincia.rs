use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cursos_provincias")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_curso: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_provincia: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::curso::Entity",
        from = "Column::IdCurso",
        to = "super::curso::Column::Id"
    )]
    Curso,
    #[sea_orm(
        belongs_to = "super::provincia::Entity",
        from = "Column::IdProvincia",
        to = "super::provincia::Column::Id"
    )]
    Provincia,
}

impl Related<super::curso::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Curso.def()
    }
}

impl Related<super::provincia::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provincia.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
