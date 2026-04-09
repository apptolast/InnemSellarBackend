use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::{EstadoReporte, MotivoReporte, TipoContenido};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "reportes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tipo_contenido: Option<TipoContenido>,
    pub id_contenido: Option<Uuid>,
    pub id_reportero: Uuid,
    pub motivo: Option<MotivoReporte>,
    pub detalle_motivo: Option<String>,
    pub estado: Option<EstadoReporte>,
    pub id_procesador: Option<Uuid>,
    pub procesado_en: Option<DateTimeWithTimeZone>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::usuario::Entity",
        from = "Column::IdReportero",
        to = "super::usuario::Column::Id"
    )]
    Reportero,
}

impl Related<super::usuario::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reportero.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
