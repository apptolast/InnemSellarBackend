//! Handlers de votos: crear/actualizar, consultar y eliminar votos.
//!
//! La tabla `votos` es **polimorfica**: un mismo usuario puede votar
//! ofertas, consejos y cursos. El campo `tipo_contenido` indica a que
//! tabla apunta `id_contenido`.
//!
//! Todos los endpoints requieren autenticacion JWT porque un voto siempre
//! esta ligado a un usuario concreto — sin identidad no tiene sentido votar.

use salvo::oapi::extract::{JsonBody, QueryParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::enums::TipoContenido;
use crate::repositories::{SeaVotoRepo, VotoRepo};

// ─── Helpers de parseo ────────────────────────────────────────────

/// Convierte un `String` recibido del cliente en el enum `TipoContenido`.
///
/// # Por que parsear manualmente y no usar el enum directamente en el DTO
/// Los enums de SeaORM (`DeriveActiveEnum`) no implementan `ToSchema` — el
/// trait que Salvo OAPI necesita para documentar los tipos en Swagger.
/// La solucion es recibir el valor como `String` en el DTO (que si implementa
/// `ToSchema`) y convertirlo manualmente aqui. El coste es minimo: una
/// comparacion de strings en cada peticion.
///
/// # Por que `Result<TipoContenido, AppError>` y no `Option`
/// Si el valor es desconocido queremos devolver un error 400 (Bad Request)
/// con un mensaje claro, no un 500 o un comportamiento silencioso.
/// `Result` fuerza al llamante a manejar el error explicitamente gracias al
/// operador `?`, lo que en Dart seria similar a lanzar una excepcion tipada.
fn parsear_tipo_contenido(s: &str) -> Result<TipoContenido, AppError> {
    match s {
        "oferta" => Ok(TipoContenido::Oferta),
        "consejo" => Ok(TipoContenido::Consejo),
        "curso" => Ok(TipoContenido::Curso),
        otro => Err(AppError::BadRequest(format!(
            "tipo_contenido invalido: '{otro}'. Valores validos: oferta, consejo, curso"
        ))),
    }
}

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para POST /api/v1/votos — crear o actualizar un voto.
///
/// # Por que `ToSchema`
/// `ToSchema` es el trait de Salvo OAPI que permite que `#[endpoint]`
/// documente automaticamente la estructura del body en Swagger UI.
/// Sin el, el generador de OpenAPI no sabe que campos acepta este endpoint.
/// Es analogo a las anotaciones `@JsonSerializable` en Dart, pero resuelto
/// en tiempo de compilacion (coste cero en runtime).
///
/// # Por que `tipo_contenido` es `String` y no el enum `TipoContenido`
/// Ver `parsear_tipo_contenido` — los enums de SeaORM no implementan
/// `ToSchema`, asi que usamos `String` en el DTO y parseamos manualmente.
#[derive(Deserialize, ToSchema)]
pub struct VotarRequest {
    /// Tipo de contenido a votar. Valores: `"oferta"`, `"consejo"`, `"curso"`.
    pub tipo_contenido: String,
    /// UUID del contenido a votar (oferta, consejo o curso segun `tipo_contenido`).
    pub id_contenido: Uuid,
    /// Tipo de voto: `1` = upvote, `-1` = downvote.
    pub tipo_voto: i32,
}

/// Body para DELETE /api/v1/votos — eliminar un voto existente.
#[derive(Deserialize, ToSchema)]
pub struct EliminarVotoRequest {
    /// Tipo de contenido del voto a eliminar. Valores: `"oferta"`, `"consejo"`, `"curso"`.
    pub tipo_contenido: String,
    /// UUID del contenido cuyo voto se quiere eliminar.
    pub id_contenido: Uuid,
}

/// Respuesta de confirmacion de operacion exitosa.
///
/// # Por que un struct tipado en vez de `serde_json::json!(...)`
/// Con `ToSchema` podemos documentar la respuesta en OpenAPI.
/// `serde_json::json!` genera un `Value` opaco — Swagger no puede
/// describir su estructura. Este struct resuelve esa limitacion.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Descripcion textual de la operacion realizada con exito.
    pub mensaje: String,
}

// ─── Handlers ────────────────────────────────────────────────────

/// POST /api/v1/votos — Votar un contenido (crear o actualizar voto).
///
/// Si el usuario ya habia votado ese contenido, el voto anterior se
/// actualiza con el nuevo valor. La tabla `votos` usa una clave primaria
/// compuesta `(id_usuario, tipo_contenido, id_contenido)` para garantizar
/// que cada usuario solo tenga un voto por contenido.
///
/// El trigger de PostgreSQL `trg_votos_contadores` actualiza automaticamente
/// los campos `cantidad_upvotes` / `cantidad_downvotes` en la tabla de
/// contenido correspondiente tras cada operacion de voto.
///
/// # Por que `#[endpoint]` en vez de `#[handler]`
/// `#[endpoint]` genera metadata OpenAPI en tiempo de compilacion ademas
/// de registrar el handler HTTP. `#[handler]` solo hace lo segundo.
/// El comportamiento en runtime es identico.
///
/// # Por que `JsonBody<VotarRequest>` en vez de `req.parse_json()`
/// `JsonBody<T>` es un extractor tipado de Salvo OAPI. Al aparecer en
/// la firma de la funcion, Salvo sabe que este endpoint espera un cuerpo
/// JSON de tipo `VotarRequest` y lo documenta en Swagger automaticamente.
#[endpoint(tags("Votos"), security(("bearer_auth" = [])))]
pub async fn votar(
    body: JsonBody<VotarRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Extraer el UUID del usuario autenticado inyectado por el middleware JWT.
    // `depot` es el almacen de datos por-peticion de Salvo — similar al
    // InheritedWidget de Flutter: datos que fluyen hacia abajo en el arbol
    // de handlers sin pasarlos explicitamente en cada llamada.
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    // Validar el tipo de voto antes de ir a la BD — fail fast.
    if body.tipo_voto != 1 && body.tipo_voto != -1 {
        return Err(AppError::BadRequest(
            "tipo_voto debe ser 1 (upvote) o -1 (downvote)".into(),
        ));
    }

    // Parsear el string a enum — error 400 si el valor no es valido.
    let tipo = parsear_tipo_contenido(&body.tipo_contenido)?;

    // Obtener el repositorio del Depot.
    // `obtain::<T>()` busca una referencia del tipo T registrado en el Depot.
    // En Rust, `clone()` aqui es barato: `SeaVotoRepo` solo contiene
    // un `Arc<DatabaseConnection>` internamente, asi que clonar copia el
    // puntero atomico, no la conexion entera.
    let repo = depot
        .obtain::<SeaVotoRepo>()
        .map_err(|_| AppError::Internal("VotoRepo no disponible".into()))?
        .clone();

    let voto = repo
        .votar(id_usuario, tipo, body.id_contenido, body.tipo_voto)
        .await?;

    Ok(Json(serde_json::to_value(voto).unwrap_or_default()))
}

