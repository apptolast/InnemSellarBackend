// src/handlers/auth.rs
//
// Handlers de autenticacion: registro, login, refrescar, logout.
// Cada handler coordina servicio + repositorio, pero NO contiene
// logica de criptografia ni acceso a BD directamente.

use chrono::{Duration, Utc};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::repositories::{AuthRepo, SeaAuthRepo};
use crate::services::AuthService;

// ─── DTOs (Data Transfer Objects) ────────────────────────────────
// Structs que definen la forma del JSON de entrada y salida.
// Son diferentes a las entidades de BD — solo exponen lo necesario.

/// Body del POST /api/v1/auth/registro
#[derive(Deserialize)]
pub struct RegistroRequest {
    pub email: String,
    pub contrasena: String,
    pub nombre_visible: Option<String>,
}

/// Body del POST /api/v1/auth/login
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub contrasena: String,
}

/// Body del POST /api/v1/auth/refrescar
#[derive(Deserialize)]
pub struct RefrescarRequest {
    pub refresh_token: String,
}

/// Body del POST /api/v1/auth/logout
#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

/// Respuesta de registro y login — contiene tokens + datos basicos del usuario
#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub usuario: UsuarioResponse,
}

/// Datos publicos del usuario (sin hash_contrasena)
#[derive(Serialize)]
pub struct UsuarioResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub nombre_visible: Option<String>,
}

/// Respuesta de refrescar — solo tokens nuevos
#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

/// Respuesta generica con mensaje
#[derive(Serialize)]
pub struct MensajeResponse {
    pub mensaje: String,
}

// ─── Handlers ────────────────────────────────────────────────────

/// POST /api/v1/auth/registro — Crear cuenta nueva.
///
/// # Flujo
/// 1. Parsear body JSON → RegistroRequest
/// 2. Verificar que el email no exista (409 si ya existe)
/// 3. Hashear la contrasena con Argon2id
/// 4. Crear el usuario en la BD
/// 5. Generar access token (JWT) + refresh token
/// 6. Guardar hash del refresh token en BD
/// 7. Devolver tokens + datos del usuario
#[handler]
pub async fn registro(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<AuthResponse>, AppError> {
    let body: RegistroRequest = req
        .parse_json()
        .await
        .map_err(|e| AppError::BadRequest(format!("JSON invalido: {e}")))?;

    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();

    // Verificar que el email no este registrado
    if auth_repo.buscar_por_email(&body.email).await?.is_some() {
        return Err(AppError::Conflict("El email ya esta registrado".into()));
    }

    // Hashear contrasena y crear usuario
    let hash = auth_service.hashear_contrasena(&body.contrasena)?;
    let usuario = auth_repo
        .crear_usuario(&body.email, &hash, body.nombre_visible.as_deref())
        .await?;

    // Generar tokens
    let access_token = auth_service.generar_access_token(usuario.id)?;
    let refresh_raw = auth_service.generar_refresh_token();
    let refresh_hash = auth_service.hashear_refresh_token(&refresh_raw);

    // Guardar refresh token en BD (expira en 30 dias)
    let expira = (Utc::now() + Duration::days(30)).fixed_offset();
    auth_repo
        .guardar_refresh_token(usuario.id, &refresh_hash, None, expira)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: refresh_raw,
        usuario: UsuarioResponse {
            id: usuario.id,
            email: usuario.email.clone(),
            nombre_visible: usuario.nombre_visible.clone(),
        },
    }))
}

