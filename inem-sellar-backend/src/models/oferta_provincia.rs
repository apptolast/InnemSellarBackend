use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// Tabla de union N:M — PK compuesta (id_oferta + id_provincia)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "ofertas_provincias")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_oferta: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_provincia: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::oferta_empleo::Entity",
        from = "Column::IdOferta",
        to = "super::oferta_empleo::Column::Id"
    )]
    OfertaEmpleo,
    #[sea_orm(
        belongs_to = "super::provincia::Entity",
        from = "Column::IdProvincia",
        to = "super::provincia::Column::Id"
    )]
    Provincia,
}

impl Related<super::oferta_empleo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OfertaEmpleo.def()
    }
}

impl Related<super::provincia::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provincia.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
