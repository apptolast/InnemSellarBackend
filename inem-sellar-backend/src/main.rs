#![warn(missing_docs)]
//! Backend de InemSellar — API REST para la app de ayuda a desempleados en Espana.
//!
//! # Arquitectura
//! Este crate usa [Salvo](https://salvo.rs) como framework web, SQLx para acceso
//! a PostgreSQL, y JWT para autenticacion. Cada modulo incluye documentacion
//! educativa que explica los conceptos de Rust utilizados.
//!
//! # Como generar la documentacion en HTML
//! ```bash
//! cargo doc --no-deps --open
//! ```

mod config;
mod db;

use salvo::prelude::*;

/// Handler basico que responde con "Hello World".
///
/// # Por que `#[handler]`
/// `#[handler]` es una macro de Salvo que convierte una funcion async en un handler HTTP.
/// Gestiona automaticamente la extraccion de parametros de la request y la serializacion
/// de la respuesta. Es similar a las anotaciones de ruta en frameworks como Shelf en Dart.
///
/// # Por que `&'static str`
/// `'static` es un lifetime que indica que el string vive durante toda la ejecucion del programa.
/// Los literales de string en Rust (`"Hello World"`) siempre son `&'static str` porque se
/// almacenan en el binario compilado, no en el heap. No requieren allocacion ni liberacion.
#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

/// Punto de entrada de la aplicacion.
///
/// # Por que `#[tokio::main]`
/// Rust no tiene async runtime integrado (a diferencia de Dart que tiene su event loop).
/// `#[tokio::main]` inicializa el runtime async de tokio, que gestiona las tareas
/// concurrentes (conexiones HTTP, queries a DB, etc.). Es como arrancar el event loop
/// de Dart, pero de forma explicita.
///
/// # Por que `async fn main()`
/// `async` permite usar `.await` dentro de la funcion. Cada `.await` cede el control
/// al runtime para que pueda ejecutar otras tareas mientras esperamos I/O (red, disco).
/// Igual que en Dart con `async/await`, pero en Rust el compilador verifica en tiempo
/// de compilacion que no haya data races.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new().get(hello);
    let acceptor = TcpListener::new("0.0.0.0:8080").bind().await;

    tracing::info!("Servidor escuchando en http://localhost:8080");
    Server::new(acceptor).serve(router).await;
}
