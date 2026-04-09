//! Handlers de prestaciones SEPE (RAI, SED, subsidios, etc.).
//!
//! Las prestaciones son datos nacionales gestionados por administradores.
//! GET = publico, POST/PUT/DELETE = requiere autenticacion (admin).
//!
//! Todos los handlers usan `#[endpoint]` para documentacion OpenAPI.

use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::repositories::prestacion_repo::{ActualizarPrestacionDto, CrearPrestacionDto};
use crate::repositories::{PrestacionRepo, SeaPrestacionRepo};

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para crear una nueva prestacion SEPE.
#[derive(Deserialize, ToSchema)]
pub struct CrearPrestacionRequest {
    /// Nombre de la prestacion. Ej: "Renta Activa de Insercion (RAI)".
    pub titulo: Option<String>,
    /// Descripcion detallada de la prestacion y sus condiciones.
    pub descripcion: Option<String>,
    /// Lista de requisitos para acceder a la prestacion.
    #[serde(default)]
    pub requisitos: Option<Vec<String>>,
    /// Enlace a la pagina oficial del SEPE con informacion.
    pub url: Option<String>,
}

/// Body para actualizar una prestacion existente.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarPrestacionRequest {
    /// Nuevo titulo de la prestacion.
    pub titulo: Option<String>,
    /// Nueva descripcion.
    pub descripcion: Option<String>,
    /// Nuevos requisitos.
    pub requisitos: Option<Vec<String>>,
    /// Nueva URL.
    pub url: Option<String>,
    /// Activar o desactivar la prestacion.
    pub activo: Option<bool>,
}

/// Respuesta con mensaje de confirmacion.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Mensaje descriptivo de la operacion realizada.
    pub mensaje: String,
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/prestaciones — Listar todas las prestaciones SEPE activas.
///
/// Endpoint publico — no requiere autenticacion.
/// Devuelve prestaciones nacionales (RAI, SED, subsidios, etc.).
#[endpoint(tags("Prestaciones"))]
pub async fn listar_prestaciones(depot: &mut Depot) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaPrestacionRepo>()
        .map_err(|_| AppError::Internal("PrestacionRepo no disponible".into()))?
        .clone();

    let prestaciones = repo.listar_prestaciones().await?;

    Ok(Json(serde_json::to_value(prestaciones).unwrap_or_default()))
}

/// GET /api/v1/prestaciones/{id} — Obtener una prestacion por su ID.
///
/// Endpoint publico — no requiere autenticacion.
#[endpoint(tags("Prestaciones"))]
pub async fn obtener_prestacion(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaPrestacionRepo>()
        .map_err(|_| AppError::Internal("PrestacionRepo no disponible".into()))?
        .clone();

    let prestacion = repo.obtener_prestacion(*id).await?;

    Ok(Json(serde_json::to_value(prestacion).unwrap_or_default()))
}

/// POST /api/v1/prestaciones — Crear una nueva prestacion (admin).
///
/// Requiere autenticacion JWT. En el futuro se verificara rol de admin.
#[endpoint(tags("Prestaciones"), security(("bearer_auth" = [])))]
pub async fn crear_prestacion(
    body: JsonBody<CrearPrestacionRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaPrestacionRepo>()
        .map_err(|_| AppError::Internal("PrestacionRepo no disponible".into()))?
        .clone();

    let dto = CrearPrestacionDto {
        titulo: body.titulo.clone(),
        descripcion: body.descripcion.clone(),
        requisitos: body.requisitos.clone(),
        url: body.url.clone(),
    };

    let prestacion = repo.crear_prestacion(dto).await?;

    Ok(Json(serde_json::to_value(prestacion).unwrap_or_default()))
}

/// PUT /api/v1/prestaciones/{id} — Actualizar una prestacion (admin).
///
/// Requiere autenticacion JWT. Solo se actualizan los campos enviados.
#[endpoint(tags("Prestaciones"), security(("bearer_auth" = [])))]
pub async fn actualizar_prestacion(
    id: PathParam<i32>,
    body: JsonBody<ActualizarPrestacionRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaPrestacionRepo>()
        .map_err(|_| AppError::Internal("PrestacionRepo no disponible".into()))?
        .clone();

    let dto = ActualizarPrestacionDto {
        titulo: body.titulo.clone(),
        descripcion: body.descripcion.clone(),
        requisitos: body.requisitos.clone(),
        url: body.url.clone(),
        activo: body.activo,
    };

    let prestacion = repo.actualizar_prestacion(*id, dto).await?;

    Ok(Json(serde_json::to_value(prestacion).unwrap_or_default()))
}

/// DELETE /api/v1/prestaciones/{id} — Eliminar una prestacion (admin).
///
/// Elimina fisicamente la prestacion de la BD.
#[endpoint(tags("Prestaciones"), security(("bearer_auth" = [])))]
pub async fn eliminar_prestacion(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaPrestacionRepo>()
        .map_err(|_| AppError::Internal("PrestacionRepo no disponible".into()))?
        .clone();

    repo.eliminar_prestacion(*id).await?;

    Ok(Json(MensajeResponse {
        mensaje: "Prestacion eliminada correctamente".into(),
    }))
}
