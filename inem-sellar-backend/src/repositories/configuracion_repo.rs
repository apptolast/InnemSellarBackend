use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};

use crate::errors::AppError;
use crate::models::configuracion_aplicacion;

/// DTO para crear o actualizar una entrada de configuracion.
pub struct UpsertConfiguracionDto {
    pub valor: Option<String>,
    pub descripcion: Option<String>,
}

pub trait ConfiguracionRepo: Send + Sync {
    fn listar_configuracion(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<configuracion_aplicacion::Model>, AppError>> + Send;

    fn obtener_configuracion(
        &self,
        clave: &str,
    ) -> impl std::future::Future<Output = Result<configuracion_aplicacion::Model, AppError>> + Send;

    fn crear_configuracion(
        &self,
        clave: &str,
        datos: UpsertConfiguracionDto,
    ) -> impl std::future::Future<Output = Result<configuracion_aplicacion::Model, AppError>> + Send;

    fn actualizar_configuracion(
        &self,
        clave: &str,
        datos: UpsertConfiguracionDto,
    ) -> impl std::future::Future<Output = Result<configuracion_aplicacion::Model, AppError>> + Send;

    fn eliminar_configuracion(
        &self,
        clave: &str,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaConfiguracionRepo {
    db: DatabaseConnection,
}

impl SeaConfiguracionRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ConfiguracionRepo for SeaConfiguracionRepo {
    async fn listar_configuracion(&self) -> Result<Vec<configuracion_aplicacion::Model>, AppError> {
        configuracion_aplicacion::Entity::find()
            .all(&self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn obtener_configuracion(
        &self,
        clave: &str,
    ) -> Result<configuracion_aplicacion::Model, AppError> {
        configuracion_aplicacion::Entity::find_by_id(clave.to_string())
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Configuracion con clave '{clave}'")))
    }

    async fn crear_configuracion(
        &self,
        clave: &str,
        datos: UpsertConfiguracionDto,
    ) -> Result<configuracion_aplicacion::Model, AppError> {
        // Verificar que no exista ya
        let existente = configuracion_aplicacion::Entity::find_by_id(clave.to_string())
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if existente.is_some() {
            return Err(AppError::Conflict(format!(
                "Configuracion con clave '{clave}' ya existe"
            )));
        }

        let nueva = configuracion_aplicacion::ActiveModel {
            clave: Set(clave.to_string()),
            valor: Set(datos.valor),
            descripcion: Set(datos.descripcion),
            ..Default::default()
        };
        nueva.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn actualizar_configuracion(
        &self,
        clave: &str,
        datos: UpsertConfiguracionDto,
    ) -> Result<configuracion_aplicacion::Model, AppError> {
        let config = configuracion_aplicacion::Entity::find_by_id(clave.to_string())
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Configuracion con clave '{clave}'")))?;

        let mut active: configuracion_aplicacion::ActiveModel = config.into();

        if datos.valor.is_some() {
            active.valor = Set(datos.valor);
        }
        if datos.descripcion.is_some() {
            active.descripcion = Set(datos.descripcion);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }

    async fn eliminar_configuracion(&self, clave: &str) -> Result<(), AppError> {
        let result = configuracion_aplicacion::Entity::delete_by_id(clave.to_string())
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Configuracion con clave '{clave}'"
            )));
        }
        Ok(())
    }
}
