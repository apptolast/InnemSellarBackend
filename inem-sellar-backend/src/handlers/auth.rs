//! Handlers de autenticacion: registro, login, refrescar, logout.
//!
//! Cada handler coordina servicio + repositorio, pero NO contiene
//! logica de criptografia ni acceso a BD directamente.
//!
//! Los handlers usan `#[endpoint]` (en vez de `#[handler]`) para que
//! Salvo pueda generar la documentacion OpenAPI automaticamente.

use chrono::{Duration, Utc};
use salvo::oapi::extract::JsonBody;
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
///
/// # Por que `ToSchema`
/// `ToSchema` es un trait de Salvo OAPI que le dice al generador de OpenAPI
/// como representar este struct en la documentacion como un JSON Schema.
/// Sin el, `#[endpoint]` no puede documentar el cuerpo de la peticion.
/// Es como anotaciones `@JsonSerializable` en Dart pero para documentacion API.
#[derive(Deserialize, ToSchema)]
pub struct RegistroRequest {
    /// Email del nuevo usuario. Debe ser unico en el sistema.
    pub email: String,
    /// Contrasena en texto plano. Se hashea con Argon2id antes de guardar.
    pub contrasena: String,
    /// Nombre que se mostrara publicamente. Opcional.
    pub nombre_visible: Option<String>,
}

/// Body del POST /api/v1/auth/login
#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Email registrado del usuario.
    pub email: String,
    /// Contrasena en texto plano para verificar contra el hash almacenado.
    pub contrasena: String,
}

/// Body del POST /api/v1/auth/refrescar
#[derive(Deserialize, ToSchema)]
pub struct RefrescarRequest {
    /// Refresh token emitido en registro o login. Se revoca tras usarlo (rotacion).
    pub refresh_token: String,
}

/// Body del POST /api/v1/auth/logout
#[derive(Deserialize, ToSchema)]
pub struct LogoutRequest {
    /// Refresh token a revocar. Tras el logout el token queda invalido.
    pub refresh_token: String,
}

/// Respuesta de registro y login — contiene tokens + datos basicos del usuario.
///
/// # Por que `Serialize` y `ToSchema`
/// `Serialize` (de serde) convierte el struct a JSON para la respuesta HTTP.
/// `ToSchema` permite que OpenAPI documente la estructura de la respuesta.
/// Juntos, forman el contrato completo: implementacion + documentacion.
#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    /// Token JWT de acceso. Corta duracion (15 min por defecto).
    /// Incluirlo en `Authorization: Bearer <token>` en cada peticion protegida.
    pub access_token: String,
    /// Token de refresco. Larga duracion (30 dias). Guardar de forma segura.
    pub refresh_token: String,
    /// Datos basicos del usuario recien autenticado.
    pub usuario: UsuarioResponse,
}

/// Datos publicos del usuario (sin hash_contrasena).
#[derive(Serialize, ToSchema)]
pub struct UsuarioResponse {
    /// UUID del usuario en la base de datos.
    pub id: Uuid,
    /// Email del usuario. Puede ser None si se registro solo con OAuth.
    pub email: Option<String>,
    /// Nombre visible publicamente. Puede ser None si no lo proporcionó.
    pub nombre_visible: Option<String>,
}

/// Respuesta de refrescar — solo tokens nuevos.
#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    /// Nuevo access token JWT.
    pub access_token: String,
    /// Nuevo refresh token (el anterior queda revocado).
    pub refresh_token: String,
}

/// Respuesta generica con mensaje de confirmacion.
#[derive(Serialize, ToSchema)]
pub struct MensajeResponse {
    /// Mensaje descriptivo de la operacion realizada.
    pub mensaje: String,
}

// ─── Handlers ────────────────────────────────────────────────────

/// POST /api/v1/auth/registro — Crear cuenta nueva.
///
/// # Flujo
/// 1. Parsear body JSON → RegistroRequest (via `JsonBody<T>` extractor)
/// 2. Verificar que el email no exista (409 si ya existe)
/// 3. Hashear la contrasena con Argon2id
/// 4. Crear el usuario en la BD
/// 5. Generar access token (JWT) + refresh token
/// 6. Guardar hash del refresh token en BD
/// 7. Devolver tokens + datos del usuario
///
/// # Por que `#[endpoint]` en vez de `#[handler]`
/// `#[endpoint]` hace todo lo que hace `#[handler]` PERO ademas genera
/// metadata OpenAPI (path, metodo HTTP, parametros, respuestas) que Salvo
/// usa para construir la documentacion Swagger automaticamente.
/// La diferencia es solo en tiempo de compilacion — el comportamiento
/// en runtime es identico.
///
/// # Por que `JsonBody<RegistroRequest>` en vez de `req.parse_json()`
/// `JsonBody<T>` es un "extractor" tipado de Salvo OAPI. Salvo lo detecta
/// en la firma de la funcion y lo documenta como el body esperado en OpenAPI.
/// Con `req.parse_json()`, Salvo no puede inferir el tipo del body para documentarlo.
#[endpoint(tags("Auth"))]
pub async fn registro(
    body: JsonBody<RegistroRequest>,
    depot: &mut Depot,
) -> Result<Json<AuthResponse>, AppError> {
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
#[endpoint(tags("Auth"))]
pub async fn login(
    body: JsonBody<LoginRequest>,
    depot: &mut Depot,
) -> Result<Json<AuthResponse>, AppError> {
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
#[endpoint(tags("Auth"))]
pub async fn refrescar(
    body: JsonBody<RefrescarRequest>,
    depot: &mut Depot,
) -> Result<Json<TokenResponse>, AppError> {
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
///
/// # Por que `security(("bearer_auth" = []))`
/// Esta anotacion le dice a OpenAPI que este endpoint requiere el esquema
/// de seguridad `bearer_auth` definido en main.rs (JWT Bearer token).
/// Swagger UI mostrara un candado y permitira al usuario introducir su token
/// para probar el endpoint directamente desde la documentacion.
#[endpoint(tags("Auth"), security(("bearer_auth" = [])))]
pub async fn logout(
    body: JsonBody<LogoutRequest>,
    depot: &mut Depot,
) -> Result<Json<MensajeResponse>, AppError> {
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
