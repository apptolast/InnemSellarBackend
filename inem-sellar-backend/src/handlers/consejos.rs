//! CRUD de consejos de empleo.
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
use crate::repositories::consejo_repo::{ActualizarConsejoDto, CrearConsejoDto};
use crate::repositories::{ConsejoRepo, SeaConsejoRepo};

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para crear un nuevo consejo de empleo.
///
/// # Por que `ToSchema`
/// El derive `ToSchema` de Salvo genera automaticamente el esquema JSON
/// de este struct en la documentacion OpenAPI. Sin el, Swagger UI no
/// conoce los campos que acepta el endpoint y no puede mostrarlos.
/// Es el equivalente a anotar un DTO en OpenAPI/Swagger con `@ApiProperty`
/// en NestJS o con `class fields` en Dart.
///
/// # Por que todos los campos son `Option<String>`
/// En Rust, `Option<T>` es el equivalente a un valor nullable. Aqui usamos
/// `Option` en todos los campos de texto para que el cliente pueda enviar
/// solo los campos que tiene, sin que el servidor rechace el request.
/// En Dart seria declarar los campos como `String?`.
#[derive(Deserialize, ToSchema)]
pub struct CrearConsejoRequest {
    /// Titulo del consejo. Ej: "Como preparar una entrevista en el SEPE".
    pub titulo: Option<String>,
    /// Contenido principal del consejo en texto libre.
    pub cuerpo: Option<String>,
    /// Enlace adicional de referencia (articulo, video, recurso oficial).
    pub web: Option<String>,
    /// URL de imagen ilustrativa del consejo (alojada en storage externo).
    pub imagen_url: Option<String>,
    /// Lista de IDs de provincias donde aplica este consejo (codigos INE).
    /// Lista vacia o ausente = consejo nacional (aplica a toda Espana).
    ///
    /// # Por que `#[serde(default)]`
    /// Sin esta anotacion, si el cliente no envia el campo `provincias`,
    /// serde falla con un error de deserializacion. Con `default`, serde
    /// asigna `Vec::new()` (lista vacia) cuando el campo no viene en el JSON.
    #[serde(default)]
    pub provincias: Vec<i32>,
}

/// Body para actualizar un consejo existente.
///
/// Solo el autor original puede actualizar el consejo.
/// Solo se actualizan los campos que se envien (actualizacion parcial).
///
/// # Por que `provincias: Option<Vec<i32>>` y no `Vec<i32>`
/// Usamos doble nivel de opcionalidad para distinguir dos casos:
/// - `None` (campo ausente en el JSON): no tocar las provincias actuales.
/// - `Some(vec![])` (campo presente con lista vacia): eliminar todas las provincias.
/// - `Some(vec![28, 41])` (campo presente con valores): reemplazar provincias.
///
/// Con `Vec<i32>` simple no hay forma de distinguir campo omitido de lista vacia.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarConsejoRequest {
    /// Nuevo titulo del consejo (si no se envia, no cambia).
    pub titulo: Option<String>,
    /// Nuevo cuerpo del consejo.
    pub cuerpo: Option<String>,
    /// Nuevo enlace web adicional.
    pub web: Option<String>,
    /// Nueva URL de imagen.
    pub imagen_url: Option<String>,
    /// Nuevas provincias asociadas. Si se envia, reemplaza las existentes.
    /// No enviar este campo = no tocar provincias. Enviar `[]` = eliminar todas.
    pub provincias: Option<Vec<i32>>,
}

