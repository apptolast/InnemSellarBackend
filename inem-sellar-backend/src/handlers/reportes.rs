//! Handlers de reportes: crear, listar y procesar reportes de contenido.
//!
//! Un reporte es la forma en que los usuarios notifican contenido inapropiado
//! (spam, incorrecto, duplicado, etc.) a los moderadores. La tabla `reportes`
//! es polimorfica — un mismo reporte puede referirse a una oferta, un consejo
//! o un curso, identificado por el par `(tipo_contenido, id_contenido)`.
//!
//! Flujo de moderacion:
//!   1. Usuario crea un reporte (`crear_reporte`) — estado: `pendiente`
//!   2. Admin lista reportes pendientes (`listar_reportes_pendientes`)
//!   3. Admin acepta o rechaza (`procesar_reporte`) — estado: `aceptado` | `rechazado`

use salvo::oapi::extract::{JsonBody, PathParam};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::enums::{MotivoReporte, TipoContenido};
use crate::repositories::reporte_repo::CrearReporteDto;
use crate::repositories::{ReporteRepo, SeaReporteRepo};

// ─── Helpers de parseo ────────────────────────────────────────────

/// Convierte un `String` recibido del cliente en el enum `TipoContenido`.
///
/// # Por que `String` en el DTO en lugar del enum directamente
/// Los enums de SeaORM (`DeriveActiveEnum`) no implementan el trait `ToSchema`
/// de Salvo OAPI. Sin `ToSchema`, `#[endpoint]` no puede generar la
/// documentacion OpenAPI del campo. Solución: recibir como `String` y
/// convertir aqui. Es el mismo patron que usa `votos.rs`.
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

/// Convierte un `String` recibido del cliente en el enum `MotivoReporte`.
///
/// `match` en Rust es exhaustivo — el compilador exige que cubras todos los
/// casos posibles. Aqui el caso `otro` actua como el "else" que captura
/// cualquier string no reconocido y devuelve un error descriptivo.
fn parsear_motivo(s: &str) -> Result<MotivoReporte, AppError> {
    match s {
        "spam" => Ok(MotivoReporte::Spam),
        "inapropiado" => Ok(MotivoReporte::Inapropiado),
        "desactualizado" => Ok(MotivoReporte::Desactualizado),
        "incorrecto" => Ok(MotivoReporte::Incorrecto),
        "duplicado" => Ok(MotivoReporte::Duplicado),
        "otro" => Ok(MotivoReporte::Otro),
        otro => Err(AppError::BadRequest(format!(
            "motivo invalido: '{otro}'. Valores validos: spam, inapropiado, desactualizado, incorrecto, duplicado, otro"
        ))),
    }
}

// ─── DTOs ────────────────────────────────────────────────────────

/// Body para POST /api/v1/reportes — reportar un contenido.
///
/// # Por que `ToSchema`
/// Permite que Salvo OAPI documente automaticamente la estructura del body
/// en Swagger UI. Sin el, el generador de OpenAPI no puede describir los
/// campos esperados para este endpoint.
///
/// # Por que `Deserialize`
/// `Deserialize` (de serde) genera el codigo para convertir el JSON de la
/// peticion en este struct de Rust. Es el equivalente del `fromJson()` en
/// Dart, pero generado por una macro en tiempo de compilacion.
#[derive(Deserialize, ToSchema)]
pub struct CrearReporteRequest {
    /// Tipo de contenido reportado. Valores: `"oferta"`, `"consejo"`, `"curso"`.
    pub tipo_contenido: String,
    /// UUID del contenido que se reporta.
    pub id_contenido: Uuid,
    /// Motivo del reporte. Valores: `"spam"`, `"inapropiado"`, `"desactualizado"`,
    /// `"incorrecto"`, `"duplicado"`, `"otro"`.
    pub motivo: String,
    /// Explicacion adicional del reporte (opcional).
    pub detalle_motivo: Option<String>,
}

