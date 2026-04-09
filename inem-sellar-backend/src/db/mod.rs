//! Modulo de acceso a la base de datos.
//!
//! Gestiona la conexion a PostgreSQL usando SeaORM.
//! SeaORM usa SQLx internamente pero expone su propia API
//! a traves de `DatabaseConnection`.

use crate::config::AppConfig;
use sea_orm::{Database, DatabaseConnection};

/// Inicializa la conexion a PostgreSQL via SeaORM.
///
/// # Cambio vs version anterior (SQLx directo)
/// Antes: `PgPoolOptions::new().connect(&url)` → devuelve `PgPool`
/// Ahora: `Database::connect(&url)` → devuelve `DatabaseConnection`
///
/// SeaORM gestiona el pool internamente (usa SQLx por debajo).
/// `DatabaseConnection` es `Clone` y `Send + Sync` — seguro para
/// compartir entre hilos y handlers.
///
/// # Por que seguimos usando `expect()` aqui
/// Es codigo de inicializacion. Si no hay BD, el servidor no puede
/// funcionar. Mejor fallar rapido con mensaje claro que arrancar
/// en estado inconsistente.
pub async fn init_db(cfg: &AppConfig) -> DatabaseConnection {
    let db = Database::connect(&cfg.database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL via SeaORM");

    tracing::info!("Conexion a PostgreSQL establecida correctamente (SeaORM)");

    db
}
