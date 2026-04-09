use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::enums::TipoContenido;
use crate::models::voto;

pub trait VotoRepo: Send + Sync {
    fn votar(
        &self,
        id_usuario: Uuid,
        tipo_contenido: TipoContenido,
        id_contenido: Uuid,
        tipo_voto: i32,
    ) -> impl std::future::Future<Output = Result<voto::Model, AppError>> + Send;

    fn obtener_voto(
        &self,
        id_usuario: Uuid,
        tipo_contenido: TipoContenido,
        id_contenido: Uuid,
    ) -> impl std::future::Future<Output = Result<Option<voto::Model>, AppError>> + Send;

    fn eliminar_voto(
        &self,
        id_usuario: Uuid,
        tipo_contenido: TipoContenido,
        id_contenido: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaVotoRepo {
    db: DatabaseConnection,
}

impl SeaVotoRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl VotoRepo for SeaVotoRepo {
    async fn votar(
        &self,
        id_usuario: Uuid,
        tipo_contenido: TipoContenido,
        id_contenido: Uuid,
        tipo_voto: i32,
    ) -> Result<voto::Model, AppError> {
        // Buscar voto existente
        let existente = self
            .obtener_voto(id_usuario, tipo_contenido.clone(), id_contenido)
            .await?;

        match existente {
            Some(voto_existente) => {
                // Actualizar voto existente
                let mut active: voto::ActiveModel = voto_existente.into();
                active.tipo_voto = Set(Some(tipo_voto));
                active.update(&self.db).await.map_err(AppError::from_db)
            }
            None => {
                // Crear voto nuevo
                let nuevo = voto::ActiveModel {
                    id_usuario: Set(id_usuario),
                    tipo_contenido: Set(tipo_contenido),
                    id_contenido: Set(id_contenido),
                    tipo_voto: Set(Some(tipo_voto)),
                    ..Default::default()
                };
                nuevo.insert(&self.db).await.map_err(AppError::from_db)
            }
        }
    }

    async fn obtener_voto(
        &self,
        id_usuario: Uuid,
        tipo_contenido: TipoContenido,
        id_contenido: Uuid,
    ) -> Result<Option<voto::Model>, AppError> {
        voto::Entity::find()
            .filter(voto::Column::IdUsuario.eq(id_usuario))
            .filter(voto::Column::TipoContenido.eq(tipo_contenido))
            .filter(voto::Column::IdContenido.eq(id_contenido))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn eliminar_voto(
        &self,
        id_usuario: Uuid,
        tipo_contenido: TipoContenido,
        id_contenido: Uuid,
    ) -> Result<(), AppError> {
        let voto = self
            .obtener_voto(id_usuario, tipo_contenido, id_contenido)
            .await?
            .ok_or_else(|| AppError::NotFound("Voto no encontrado".into()))?;

        let active: voto::ActiveModel = voto.into();
        active.delete(&self.db).await.map_err(AppError::from_db)?;
        Ok(())
    }
}