/// GET /api/v1/votos — Obtener el voto del usuario autenticado sobre un contenido.
///
/// Devuelve el voto actual (`tipo_voto`: 1 o -1) si existe, o 404 si
/// el usuario no ha votado ese contenido todavia.
///
/// Util en el cliente para saber si mostrar el boton de upvote o downvote
/// como activo cuando el usuario carga la pantalla de detalle.
///
/// # Parametros de query
/// - `tipo_contenido`: `"oferta"`, `"consejo"` o `"curso"`
/// - `id_contenido`: UUID del contenido a consultar
///
/// # Por que `QueryParam<String, true>` y no `req.query::<String>(...)`
/// `QueryParam<T, REQUIRED>` es el extractor tipado de Salvo OAPI para
/// parametros de query. El segundo parametro generico (`true`) indica que
/// el parametro es obligatorio — Salvo devolvera 400 si falta.
/// Usando este extractor, Swagger UI los documenta automaticamente.
#[endpoint(tags("Votos"), security(("bearer_auth" = [])))]
pub async fn obtener_voto(
    tipo_contenido: QueryParam<String, true>,
    id_contenido: QueryParam<String, true>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    // Parsear tipo_contenido de String a enum
    let tipo = parsear_tipo_contenido(&tipo_contenido)?;

    // Parsear id_contenido de String a UUID
    // `into_inner()` extrae el valor del wrapper `QueryParam<T, REQUIRED>`.
    let uuid = Uuid::parse_str(&id_contenido)
        .map_err(|_| AppError::BadRequest("id_contenido no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaVotoRepo>()
        .map_err(|_| AppError::Internal("VotoRepo no disponible".into()))?
        .clone();

    // `obtener_voto` devuelve `Result<Option<Model>, AppError>`.
    // `Option` es el equivalente de Rust a `T?` en Dart — puede ser
    // `Some(valor)` o `None`. Lo mapeamos a 404 si no existe.
    let voto = repo
        .obtener_voto(id_usuario, tipo, uuid)
        .await?
        .ok_or_else(|| AppError::NotFound("El usuario no ha votado este contenido".into()))?;

    Ok(Json(serde_json::to_value(voto).unwrap_or_default()))
}

/// DELETE /api/v1/votos — Eliminar el voto del usuario sobre un contenido.
///
/// Solo puede eliminar su propio voto — el `id_usuario` se toma del JWT,
/// no del body, para evitar que un usuario borre votos de otros.
/// El trigger de PostgreSQL actualiza los contadores automaticamente.
#[endpoint(tags("Votos"), security(("bearer_auth" = [])))]
pub async fn eliminar_voto(
    body: JsonBody<EliminarVotoRequest>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let id_usuario = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let tipo = parsear_tipo_contenido(&body.tipo_contenido)?;

    let repo = depot
        .obtain::<SeaVotoRepo>()
        .map_err(|_| AppError::Internal("VotoRepo no disponible".into()))?
        .clone();

    repo.eliminar_voto(id_usuario, tipo, body.id_contenido)
        .await?;

    Ok(Json(MensajeResponse {
        mensaje: "Voto eliminado correctamente".into(),
    }))
}

// ─── Tests unitarios ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifica que `parsear_tipo_contenido` acepta todos los valores validos
    /// del enum y rechaza los desconocidos con un error 400.
    ///
    /// # Por que `#[test]` y no un test de integracion
    /// Esta funcion es pura (sin IO, sin BD) — el test unitario es suficiente
    /// y mucho mas rapido que levantar un servidor de prueba.
    #[test]
    fn parsear_tipo_contenido_valores_validos() {
        assert!(matches!(
            parsear_tipo_contenido("oferta"),
            Ok(TipoContenido::Oferta)
        ));
        assert!(matches!(
            parsear_tipo_contenido("consejo"),
            Ok(TipoContenido::Consejo)
        ));
        assert!(matches!(
            parsear_tipo_contenido("curso"),
            Ok(TipoContenido::Curso)
        ));
    }

    #[test]
    fn parsear_tipo_contenido_valor_invalido_devuelve_error() {
        // `matches!` verifica que el patron coincide sin hacer unwrap.
        // Es la forma idiomatica de testear variantes de Result/Option en Rust.
        assert!(matches!(
            parsear_tipo_contenido("desconocido"),
            Err(AppError::BadRequest(_))
        ));
    }
}
