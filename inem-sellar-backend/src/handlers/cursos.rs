//! CRUD de cursos de formacion.
//!
//! GET = publico, POST/PUT/DELETE = requiere autenticacion JWT.
//!
//! Los handlers usan `#[endpoint]` (en lugar del antiguo `#[handler]`) para
//! generacion automatica de documentacion OpenAPI. La diferencia clave es que
//! `#[endpoint]` analiza los tipos de los parametros en tiempo de compilacion
//! y construye el esquema OpenAPI sin necesidad de escribirlo a mano.
//!
//! Los endpoints protegidos llevan `security(("bearer_auth" = []))`, lo que
//! indica a Swagger UI que muestren un candado y permitan introducir un JWT
//! para probar el endpoint directamente desde la documentacion.

use salvo::oapi::extract::{JsonBody, PathParam, QueryParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::curso_repo::{ActualizarCursoDto, CrearCursoDto};
use crate::repositories::{CursoRepo, SeaCursoRepo};

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para crear un nuevo curso de formacion.
///
/// # Por que `ToSchema`
/// El derive `ToSchema` le indica a Salvo que genere el esquema JSON Schema
/// de este struct automaticamente. Asi Swagger UI puede mostrar un ejemplo
/// del body con todos los campos y sus tipos — equivalente a documentacion
/// de parametros en Flutter/Dart pero generada 100% desde el codigo Rust.
///
/// # Por que todos los campos son `Option<T>`
/// En Rust, un campo `Option<String>` puede ser `Some("valor")` o `None`
/// (ausente). Esto modela publicaciones parciales: el usuario no tiene por
/// que rellenar todos los campos al crear un curso.
/// En Dart usarias `String?` — aqui es exactamente lo mismo.
#[derive(Deserialize, ToSchema)]
pub struct CrearCursoRequest {
    /// Titulo del curso. Ej: "Introduccion a la Ciberseguridad".
    pub titulo: Option<String>,
    /// Descripcion breve del curso (resumen de una o dos frases).
    pub descripcion: Option<String>,
    /// Contenido detallado o temario del curso (puede ser largo).
    pub contenido: Option<String>,
    /// Enlace a la pagina web del curso o entidad organizadora.
    pub web: Option<String>,
    /// URL de una imagen representativa del curso.
    pub imagen_url: Option<String>,
    /// Duracion total del curso en horas. Ej: 40.
    pub duracion_horas: Option<i32>,
    /// Fecha de inicio en formato ISO 8601: "YYYY-MM-DD". Ej: "2026-05-01".
    pub fecha_inicio: Option<String>,
    /// Fecha de fin en formato ISO 8601: "YYYY-MM-DD". Ej: "2026-06-30".
    pub fecha_fin: Option<String>,
    /// Indica si el curso esta homologado oficialmente por la administracion.
    pub curso_homologado: Option<bool>,
    /// Telefono de contacto para informacion sobre el curso.
    pub telefono_contacto: Option<String>,
    /// Email de contacto para inscripciones o consultas.
    pub email_contacto: Option<String>,
    /// Lista de IDs de provincias donde se imparte el curso (codigos INE).
    /// Lista vacia = curso disponible en toda Espana (nacional).
    ///
    /// # Por que `#[serde(default)]`
    /// Si el cliente no envia el campo `provincias` en el JSON, serde usara
    /// el valor por defecto de `Vec<i32>`, que es un vector vacio `[]`.
    /// Sin esta anotacion, la deserializacion fallaria si falta el campo.
    #[serde(default)]
    pub provincias: Vec<i32>,
}

/// Body para actualizar un curso existente.
///
/// Solo el autor original puede actualizar el curso.
/// Unicamente se modifican los campos que se incluyan en el JSON
/// (partial update / PATCH semantics sobre un endpoint PUT).
///
/// # Por que `Option<Vec<i32>>` para provincias
/// Distingue tres estados posibles:
/// - Campo ausente (`None`): no tocar las provincias actuales.
/// - Campo presente con valores (`Some([1, 2])`): reemplazar todas las provincias.
/// - Campo presente y vacio (`Some([])`): eliminar todas las provincias.
///
/// Un simple `Vec<i32>` no podria representar "no tocar" vs "eliminar todas".
#[derive(Deserialize, ToSchema)]
pub struct ActualizarCursoRequest {
    /// Nuevo titulo del curso (si no se envia, el titulo no cambia).
    pub titulo: Option<String>,
    /// Nueva descripcion breve.
    pub descripcion: Option<String>,
    /// Nuevo contenido detallado.
    pub contenido: Option<String>,
    /// Nueva URL de la pagina web del curso.
    pub web: Option<String>,
    /// Nueva URL de imagen.
    pub imagen_url: Option<String>,
    /// Nueva duracion en horas.
    pub duracion_horas: Option<i32>,
    /// Nueva fecha de inicio "YYYY-MM-DD".
    pub fecha_inicio: Option<String>,
    /// Nueva fecha de fin "YYYY-MM-DD".
    pub fecha_fin: Option<String>,
    /// Nuevo valor de homologacion.
    pub curso_homologado: Option<bool>,
    /// Nuevo telefono de contacto.
    pub telefono_contacto: Option<String>,
    /// Nuevo email de contacto.
    pub email_contacto: Option<String>,
    /// Nuevas provincias asociadas. Si se envia, reemplaza las existentes.
    /// No enviar este campo = no tocar provincias. Enviar `[]` = eliminar todas.
    pub provincias: Option<Vec<i32>>,
}

/// Respuesta paginada del listado de cursos.
///
/// # Por que `Vec<serde_json::Value>` en lugar de `Vec<curso::Model>`
/// Los cursos pueden tener datos agregados dinamicamente (provincias, voto
/// del usuario actual) que no estan en el Model base de SeaORM. Usando
/// `serde_json::Value` tenemos flexibilidad para enriquecer la respuesta
/// sin crear structs adicionales. La contrapartida es que OpenAPI no puede
/// documentar la estructura interna de cada curso — limitacion conocida y aceptada.
#[derive(Serialize, ToSchema)]
pub struct ListaCursosResponse {
    /// Lista de cursos de la pagina actual.
    pub cursos: Vec<serde_json::Value>,
    /// Total de cursos que coinciden con el filtro (para calcular el numero de paginas).
    pub total: u64,
    /// Numero de pagina actual, base 1.
    pub pagina: u64,
    /// Numero de cursos incluidos por pagina.
    pub por_pagina: u64,
}

// ─── Helper ──────────────────────────────────────────────────────

/// Convierte un string con formato "YYYY-MM-DD" a `chrono::NaiveDate`.
///
/// # Por que una funcion helper separada
/// La logica de parseo de fechas se repite en `crear_curso` y `actualizar_curso`.
/// Extraerla a una funcion evita duplicacion (principio DRY) y centraliza
/// el mensaje de error para que sea consistente en ambos endpoints.
///
/// `NaiveDate` es la representacion de chrono para una fecha sin zona horaria
/// (solo dia, mes, anho). Es lo que espera SeaORM para columnas `DATE` en PostgreSQL.
fn parsear_fecha(s: &str) -> Result<chrono::NaiveDate, AppError> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest(format!("Fecha invalida: {s}. Formato esperado: YYYY-MM-DD"))
    })
}

