//! Handlers de perfil de usuario.
//!
//! Endpoints para que el usuario vea y edite su propio perfil,
//! y para ver perfiles publicos de otros usuarios.
//!
//! # Diferencia con auth.rs
//! `auth.rs` maneja registro/login/tokens (identidad).
//! `usuarios.rs` maneja el perfil (datos personales editables).
//!
//! Todos los handlers usan `#[endpoint]` para documentacion OpenAPI.

use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::usuario_repo::ActualizarPerfilDto;
use crate::repositories::{SeaUsuarioRepo, UsuarioRepo};

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para actualizar el perfil del usuario autenticado.
///
/// Todos los campos son opcionales — solo se actualizan los que se envian.
/// El `id` NO va en el body: se toma del JWT para evitar suplantacion.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarPerfilRequest {
    /// Nombre que se muestra publicamente en la app.
    pub nombre_visible: Option<String>,
    /// URL de la foto de perfil (almacenada en storage externo).
    pub url_avatar: Option<String>,
    /// Enlace al perfil de LinkedIn del usuario.
    pub url_linkedin: Option<String>,
    /// Enlace al CV en PDF (almacenado en storage externo).
    pub url_curriculum: Option<String>,
    /// Codigo INE de la provincia del usuario (1-52).
    pub id_provincia: Option<i32>,
}

/// Respuesta con datos del perfil del usuario.
///
/// Excluye `hash_contrasena` por seguridad — el Model ya tiene
/// `#[serde(skip_serializing)]` en ese campo, pero este DTO
/// explicita lo que se devuelve al cliente.
#[derive(Serialize, ToSchema)]
pub struct PerfilResponse {
    /// UUID del usuario.
    pub id: Uuid,
    /// Email del usuario.
    pub email: Option<String>,
    /// Nombre publico.
    pub nombre_visible: Option<String>,
    /// URL del avatar.
    pub url_avatar: Option<String>,
    /// URL de LinkedIn.
    pub url_linkedin: Option<String>,
    /// URL del CV.
    pub url_curriculum: Option<String>,
    /// Si la cuenta esta activa.
    pub activo: Option<bool>,
    /// Codigo INE de la provincia.
    pub id_provincia: Option<i32>,
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/perfil — Obtener el perfil del usuario autenticado.
///
/// Requiere JWT en el header `Authorization: Bearer <token>`.
/// Devuelve todos los datos del perfil excepto el hash de contrasena.
///
/// # Por que un endpoint separado de GET /usuarios/{id}
/// El perfil propio devuelve TODOS los datos (email, provincia, etc.).
/// El perfil publico de otro usuario solo devuelve datos no sensibles.
#[endpoint(tags("Usuarios"), security(("bearer_auth" = [])))]
pub async fn obtener_perfil(depot: &mut Depot) -> Result<Json<PerfilResponse>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaUsuarioRepo>()
        .map_err(|_| AppError::Internal("UsuarioRepo no disponible".into()))?
        .clone();

    let usuario = repo.obtener_perfil(id_usuario).await?;

    Ok(Json(PerfilResponse {
        id: usuario.id,
        email: usuario.email,
        nombre_visible: usuario.nombre_visible,
        url_avatar: usuario.url_avatar,
        url_linkedin: usuario.url_linkedin,
        url_curriculum: usuario.url_curriculum,
        activo: usuario.activo,
        id_provincia: usuario.id_provincia,
    }))
}

/// PUT /api/v1/perfil — Actualizar el perfil del usuario autenticado.
///
/// Solo actualiza los campos que se envian en el body.
/// El usuario solo puede editar SU PROPIO perfil (el id viene del JWT).
#[endpoint(tags("Usuarios"), security(("bearer_auth" = [])))]
pub async fn actualizar_perfil(
    body: JsonBody<ActualizarPerfilRequest>,
    depot: &mut Depot,
) -> Result<Json<PerfilResponse>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaUsuarioRepo>()
        .map_err(|_| AppError::Internal("UsuarioRepo no disponible".into()))?
        .clone();

    let dto = ActualizarPerfilDto {
        nombre_visible: body.nombre_visible.clone(),
        url_avatar: body.url_avatar.clone(),
        url_linkedin: body.url_linkedin.clone(),
        url_curriculum: body.url_curriculum.clone(),
        id_provincia: body.id_provincia,
    };

    let usuario = repo.actualizar_perfil(id_usuario, dto).await?;

    Ok(Json(PerfilResponse {
        id: usuario.id,
        email: usuario.email,
        nombre_visible: usuario.nombre_visible,
        url_avatar: usuario.url_avatar,
        url_linkedin: usuario.url_linkedin,
        url_curriculum: usuario.url_curriculum,
        activo: usuario.activo,
        id_provincia: usuario.id_provincia,
    }))
}

/// GET /api/v1/usuarios/{id} — Obtener perfil publico de un usuario.
///
/// Endpoint publico — no requiere autenticacion.
/// Solo devuelve datos no sensibles (nombre, avatar).
/// No expone email, provincia ni otros datos privados.
#[endpoint(tags("Usuarios"))]
pub async fn obtener_usuario_publico(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de usuario no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaUsuarioRepo>()
        .map_err(|_| AppError::Internal("UsuarioRepo no disponible".into()))?
        .clone();

    let usuario = repo.obtener_perfil(uuid).await?;

    // Solo exponemos datos publicos — sin email, sin provincia
    Ok(Json(serde_json::json!({
        "id": usuario.id,
        "nombre_visible": usuario.nombre_visible,
        "url_avatar": usuario.url_avatar,
    })))
}