/// POST /api/v1/auth/login — Iniciar sesion.
#[handler]
pub async fn login(req: &mut Request, depot: &mut Depot) -> Result<Json<AuthResponse>, AppError> {
    let body: LoginRequest = req
        .parse_json()
        .await
        .map_err(|e| AppError::BadRequest(format!("JSON invalido: {e}")))?;

    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();

    // Buscar usuario por email
    let usuario = auth_repo
        .buscar_por_email(&body.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verificar contrasena
    let hash = usuario
        .hash_contrasena
        .as_deref()
        .ok_or(AppError::Unauthorized)?;
    if !auth_service.verificar_contrasena(&body.contrasena, hash)? {
        return Err(AppError::Unauthorized);
    }

    // Actualizar ultimo login
    auth_repo.actualizar_ultimo_login(usuario.id).await?;

    // Generar tokens
    let access_token = auth_service.generar_access_token(usuario.id)?;
    let refresh_raw = auth_service.generar_refresh_token();
    let refresh_hash = auth_service.hashear_refresh_token(&refresh_raw);

    let expira = (Utc::now() + Duration::days(30)).fixed_offset();
    auth_repo
        .guardar_refresh_token(usuario.id, &refresh_hash, None, expira)
        .await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: refresh_raw,
        usuario: UsuarioResponse {
            id: usuario.id,
            email: usuario.email.clone(),
            nombre_visible: usuario.nombre_visible.clone(),
        },
    }))
}

/// POST /api/v1/auth/refrescar — Obtener nuevos tokens con un refresh token.
///
/// # Rotacion de tokens
/// Al refrescar, el refresh token anterior se revoca y se emite uno nuevo.
/// Esto limita el dano si un refresh token es comprometido: el atacante
/// solo tiene una ventana corta antes de que el token se invalide.
#[handler]
pub async fn refrescar(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<TokenResponse>, AppError> {
    let body: RefrescarRequest = req
        .parse_json()
        .await
        .map_err(|e| AppError::BadRequest(format!("JSON invalido: {e}")))?;

    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();

    // Hashear el refresh token recibido y buscarlo en BD
    let hash = auth_service.hashear_refresh_token(&body.refresh_token);
    let token_db = auth_repo
        .buscar_refresh_token_por_hash(&hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    // Verificar que no ha expirado
    if let Some(expira) = token_db.expira_en
        && expira < Utc::now().fixed_offset()
    {
        return Err(AppError::Unauthorized);
    }

    // Revocar el token anterior (rotacion)
    auth_repo.revocar_refresh_token(token_db.id).await?;

    // Generar nuevo par de tokens
    let access_token = auth_service.generar_access_token(token_db.id_usuario)?;
    let new_refresh_raw = auth_service.generar_refresh_token();
    let new_refresh_hash = auth_service.hashear_refresh_token(&new_refresh_raw);

    let expira = (Utc::now() + Duration::days(30)).fixed_offset();
    auth_repo
        .guardar_refresh_token(token_db.id_usuario, &new_refresh_hash, None, expira)
        .await?;

    Ok(Json(TokenResponse {
        access_token,
        refresh_token: new_refresh_raw,
    }))
}

/// POST /api/v1/auth/logout — Cerrar sesion (revocar refresh token).
/// Requiere autenticacion (access token en header Authorization).
#[handler]
pub async fn logout(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
    let body: LogoutRequest = req
        .parse_json()
        .await
        .map_err(|e| AppError::BadRequest(format!("JSON invalido: {e}")))?;

    let auth_service = depot
        .obtain::<AuthService>()
        .map_err(|_| AppError::Internal("AuthService no disponible".into()))?
        .clone();
    let auth_repo = depot
        .obtain::<SeaAuthRepo>()
        .map_err(|_| AppError::Internal("AuthRepo no disponible".into()))?
        .clone();

    // Buscar el refresh token en BD y revocarlo
    let hash = auth_service.hashear_refresh_token(&body.refresh_token);
    let token_db = auth_repo
        .buscar_refresh_token_por_hash(&hash)
        .await?
        .ok_or(AppError::Unauthorized)?;

    auth_repo.revocar_refresh_token(token_db.id).await?;

    Ok(Json(MensajeResponse {
        mensaje: "Sesion cerrada".into(),
    }))
}