// ─── Handlers ────────────────────────────────────────────────────

/// GET /api/v1/cursos — Listar cursos activos con paginacion y filtro opcional por provincia.
///
/// Endpoint publico — no requiere autenticacion.
///
/// # Por que `QueryParam<i32, false>` y no `req.query::<i32>(...)`
/// `QueryParam<T, false>` es un extractor tipado de Salvo que:
/// 1. Deserializa el query param automaticamente al tipo `T`.
/// 2. El segundo argumento `false` indica que el parametro es OPCIONAL
///    (si fuera `true` seria obligatorio y devolveria 400 si falta).
/// 3. Salvo lo incluye en el esquema OpenAPI para que Swagger UI muestre
///    los parametros de query con sus tipos.
///
/// # Parametros de query (todos opcionales)
/// - `id_provincia`: filtrar por provincia (codigo INE 1-52)
/// - `pagina`: numero de pagina, base 1 (default: 1)
/// - `por_pagina`: resultados por pagina (default: 20)
#[endpoint(tags("Cursos"))]
pub async fn listar_cursos(
    id_provincia: QueryParam<i32, false>,
    pagina: QueryParam<u64, false>,
    por_pagina: QueryParam<u64, false>,
    depot: &mut Depot,
) -> Result<Json<ListaCursosResponse>, AppError> {
    // `into_inner()` extrae el valor del wrapper QueryParam.
    // `unwrap_or` proporciona el valor por defecto cuando el param no se envio.
    let pagina_val = pagina.into_inner().unwrap_or(1);
    let por_pagina_val = por_pagina.into_inner().unwrap_or(20);

    let repo = depot
        .obtain::<SeaCursoRepo>()
        .map_err(|_| AppError::Internal("CursoRepo no disponible".into()))?
        .clone();

    let (cursos, total) = repo
        .listar_cursos(id_provincia.into_inner(), pagina_val, por_pagina_val)
        .await?;

    // Convertimos cada Model de SeaORM a serde_json::Value para la respuesta.
    // `unwrap_or_default()` devuelve `null` si la serializacion falla (nunca deberia).
    let cursos_json: Vec<serde_json::Value> = cursos
        .into_iter()
        .map(|c| serde_json::to_value(c).unwrap_or_default())
        .collect();

    Ok(Json(ListaCursosResponse {
        cursos: cursos_json,
        total,
        pagina: pagina_val,
        por_pagina: por_pagina_val,
    }))
}

