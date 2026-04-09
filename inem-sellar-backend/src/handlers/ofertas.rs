// src/handlers/ofertas.rs
//
// CRUD de ofertas de empleo.
// GET = publico, POST/PUT/DELETE = requiere autenticacion.

use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::oferta_repo::{ActualizarOfertaDto, CrearOfertaDto};
use crate::repositories::{OfertaRepo, SeaOfertaRepo};

// ─── DTOs ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CrearOfertaRequest {
    pub titulo_puesto: Option<String>,
    pub empresa: Option<String>,
    pub ubicacion: Option<String>,
    pub descripcion: Option<String>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub web_contacto: Option<String>,
    #[serde(default)]
    pub provincias: Vec<i32>,
}

#[derive(Deserialize)]
pub struct ActualizarOfertaRequest {
    pub titulo_puesto: Option<String>,
    pub empresa: Option<String>,
    pub ubicacion: Option<String>,
    pub descripcion: Option<String>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub web_contacto: Option<String>,
}

#[derive(Serialize)]
pub struct ListaOfertasResponse {
    pub ofertas: Vec<serde_json::Value>,
    pub total: u64,
    pub pagina: u64,
    pub por_pagina: u64,
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/ofertas — Listar ofertas activas.
/// Query params opcionales: ?id_provincia=X&pagina=1&por_pagina=20
#[handler]
pub async fn listar_ofertas(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<ListaOfertasResponse>, AppError> {
    let id_provincia = req.query::<i32>("id_provincia");
    let pagina = req.query::<u64>("pagina").unwrap_or(1);
    let por_pagina = req.query::<u64>("por_pagina").unwrap_or(20);

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    let (ofertas, total) = repo
        .listar_ofertas(id_provincia, pagina, por_pagina)
        .await?;

    // Serializamos las ofertas a JSON Values para la respuesta
    let ofertas_json: Vec<serde_json::Value> = ofertas
        .into_iter()
        .map(|o| serde_json::to_value(o).unwrap_or_default())
        .collect();

    Ok(Json(ListaOfertasResponse {
        ofertas: ofertas_json,
        total,
        pagina,
        por_pagina,
    }))
}

/// GET /api/v1/ofertas/{id} — Obtener una oferta por UUID.
#[handler]
pub async fn obtener_oferta(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = req
        .param::<String>("id")
        .ok_or_else(|| AppError::BadRequest("ID de oferta faltante".into()))?;
    let id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de oferta no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    let oferta = repo.obtener_oferta(id).await?;

    Ok(Json(serde_json::to_value(oferta).unwrap_or_default()))
}

/// POST /api/v1/ofertas — Crear oferta (requiere auth).
/// El id_autor se toma del token JWT (no del body).
#[handler]
pub async fn crear_oferta(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Obtener el usuario autenticado del Depot (inyectado por auth_middleware)
    let id_autor = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let body: CrearOfertaRequest = req
        .parse_json()
        .await
        .map_err(|e| AppError::BadRequest(format!("JSON invalido: {e}")))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    let dto = CrearOfertaDto {
        titulo_puesto: body.titulo_puesto,
        empresa: body.empresa,
        ubicacion: body.ubicacion,
        descripcion: body.descripcion,
        telefono_contacto: body.telefono_contacto,
        email_contacto: body.email_contacto,
        web_contacto: body.web_contacto,
        provincias: body.provincias,
    };

    let oferta = repo.crear_oferta(id_autor, dto).await?;

    Ok(Json(serde_json::to_value(oferta).unwrap_or_default()))
}

/// PUT /api/v1/ofertas/{id} — Actualizar oferta (solo el autor).
#[handler]
pub async fn actualizar_oferta(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let id = req
        .param::<String>("id")
        .ok_or_else(|| AppError::BadRequest("ID de oferta faltante".into()))?;
    let id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de oferta no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    // Verificar que el usuario es el autor de la oferta
    let oferta_existente = repo.obtener_oferta(id).await?;
    if oferta_existente.id_autor != id_usuario {
        return Err(AppError::Forbidden);
    }

    let body: ActualizarOfertaRequest = req
        .parse_json()
        .await
        .map_err(|e| AppError::BadRequest(format!("JSON invalido: {e}")))?;

    let dto = ActualizarOfertaDto {
        titulo_puesto: body.titulo_puesto,
        empresa: body.empresa,
        ubicacion: body.ubicacion,
        descripcion: body.descripcion,
        telefono_contacto: body.telefono_contacto,
        email_contacto: body.email_contacto,
        web_contacto: body.web_contacto,
    };

    let oferta = repo.actualizar_oferta(id, dto).await?;

    Ok(Json(serde_json::to_value(oferta).unwrap_or_default()))
}

/// DELETE /api/v1/ofertas/{id} — Eliminar oferta (solo el autor).
#[handler]
pub async fn eliminar_oferta(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let id = req
        .param::<String>("id")
        .ok_or_else(|| AppError::BadRequest("ID de oferta faltante".into()))?;
    let id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de oferta no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    // Verificar que el usuario es el autor
    let oferta = repo.obtener_oferta(id).await?;
    if oferta.id_autor != id_usuario {
        return Err(AppError::Forbidden);
    }

    repo.eliminar_oferta(id).await?;

    Ok(Json(
        serde_json::json!({"mensaje": "Oferta eliminada correctamente"}),
    ))
}
