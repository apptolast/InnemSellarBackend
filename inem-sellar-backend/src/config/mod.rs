use serde::Deserialize;
use crate::config;

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    #[serde(default = "default_addr")]
    pub server_addr: String,
}

fn default_addr() -> String {
    "0.0.0.0:8080".to_string()
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        envy::from_env::<Self>().expect("Error cargando configuración desde variables de entorno")
    }
}