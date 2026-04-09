//! CRUD de ofertas de empleo.
//!
//! GET = publico, POST/PUT/DELETE = requiere autenticacion JWT.
//!
//! Los handlers usan `#[endpoint]` para generacion automatica de documentacion
//! OpenAPI. Los endpoints protegidos llevan `security(("bearer_auth" = []))`.

use salvo::oapi::extract::{JsonBody, PathParam, QueryParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::oferta_repo::{ActualizarOfertaDto, CrearOfertaDto};
use crate::repositories::{OfertaRepo, SeaOfertaRepo};

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para crear una nueva oferta de empleo.
///
/// Todos los campos son opcionales para permitir publicaciones parciales.
/// El `id_autor` NO va en el body — se toma del JWT para evitar suplantacion.
#[derive(Deserialize, ToSchema)]
pub struct CrearOfertaRequest {
    /// Titulo del puesto de trabajo. Ej: "Desarrollador Backend Senior".
    pub titulo_puesto: Option<String>,
    /// Nombre de la empresa que oferta. Ej: "AppToLast SL".
    pub empresa: Option<String>,
    /// Ciudad o region donde se realiza el trabajo. Ej: "Madrid (hibrido)".
    pub ubicacion: Option<String>,
    /// Descripcion detallada del puesto, requisitos y condiciones.
    pub descripcion: Option<String>,
    /// Telefono de contacto para candidatos.
    pub telefono_contacto: Option<String>,
    /// Email de contacto para candidatos.
    pub email_contacto: Option<String>,
    /// Web o enlace directo a la oferta original.
    pub web_contacto: Option<String>,
    /// Lista de IDs de provincias donde aplica esta oferta (codigos INE).
    /// Lista vacia = oferta nacional.
    #[serde(default)]
    pub provincias: Vec<i32>,
}

/// Body para actualizar una oferta existente.
///
/// Solo el autor original puede actualizar la oferta.
/// Solo se actualizan los campos que se envien (parcial update).
#[derive(Deserialize, ToSchema)]
pub struct ActualizarOfertaRequest {
    /// Nuevo titulo del puesto (opcional — si no se envia, no cambia).
    pub titulo_puesto: Option<String>,
    /// Nuevo nombre de empresa.
    pub empresa: Option<String>,
    /// Nueva ubicacion.
    pub ubicacion: Option<String>,
    /// Nueva descripcion.
    pub descripcion: Option<String>,
    /// Nuevo telefono de contacto.
    pub telefono_contacto: Option<String>,
    /// Nuevo email de contacto.
    pub email_contacto: Option<String>,
    /// Nueva web de contacto.
    pub web_contacto: Option<String>,
    /// Nuevas provincias asociadas. Si se envia, reemplaza las existentes.
    /// No enviar este campo = no tocar provincias. Enviar `[]` = eliminar todas.
    pub provincias: Option<Vec<i32>>,
}

/// Respuesta paginada de listado de ofertas.
///
/// # Por que `Vec<serde_json::Value>` en lugar de `Vec<OfertaModel>`
/// Las ofertas pueden tener campos adicionales dinamicos (provincias asociadas)
/// que no estan en el struct Model base. Usando `serde_json::Value` tenemos
/// flexibilidad para incluir datos agregados sin crear nuevos structs.
/// La contrapartida es que OpenAPI no puede documentar la estructura interna
/// de cada oferta — esta es una limitacion conocida y aceptada.
#[derive(Serialize, ToSchema)]
pub struct ListaOfertasResponse {
    /// Lista de ofertas de la pagina actual.
    pub ofertas: Vec<serde_json::Value>,
    /// Total de ofertas que coinciden con los filtros (para calcular paginas).
    pub total: u64,
    /// Numero de pagina actual (base 1).
    pub pagina: u64,
    /// Numero de ofertas por pagina.
    pub por_pagina: u64,
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/ofertas — Listar ofertas activas con paginacion y filtro opcional.
///
/// Endpoint publico — no requiere autenticacion.
///
/// # Parametros de query (todos opcionales)
/// - `id_provincia`: filtrar por provincia (codigo INE)
/// - `pagina`: numero de pagina, base 1 (default: 1)
/// - `por_pagina`: resultados por pagina (default: 20)
#[endpoint(tags("Ofertas"))]
pub async fn listar_ofertas(
    id_provincia: QueryParam<i32, false>,
    pagina: QueryParam<u64, false>,
    por_pagina: QueryParam<u64, false>,
    depot: &mut Depot,
) -> Result<Json<ListaOfertasResponse>, AppError> {
    let pagina_val = pagina.into_inner().unwrap_or(1);
    let por_pagina_val = por_pagina.into_inner().unwrap_or(20);

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    let (ofertas, total) = repo
        .listar_ofertas(id_provincia.into_inner(), pagina_val, por_pagina_val)
        .await?;

    // Serializamos las ofertas a JSON Values para la respuesta
    let ofertas_json: Vec<serde_json::Value> = ofertas
        .into_iter()
        .map(|o| serde_json::to_value(o).unwrap_or_default())
        .collect();

    Ok(Json(ListaOfertasResponse {
        ofertas: ofertas_json,
        total,
        pagina: pagina_val,
        por_pagina: por_pagina_val,
    }))
}

/// GET /api/v1/ofertas/{id} — Obtener una oferta por su UUID.
///
/// Endpoint publico — no requiere autenticacion.
#[endpoint(tags("Ofertas"))]
pub async fn obtener_oferta(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Parseamos el String del path param como UUID con manejo de error explicito
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de oferta no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    let oferta = repo.obtener_oferta(uuid).await?;

    Ok(Json(serde_json::to_value(oferta).unwrap_or_default()))
}

/// POST /api/v1/ofertas — Crear oferta (requiere autenticacion JWT).
///
/// El `id_autor` se extrae del token JWT en el middleware — no se acepta
/// del body para evitar que un usuario publique ofertas en nombre de otro.
///
/// # Por que `security(("bearer_auth" = []))`
/// Esta anotacion documenta en OpenAPI que este endpoint requiere un JWT
/// en el header `Authorization: Bearer <token>`. Swagger UI mostrara
/// un candado y permitira introducir el token para probar el endpoint.
#[endpoint(tags("Ofertas"), security(("bearer_auth" = [])))]
pub async fn crear_oferta(
    body: JsonBody<CrearOfertaRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Obtener el usuario autenticado del Depot (inyectado por auth_middleware)
    let id_autor = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    let dto = CrearOfertaDto {
        titulo_puesto: body.titulo_puesto.clone(),
        empresa: body.empresa.clone(),
        ubicacion: body.ubicacion.clone(),
        descripcion: body.descripcion.clone(),
        telefono_contacto: body.telefono_contacto.clone(),
        email_contacto: body.email_contacto.clone(),
        web_contacto: body.web_contacto.clone(),
        provincias: body.provincias.clone(),
    };

    let oferta = repo.crear_oferta(id_autor, dto).await?;

    Ok(Json(serde_json::to_value(oferta).unwrap_or_default()))
}

/// PUT /api/v1/ofertas/{id} — Actualizar oferta (solo el autor original).
///
/// Verifica que el usuario autenticado sea el autor antes de aplicar cambios.
#[endpoint(tags("Ofertas"), security(("bearer_auth" = [])))]
pub async fn actualizar_oferta(
    id: PathParam<String>,
    body: JsonBody<ActualizarOfertaRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de oferta no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    // Verificar que el usuario es el autor de la oferta
    let oferta_existente = repo.obtener_oferta(uuid).await?;
    if oferta_existente.id_autor != id_usuario {
        return Err(AppError::Forbidden);
    }

    let dto = ActualizarOfertaDto {
        titulo_puesto: body.titulo_puesto.clone(),
        empresa: body.empresa.clone(),
        ubicacion: body.ubicacion.clone(),
        descripcion: body.descripcion.clone(),
        telefono_contacto: body.telefono_contacto.clone(),
        email_contacto: body.email_contacto.clone(),
        web_contacto: body.web_contacto.clone(),
        provincias: body.provincias.clone(),
    };

    let oferta = repo.actualizar_oferta(uuid, dto).await?;

    Ok(Json(serde_json::to_value(oferta).unwrap_or_default()))
}

/// DELETE /api/v1/ofertas/{id} — Eliminar oferta (solo el autor original).
///
/// Elimina fisicamente la oferta de la BD. La verificacion de autoria
/// se hace antes de borrar para evitar eliminar contenido de otros usuarios.
#[endpoint(tags("Ofertas"), security(("bearer_auth" = [])))]
pub async fn eliminar_oferta(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de oferta no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaOfertaRepo>()
        .map_err(|_| AppError::Internal("OfertaRepo no disponible".into()))?
        .clone();

    // Verificar que el usuario es el autor
    let oferta = repo.obtener_oferta(uuid).await?;
    if oferta.id_autor != id_usuario {
        return Err(AppError::Forbidden);
    }

    repo.eliminar_oferta(uuid).await?;

    Ok(Json(
        serde_json::json!({"mensaje": "Oferta eliminada correctamente"}),
    ))
}
