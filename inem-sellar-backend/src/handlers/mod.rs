// src/handlers/mod.rs
//
// Modulo que agrupa todos los handlers HTTP.
// Cada submodulo contiene los handlers de un dominio funcional.

use crate::errors::AppError;
use crate::models::enums::EstadoModeracion;

pub mod auth;
pub mod configuracion;
pub mod consejos;
pub mod cursos;
pub mod geografia;
pub mod ofertas;
pub mod prestaciones;
pub mod reportes;
pub mod usuarios;
pub mod votos;

pub(crate) fn parsear_estado_moderacion(s: &str) -> Result<EstadoModeracion, AppError> {
    EstadoModeracion::desde_api(s).ok_or_else(|| {
        AppError::BadRequest(format!(
            "estado_moderacion invalido: '{s}'. Valores validos: {}",
            EstadoModeracion::valores_api()
        ))
    })
}
