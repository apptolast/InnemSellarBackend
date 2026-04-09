// src/errors/mod.rs
//
// DONDE VA: src/errors/ (carpeta que ya existia vacia)
//
// QUE ES: el tipo de error unificado de toda la aplicacion.
// Todos los handlers devuelven `Result<T, AppError>`, y Salvo
// convierte automaticamente el AppError en una respuesta HTTP
// con el status code y mensaje JSON apropiados.
//
// POR QUE EXISTE: sin un tipo de error comun, cada handler
// tendria su propia forma de manejar errores. Con AppError,
// el manejo de errores es consistente en toda la API:
// - 400 para datos invalidos
// - 401 para no autenticado
// - 403 para sin permisos
// - 404 para no encontrado
// - 409 para conflictos (ej: email duplicado)
// - 500 para errores internos (BD, bugs)

use salvo::async_trait;
use salvo::http::StatusCode;
use salvo::oapi::{Components, EndpointOutRegister, Operation};
use salvo::prelude::*;
use thiserror::Error;

/// Error unificado de la API de InemSellar.
///
/// # Por que `#[derive(Error)]`
/// `Error` viene del crate `thiserror`. Genera automaticamente la
/// implementacion del trait `std::error::Error` para nuestro enum.
/// Sin esto, tendriamos que implementar `Error`, `Display` y `Debug`
/// a mano — unas 30 lineas de boilerplate por variante.
///
/// # Por que `#[error("...")]` en cada variante
/// `thiserror` usa estos atributos para generar el `Display` del error.
/// El texto entre comillas es lo que devuelve `self.to_string()`.
/// `{0}` se reemplaza por el primer campo de la variante (interpolacion).
///
/// # Por que `#[from]` en `Database`
/// `#[from]` genera automaticamente `impl From<sqlx::Error> for AppError`.
/// Esto permite usar el operador `?` en handlers: si una query SQLx falla,
/// el error se convierte automaticamente en `AppError::Database(...)`.
/// Es como si Rust hiciera `.map_err(AppError::Database)` por ti.
///
/// # Equivalencia en Dart
/// En Dart harias algo como:
/// ```dart
/// sealed class AppError {
///   const factory AppError.notFound(String msg) = NotFoundError;
///   const factory AppError.badRequest(String msg) = BadRequestError;
///   // ...
/// }
/// ```
/// En Rust, un `enum` con `thiserror` logra lo mismo pero con pattern matching
/// exhaustivo: el compilador te obliga a manejar TODAS las variantes.
#[derive(Error, Debug)]
pub enum AppError {
    /// Recurso no encontrado (HTTP 404).
    /// Uso: `AppError::NotFound("Usuario con id X".into())`
    #[error("Recurso no encontrado: {0}")]
    NotFound(String),

    /// Datos de entrada invalidos (HTTP 400).
    /// Uso: `AppError::BadRequest("Email no valido".into())`
    #[error("Datos invalidos: {0}")]
    BadRequest(String),

    /// No autenticado — falta token JWT o es invalido (HTTP 401).
    /// No tiene campo String porque el mensaje siempre es el mismo.
    #[error("No autenticado")]
    Unauthorized,

    /// Autenticado pero sin permisos para esta accion (HTTP 403).
    /// Ej: un usuario normal intenta acceder a un endpoint de admin.
    #[error("Sin permisos para esta accion")]
    Forbidden,

    /// Conflicto con el estado actual (HTTP 409).
    /// Ej: intentar registrar un email que ya existe (violacion UNIQUE).
    #[error("Conflicto: {0}")]
    Conflict(String),

    /// Error de base de datos (HTTP 500).
    /// `#[from]` permite que `?` convierta sea_orm::DbErr en AppError.
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),

    /// Error interno generico (HTTP 500).
    #[error("Error interno: {0}")]
    Internal(String),
}

impl AppError {
    /// Metodo helper para convertir DbErr en AppError.
    /// Util con `.map_err(AppError::from_db)` en los repositorios.
    pub fn from_db(err: sea_orm::DbErr) -> Self {
        AppError::Database(err)
    }
}