/// Body para PUT /api/v1/reportes/{id} — procesar un reporte (aceptar o rechazar).
///
/// Solo los administradores deberan llamar a este endpoint. La verificacion
/// de rol admin es una mejora futura — actualmente solo requiere autenticacion.
#[derive(Deserialize, ToSchema)]
pub struct ProcesarReporteRequest {
    /// `true` = aceptar el reporte (contenido sera moderado).
    /// `false` = rechazar el reporte (contenido se mantiene).
    pub aceptar: bool,
}

/// Respuesta de confirmacion de operacion exitosa.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Descripcion textual de la operacion realizada con exito.
    pub mensaje: String,
}

// ─── Handlers ────────────────────────────────────────────────────

/// POST /api/v1/reportes — Reportar un contenido.
///
/// Un usuario solo puede reportar el mismo contenido una vez — la BD tiene
/// una restriccion `UNIQUE (tipo_contenido, id_contenido, id_reportero)`.
/// Si intenta reportar dos veces el mismo contenido, recibe un error 409 Conflict.
///
/// El `id_reportero` se extrae del JWT, no del body, para evitar suplantacion.
///
/// # Por que `#[endpoint]` y `JsonBody<T>`
/// `#[endpoint]` genera metadata OpenAPI ademas de registrar el handler HTTP.
/// `JsonBody<T>` es el extractor tipado que Salvo OAPI necesita para documentar
/// el body en Swagger. Ver `auth.rs` para una explicacion mas detallada del patron.
#[endpoint(tags("Reportes"), security(("bearer_auth" = [])))]
pub async fn crear_reporte(
    body: JsonBody<CrearReporteRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Extraer el UUID del reportero del Depot (inyectado por el middleware JWT).
    // `depot.get::<T>(key)` devuelve `Result<&T, _>`. El `*` desreferencia
    // la referencia para obtener el `Uuid` por valor (es `Copy`, no `Clone`).
    let id_reportero = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    // Parsear los campos String a sus enums correspondientes.
    // Si cualquiera falla, el operador `?` propaga el AppError::BadRequest
    // al caller (Salvo), que lo convierte en una respuesta 400.
    let tipo = parsear_tipo_contenido(&body.tipo_contenido)?;
    let motivo = parsear_motivo(&body.motivo)?;

    let repo = depot
        .obtain::<SeaReporteRepo>()
        .map_err(|_| AppError::Internal("ReporteRepo no disponible".into()))?
        .clone();

    let dto = CrearReporteDto {
        tipo_contenido: tipo,
        id_contenido: body.id_contenido,
        motivo,
        detalle_motivo: body.detalle_motivo.clone(),
    };

    let reporte = repo.crear_reporte(id_reportero, dto).await?;

    Ok(Json(serde_json::to_value(reporte).unwrap_or_default()))
}

/// GET /api/v1/reportes/pendientes — Listar reportes pendientes de revision.
///
/// Endpoint de administracion — devuelve todos los reportes cuyo estado
/// es `pendiente`. En el futuro se anadira verificacion de rol admin.
/// Por ahora requiere autenticacion valida.
#[endpoint(tags("Reportes"), security(("bearer_auth" = [])))]
pub async fn listar_reportes_pendientes(
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verificar autenticacion — el middleware ya lo comprueba, pero este
    // `get` hace la verificacion explicita en el handler para claridad.
    let _id_usuario = depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let repo = depot
        .obtain::<SeaReporteRepo>()
        .map_err(|_| AppError::Internal("ReporteRepo no disponible".into()))?
        .clone();

    let reportes = repo.listar_reportes_pendientes().await?;

    Ok(Json(serde_json::to_value(reportes).unwrap_or_default()))
}

