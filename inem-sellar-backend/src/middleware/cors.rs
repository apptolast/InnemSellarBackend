//! Middleware CORS global de la API.
//!
//! En Salvo el handler CORS debe aplicarse al `Service`, no al `Router`, para
//! interceptar las preflight `OPTIONS` aunque no exista una ruta `OPTIONS`
//! explicita.

use salvo::cors::Cors;
use salvo::http::Method;
use salvo::prelude::*;

/// Origenes admin permitidos por defecto.
///
/// `Origin` no lleva slash final: `https://inemadmin.apptolast.com`, no
/// `https://inemadmin.apptolast.com/`.
pub const DEFAULT_ADMIN_WEB_ORIGINS: &str = "https://inemadmin.apptolast.com,http://localhost:8081";

/// Construye el middleware CORS para toda la API.
///
/// `admin_web_origins` debe ser una lista separada por comas, por ejemplo:
/// `https://inemadmin.apptolast.com,http://localhost:8081`.
pub fn cors_handler(admin_web_origins: &str) -> impl Handler {
    let origins = parse_origins(admin_web_origins);

    Cors::new()
        .allow_origin(&origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec!["content-type", "authorization"])
        .expose_headers(vec!["authorization", "content-type"])
        .into_handler()
}

fn parse_origins(admin_web_origins: &str) -> Vec<String> {
    let origins = admin_web_origins
        .split(',')
        .filter_map(normalize_origin)
        .collect::<Vec<_>>();

    if origins.is_empty() {
        DEFAULT_ADMIN_WEB_ORIGINS
            .split(',')
            .filter_map(normalize_origin)
            .collect()
    } else {
        origins
    }
}

fn normalize_origin(raw_origin: &str) -> Option<String> {
    let origin = raw_origin.trim().trim_end_matches('/');
    if origin.is_empty() {
        return None;
    }

    assert_ne!(
        origin, "*",
        "ADMIN_WEB_ORIGINS no puede usar '*'; configura origenes concretos separados por coma"
    );
    assert!(
        origin.starts_with("http://") || origin.starts_with("https://"),
        "ADMIN_WEB_ORIGINS contiene un origen sin protocolo http/https: {origin}"
    );

    Some(origin.to_string())
}
