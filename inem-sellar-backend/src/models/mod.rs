// src/models/mod.rs
//
// DONDE VA: es el "index" de la carpeta models/
// QUE HACE: declara que archivos existen (pub mod) y re-exporta
//           los tipos para que otros modulos los usen facilmente.
//
// # Por que `pub mod` + `pub use`
//
// `pub mod usuario;` hace dos cosas:
//   1. Le dice a Rust que el archivo usuario.rs existe y es parte de este modulo
//   2. `pub` lo hace visible fuera de models/
//
// `pub use usuario::Usuario;` re-exporta el struct para que desde
// fuera de models/ puedas escribir:
//   use crate::models::Usuario;
// en vez de:
//   use crate::models::usuario::Usuario;
//
// Es pura comodidad — no cambia funcionalidad, solo ahorra escritura.

// --- Enums (tipos de PostgreSQL) ---
pub mod enums;
pub use enums::*;

// --- Geografia ---
pub mod comunidad_autonoma;
pub mod oficina_sepe;
pub mod provincia;

pub use comunidad_autonoma::ComunidadAutonoma;
pub use oficina_sepe::OficinaSepe;
pub use provincia::Provincia;

// --- Autenticacion ---
pub mod proveedor_autenticacion;
pub mod token_refresco;
pub mod usuario;

pub use proveedor_autenticacion::ProveedorAutenticacion;
pub use token_refresco::TokenRefresco;
pub use usuario::Usuario;

// --- Contenido ---
pub mod consejo;
pub mod curso;
pub mod oferta_empleo;

pub use consejo::Consejo;
pub use curso::Curso;
pub use oferta_empleo::OfertaEmpleo;

// --- Relaciones N:M ---
pub mod consejo_provincia;
pub mod curso_provincia;
pub mod oferta_provincia;

pub use consejo_provincia::ConsejoProvincia;
pub use curso_provincia::CursoProvincia;
pub use oferta_provincia::OfertaProvincia;

// --- Interacciones (polimorficas) ---
pub mod reporte;
pub mod voto;

pub use reporte::Reporte;
pub use voto::Voto;

// --- Sistema ---
pub mod configuracion_aplicacion;
pub mod prestacion;

pub use configuracion_aplicacion::ConfiguracionAplicacion;
pub use prestacion::Prestacion;