/// GET /api/v1/cursos/{id} — Obtener un curso por su UUID.
///
/// Endpoint publico — no requiere autenticacion.
///
/// # Por que `PathParam<String>` y no directamente `PathParam<Uuid>`
/// Salvo puede extraer el path param como `String` de forma segura y sin
/// dependencias extra. Luego parseamos el UUID manualmente con `Uuid::parse_str`
/// para producir un error 400 descriptivo si el formato no es correcto.
/// Esto da mas control sobre el mensaje de error que dejar que Salvo falle internamente.
#[endpoint(tags("Cursos"))]
pub async fn obtener_curso(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // El operador `?` propaga el error si el UUID no es valido, retornando 400 Bad Request.
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de curso no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaCursoRepo>()
        .map_err(|_| AppError::Internal("CursoRepo no disponible".into()))?
        .clone();

    let curso = repo.obtener_curso(uuid).await?;

    Ok(Json(serde_json::to_value(curso).unwrap_or_default()))
}

/// POST /api/v1/cursos — Crear un nuevo curso (requiere autenticacion JWT).
///
/// El `id_autor` se extrae del JWT en el middleware de autenticacion —
/// nunca se acepta del body para evitar que un usuario publique contenido
/// en nombre de otro.
///
/// # Por que `JsonBody<CrearCursoRequest>`
/// `JsonBody<T>` es el extractor tipado de Salvo para el body de la peticion.
/// Deserializa el JSON automaticamente al tipo `T` y genera el esquema en OpenAPI.
/// Equivale a `req.parse_json::<T>().await` del antiguo `#[handler]`, pero
/// con documentacion automatica incluida.
///
/// # Por que `security(("bearer_auth" = []))`
/// Esta anotacion indica a OpenAPI que este endpoint requiere autenticacion JWT.
/// Swagger UI mostrara un candado y permitira introducir el token Bearer para
/// probar el endpoint directamente desde la documentacion.
#[endpoint(tags("Cursos"), security(("bearer_auth" = [])))]
pub async fn crear_curso(
    body: JsonBody<CrearCursoRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // `depot.get::<Uuid>("id_usuario")` recupera el UUID inyectado por auth_middleware.
    // El `*` dereferencia el `&Uuid` a `Uuid` (Copy trait — no necesita clone).
    let id_autor = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaCursoRepo>()
        .map_err(|_| AppError::Internal("CursoRepo no disponible".into()))?
        .clone();

    // Construimos el DTO (Data Transfer Object) que el repositorio espera.
    // Las fechas vienen como String en el JSON y las convertimos a NaiveDate aqui,
    // antes de pasarlas al repositorio que solo conoce tipos de dominio.
    //
    // `as_deref()` convierte `Option<String>` a `Option<&str>` sin consumir el valor.
    // `map(parsear_fecha)` aplica la conversion a cada `&str` si existe.
    // `transpose()` convierte `Option<Result<T, E>>` a `Result<Option<T>, E>`,
    // propagando el error con `?` si la fecha es invalida.
    let dto = CrearCursoDto {
        titulo: body.titulo.clone(),
        descripcion: body.descripcion.clone(),
        contenido: body.contenido.clone(),
        web: body.web.clone(),
        imagen_url: body.imagen_url.clone(),
        duracion_horas: body.duracion_horas,
        fecha_inicio: body
            .fecha_inicio
            .as_deref()
            .map(parsear_fecha)
            .transpose()?,
        fecha_fin: body.fecha_fin.as_deref().map(parsear_fecha).transpose()?,
        curso_homologado: body.curso_homologado,
        telefono_contacto: body.telefono_contacto.clone(),
        email_contacto: body.email_contacto.clone(),
        provincias: body.provincias.clone(),
    };

    let curso = repo.crear_curso(id_autor, dto).await?;

    Ok(Json(serde_json::to_value(curso).unwrap_or_default()))
}

