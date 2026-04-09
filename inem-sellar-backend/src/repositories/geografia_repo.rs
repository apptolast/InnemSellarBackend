// src/repositories/geografia_repo.rs
//
// Repositorio de datos geograficos con SeaORM.
//
// CAMBIO CLAVE vs version SQLx:
//   Antes: sqlx::query_as::<_, Struct>("SELECT * FROM tabla").fetch_all(&pool)
//   Ahora: tabla::Entity::find().all(&db)
//
// El SQL ya NO se escribe a mano. SeaORM genera las queries
// automaticamente a partir de las entidades.

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

use crate::errors::AppError;
use crate::models::{comunidad_autonoma, oficina_sepe, provincia};

/// Contrato (interfaz) para acceso a datos geograficos.
pub trait GeografiaRepo: Send + Sync {
    fn listar_comunidades(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<comunidad_autonoma::Model>, AppError>> + Send;

    fn obtener_comunidad(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<comunidad_autonoma::Model, AppError>> + Send;

    fn listar_provincias(
        &self,
        id_comunidad: Option<i32>,
    ) -> impl std::future::Future<Output = Result<Vec<provincia::Model>, AppError>> + Send;

    fn obtener_provincia(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<provincia::Model, AppError>> + Send;

    fn obtener_oficina_por_provincia(
        &self,
        id_provincia: i32,
    ) -> impl std::future::Future<Output = Result<oficina_sepe::Model, AppError>> + Send;
}

/// Implementacion con SeaORM + PostgreSQL.
#[derive(Clone)]
pub struct SeaGeografiaRepo {
    db: DatabaseConnection,
}

impl SeaGeografiaRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

/// # Por que Entity::find() en vez de SQL crudo
/// SeaORM genera el SQL por ti. Cuando escribes:
///   `comunidad_autonoma::Entity::find().all(&self.db).await`
/// SeaORM genera internamente:
///   `SELECT * FROM comunidades_autonomas`
/// Y mapea el resultado a `comunidad_autonoma::Model`.
///
/// Para filtrar:
///   `.filter(provincia::Column::IdComunidad.eq(5))`
/// Genera:
///   `WHERE id_comunidad = 5`
///
/// Para ordenar:
///   `.order_by_asc(comunidad_autonoma::Column::Id)`
/// Genera:
///   `ORDER BY id ASC`
///
/// # Por que `From<sea_orm::DbErr>` en AppError
/// SeaORM usa `DbErr` como tipo de error. Necesitamos convertirlo
/// a nuestro `AppError`. Lo hacemos con un impl From mas abajo.
impl GeografiaRepo for SeaGeografiaRepo {
    async fn listar_comunidades(&self) -> Result<Vec<comunidad_autonoma::Model>, AppError> {
        let comunidades = comunidad_autonoma::Entity::find()
            .order_by_asc(comunidad_autonoma::Column::Id)
            .all(&self.db)
            .await
            .map_err(AppError::from_db)?;

        Ok(comunidades)
    }

    async fn obtener_comunidad(&self, id: i32) -> Result<comunidad_autonoma::Model, AppError> {
        comunidad_autonoma::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Comunidad autonoma con id {id}")))
    }

    async fn listar_provincias(
        &self,
        id_comunidad: Option<i32>,
    ) -> Result<Vec<provincia::Model>, AppError> {
        let mut query = provincia::Entity::find().order_by_asc(provincia::Column::Id);

        // Si viene filtro, anadimos WHERE. Si no, devolvemos todas.
        if let Some(id_com) = id_comunidad {
            query = query.filter(provincia::Column::IdComunidad.eq(id_com));
        }

        let provincias = query.all(&self.db).await.map_err(AppError::from_db)?;

        Ok(provincias)
    }

    async fn obtener_provincia(&self, id: i32) -> Result<provincia::Model, AppError> {
        provincia::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Provincia con id {id}")))
    }

    async fn obtener_oficina_por_provincia(
        &self,
        id_provincia: i32,
    ) -> Result<oficina_sepe::Model, AppError> {
        oficina_sepe::Entity::find()
            .filter(oficina_sepe::Column::IdProvincia.eq(id_provincia))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Oficina SEPE para provincia con id {id_provincia}"))
            })
    }
}
