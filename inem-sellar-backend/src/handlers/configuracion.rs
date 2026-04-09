//! Handlers de configuracion de la aplicacion.
//!
//! La tabla `configuracion_aplicacion` es un key-value store para
//! ajustes globales de la app (modo mantenimiento, version minima, etc.).
//! GET = publico, POST/PUT/DELETE = requiere autenticacion (admin).
//!
//! Todos los handlers usan `#[endpoint]` para documentacion OpenAPI.

use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::repositories::configuracion_repo::UpsertConfiguracionDto;
use crate::repositories::{ConfiguracionRepo, SeaConfiguracionRepo};

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para crear una nueva entrada de configuracion.
#[derive(Deserialize, ToSchema)]
pub struct CrearConfiguracionRequest {
    /// Clave unica de configuracion. Ej: "modo_mantenimiento".
    pub clave: String,
    /// Valor de la configuracion. Ej: "false".
    pub valor: Option<String>,
    /// Descripcion de para que sirve esta configuracion.
    pub descripcion: Option<String>,
}

/// Body para actualizar una entrada de configuracion existente.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarConfiguracionRequest {
    /// Nuevo valor.
    pub valor: Option<String>,
    /// Nueva descripcion.
    pub descripcion: Option<String>,
}

/// Respuesta con mensaje de confirmacion.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Mensaje descriptivo de la operacion realizada.
    pub mensaje: String,
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/configuracion — Listar toda la configuracion de la app.
///
/// Endpoint publico. La app Flutter consulta estos valores para ajustar
/// su comportamiento sin necesidad de actualizar la app.
#[endpoint(tags("Configuracion"))]
pub async fn listar_configuracion(depot: &mut Depot) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaConfiguracionRepo>()
        .map_err(|_| AppError::Internal("ConfiguracionRepo no disponible".into()))?
        .clone();

    let config = repo.listar_configuracion().await?;

    Ok(Json(serde_json::to_value(config).unwrap_or_default()))
}

/// GET /api/v1/configuracion/{clave} — Obtener un valor de configuracion.
///
/// Endpoint publico. Permite consultar una configuracion especifica por clave.
#[endpoint(tags("Configuracion"))]
pub async fn obtener_configuracion(
    clave: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaConfiguracionRepo>()
        .map_err(|_| AppError::Internal("ConfiguracionRepo no disponible".into()))?
        .clone();

    let config = repo.obtener_configuracion(&clave).await?;

    Ok(Json(serde_json::to_value(config).unwrap_or_default()))
}

/// POST /api/v1/configuracion — Crear una nueva entrada de configuracion (admin).
///
/// Requiere autenticacion JWT. Devuelve 409 si la clave ya existe.
#[endpoint(tags("Configuracion"), security(("bearer_auth" = [])))]
pub async fn crear_configuracion(
    body: JsonBody<CrearConfiguracionRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaConfiguracionRepo>()
        .map_err(|_| AppError::Internal("ConfiguracionRepo no disponible".into()))?
        .clone();

    let dto = UpsertConfiguracionDto {
        valor: body.valor.clone(),
        descripcion: body.descripcion.clone(),
    };

    let config = repo.crear_configuracion(&body.clave, dto).await?;

    Ok(Json(serde_json::to_value(config).unwrap_or_default()))
}

/// PUT /api/v1/configuracion/{clave} — Actualizar una configuracion (admin).
///
/// Requiere autenticacion JWT. Solo actualiza campos enviados.
#[endpoint(tags("Configuracion"), security(("bearer_auth" = [])))]
pub async fn actualizar_configuracion(
    clave: PathParam<String>,
    body: JsonBody<ActualizarConfiguracionRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaConfiguracionRepo>()
        .map_err(|_| AppError::Internal("ConfiguracionRepo no disponible".into()))?
        .clone();

    let dto = UpsertConfiguracionDto {
        valor: body.valor.clone(),
        descripcion: body.descripcion.clone(),
    };

    let config = repo.actualizar_configuracion(&clave, dto).await?;

    Ok(Json(serde_json::to_value(config).unwrap_or_default()))
}

/// DELETE /api/v1/configuracion/{clave} — Eliminar una configuracion (admin).
///
/// Elimina fisicamente la entrada de configuracion.
#[endpoint(tags("Configuracion"), security(("bearer_auth" = [])))]
pub async fn eliminar_configuracion(
    clave: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaConfiguracionRepo>()
        .map_err(|_| AppError::Internal("ConfiguracionRepo no disponible".into()))?
        .clone();

    repo.eliminar_configuracion(&clave).await?;

    Ok(Json(MensajeResponse {
        mensaje: "Configuracion eliminada correctamente".into(),
    }))
}