/// PUT /api/v1/cursos/{id} — Actualizar un curso (solo el autor original).
///
/// Verifica que el usuario autenticado sea el autor del curso antes de
/// aplicar cualquier cambio. Si no lo es, devuelve 403 Forbidden.
///
/// Solo se modifican los campos que se incluyan en el JSON enviado.
/// El campo `provincias` tiene semantica especial:
/// - Ausente: no se tocan las provincias actuales.
/// - `[]`: se eliminan todas las provincias (curso nacional).
/// - `[1, 2, 3]`: se reemplazan las provincias por las indicadas.
#[endpoint(tags("Cursos"), security(("bearer_auth" = [])))]
pub async fn actualizar_curso(
    id: PathParam<String>,
    body: JsonBody<ActualizarCursoRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de curso no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaCursoRepo>()
        .map_err(|_| AppError::Internal("CursoRepo no disponible".into()))?
        .clone();

    // Verificar autoria: solo el creador del curso puede modificarlo.
    // `id_autor` es `Option<Uuid>` en el modelo — comparamos con `Some(id_usuario)`.
    let curso_existente = repo.obtener_curso(uuid).await?;
    if curso_existente.id_autor != Some(id_usuario) {
        return Err(AppError::Forbidden);
    }

    let dto = ActualizarCursoDto {
        titulo: body.titulo.clone(),
        descripcion: body.descripcion.clone(),
        contenido: body.contenido.clone(),
        web: body.web.clone(),
        imagen_url: body.imagen_url.clone(),
        duracion_horas: body.duracion_horas,
        fecha_inicio: body
            .fecha_inicio
            .as_deref()
            .map(parsear_fecha)
            .transpose()?,
        fecha_fin: body.fecha_fin.as_deref().map(parsear_fecha).transpose()?,
        curso_homologado: body.curso_homologado,
        telefono_contacto: body.telefono_contacto.clone(),
        email_contacto: body.email_contacto.clone(),
        // Pasamos las provincias tal cual vienen del body:
        // - None si el campo no estaba en el JSON => el repo no toca las provincias.
        // - Some(vec) si el campo estaba => el repo reemplaza las provincias.
        provincias: body.provincias.clone(),
    };

    let curso = repo.actualizar_curso(uuid, dto).await?;

    Ok(Json(serde_json::to_value(curso).unwrap_or_default()))
}

/// DELETE /api/v1/cursos/{id} — Eliminar un curso (solo el autor original).
///
/// Elimina fisicamente el curso de la base de datos junto con todas sus
/// relaciones de provincia (gracias a `ON DELETE CASCADE` en el esquema SQL).
///
/// La verificacion de autoria se realiza ANTES del borrado para evitar
/// eliminar contenido de otros usuarios por error.
#[endpoint(tags("Cursos"), security(("bearer_auth" = [])))]
pub async fn eliminar_curso(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de curso no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaCursoRepo>()
        .map_err(|_| AppError::Internal("CursoRepo no disponible".into()))?
        .clone();

    // Comprobar autoria antes de eliminar.
    let curso = repo.obtener_curso(uuid).await?;
    if curso.id_autor != Some(id_usuario) {
        return Err(AppError::Forbidden);
    }

    repo.eliminar_curso(uuid).await?;

    Ok(Json(
        serde_json::json!({"mensaje": "Curso eliminado correctamente"}),
    ))
}
