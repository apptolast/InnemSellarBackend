#![warn(missing_docs)]
// Entidades y variantes de error estan definidas para uso futuro inmediato.
// Este allow se eliminara cuando la API este 100% implementada.
#![allow(dead_code)]
//! Backend de InemSellar — API REST para la app de ayuda a desempleados en Espana.
//!
//! # Arquitectura
//! Este crate usa [Salvo](https://salvo.rs) como framework web, SeaORM como ORM
//! para PostgreSQL, y JWT para autenticacion. Cada modulo incluye documentacion
//! educativa que explica los conceptos de Rust utilizados.
//!
//! # Como generar la documentacion en HTML
//! ```bash
//! cargo doc --no-deps --open
//! ```

mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use salvo::affix_state;
use salvo::prelude::*;

use crate::repositories::{SeaAuthRepo, SeaGeografiaRepo, SeaOfertaRepo};
use crate::services::AuthService;

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
    // Cargamos la configuracion antes de inicializar el logger para que
    // DATABASE_URL y SERVER_ADDR esten disponibles. `from_env()` hace panic
    // con mensaje claro si faltan variables obligatorias — fallo rapido
    // intencionado en arranque.
    let cfg = config::AppConfig::from_env();

    // Inicializamos el sistema de logging estructurado.
    // `fmt()` configura el formato de salida (texto legible por humanos).
    // `with_env_filter` lee la variable de entorno RUST_LOG para decidir
    // que niveles mostrar (ej: RUST_LOG=info o RUST_LOG=debug).
    // Si RUST_LOG no esta definida, no se muestra ningun log.
    // Equivale a configurar el nivel de log en Flutter con `Logger.root.level`.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Creamos la conexion a PostgreSQL via SeaORM.
    // SeaORM gestiona el pool internamente (usa SQLx por debajo).
    let db = db::init_db(&cfg).await;

    // Creamos servicios y repositorios, inyectando la conexion.
    let auth_service = AuthService::new(cfg.jwt_secret.clone(), cfg.jwt_expiracion_minutos);
    let geo_repo = SeaGeografiaRepo::new(db.clone());
    let auth_repo = SeaAuthRepo::new(db.clone());
    let oferta_repo = SeaOfertaRepo::new(db.clone());

    // Inyectamos todos los servicios y repos en el Depot de Salvo.
    let router = Router::new()
        .get(hello)
        .hoop(affix_state::inject(auth_service))
        .hoop(affix_state::inject(geo_repo))
        .hoop(affix_state::inject(auth_repo))
        .hoop(affix_state::inject(oferta_repo))
        .push(routes::crear_router());

    // `TcpListener` abre el socket TCP en el puerto 8080 en todas las interfaces
    // de red (0.0.0.0). `.bind().await` completa el binding de forma asincrona.
    // En produccion, Nginx actua como proxy inverso delante de este puerto.
    let acceptor = TcpListener::new(cfg.port_addr).bind().await;

    tracing::info!("Servidor escuchando en http://{}", cfg.server_addr);

    // `Server::new(acceptor).serve(router).await` arranca el bucle principal
    // de aceptacion de conexiones. Esta llamada no retorna hasta que el proceso
    // recibe una senal de terminacion (SIGTERM/SIGINT). Es el equivalente al
    // `runApp(MyApp())` en Flutter: el punto sin retorno que cede el control
    // al framework.
    Server::new(acceptor).serve(router).await;
}
