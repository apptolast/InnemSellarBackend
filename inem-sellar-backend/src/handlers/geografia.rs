// src/handlers/geografia.rs
//
// Handlers HTTP para endpoints de geografia.
// NO hay SQL aqui — los handlers llaman al repositorio.
// Los tipos Model de SeaORM son serializables a JSON directamente.

use salvo::prelude::*;

use crate::errors::AppError;
use crate::models::{comunidad_autonoma, oficina_sepe, provincia};
use crate::repositories::{GeografiaRepo, SeaGeografiaRepo};

/// GET /api/v1/comunidades
#[handler]
pub async fn listar_comunidades(
    depot: &mut Depot,
) -> Result<Json<Vec<comunidad_autonoma::Model>>, AppError> {
    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let comunidades = repo.listar_comunidades().await?;

    Ok(Json(comunidades))
}

/// GET /api/v1/comunidades/{id}
#[handler]
pub async fn obtener_comunidad(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<comunidad_autonoma::Model>, AppError> {
    let id = req
        .param::<i32>("id")
        .ok_or_else(|| AppError::BadRequest("ID de comunidad invalido o faltante".into()))?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let comunidad = repo.obtener_comunidad(id).await?;

    Ok(Json(comunidad))
}

/// GET /api/v1/provincias — con filtro opcional ?id_comunidad=X
#[handler]
pub async fn listar_provincias(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<Vec<provincia::Model>>, AppError> {
    let id_comunidad = req.query::<i32>("id_comunidad");

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let provincias = repo.listar_provincias(id_comunidad).await?;

    Ok(Json(provincias))
}

/// GET /api/v1/provincias/{id}
#[handler]
pub async fn obtener_provincia(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<provincia::Model>, AppError> {
    let id = req
        .param::<i32>("id")
        .ok_or_else(|| AppError::BadRequest("ID de provincia invalido o faltante".into()))?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let provincia = repo.obtener_provincia(id).await?;

    Ok(Json(provincia))
}

/// GET /api/v1/provincias/{id}/oficina
#[handler]
pub async fn obtener_oficina_por_provincia(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<oficina_sepe::Model>, AppError> {
    let id_provincia = req
        .param::<i32>("id")
        .ok_or_else(|| AppError::BadRequest("ID de provincia invalido o faltante".into()))?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let oficina = repo.obtener_oficina_por_provincia(id_provincia).await?;

    Ok(Json(oficina))
}
