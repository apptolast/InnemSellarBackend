// src/repositories/usuario_repo.rs
//
// Repositorio de usuarios — operaciones de perfil (no auth).
//
// # Por que separar de auth_repo
// auth_repo maneja registro, login, tokens — operaciones de autenticacion.
// usuario_repo maneja el perfil del usuario — operaciones de datos personales.
// Separar responsabilidades mantiene cada modulo enfocado y testeable.

use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::usuario;

/// DTO para actualizar el perfil del usuario.
///
/// Solo contiene campos editables por el usuario. Campos como `email`,
/// `hash_contrasena`, `activo` NO estan aqui porque se gestionan
/// desde auth (cambio de contrasena) o admin (desactivar cuenta).
pub struct ActualizarPerfilDto {
    pub nombre_visible: Option<String>,
    pub url_avatar: Option<String>,
    pub url_linkedin: Option<String>,
    pub url_curriculum: Option<String>,
    pub id_provincia: Option<i32>,
}

/// Contrato (interfaz) para acceso a datos de perfil de usuario.
///
/// # Por que un trait y no funciones directas
/// Un trait en Rust es como una interfaz en Dart/Java. Define QUE operaciones
/// existen sin decir COMO se implementan. Esto permite:
/// 1. Cambiar la implementacion (SeaORM hoy, otro ORM manana) sin tocar handlers
/// 2. Crear implementaciones de prueba (mocks) para tests
/// 3. Documentar el contrato de forma explicita
pub trait UsuarioRepo: Send + Sync {
    /// Obtener el perfil completo de un usuario por su UUID.
    /// Devuelve el Model completo (el campo hash_contrasena se excluye
    /// en la serializacion gracias a `#[serde(skip_serializing)]` en el Model).
    fn obtener_perfil(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<usuario::Model, AppError>> + Send;

    /// Actualizar los datos del perfil de un usuario.
    /// Solo modifica los campos que vienen con valor (partial update).
    fn actualizar_perfil(
        &self,
        id: Uuid,
        datos: ActualizarPerfilDto,
    ) -> impl std::future::Future<Output = Result<usuario::Model, AppError>> + Send;
}

/// Implementacion con SeaORM + PostgreSQL.
#[derive(Clone)]
pub struct SeaUsuarioRepo {
    db: sea_orm::DatabaseConnection,
}

impl SeaUsuarioRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl UsuarioRepo for SeaUsuarioRepo {
    async fn obtener_perfil(&self, id: Uuid) -> Result<usuario::Model, AppError> {
        usuario::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Usuario con id {id}")))
    }

    async fn actualizar_perfil(
        &self,
        id: Uuid,
        datos: ActualizarPerfilDto,
    ) -> Result<usuario::Model, AppError> {
        let usuario = usuario::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Usuario con id {id}")))?;

        let mut active: usuario::ActiveModel = usuario.into();

        // Solo actualizamos los campos que vienen con valor.
        // `Set(valor)` marca el campo como "modificado" en SeaORM.
        // Los campos no tocados mantienen su valor original.
        if datos.nombre_visible.is_some() {
            active.nombre_visible = Set(datos.nombre_visible);
        }
        if datos.url_avatar.is_some() {
            active.url_avatar = Set(datos.url_avatar);
        }
        if datos.url_linkedin.is_some() {
            active.url_linkedin = Set(datos.url_linkedin);
        }
        if datos.url_curriculum.is_some() {
            active.url_curriculum = Set(datos.url_curriculum);
        }
        if datos.id_provincia.is_some() {
            active.id_provincia = Set(datos.id_provincia);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }
}
