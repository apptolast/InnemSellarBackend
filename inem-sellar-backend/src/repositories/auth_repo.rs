// src/repositories/auth_repo.rs
//
// Repositorio de autenticacion — acceso a datos de usuarios y tokens.

use chrono::{DateTime, FixedOffset};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{token_refresco, usuario};

/// Contrato de acceso a datos de autenticacion.
pub trait AuthRepo: Send + Sync {
    fn crear_usuario(
        &self,
        email: &str,
        hash_contrasena: &str,
        nombre_visible: Option<&str>,
    ) -> impl std::future::Future<Output = Result<usuario::Model, AppError>> + Send;

    fn buscar_por_email(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<usuario::Model>, AppError>> + Send;

    fn guardar_refresh_token(
        &self,
        id_usuario: Uuid,
        hash_token: &str,
        info_dispositivo: Option<&str>,
        expira_en: DateTime<FixedOffset>,
    ) -> impl std::future::Future<Output = Result<token_refresco::Model, AppError>> + Send;

    fn buscar_refresh_token_por_hash(
        &self,
        hash: &str,
    ) -> impl std::future::Future<Output = Result<Option<token_refresco::Model>, AppError>> + Send;

    fn revocar_refresh_token(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn actualizar_ultimo_login(
        &self,
        id_usuario: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

/// Implementacion con SeaORM.
#[derive(Clone)]
pub struct SeaAuthRepo {
    db: DatabaseConnection,
}

impl SeaAuthRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl AuthRepo for SeaAuthRepo {
    async fn crear_usuario(
        &self,
        email: &str,
        hash_contrasena: &str,
        nombre_visible: Option<&str>,
    ) -> Result<usuario::Model, AppError> {
        let new_user = usuario::ActiveModel {
            id: Set(Uuid::new_v4()),
            email: Set(Some(email.to_string())),
            hash_contrasena: Set(Some(hash_contrasena.to_string())),
            nombre_visible: Set(nombre_visible.map(String::from)),
            activo: Set(Some(true)),
            ..Default::default()
        };

        new_user.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn buscar_por_email(&self, email: &str) -> Result<Option<usuario::Model>, AppError> {
        usuario::Entity::find()
            .filter(usuario::Column::Email.eq(Some(email.to_string())))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn guardar_refresh_token(
        &self,
        id_usuario: Uuid,
        hash_token: &str,
        info_dispositivo: Option<&str>,
        expira_en: DateTime<FixedOffset>,
    ) -> Result<token_refresco::Model, AppError> {
        let new_token = token_refresco::ActiveModel {
            id: Set(Uuid::new_v4()),
            id_usuario: Set(id_usuario),
            hash_token: Set(Some(hash_token.to_string())),
            informacion_dispositivo: Set(info_dispositivo.map(String::from)),
            expira_en: Set(Some(expira_en)),
            revocado: Set(Some(false)),
            ..Default::default()
        };

        new_token.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn buscar_refresh_token_por_hash(
        &self,
        hash: &str,
    ) -> Result<Option<token_refresco::Model>, AppError> {
        token_refresco::Entity::find()
            .filter(token_refresco::Column::HashToken.eq(Some(hash.to_string())))
            .filter(token_refresco::Column::Revocado.eq(Some(false)))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn revocar_refresh_token(&self, id: Uuid) -> Result<(), AppError> {
        let token = token_refresco::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound("Token no encontrado".into()))?;

        let mut active: token_refresco::ActiveModel = token.into();
        active.revocado = Set(Some(true));
        active.update(&self.db).await.map_err(AppError::from_db)?;

        Ok(())
    }

    async fn actualizar_ultimo_login(&self, id_usuario: Uuid) -> Result<(), AppError> {
        let user = usuario::Entity::find_by_id(id_usuario)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound("Usuario no encontrado".into()))?;

        let mut active: usuario::ActiveModel = user.into();
        active.ultimo_login = Set(Some(chrono::Utc::now().fixed_offset()));
        active.update(&self.db).await.map_err(AppError::from_db)?;

        Ok(())
    }
}
