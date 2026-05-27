use salvo::affix_state;
use salvo::http::header::{
    ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_EXPOSE_HEADERS, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD,
    HeaderName, ORIGIN, VARY,
};
use salvo::prelude::*;
use salvo::test::TestClient;

use inem_sellar_backend::middleware::{auth_middleware, cors_handler};
use inem_sellar_backend::services::AuthService;

const ALLOWED_ORIGINS: &str = " https://inemadmin.apptolast.com/ , http://localhost:8081/ ";
const ADMIN_ORIGIN: &str = "https://inemadmin.apptolast.com";
const JWT_SECRET: &str = "test-jwt-secret-no-usado-en-prod";

#[handler]
async fn ok() -> &'static str {
    "ok"
}

fn service_auth_firebase() -> Service {
    let router = Router::new().push(Router::with_path("api/v1/auth/firebase").post(ok));
    Service::new(router).hoop(cors_handler(ALLOWED_ORIGINS))
}

fn service_protected_cursos() -> Service {
    let auth_service = AuthService::new(JWT_SECRET.to_string(), 15);
    let router = Router::new().hoop(affix_state::inject(auth_service)).push(
        Router::with_path("api/v1/cursos")
            .hoop(auth_middleware)
            .get(ok),
    );
    Service::new(router).hoop(cors_handler(ALLOWED_ORIGINS))
}

fn header<'a>(res: &'a Response, name: &HeaderName) -> Option<&'a str> {
    res.headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

fn header_values(res: &Response, name: &HeaderName) -> Vec<String> {
    res.headers()
        .get_all(name)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .map(str::to_string)
        .collect()
}

fn assert_header_contains(res: &Response, name: &HeaderName, expected: &str) {
    let values = header_values(res, name);
    assert!(
        values
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(expected)),
        "header {name} no contiene {expected:?}; valores: {values:?}"
    );
}

#[tokio::test]
async fn preflight_auth_firebase_devuelve_204_con_cors() {
    let service = service_auth_firebase();

    let res = TestClient::options("http://127.0.0.1/api/v1/auth/firebase")
        .add_header(ORIGIN, ADMIN_ORIGIN, true)
        .add_header(ACCESS_CONTROL_REQUEST_METHOD, "POST", true)
        .add_header(ACCESS_CONTROL_REQUEST_HEADERS, "content-type", true)
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::NO_CONTENT));
    assert_eq!(
        header(&res, &ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(ADMIN_ORIGIN)
    );
    assert_header_contains(&res, &ACCESS_CONTROL_ALLOW_METHODS, "post");
    assert_header_contains(&res, &ACCESS_CONTROL_ALLOW_HEADERS, "content-type");
    assert_header_contains(&res, &VARY, "origin");
}

#[tokio::test]
async fn preflight_endpoint_protegido_no_exige_authorization() {
    let service = service_protected_cursos();

    let res = TestClient::options("http://127.0.0.1/api/v1/cursos")
        .add_header(ORIGIN, ADMIN_ORIGIN, true)
        .add_header(ACCESS_CONTROL_REQUEST_METHOD, "GET", true)
        .add_header(ACCESS_CONTROL_REQUEST_HEADERS, "authorization", true)
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::NO_CONTENT));
    assert_eq!(
        header(&res, &ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(ADMIN_ORIGIN)
    );
    assert_header_contains(&res, &ACCESS_CONTROL_ALLOW_METHODS, "get");
    assert_header_contains(&res, &ACCESS_CONTROL_ALLOW_HEADERS, "authorization");
}

#[tokio::test]
async fn errores_de_auth_tambien_llevan_cors() {
    let service = service_protected_cursos();

    let res = TestClient::get("http://127.0.0.1/api/v1/cursos")
        .add_header(ORIGIN, ADMIN_ORIGIN, true)
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::UNAUTHORIZED));
    assert_eq!(
        header(&res, &ACCESS_CONTROL_ALLOW_ORIGIN),
        Some(ADMIN_ORIGIN)
    );
    assert_header_contains(&res, &ACCESS_CONTROL_EXPOSE_HEADERS, "authorization");
}

#[tokio::test]
async fn origen_no_permitido_no_recibe_allow_origin() {
    let service = service_auth_firebase();

    let res = TestClient::options("http://127.0.0.1/api/v1/auth/firebase")
        .add_header(ORIGIN, "https://otro-admin.example", true)
        .add_header(ACCESS_CONTROL_REQUEST_METHOD, "POST", true)
        .add_header(ACCESS_CONTROL_REQUEST_HEADERS, "content-type", true)
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::NO_CONTENT));
    assert_eq!(header(&res, &ACCESS_CONTROL_ALLOW_ORIGIN), None);
}
