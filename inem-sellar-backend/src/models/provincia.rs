use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "provincias")]
pub struct Model {
    /// INTEGER PK — codigo INE, NO auto_increment (se inserta manualmente)
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub nombre: Option<String>,
    pub id_comunidad: i32,
    pub logo_asset: Option<String>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /// Provincia pertenece a una ComunidadAutonoma (N:1)
    #[sea_orm(
        belongs_to = "super::comunidad_autonoma::Entity",
        from = "Column::IdComunidad",
        to = "super::comunidad_autonoma::Column::Id"
    )]
    ComunidadAutonoma,
    /// Provincia tiene una OficinaSepe (1:1)
    #[sea_orm(has_one = "super::oficina_sepe::Entity")]
    OficinaSepe,
}

impl Related<super::comunidad_autonoma::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ComunidadAutonoma.def()
    }
}

impl Related<super::oficina_sepe::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OficinaSepe.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
