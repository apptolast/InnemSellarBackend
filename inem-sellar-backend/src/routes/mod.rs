// src/routes/mod.rs
//
// DONDE VA: src/routes/
//
// QUE ES: define el arbol de rutas de la API REST.
// Conecta URLs con handlers usando el Router de Salvo.
//
// # Patron de Salvo para rutas
// Salvo usa un arbol de routers composable:
// - `Router::with_path("x")` crea un nodo con path "x"
// - `.get(handler)` asocia un handler al metodo GET
// - `.push(Router::...)` anade un hijo (ruta anidada)
// - `.hoop(middleware)` aplica middleware a ese nodo y sus hijos
//
// Es similar a go_router en Flutter:
//   GoRoute(path: '/comunidades', builder: listaComunidades,
//     routes: [GoRoute(path: ':id', builder: detalleComunidad)])

use salvo::prelude::*;

use crate::handlers::{auth, geografia, ofertas};
use crate::middleware;

/// Crea el router completo de la API.
///
/// Estructura de URLs:
/// ```text
/// /                                         → hello (health check)
/// /api/v1/auth/registro                     → POST registro
/// /api/v1/auth/login                        → POST login
/// /api/v1/auth/refrescar                    → POST refrescar token
/// /api/v1/auth/logout                       → POST logout (auth requerida)
/// /api/v1/comunidades                       → GET listar
/// /api/v1/comunidades/{id}                  → GET obtener por id
/// /api/v1/provincias                        → GET listar (?id_comunidad=X)
/// /api/v1/provincias/{id}                   → GET obtener por id
/// /api/v1/provincias/{id}/oficina           → GET oficina SEPE
/// /api/v1/ofertas                           → GET listar (publico)
/// /api/v1/ofertas/{id}                      → GET obtener (publico)
/// /api/v1/ofertas                           → POST crear (auth)
/// /api/v1/ofertas/{id}                      → PUT actualizar (auth, solo autor)
/// /api/v1/ofertas/{id}                      → DELETE eliminar (auth, solo autor)
/// ```
///
/// # Por que `{id}` y no `:id` o `<id>`
/// Salvo v0.76+ usa la sintaxis `{nombre}` para path params.
/// `:id` es de Express/Actix, `<id>` era la sintaxis antigua de Salvo.
/// `{id:num}` restringiria a numeros, pero no lo usamos porque
/// nuestro param es i32 y la conversion falla con 400 si no es numero.
pub fn crear_router() -> Router {
    Router::new().push(
        Router::with_path("api/v1")
            // /api/v1/auth — rutas publicas (sin middleware auth)
            .push(
                Router::with_path("auth")
                    .push(Router::with_path("registro").post(auth::registro))
                    .push(Router::with_path("login").post(auth::login))
                    .push(Router::with_path("refrescar").post(auth::refrescar))
                    .push(
                        // logout requiere auth
                        Router::with_path("logout")
                            .hoop(middleware::auth_middleware)
                            .post(auth::logout),
                    ),
            )
            // /api/v1/comunidades — rutas publicas
            .push(
                Router::with_path("comunidades")
                    .get(geografia::listar_comunidades)
                    .push(
                        // /api/v1/comunidades/{id}
                        Router::with_path("{id}").get(geografia::obtener_comunidad),
                    ),
            )
            // /api/v1/provincias
            .push(
                Router::with_path("provincias")
                    .get(geografia::listar_provincias)
                    .push(
                        Router::with_path("{id}")
                            .get(geografia::obtener_provincia)
                            .push(
                                Router::with_path("oficina")
                                    .get(geografia::obtener_oficina_por_provincia),
                            ),
                    ),
            )
            // /api/v1/ofertas — GET publico, POST/PUT/DELETE protegido
            .push(
                Router::with_path("ofertas")
                    .get(ofertas::listar_ofertas)
                    .push(
                        // POST crear — requiere auth
                        Router::new()
                            .hoop(middleware::auth_middleware)
                            .post(ofertas::crear_oferta),
                    )
                    .push(
                        Router::with_path("{id}").get(ofertas::obtener_oferta).push(
                            // PUT/DELETE — requieren auth
                            Router::new()
                                .hoop(middleware::auth_middleware)
                                .put(ofertas::actualizar_oferta)
                                .delete(ofertas::eliminar_oferta),
                        ),
                    ),
            ),
    )
}
