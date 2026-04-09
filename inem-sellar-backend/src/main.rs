#![warn(missing_docs)]
//! Backend de InemSellar — API REST para la app de ayuda a desempleados en Espana.
//!
//! # Arquitectura
//! Este crate usa [Salvo](https://salvo.rs) como framework web, SeaORM como ORM
//! para PostgreSQL, y JWT para autenticacion. Cada modulo incluye documentacion
//! educativa que explica los conceptos de Rust utilizados.
//!
//! # Documentacion de la API
//! Con el servidor en marcha, la documentacion Swagger UI esta disponible en:
//! - Swagger UI: `http://localhost:8080/swagger-ui`
//! - JSON OpenAPI: `http://localhost:8080/api-doc/openapi.json`
//!
//! # Como generar la documentacion Rust en HTML
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
use salvo::oapi::security::{Http, HttpAuthScheme, SecurityScheme};
use salvo::oapi::{Info, OpenApi};
use salvo::prelude::*;

use crate::repositories::{
    SeaAuthRepo, SeaConfiguracionRepo, SeaConsejoRepo, SeaCursoRepo, SeaGeografiaRepo,
    SeaOfertaRepo, SeaPrestacionRepo, SeaReporteRepo, SeaUsuarioRepo, SeaVotoRepo,
};
use crate::services::AuthService;

