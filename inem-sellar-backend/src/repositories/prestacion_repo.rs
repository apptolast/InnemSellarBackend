use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::errors::AppError;
use crate::models::prestacion;

/// DTO para crear una prestacion SEPE.
pub struct CrearPrestacionDto {
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub requisitos: Option<Vec<String>>,
    pub url: Option<String>,
}

/// DTO para actualizar una prestacion SEPE.
pub struct ActualizarPrestacionDto {
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub requisitos: Option<Vec<String>>,
    pub url: Option<String>,
    pub activo: Option<bool>,
}

pub trait PrestacionRepo: Send + Sync {
    fn listar_prestaciones(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<prestacion::Model>, AppError>> + Send;

    fn obtener_prestacion(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<prestacion::Model, AppError>> + Send;

    fn crear_prestacion(
        &self,
        datos: CrearPrestacionDto,
    ) -> impl std::future::Future<Output = Result<prestacion::Model, AppError>> + Send;

    fn actualizar_prestacion(
        &self,
        id: i32,
        datos: ActualizarPrestacionDto,
    ) -> impl std::future::Future<Output = Result<prestacion::Model, AppError>> + Send;

    fn eliminar_prestacion(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaPrestacionRepo {
    db: DatabaseConnection,
}

impl SeaPrestacionRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PrestacionRepo for SeaPrestacionRepo {
    async fn listar_prestaciones(&self) -> Result<Vec<prestacion::Model>, AppError> {
        prestacion::Entity::find()
            .filter(prestacion::Column::Activo.eq(Some(true)))
            .all(&self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn obtener_prestacion(&self, id: i32) -> Result<prestacion::Model, AppError> {
        prestacion::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Prestacion con id {id}")))
    }

    async fn crear_prestacion(
        &self,
        datos: CrearPrestacionDto,
    ) -> Result<prestacion::Model, AppError> {
        let nueva = prestacion::ActiveModel {
            titulo: Set(datos.titulo),
            descripcion: Set(datos.descripcion),
            requisitos: Set(datos.requisitos),
            url: Set(datos.url),
            activo: Set(Some(true)),
            ..Default::default()
        };
        nueva.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn actualizar_prestacion(
        &self,
        id: i32,
        datos: ActualizarPrestacionDto,
    ) -> Result<prestacion::Model, AppError> {
        let prestacion = prestacion::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Prestacion con id {id}")))?;

        let mut active: prestacion::ActiveModel = prestacion.into();

        if datos.titulo.is_some() {
            active.titulo = Set(datos.titulo);
        }
        if datos.descripcion.is_some() {
            active.descripcion = Set(datos.descripcion);
        }
        if datos.requisitos.is_some() {
            active.requisitos = Set(datos.requisitos);
        }
        if datos.url.is_some() {
            active.url = Set(datos.url);
        }
        if datos.activo.is_some() {
            active.activo = Set(datos.activo);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }

    async fn eliminar_prestacion(&self, id: i32) -> Result<(), AppError> {
        let result = prestacion::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Prestacion con id {id}")));
        }
        Ok(())
    }
}
