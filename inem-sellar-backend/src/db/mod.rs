use crate::config::AppConfig;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn init_pool(cfg: &AppConfig) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&cfg.database_url)
        .await
        .expect("No se pudo conectar a PostgreSQL");

    tracing::info!("Conexión a PostgreSQL establecida correctamente");

    pool
}
