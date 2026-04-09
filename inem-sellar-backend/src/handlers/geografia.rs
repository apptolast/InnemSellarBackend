//! Handlers HTTP para endpoints de geografia.
//!
//! NO hay SQL aqui — los handlers llaman al repositorio.
//! Los tipos Model de SeaORM son serializables a JSON directamente.
//!
//! Todos los handlers usan `#[endpoint]` para que Salvo genere
//! documentacion OpenAPI automaticamente de cada ruta.
//!
//! # Por que `Json<serde_json::Value>` para respuestas de SeaORM
//! Los modelos de SeaORM (`comunidad_autonoma::Model`, etc.) no implementan
//! `ToSchema` de Salvo OAPI — implementarlo requeriria modificar los derives
//! de `DeriveEntityModel`, lo que puede generar conflictos. En vez de eso,
//! serializamos a `serde_json::Value` (JSON opaco) para la respuesta.
//! OpenAPI documenta estos endpoints como que devuelven un objeto JSON generico.
//! Los DTOs que controlamos directamente (auth, ofertas) si tienen `ToSchema`.

use salvo::oapi::extract::{PathParam, QueryParam};
use salvo::prelude::*;

use crate::errors::AppError;
use crate::repositories::{GeografiaRepo, SeaGeografiaRepo};

/// GET /api/v1/comunidades — Listar todas las comunidades autonomas de Espana.
///
/// Devuelve las 17 CCAA + Ceuta + Melilla con datos del servicio de empleo regional
/// (nombre, web, URL de sellado de demanda de empleo).
///
/// # Por que no necesita parametros de ruta ni query
/// Este endpoint devuelve siempre la lista completa (son solo 19 registros estaticos).
/// No necesita paginacion ni filtros porque el dataset es pequeno y no cambia.
///
/// # Por que `depot: &mut Depot` sigue siendo necesario
/// OpenAPI ignora `depot` en la firma — no lo documenta como parametro.
/// Pero el runtime de Salvo lo necesita para que el handler pueda acceder
/// a los servicios/repositorios inyectados en main.rs con `affix_state::inject`.
#[endpoint(tags("Geografia"))]
pub async fn listar_comunidades(depot: &mut Depot) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let comunidades = repo.listar_comunidades().await?;

    Ok(Json(serde_json::to_value(comunidades).unwrap_or_default()))
}

/// GET /api/v1/comunidades/{id} — Obtener una comunidad autonoma por su ID.
///
/// # Por que `PathParam<i32>` en vez de `req.param::<i32>("id")`
/// `PathParam<i32>` es un extractor tipado de Salvo OAPI. Salvo lo detecta
/// en la firma de la funcion y lo documenta como parametro de ruta en OpenAPI
/// con tipo `integer`. Con `req.param()`, Salvo no puede inferir el tipo
/// ni incluirlo en la documentacion automatica.
///
/// # Por que `*id` (dereference)
/// `id` es de tipo `PathParam<i32>`, no `i32` directamente.
/// `PathParam<T>` implementa `Deref<Target = T>`, lo que permite usar `*id`
/// para obtener el `i32` subyacente. Es como unwrap pero a nivel de tipos.
#[endpoint(tags("Geografia"))]
pub async fn obtener_comunidad(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let comunidad = repo.obtener_comunidad(*id).await?;

    Ok(Json(serde_json::to_value(comunidad).unwrap_or_default()))
}

/// GET /api/v1/provincias — Listar provincias con filtro opcional por comunidad.
///
/// # Parametros de query
/// - `id_comunidad` (opcional): si se pasa, filtra las provincias de esa CCAA.
///   Sin este parametro, devuelve las 52 provincias espanolas.
///
/// # Por que `QueryParam<i32, false>`
/// `QueryParam<T, REQUIRED>` es el extractor de parametros de query URL de Salvo OAPI.
/// El segundo parametro de tipo (`false`) indica que es OPCIONAL.
/// `QueryParam<i32, true>` seria obligatorio.
/// Salvo lo documenta en OpenAPI como `?id_comunidad=X` (query param).
///
/// # Por que `id_comunidad.into_inner()`
/// `QueryParam<i32, false>` wrappea un `Option<i32>` (puede no venir en la URL).
/// `.into_inner()` extrae el `Option<i32>` del wrapper para pasarlo al repositorio.
#[endpoint(tags("Geografia"))]
pub async fn listar_provincias(
    id_comunidad: QueryParam<i32, false>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let provincias = repo.listar_provincias(id_comunidad.into_inner()).await?;

    Ok(Json(serde_json::to_value(provincias).unwrap_or_default()))
}

/// GET /api/v1/provincias/{id} — Obtener una provincia por su codigo INE.
#[endpoint(tags("Geografia"))]
pub async fn obtener_provincia(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let provincia = repo.obtener_provincia(*id).await?;

    Ok(Json(serde_json::to_value(provincia).unwrap_or_default()))
}

/// GET /api/v1/provincias/{id}/oficina — Obtener la oficina SEPE de una provincia.
///
/// Cada provincia tiene exactamente una oficina SEPE con telefono, web y URLs
/// de recursos de empleo especificos de esa provincia.
#[endpoint(tags("Geografia"))]
pub async fn obtener_oficina_por_provincia(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?;

    let oficina = repo.obtener_oficina_por_provincia(*id).await?;

    Ok(Json(serde_json::to_value(oficina).unwrap_or_default()))
}