/// Respuesta paginada del listado de consejos.
///
/// # Por que `Vec<serde_json::Value>` en lugar de `Vec<consejo::Model>`
/// Los consejos pueden incluir datos agregados dinamicos (provincias asociadas)
/// que no forman parte del struct `Model` base de SeaORM. Usando
/// `serde_json::Value` tenemos flexibilidad para incluir esos datos sin
/// crear nuevos structs intermedios. La contrapartida es que OpenAPI no
/// puede documentar la estructura interna de cada consejo — limitacion
/// conocida y aceptada en este diseno.
#[derive(Serialize, ToSchema)]
pub struct ListaConsejosResponse {
    /// Lista de consejos de la pagina actual.
    pub consejos: Vec<serde_json::Value>,
    /// Total de consejos que coinciden con los filtros (para calcular paginas).
    pub total: u64,
    /// Numero de pagina actual (base 1).
    pub pagina: u64,
    /// Numero de consejos por pagina.
    pub por_pagina: u64,
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/consejos — Listar consejos activos con paginacion y filtro opcional.
///
/// Endpoint publico — no requiere autenticacion.
///
/// # Por que `QueryParam<i32, false>` y no `req.query::<i32>("...")`
/// `QueryParam<T, const REQUIRED: bool>` es un extractor tipado de Salvo
/// que documenta el parametro en OpenAPI automaticamente. El segundo parametro
/// generics (`false`) indica que el parametro es opcional, por lo que Salvo
/// retorna `Option<T>` al llamar `.into_inner()`.
/// Con `req.query()` el codigo funciona igual pero OpenAPI no sabe que
/// parametros acepta el endpoint.
///
/// # Parametros de query (todos opcionales)
/// - `id_provincia`: filtrar por provincia (codigo INE, 1-52)
/// - `pagina`: numero de pagina, base 1 (default: 1)
/// - `por_pagina`: resultados por pagina (default: 20)
#[endpoint(tags("Consejos"))]
pub async fn listar_consejos(
    id_provincia: QueryParam<i32, false>,
    pagina: QueryParam<u64, false>,
    por_pagina: QueryParam<u64, false>,
    depot: &mut Depot,
) -> Result<Json<ListaConsejosResponse>, AppError> {
    let pagina_val = pagina.into_inner().unwrap_or(1);
    let por_pagina_val = por_pagina.into_inner().unwrap_or(20);

    let repo = depot
        .obtain::<SeaConsejoRepo>()
        .map_err(|_| AppError::Internal("ConsejoRepo no disponible".into()))?
        .clone();

    let (consejos, total) = repo
        .listar_consejos(id_provincia.into_inner(), pagina_val, por_pagina_val)
        .await?;

    // Serializamos los consejos a JSON Values para la respuesta
    let consejos_json: Vec<serde_json::Value> = consejos
        .into_iter()
        .map(|c| serde_json::to_value(c).unwrap_or_default())
        .collect();

    Ok(Json(ListaConsejosResponse {
        consejos: consejos_json,
        total,
        pagina: pagina_val,
        por_pagina: por_pagina_val,
    }))
}

/// GET /api/v1/consejos/{id} — Obtener un consejo por su UUID.
///
/// Endpoint publico — no requiere autenticacion.
///
/// # Por que `PathParam<String>` y no `PathParam<Uuid>`
/// Salvo puede extraer el path param como `String` y nosotros lo parseamos
/// manualmente a `Uuid`. Esto nos da un mensaje de error controlado en lugar
/// de un error generico del framework cuando el UUID tiene formato invalido.
#[endpoint(tags("Consejos"))]
pub async fn obtener_consejo(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Parseamos el String del path param como UUID con manejo de error explicito
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de consejo no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaConsejoRepo>()
        .map_err(|_| AppError::Internal("ConsejoRepo no disponible".into()))?
        .clone();

    let consejo = repo.obtener_consejo(uuid).await?;

    Ok(Json(serde_json::to_value(consejo).unwrap_or_default()))
}

/// POST /api/v1/consejos — Crear consejo (requiere autenticacion JWT).
///
/// El `id_autor` se extrae del token JWT en el middleware — no se acepta
/// del body para evitar que un usuario publique consejos en nombre de otro.
///
/// # Por que `JsonBody<CrearConsejoRequest>` y no `req.parse_json()`
/// `JsonBody<T>` es el extractor tipado de Salvo para request bodies JSON.
/// Al igual que `QueryParam`, genera documentacion OpenAPI automatica
/// con el esquema del struct `T`. Ademas, si el body no cumple el esquema,
/// Salvo retorna un error 400 antes de que el handler se ejecute.
///
/// # Por que `security(("bearer_auth" = []))`
/// Esta anotacion documenta en OpenAPI que este endpoint requiere un JWT
/// en el header `Authorization: Bearer <token>`. Swagger UI mostrara
/// un candado y permitira introducir el token para probar el endpoint.
#[endpoint(tags("Consejos"), security(("bearer_auth" = [])))]
pub async fn crear_consejo(
    body: JsonBody<CrearConsejoRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Obtener el usuario autenticado del Depot (inyectado por auth_middleware)
    let id_autor = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaConsejoRepo>()
        .map_err(|_| AppError::Internal("ConsejoRepo no disponible".into()))?
        .clone();

    let dto = CrearConsejoDto {
        titulo: body.titulo.clone(),
        cuerpo: body.cuerpo.clone(),
        web: body.web.clone(),
        imagen_url: body.imagen_url.clone(),
        provincias: body.provincias.clone(),
    };

    let consejo = repo.crear_consejo(id_autor, dto).await?;

    Ok(Json(serde_json::to_value(consejo).unwrap_or_default()))
}

/// PUT /api/v1/consejos/{id} — Actualizar consejo (solo el autor original).
///
/// Verifica que el usuario autenticado sea el autor antes de aplicar cambios.
/// Soporta actualizacion parcial — solo se modifican los campos que se envien.
#[endpoint(tags("Consejos"), security(("bearer_auth" = [])))]
pub async fn actualizar_consejo(
    id: PathParam<String>,
    body: JsonBody<ActualizarConsejoRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de consejo no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaConsejoRepo>()
        .map_err(|_| AppError::Internal("ConsejoRepo no disponible".into()))?
        .clone();

    // Verificar que el usuario es el autor del consejo
    let consejo_existente = repo.obtener_consejo(uuid).await?;
    if consejo_existente.id_autor != Some(id_usuario) {
        return Err(AppError::Forbidden);
    }

    let dto = ActualizarConsejoDto {
        titulo: body.titulo.clone(),
        cuerpo: body.cuerpo.clone(),
        web: body.web.clone(),
        imagen_url: body.imagen_url.clone(),
        provincias: body.provincias.clone(),
    };

    let consejo = repo.actualizar_consejo(uuid, dto).await?;

    Ok(Json(serde_json::to_value(consejo).unwrap_or_default()))
}

/// DELETE /api/v1/consejos/{id} — Eliminar consejo (solo el autor original).
///
/// Elimina fisicamente el consejo de la BD. La verificacion de autoria
/// se hace antes de borrar para evitar eliminar contenido de otros usuarios.
#[endpoint(tags("Consejos"), security(("bearer_auth" = [])))]
pub async fn eliminar_consejo(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de consejo no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaConsejoRepo>()
        .map_err(|_| AppError::Internal("ConsejoRepo no disponible".into()))?
        .clone();

    // Verificar que el usuario es el autor antes de eliminar
    let consejo = repo.obtener_consejo(uuid).await?;
    if consejo.id_autor != Some(id_usuario) {
        return Err(AppError::Forbidden);
    }

    repo.eliminar_consejo(uuid).await?;

    Ok(Json(
        serde_json::json!({"mensaje": "Consejo eliminado correctamente"}),
    ))
}
