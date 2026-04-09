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

use crate::handlers::geografia;

/// Crea el router completo de la API.
///
/// Estructura de URLs:
/// ```text
/// /                                         → hello (health check)
/// /api/v1/comunidades                       → GET listar
/// /api/v1/comunidades/{id}                  → GET obtener por id
/// /api/v1/provincias                        → GET listar (?id_comunidad=X)
/// /api/v1/provincias/{id}                   → GET obtener por id
/// /api/v1/provincias/{id}/oficina           → GET oficina SEPE
/// ```
///
/// # Por que `{id}` y no `:id` o `<id>`
/// Salvo v0.76+ usa la sintaxis `{nombre}` para path params.
/// `:id` es de Express/Actix, `<id>` era la sintaxis antigua de Salvo.
/// `{id:num}` restringiria a numeros, pero no lo usamos porque
/// nuestro param es i32 y la conversion falla con 400 si no es numero.
pub fn crear_router() -> Router {
    Router::new()
        // Rutas de geografia bajo /api/v1
        .push(
            Router::with_path("api/v1")
                // /api/v1/comunidades
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
                                // /api/v1/provincias/{id}
                                .get(geografia::obtener_provincia)
                                .push(
                                    // /api/v1/provincias/{id}/oficina
                                    Router::with_path("oficina")
                                        .get(geografia::obtener_oficina_por_provincia),
                                ),
                        ),
                ),
        )
}