/// PUT /api/v1/reportes/{id} — Procesar un reporte: aceptar o rechazar.
///
/// Registra quien proceso el reporte (`id_procesador` del JWT) y cuando
/// (`procesado_en` se guarda en el repositorio). El estado del reporte pasa
/// de `pendiente` a `aceptado` o `rechazado` segun el campo `aceptar`.
///
/// # Por que `PathParam<String>` en vez de `PathParam<Uuid>`
/// Salvo OAPI puede documentar `PathParam<String>` sin problemas.
/// Convertimos a `Uuid` manualmente para un mensaje de error claro
/// si el formato es invalido, en lugar de dejar que Salvo devuelva
/// un error generico de parseo.
#[endpoint(tags("Reportes"), security(("bearer_auth" = [])))]
pub async fn procesar_reporte(
    id: PathParam<String>,
    body: JsonBody<ProcesarReporteRequest>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    // El procesador es el usuario autenticado (admin en el futuro).
    let id_procesador = *depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    // `into_inner()` extrae el `String` del wrapper `PathParam<String>`.
    // Es el patron idiomatico de Salvo para acceder al valor del extractor.
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de reporte no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaReporteRepo>()
        .map_err(|_| AppError::Internal("ReporteRepo no disponible".into()))?
        .clone();

    let reporte = repo
        .procesar_reporte(uuid, id_procesador, body.aceptar)
        .await?;

    Ok(Json(serde_json::to_value(reporte).unwrap_or_default()))
}

/// GET /api/v1/reportes/{id} — Obtener un reporte por su UUID.
///
/// Requiere autenticacion. Devuelve todos los datos del reporte incluyendo
/// el estado de moderacion, el motivo, y quien lo proceso.
#[endpoint(tags("Reportes"), security(("bearer_auth" = [])))]
pub async fn obtener_reporte(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<serde_json::Value>, AppError> {
    let _id_usuario = depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de reporte no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaReporteRepo>()
        .map_err(|_| AppError::Internal("ReporteRepo no disponible".into()))?
        .clone();

    let reporte = repo.obtener_reporte(uuid).await?;

    Ok(Json(serde_json::to_value(reporte).unwrap_or_default()))
}

/// DELETE /api/v1/reportes/{id} — Eliminar un reporte (admin).
///
/// Elimina fisicamente el reporte de la BD. Solo deberia usarse
/// por administradores. La verificacion de rol admin es futura.
#[endpoint(tags("Reportes"), security(("bearer_auth" = [])))]
pub async fn eliminar_reporte(
    id: PathParam<String>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let _id_usuario = depot
        .get::<Uuid>("id_usuario")
        .map_err(|_| AppError::Unauthorized)?;

    let uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("ID de reporte no es un UUID valido".into()))?;

    let repo = depot
        .obtain::<SeaReporteRepo>()
        .map_err(|_| AppError::Internal("ReporteRepo no disponible".into()))?
        .clone();

    repo.eliminar_reporte(uuid).await?;

    Ok(Json(MensajeResponse {
        mensaje: "Reporte eliminado correctamente".into(),
    }))
}

// ─── Tests unitarios ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifica que `parsear_tipo_contenido` acepta los tres valores validos
    /// y devuelve error para cualquier otro string.
    #[test]
    fn parsear_tipo_contenido_correcto() {
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
        assert!(matches!(
            parsear_tipo_contenido("invalido"),
            Err(AppError::BadRequest(_))
        ));
    }

    /// Verifica que `parsear_motivo` acepta todos los motivos del esquema
    /// y rechaza valores no reconocidos.
    #[test]
    fn parsear_motivo_todos_los_valores() {
        // Valores validos del schema.sql
        for motivo in [
            "spam",
            "inapropiado",
            "desactualizado",
            "incorrecto",
            "duplicado",
            "otro",
        ] {
            assert!(
                parsear_motivo(motivo).is_ok(),
                "Motivo '{motivo}' deberia ser valido"
            );
        }

        // Valor invalido
        assert!(matches!(
            parsear_motivo("mentira"),
            Err(AppError::BadRequest(_))
        ));
    }
}
