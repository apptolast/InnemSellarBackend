// src/middleware/auth.rs
//
// Middleware de autenticacion JWT para Salvo.
//
// # Como funciona un middleware en Salvo
// Un middleware es un handler que decide si la peticion puede continuar.
// Si el token es valido: inserta el id_usuario en Depot y llama a call_next.
// Si no: responde 401 y llama a skip_rest (no llega al handler).
//
// Es como un middleware de Express.js o un interceptor de Dio en Flutter:
//   interceptor.onRequest = (options, handler) {
//     if (tokenValido) handler.next(options);
//     else handler.reject(DioException.unauthorized);
//   };

use salvo::http::StatusCode;
use salvo::prelude::*;
use uuid::Uuid;

use crate::services::AuthService;

/// Middleware que verifica el JWT en el header Authorization.
///
/// # Flujo
/// 1. Busca el header `Authorization: Bearer <token>`
/// 2. Extrae el token (quita el prefijo "Bearer ")
/// 3. Verifica el JWT con AuthService
/// 4. Si es valido: inserta `id_usuario` en Depot → call_next
/// 5. Si no: responde 401 → skip_rest
#[handler]
pub async fn auth_middleware(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    // Obtenemos el AuthService del Depot (inyectado en main.rs)
    let auth_service = match depot.obtain::<AuthService>() {
        Ok(service) => service.clone(),
        Err(_) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Json(serde_json::json!({
                "error": "Error interno del servidor",
                "codigo": 500
            })));
            ctrl.skip_rest();
            return;
        }
    };

    // Extraemos el token del header Authorization
    let token = match req.header::<String>("authorization") {
        Some(header) if header.starts_with("Bearer ") => header[7..].to_string(),
        _ => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "No autenticado",
                "codigo": 401
            })));
            ctrl.skip_rest();
            return;
        }
    };

    // Verificamos el JWT
    match auth_service.verificar_access_token(&token) {
        Ok(claims) => {
            // Token valido: extraemos el UUID del usuario y lo metemos en Depot
            if let Ok(id_usuario) = Uuid::parse_str(&claims.sub) {
                depot.insert("id_usuario", id_usuario);
                ctrl.call_next(req, depot, res).await;
            } else {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(serde_json::json!({
                    "error": "Token con formato invalido",
                    "codigo": 401
                })));
                ctrl.skip_rest();
            }
        }
        Err(_) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Json(serde_json::json!({
                "error": "Token invalido o expirado",
                "codigo": 401
            })));
            ctrl.skip_rest();
        }
    }
}
