// src/routes/mod.rs
//
// Define el arbol de rutas de la API REST.
// Conecta URLs con handlers usando el Router de Salvo.

use salvo::prelude::*;

use crate::handlers::{
    auth, configuracion, consejos, cursos, geografia, ofertas, prestaciones, reportes, usuarios,
    votos,
};
use crate::middleware;

/// Crea el router completo de la API.
///
/// Estructura de URLs (~40 endpoints):
/// ```text
/// /api/v1/auth/registro                     → POST registro
/// /api/v1/auth/login                        → POST login
/// /api/v1/auth/refrescar                    → POST refrescar token
/// /api/v1/auth/logout                       → POST logout (auth)
///
/// /api/v1/perfil                            → GET/PUT perfil propio (auth)
/// /api/v1/usuarios/{id}                     → GET perfil publico
///
/// /api/v1/comunidades                       → GET listar
/// /api/v1/comunidades/{id}                  → GET obtener
/// /api/v1/provincias                        → GET listar (?id_comunidad=X)
/// /api/v1/provincias/{id}                   → GET obtener
/// /api/v1/provincias/{id}/oficina           → GET oficina SEPE
///
/// /api/v1/ofertas                           → GET listar / POST crear (auth)
/// /api/v1/ofertas/{id}                      → GET / PUT (auth) / DELETE (auth)
///
/// /api/v1/consejos                          → GET listar / POST crear (auth)
/// /api/v1/consejos/{id}                     → GET / PUT (auth) / DELETE (auth)
///
/// /api/v1/cursos                            → GET listar / POST crear (auth)
/// /api/v1/cursos/{id}                       → GET / PUT (auth) / DELETE (auth)
///
/// /api/v1/votos                             → GET / POST / DELETE (auth)
///
/// /api/v1/reportes                          → POST crear (auth)
/// /api/v1/reportes/pendientes               → GET listar (auth/admin)
/// /api/v1/reportes/{id}                     → PUT procesar (auth/admin)
///
/// /api/v1/prestaciones                      → GET listar / POST crear (auth/admin)
/// /api/v1/prestaciones/{id}                 → GET / PUT (auth/admin) / DELETE (auth/admin)
///
/// /api/v1/configuracion                     → GET listar / POST crear (auth/admin)
/// /api/v1/configuracion/{clave}             → GET / PUT (auth/admin) / DELETE (auth/admin)
/// ```
pub fn crear_router() -> Router {
    Router::new().push(
        Router::with_path("api/v1")
            // ── Auth ──────────────────────────────────────────────
            .push(
                Router::with_path("auth")
                    .push(Router::with_path("registro").post(auth::registro))
                    .push(Router::with_path("login").post(auth::login))
                    .push(Router::with_path("refrescar").post(auth::refrescar))
                    .push(
                        Router::with_path("logout")
                            .hoop(middleware::auth_middleware)
                            .post(auth::logout),
                    ),
            )
            // ── Perfil de usuario ─────────────────────────────────
            .push(
                Router::with_path("perfil")
                    .hoop(middleware::auth_middleware)
                    .get(usuarios::obtener_perfil)
                    .put(usuarios::actualizar_perfil),
            )
            .push(Router::with_path("usuarios/{id}").get(usuarios::obtener_usuario_publico))
            // ── Geografia ─────────────────────────────────────────
            .push(
                Router::with_path("comunidades")
                    .get(geografia::listar_comunidades)
                    .push(Router::with_path("{id}").get(geografia::obtener_comunidad)),
            )
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
            // ── Ofertas de empleo ─────────────────────────────────
            .push(
                Router::with_path("ofertas")
                    .get(ofertas::listar_ofertas)
                    .push(
                        Router::new()
                            .hoop(middleware::auth_middleware)
                            .post(ofertas::crear_oferta),
                    )
                    .push(
                        Router::with_path("{id}").get(ofertas::obtener_oferta).push(
                            Router::new()
                                .hoop(middleware::auth_middleware)
                                .put(ofertas::actualizar_oferta)
                                .delete(ofertas::eliminar_oferta),
                        ),
                    ),
            )
            // ── Consejos ──────────────────────────────────────────
            .push(
                Router::with_path("consejos")
                    .get(consejos::listar_consejos)
                    .push(
                        Router::new()
                            .hoop(middleware::auth_middleware)
                            .post(consejos::crear_consejo),
                    )
                    .push(
                        Router::with_path("{id}")
                            .get(consejos::obtener_consejo)
                            .push(
                                Router::new()
                                    .hoop(middleware::auth_middleware)
                                    .put(consejos::actualizar_consejo)
                                    .delete(consejos::eliminar_consejo),
                            ),
                    ),
            )
            // ── Cursos ────────────────────────────────────────────
            .push(
                Router::with_path("cursos")
                    .get(cursos::listar_cursos)
                    .push(
                        Router::new()
                            .hoop(middleware::auth_middleware)
                            .post(cursos::crear_curso),
                    )
                    .push(
                        Router::with_path("{id}").get(cursos::obtener_curso).push(
                            Router::new()
                                .hoop(middleware::auth_middleware)
                                .put(cursos::actualizar_curso)
                                .delete(cursos::eliminar_curso),
                        ),
                    ),
            )
            // ── Votos ─────────────────────────────────────────────
            .push(
                Router::with_path("votos")
                    .hoop(middleware::auth_middleware)
                    .get(votos::obtener_voto)
                    .post(votos::votar)
                    .delete(votos::eliminar_voto),
            )
            // ── Reportes ──────────────────────────────────────────
            .push(
                Router::with_path("reportes")
                    .hoop(middleware::auth_middleware)
                    .post(reportes::crear_reporte)
                    .push(Router::with_path("pendientes").get(reportes::listar_reportes_pendientes))
                    .push(Router::with_path("{id}").put(reportes::procesar_reporte)),
            )
            // ── Prestaciones ──────────────────────────────────────
            .push(
                Router::with_path("prestaciones")
                    .get(prestaciones::listar_prestaciones)
                    .push(
                        Router::new()
                            .hoop(middleware::auth_middleware)
                            .post(prestaciones::crear_prestacion),
                    )
                    .push(
                        Router::with_path("{id}")
                            .get(prestaciones::obtener_prestacion)
                            .push(
                                Router::new()
                                    .hoop(middleware::auth_middleware)
                                    .put(prestaciones::actualizar_prestacion)
                                    .delete(prestaciones::eliminar_prestacion),
                            ),
                    ),
            )
            // ── Configuracion ─────────────────────────────────────
            .push(
                Router::with_path("configuracion")
                    .get(configuracion::listar_configuracion)
                    .push(
                        Router::new()
                            .hoop(middleware::auth_middleware)
                            .post(configuracion::crear_configuracion),
                    )
                    .push(
                        Router::with_path("{clave}")
                            .get(configuracion::obtener_configuracion)
                            .push(
                                Router::new()
                                    .hoop(middleware::auth_middleware)
                                    .put(configuracion::actualizar_configuracion)
                                    .delete(configuracion::eliminar_configuracion),
                            ),
                    ),
            ),
    )
}
