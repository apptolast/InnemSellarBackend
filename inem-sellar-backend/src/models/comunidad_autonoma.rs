// src/models/comunidad_autonoma.rs
//
// Entidad SeaORM para la tabla comunidades_autonomas.
//
// CAMBIO CLAVE vs version anterior:
//   Antes: struct ComunidadAutonoma con #[derive(FromRow)]
//   Ahora: struct Model con #[derive(DeriveEntityModel)]
//
// En SeaORM, cada entidad tiene:
//   - `Model` (struct con los datos — lo que antes era ComunidadAutonoma)
//   - `Entity` (tipo para hacer queries — generado automaticamente)
//   - `Column` (enum con los nombres de columna — generado automaticamente)
//   - `ActiveModel` (para inserts/updates — generado automaticamente)
//   - `Relation` (enum con relaciones a otras tablas — lo defines tu)
//
// Para hacer queries, usas Entity:
//   comunidad_autonoma::Entity::find().all(&db).await
// En vez de:
//   sqlx::query_as::<_, ComunidadAutonoma>("SELECT * FROM ...").fetch_all(&pool).await

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// # Por que `DeriveEntityModel`
/// Genera automaticamente Entity, Column, PrimaryKey y ActiveModel.
/// Es como si SeaORM escribiera ~200 lineas de codigo por ti.
/// `table_name` debe coincidir con el nombre de la tabla en PostgreSQL.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "comunidades_autonomas")]
pub struct Model {
    /// `primary_key` indica que esta columna es la PK.
    /// `auto_increment = true` porque es SERIAL en PostgreSQL.
    #[sea_orm(primary_key)]
    pub id: i32,
    pub nombre: Option<String>,
    pub nombre_servicio_empleo: Option<String>,
    pub web_servicio_empleo: Option<String>,
    pub url_sellado: Option<String>,
    pub creado_en: Option<DateTimeWithTimeZone>,
    pub actualizado_en: Option<DateTimeWithTimeZone>,
}

/// Relaciones con otras tablas.
/// ComunidadAutonoma tiene muchas Provincias (1:N).
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::provincia::Entity")]
    Provincias,
}

/// Implementa la relacion inversa para que SeaORM pueda hacer JOINs.
impl Related<super::provincia::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Provincias.def()
    }
}

/// ActiveModelBehavior permite hooks de ciclo de vida (before_save, etc.)
/// Lo dejamos vacio = comportamiento por defecto.
impl ActiveModelBehavior for ActiveModel {}
