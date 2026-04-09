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

use salvo::oapi::extract::{JsonBody, PathParam, QueryParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::repositories::geografia_repo::{
    ActualizarComunidadDto, ActualizarOficinaDto, ActualizarProvinciaDto, CrearComunidadDto,
    CrearOficinaDto, CrearProvinciaDto,
};
use crate::repositories::{GeografiaRepo, SeaGeografiaRepo};

// ─── DTOs de request (entrada del cliente) ───────────────────────────────────

/// Body para crear una comunidad autonoma.
///
/// # Por que `#[derive(Deserialize, ToSchema)]`
/// `Deserialize` permite que Salvo convierta el JSON del body en este struct.
/// `ToSchema` permite que Salvo lo incluya en la documentacion OpenAPI
/// (equivalente a un esquema JSON Schema en Swagger UI).
#[derive(Deserialize, ToSchema)]
pub struct CrearComunidadRequest {
    /// Nombre oficial de la comunidad. Ej: "Andalucia".
    pub nombre: Option<String>,
    /// Nombre del servicio regional de empleo. Ej: "SAE".
    pub nombre_servicio_empleo: Option<String>,
    /// URL del portal del servicio de empleo.
    pub web_servicio_empleo: Option<String>,
    /// URL de la pagina de sellado/renovacion de la demanda de empleo.
    pub url_sellado: Option<String>,
}

/// Body para actualizar una comunidad autonoma existente.
///
/// Todos los campos son opcionales: solo se actualizan los campos que llegan
/// con valor. Los que llegan como `null` o ausentes no se modifican.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarComunidadRequest {
    /// Nuevo nombre. Omitir = no modificar.
    pub nombre: Option<String>,
    /// Nuevo nombre del servicio regional. Omitir = no modificar.
    pub nombre_servicio_empleo: Option<String>,
    /// Nueva URL del portal. Omitir = no modificar.
    pub web_servicio_empleo: Option<String>,
    /// Nueva URL de sellado. Omitir = no modificar.
    pub url_sellado: Option<String>,
}

/// Body para crear una provincia.
///
/// # Por que `id` es obligatorio aqui
/// Las provincias usan el codigo INE oficial (1-52) como PK, que NO es
/// auto-incrementado por la base de datos. El cliente debe enviar el
/// codigo INE correcto al crear una provincia.
#[derive(Deserialize, ToSchema)]
pub struct CrearProvinciaRequest {
    /// Codigo INE de la provincia (1-52). Es la clave primaria.
    pub id: i32,
    /// Nombre de la provincia. Ej: "Sevilla".
    pub nombre: Option<String>,
    /// ID de la comunidad autonoma a la que pertenece.
    pub id_comunidad: i32,
    /// Ruta al logo en los assets de la app Flutter.
    pub logo_asset: Option<String>,
}

/// Body para actualizar una provincia existente.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarProvinciaRequest {
    /// Nuevo nombre. Omitir = no modificar.
    pub nombre: Option<String>,
    /// Nueva comunidad autonoma. Omitir = no modificar.
    pub id_comunidad: Option<i32>,
    /// Nuevo logo asset. Omitir = no modificar.
    pub logo_asset: Option<String>,
}

/// Body para crear una oficina SEPE.
///
/// El `id_provincia` viene del path param `{id}` en la ruta, no del body.
/// Aqui solo se incluyen los datos de contacto de la oficina.
#[derive(Deserialize, ToSchema)]
pub struct CrearOficinaRequest {
    /// Telefono de atencion al ciudadano.
    pub telefono: Option<String>,
    /// URL del portal web de la oficina.
    pub web: Option<String>,
    /// URL del catalogo de cursos de formacion provincial.
    pub url_cursos: Option<String>,
    /// URL del servicio de orientacion laboral provincial.
    pub url_orientacion: Option<String>,
}

/// Body para actualizar una oficina SEPE existente.
#[derive(Deserialize, ToSchema)]
pub struct ActualizarOficinaRequest {
    /// Nuevo telefono. Omitir = no modificar.
    pub telefono: Option<String>,
    /// Nueva URL web. Omitir = no modificar.
    pub web: Option<String>,
    /// Nueva URL de cursos. Omitir = no modificar.
    pub url_cursos: Option<String>,
    /// Nueva URL de orientacion. Omitir = no modificar.
    pub url_orientacion: Option<String>,
}