/// Handler basico que responde con "Hello World".
///
/// # Por que `#[handler]`
/// `#[handler]` es una macro de Salvo que convierte una funcion async en un handler HTTP.
/// Gestiona automaticamente la extraccion de parametros de la request y la serializacion
/// de la respuesta. Es similar a las anotaciones de ruta en frameworks como Shelf en Dart.
///
/// # Por que `#[handler]` aqui y no `#[endpoint]`
/// Este handler de health-check es tan simple que no aporta valor documentarlo en OpenAPI.
/// `#[endpoint]` se reserva para los handlers de negocio que necesitan documentacion.
/// Tecnicamente ambos funcionan igual en runtime; la diferencia es solo en que
/// `#[endpoint]` genera metadata OpenAPI y `#[handler]` no.
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
    let consejo_repo = SeaConsejoRepo::new(db.clone());
    let curso_repo = SeaCursoRepo::new(db.clone());
    let voto_repo = SeaVotoRepo::new(db.clone());
    let reporte_repo = SeaReporteRepo::new(db.clone());
    let prestacion_repo = SeaPrestacionRepo::new(db.clone());
    let configuracion_repo = SeaConfiguracionRepo::new(db.clone());
    let usuario_repo = SeaUsuarioRepo::new(db.clone());

    // Inyectamos todos los servicios y repos en el Depot de Salvo.
    //
    // # Por que inyectar en el router raiz
    // `affix_state::inject(valor)` es un middleware que inserta el valor en el `Depot`
    // de cada peticion. Al ponerlo en el router raiz con `.hoop(...)`, todos los
    // handlers hijo pueden acceder a el con `depot.obtain::<Tipo>()`.
    // Es el patron de "inyeccion de dependencias" de Salvo: similar a los Providers
    // de Riverpod en Flutter, pero sin arbol de widgets — solo un mapa tipado por request.
    let router = Router::new()
        .get(hello)
        .hoop(affix_state::inject(auth_service))
        .hoop(affix_state::inject(geo_repo))
        .hoop(affix_state::inject(auth_repo))
        .hoop(affix_state::inject(oferta_repo))
        .hoop(affix_state::inject(consejo_repo))
        .hoop(affix_state::inject(curso_repo))
        .hoop(affix_state::inject(voto_repo))
        .hoop(affix_state::inject(reporte_repo))
        .hoop(affix_state::inject(prestacion_repo))
        .hoop(affix_state::inject(configuracion_repo))
        .hoop(affix_state::inject(usuario_repo))
        .push(routes::crear_router());

    // ── Documentacion OpenAPI / Swagger UI ──────────────────────────────────
    //
    // # Por que OpenAPI
    // OpenAPI (antes Swagger) es el estandar de la industria para documentar APIs REST.
    // El spec JSON describe todos los endpoints, parametros, esquemas de request/response
    // y requisitos de autenticacion. Swagger UI lo convierte en una interfaz web interactiva
    // donde los desarrolladores (y el equipo de Flutter) pueden explorar y probar la API.
    //
    // # Como funciona con Salvo OAPI
    // `OpenApi::new(...)` recorre el router y recopila la metadata que generaron los
    // `#[endpoint]` (tags, parametros, esquemas de DTOs marcados con `ToSchema`,
    // esquemas de seguridad). El resultado es un JSON valido segun la spec OpenAPI 3.x.
    //
    // `.merge_router(&router)` es el paso clave: Salvo inspecciona el arbol de routers
    // buscando handlers marcados con `#[endpoint]` y extrae su informacion de tipos
    // para construir el spec OpenAPI automaticamente.
    //
    // # Por que `add_security_scheme("bearer_auth", ...)`
    // Define el esquema de autenticacion JWT que referencian los endpoints con
    // `security(("bearer_auth" = []))`. Le dice a OpenAPI: "hay un esquema llamado
    // bearer_auth que es HTTP Bearer con formato JWT". Swagger UI mostrara un boton
    // "Authorize" donde el usuario puede pegar su JWT para probar endpoints protegidos.
    let doc = OpenApi::new("InemSellar API", "0.1.0")
        .info(
            Info::new("InemSellar API", "0.1.0")
                .description(
                    "API REST del backend de InemSellar\n\n\
                     App de ayuda a desempleados en Espana — SEPE/INEM.\n\n\
                     **Autenticacion**: usa `POST /api/v1/auth/login` para obtener un \
                     `access_token` JWT y pulsalo en el boton Authorize (arriba a la derecha).",
                )
                .contact(
                    salvo::oapi::Contact::new()
                        .name("AppToLast")
                        .email("admin@apptolast.com"),
                ),
        )
        .add_security_scheme(
            "bearer_auth",
            // `Http` define un esquema de autenticacion HTTP estandar.
            // `HttpAuthScheme::Bearer` indica que el token va en el header
            // `Authorization: Bearer <token>`. `.bearer_format("JWT")` es
            // solo un hint para Swagger UI (no afecta a la validacion).
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer).bearer_format("JWT")),
        )
        .merge_router(&router);

    // Anadimos las rutas de documentacion al router existente.
    //
    // # Por que `.unshift(...)` y no `.push(...)`
    // `.unshift()` inserta el router al PRINCIPIO del arbol de rutas, antes que las
    // rutas de negocio. Esto es necesario para que las rutas `/api-doc/openapi.json`
    // y `/swagger-ui` se registren correctamente sin conflictos con el prefijo `/api/v1`.
    //
    // # Las dos rutas de documentacion
    // - `/api-doc/openapi.json`: el spec JSON bruto (util para herramientas externas)
    // - `/swagger-ui`: interfaz web interactiva (para humanos)
    let router = router
        .unshift(doc.into_router("/api-doc/openapi.json"))
        .unshift(SwaggerUi::new("/api-doc/openapi.json").into_router("/swagger-ui"));

    // `TcpListener` abre el socket TCP en el puerto 8080 en todas las interfaces
    // de red (0.0.0.0). `.bind().await` completa el binding de forma asincrona.
    // En produccion, Nginx actua como proxy inverso delante de este puerto.
    let acceptor = TcpListener::new(cfg.port_addr).bind().await;

    tracing::info!("Servidor escuchando en http://{}", cfg.server_addr);
    tracing::info!(
        "Documentacion Swagger UI: http://{}/swagger-ui",
        cfg.server_addr
    );

    // `Server::new(acceptor).serve(router).await` arranca el bucle principal
    // de aceptacion de conexiones. Esta llamada no retorna hasta que el proceso
    // recibe una senal de terminacion (SIGTERM/SIGINT). Es el equivalente al
    // `runApp(MyApp())` en Flutter: el punto sin retorno que cede el control
    // al framework.
    Server::new(acceptor).serve(router).await;
}
