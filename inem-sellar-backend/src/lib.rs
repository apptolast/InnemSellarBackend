//! Backend de InemSellar — biblioteca compartida entre el binario y los tests.
//!
//! # Por que lib + bin
//! Este crate expone los modulos publicos como una libreria (`src/lib.rs`)
//! para que los tests de integracion en `tests/` puedan acceder a los
//! handlers, servicios, repositorios y modelos. En Rust, un crate puramente
//! binario no permite tests integrados externos: solo se permiten cuando
//! existe un crate-libreria.
//!
//! El binario `src/main.rs` es un wrapper delgado que monta la conexion a
//! BD, inyecta los servicios en el `Depot` de Salvo, configura OpenAPI y
//! arranca el servidor. TODA la logica vive aqui.
//!
//! # Documentacion HTML
//! ```bash
//! cargo doc --no-deps --open
//! ```

pub mod config;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