/// Respuesta generica con mensaje de texto.
///
/// # Por que `#[derive(Serialize, ToSchema)]`
/// `Serialize` convierte el struct a JSON para la respuesta HTTP.
/// `ToSchema` lo documenta en Swagger UI.
/// No necesita `Deserialize` porque solo se usa como OUTPUT, nunca como INPUT.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Descripcion de la operacion realizada.
    pub mensaje: String,
}

// ─── Handlers de LECTURA (publicos) ─────────────────────────────────────────

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

// ─── Handlers de ESCRITURA para Comunidades (requieren auth) ─────────────────

/// POST /api/v1/comunidades — Crear una nueva comunidad autonoma (admin).
///
/// Requiere autenticacion JWT. Devuelve el registro creado con su ID asignado.
///
/// # Por que `JsonBody<CrearComunidadRequest>`
/// `JsonBody<T>` es el extractor de Salvo OAPI para el body JSON.
/// Salvo lo documenta en OpenAPI como `requestBody` con el schema de `T`.
/// Equivale a leer `req.body_json::<T>()` pero con documentacion automatica.
///
/// # Por que verificamos `_id_usuario`
/// El middleware de auth ya valido el JWT antes de llegar aqui.
/// Aun asi, extraemos el id del depot para confirmar que el token es valido
/// y que el middleware funciono. Es una doble comprobacion defensiva.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn crear_comunidad(
    body: JsonBody<CrearComunidadRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    let dto = CrearComunidadDto {
        nombre: body.nombre.clone(),
        nombre_servicio_empleo: body.nombre_servicio_empleo.clone(),
        web_servicio_empleo: body.web_servicio_empleo.clone(),
        url_sellado: body.url_sellado.clone(),
    };

    let comunidad = repo.crear_comunidad(dto).await?;

    Ok(Json(serde_json::to_value(comunidad).unwrap_or_default()))
}

/// PUT /api/v1/comunidades/{id} — Actualizar una comunidad autonoma (admin).
///
/// Requiere autenticacion JWT. Solo actualiza los campos enviados en el body.
/// Los campos ausentes o `null` no se modifican en base de datos.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn actualizar_comunidad(
    id: PathParam<i32>,
    body: JsonBody<ActualizarComunidadRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    let dto = ActualizarComunidadDto {
        nombre: body.nombre.clone(),
        nombre_servicio_empleo: body.nombre_servicio_empleo.clone(),
        web_servicio_empleo: body.web_servicio_empleo.clone(),
        url_sellado: body.url_sellado.clone(),
    };

    let comunidad = repo.actualizar_comunidad(*id, dto).await?;

    Ok(Json(serde_json::to_value(comunidad).unwrap_or_default()))
}

/// DELETE /api/v1/comunidades/{id} — Eliminar una comunidad autonoma (admin).
///
/// Requiere autenticacion JWT. Elimina fisicamente el registro.
/// Devuelve 404 si no existe, 200 con mensaje si se elimino correctamente.
///
/// # Atencion: cascada en base de datos
/// La eliminacion de una comunidad puede fallar si tiene provincias asociadas
/// (FK constraint). En ese caso la base de datos devuelve un error de integridad
/// referencial que se convierte en `AppError::Internal`.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn eliminar_comunidad(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    repo.eliminar_comunidad(*id).await?;

    Ok(Json(MensajeResponse {
        mensaje: format!("Comunidad autonoma con id {} eliminada correctamente", *id),
    }))
}

// ─── Handlers de ESCRITURA para Provincias (requieren auth) ──────────────────

/// POST /api/v1/provincias — Crear una nueva provincia (admin).
///
/// Requiere autenticacion JWT. El `id` (codigo INE) debe enviarse en el body.
/// Devuelve 409 si ya existe una provincia con ese codigo INE.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn crear_provincia(
    body: JsonBody<CrearProvinciaRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    let dto = CrearProvinciaDto {
        id: body.id,
        nombre: body.nombre.clone(),
        id_comunidad: body.id_comunidad,
        logo_asset: body.logo_asset.clone(),
    };

    let provincia = repo.crear_provincia(dto).await?;

    Ok(Json(serde_json::to_value(provincia).unwrap_or_default()))
}