/// Implementacion de `EndpointOutRegister` para que `#[endpoint]` pueda documentar
/// las respuestas de error en la especificacion OpenAPI generada automaticamente.
///
/// # Por que este trait
/// Cuando un handler marcado con `#[endpoint]` devuelve `Result<T, AppError>`,
/// Salvo necesita saber que codigos HTTP puede retornar el error para incluirlos
/// en la documentacion OpenAPI. Este trait le dice a Salvo: "este error puede
/// producir 400, 401, 403, 404, 409 y 500".
///
/// # Por que `let _ = components`
/// `components` se usa para registrar schemas de tipos complejos (e.g., structs
/// de respuesta de error con cuerpo JSON documentado). En nuestro caso solo
/// registramos descripciones textuales simples, asi que no necesitamos `components`.
/// La asignacion `let _ = ...` suprime el warning del compilador de variable no usada.
impl EndpointOutRegister for AppError {
    fn register(components: &mut Components, operation: &mut Operation) {
        // Usamos String::from("400") en vez de "400".into() para ayudar al compilador
        // a resolver la ambiguedad de tipos cuando hay multiples impls de Into<String>.
        operation.responses.insert(
            String::from("400"),
            salvo::oapi::Response::new(
                "Datos invalidos — el cuerpo JSON o los parametros no son correctos",
            ),
        );
        operation.responses.insert(
            String::from("401"),
            salvo::oapi::Response::new("No autenticado — falta el token JWT o es invalido"),
        );
        operation.responses.insert(
            String::from("403"),
            salvo::oapi::Response::new(
                "Sin permisos — el usuario autenticado no tiene acceso a este recurso",
            ),
        );
        operation.responses.insert(
            String::from("404"),
            salvo::oapi::Response::new("Recurso no encontrado"),
        );
        operation.responses.insert(
            String::from("409"),
            salvo::oapi::Response::new("Conflicto — el recurso ya existe (ej: email duplicado)"),
        );
        operation.responses.insert(
            String::from("500"),
            salvo::oapi::Response::new("Error interno del servidor"),
        );
        // No registramos schemas de error en components porque las respuestas
        // de error son objetos JSON simples { error: string, codigo: number }
        // que no requieren definicion formal de schema.
        let _ = components;
    }
}

/// # Por que implementar `Writer`
/// `Writer` es el trait de Salvo que convierte un tipo en una respuesta HTTP.
/// Cuando un handler devuelve `Result<Json<T>, AppError>`, Salvo llama a
/// `AppError::write()` si hay error. Aqui decidimos:
/// - Que status code HTTP devolver (404, 401, 500, etc.)
/// - Que JSON mandar al cliente
///
/// # Seguridad: no exponer errores internos
/// Para `Database` e `Internal`, logueamos el error real con `tracing::error!`
/// (se ve en los logs del servidor) pero al cliente le mandamos un mensaje
/// generico "Error interno del servidor". Esto evita filtrar nombres de tablas,
/// queries SQL, o stack traces al frontend.
///
/// # Por que `#[async_trait]`
/// `Writer::write` es una funcion async dentro de un trait. Rust aun usa
/// el crate `async_trait` para esto en muchas librerias (incluido Salvo).
/// `#[async_trait]` transforma la funcion async en un tipo que Rust puede
/// manejar en un trait. Salvo re-exporta este macro desde su prelude.
#[async_trait]
impl Writer for AppError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        // Determinamos status code y mensaje segun la variante.
        // `match` es exhaustivo: si anadimos una nueva variante a AppError
        // y olvidamos manejarla aqui, el compilador da error.
        let (status, mensaje) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "No autenticado".to_string()),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "Sin permisos para esta accion".to_string(),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),

            // SEGURIDAD: errores de BD e internos se loguean pero NO se exponen
            AppError::Database(err) => {
                tracing::error!("Error de base de datos: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".to_string(),
                )
            }
            AppError::Internal(msg) => {
                tracing::error!("Error interno: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error interno del servidor".to_string(),
                )
            }
        };

        // Escribimos la respuesta HTTP:
        // 1. Ponemos el status code (404, 401, 500, etc.)
        // 2. Renderizamos un JSON con el mensaje de error
        //
        // `Json(...)` es un wrapper de Salvo que serializa a JSON y pone
        // el header Content-Type: application/json automaticamente.
        //
        // `serde_json::json!` es un macro que crea un serde_json::Value
        // a partir de una sintaxis similar a JSON literal.
        res.status_code(status);
        res.render(Json(serde_json::json!({
            "error": mensaje,
            "codigo": status.as_u16(),
        })));
    }
}
