use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use salvo::affix_state;
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
use uuid::Uuid;

use inem_sellar_backend::middleware::{admin_middleware, auth_middleware};
use inem_sellar_backend::services::AuthService;

const JWT_SECRET: &str = "test-jwt-secret-no-usado-en-prod";

#[handler]
async fn admin_ok() -> &'static str {
    "ok"
}

fn build_service() -> Service {
    let auth_service = AuthService::new(JWT_SECRET.to_string(), 15);
    let router = Router::new().hoop(affix_state::inject(auth_service)).push(
        Router::with_path("admin")
            .hoop(auth_middleware)
            .hoop(admin_middleware)
            .get(admin_ok),
    );
    Service::new(router)
}

fn token(admin: bool) -> String {
    AuthService::new(JWT_SECRET.to_string(), 15)
        .generar_access_token_con_flags(Uuid::new_v4(), false, admin)
        .unwrap()
}

fn token_legacy_sin_admin() -> String {
    let now = Utc::now().timestamp();
    let claims = json!({
        "sub": Uuid::new_v4().to_string(),
        "exp": now + 3600,
        "iat": now,
        "anonimo": false
    });
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    )
    .unwrap()
}

#[tokio::test]
async fn admin_middleware_permita_admin_true() {
    let service = build_service();

    let res = TestClient::get("http://127.0.0.1/admin")
        .bearer_auth(token(true))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::OK));
}

#[tokio::test]
async fn admin_middleware_rechaza_usuario_no_admin() {
    let service = build_service();

    let res = TestClient::get("http://127.0.0.1/admin")
        .bearer_auth(token(false))
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::FORBIDDEN));
}

#[tokio::test]
async fn admin_middleware_rechaza_token_legacy_sin_claim_admin() {
    let service = build_service();

    let res = TestClient::get("http://127.0.0.1/admin")
        .bearer_auth(token_legacy_sin_admin())
        .send(&service)
        .await;

    assert_eq!(res.status_code, Some(StatusCode::FORBIDDEN));
}
