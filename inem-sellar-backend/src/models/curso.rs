use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

use super::enums::{EstadoModeracion, OrigenContenido};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cursos")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// FK nullable — ON DELETE SET NULL
    pub id_autor: Option<Uuid>,
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub contenido: Option<String>,
    pub web: Option<String>,
    pub imagen_url: Option<String>,
    pub duracion_horas: Option<i32>,
    /// DATE en PostgreSQL → Date en SeaORM (chrono::NaiveDate internamente)
    pub fecha_inicio: Option<Date>,
    pub fecha_fin: Option<Date>,
    pub curso_homologado: Option<bool>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub origen: Option<OrigenContenido>,
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
