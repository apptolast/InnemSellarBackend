//! Modulo de acceso a la base de datos.
//!
//! Gestiona la creacion y configuracion del pool de conexiones a PostgreSQL
//! usando SQLx. El pool es el unico punto de entrada a la base de datos en
//! toda la aplicacion; los handlers reciben una referencia al pool, nunca
//! abren conexiones directas.

use crate::config::AppConfig;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Inicializa el pool de conexiones a PostgreSQL y lo devuelve listo para usar.
///
/// Recibe la configuracion de la aplicacion como referencia (`&AppConfig`) y
/// devuelve un `PgPool` que puede clonarse y compartirse entre todos los handlers.
///
/// # Por que un pool de conexiones
/// Abrir una conexion TCP a PostgreSQL cuesta ~10-50 ms (handshake TLS, autenticacion,
/// negociacion de protocolo). Con un pool mantenemos un conjunto de conexiones ya
/// establecidas y las reutilizamos. Es el mismo patron que usa el paquete
/// `sqflite` o cualquier ORM en Flutter cuando habilitas connection pooling.
///
/// # Por que `&AppConfig` y no `AppConfig`
/// El `&` indica que tomamos prestado (*borrowed*) el dato sin ser duenos de el.
/// Si recibiramos `AppConfig` (sin `&`), el ownership se transferiria a esta
/// funcion y `cfg` quedaria inaccesible en `main`. Con `&` la funcion lee la
/// config sin consumirla, igual que pasar un objeto por referencia en Dart
/// (en Dart todos los objetos son referencias implicitas; en Rust lo hacemos
/// explicito con `&`).
///
/// # Por que `async fn` y `.await`
/// `connect()` abre la conexion de red, que es I/O puro. En Rust async,
/// `.await` cede el hilo al runtime de tokio mientras espera la respuesta de
/// la red, permitiendo que otras tareas corran en paralelo. Sin `.await`
/// simplemente tendriamos un `Future` sin ejecutar — equivalente a llamar a
/// una funcion `async` en Dart sin usar `await`.
///
/// # Por que `expect()` y no `?`
/// `expect()` hace panic con el mensaje dado si `connect()` devuelve un error.
/// Esto es aceptable aqui porque es codigo de inicializacion: si no hay base
/// de datos disponible al arrancar, el servidor no puede funcionar y es mejor
/// fallar rapido con un mensaje claro que entrar en un estado inconsistente.
/// En handlers de API usaremos `?` para propagar el error al cliente con un
/// HTTP 500, sin hacer panic.
///
/// # Parametros del pool
/// - `max_connections(20)`: maximo de conexiones simultaneas abiertas.
/// - `min_connections(5)`: conexiones que se abren al arrancar (warm-up).
/// - `acquire_timeout(10s)`: tiempo maximo de espera para obtener una conexion libre.
/// - `idle_timeout(600s)`: cierra conexiones que llevan 10 min sin usarse.
/// - `max_lifetime(1800s)`: recicla cada conexion tras 30 min para evitar
///   conexiones "fantasma" que PostgreSQL haya cerrado por su lado.
pub async fn init_pool(cfg: &AppConfig) -> PgPool {
    // PgPoolOptions implementa el patron Builder: cada metodo devuelve `Self`,
    // permitiendo encadenar configuraciones antes de llamar a `connect()`.
    // Es identico al patron fluent/builder que se usa en Flutter con, por ejemplo,
    // `ThemeData.copyWith(...)`.
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        // `&cfg.database_url` presta el String interno como &str para la llamada.
        // SQLx acepta &str, no necesita tomar ownership del String.
        .connect(&cfg.database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    // `tracing::info!` emite un evento de log al nivel INFO.
    // `tracing` es el ecosistema de observabilidad de Rust: separa la
    // instrumentacion (este macro) del destino de los logs (tracing-subscriber,
    // configurado en main). Equivale a `debugPrint` en Flutter pero estructurado
    // y configurable por entorno.
    tracing::info!("Conexion a PostgreSQL establecida correctamente");

    // Devolvemos el pool. En Rust no hay `return` obligatorio: la ultima
    // expresion de un bloque SIN punto y coma es el valor de retorno.
    // Es como el `=>` en funciones de una linea en Dart.
    pool
}