/// PUT /api/v1/provincias/{id} — Actualizar una provincia (admin).
///
/// Requiere autenticacion JWT. El `id` es el codigo INE (path param).
/// Solo actualiza los campos enviados en el body.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn actualizar_provincia(
    id: PathParam<i32>,
    body: JsonBody<ActualizarProvinciaRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    let dto = ActualizarProvinciaDto {
        nombre: body.nombre.clone(),
        id_comunidad: body.id_comunidad,
        logo_asset: body.logo_asset.clone(),
    };

    let provincia = repo.actualizar_provincia(*id, dto).await?;

    Ok(Json(serde_json::to_value(provincia).unwrap_or_default()))
}

/// DELETE /api/v1/provincias/{id} — Eliminar una provincia (admin).
///
/// Requiere autenticacion JWT. Elimina fisicamente el registro.
/// Devuelve 404 si no existe la provincia con ese codigo INE.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn eliminar_provincia(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    repo.eliminar_provincia(*id).await?;

    Ok(Json(MensajeResponse {
        mensaje: format!("Provincia con id {} eliminada correctamente", *id),
    }))
}

// ─── Handlers de ESCRITURA para Oficinas SEPE (requieren auth) ───────────────

/// POST /api/v1/provincias/{id}/oficina — Crear oficina SEPE para una provincia (admin).
///
/// Requiere autenticacion JWT. El `{id}` del path es el codigo INE de la provincia.
/// Devuelve 409 si ya existe una oficina para esa provincia.
///
/// # Por que el id de provincia viene del path y no del body
/// La URL `/provincias/{id}/oficina` ya expresa el contexto: estamos operando
/// sobre la oficina de UNA provincia especifica. Poner `id_provincia` tambien
/// en el body seria redundante y podria generar inconsistencias si difieren.
/// El path param es la fuente de verdad.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn crear_oficina(
    id: PathParam<i32>,
    body: JsonBody<CrearOficinaRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    let dto = CrearOficinaDto {
        id_provincia: *id,
        telefono: body.telefono.clone(),
        web: body.web.clone(),
        url_cursos: body.url_cursos.clone(),
        url_orientacion: body.url_orientacion.clone(),
    };

    let oficina = repo.crear_oficina(dto).await?;

    Ok(Json(serde_json::to_value(oficina).unwrap_or_default()))
}

/// PUT /api/v1/provincias/{id}/oficina — Actualizar la oficina SEPE de una provincia (admin).
///
/// Requiere autenticacion JWT. El `{id}` del path es el codigo INE de la provincia.
/// Solo actualiza los campos enviados en el body.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn actualizar_oficina(
    id: PathParam<i32>,
    body: JsonBody<ActualizarOficinaRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    let dto = ActualizarOficinaDto {
        telefono: body.telefono.clone(),
        web: body.web.clone(),
        url_cursos: body.url_cursos.clone(),
        url_orientacion: body.url_orientacion.clone(),
    };

    let oficina = repo.actualizar_oficina(*id, dto).await?;

    Ok(Json(serde_json::to_value(oficina).unwrap_or_default()))
}

/// DELETE /api/v1/provincias/{id}/oficina — Eliminar la oficina SEPE de una provincia (admin).
///
/// Requiere autenticacion JWT. El `{id}` del path es el codigo INE de la provincia.
/// Devuelve 404 si no existe oficina para esa provincia.
#[endpoint(tags("Geografia"), security(("bearer_auth" = [])))]
pub async fn eliminar_oficina(
    id: PathParam<i32>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let _id_usuario = depot
        .get::<uuid::Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaGeografiaRepo>()
        .map_err(|_| AppError::Internal("Repositorio de geografia no disponible".into()))?
        .clone();

    repo.eliminar_oficina(*id).await?;

    Ok(Json(MensajeResponse {
        mensaje: format!(
            "Oficina SEPE de la provincia con id {} eliminada correctamente",
            *id
        ),
    }))
}
