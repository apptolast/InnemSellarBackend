// src/models/configuracion_aplicacion.rs
//
// Tabla: configuracion_aplicacion (key-value global)
// CONCEPTO NUEVO: PK de tipo TEXT (String en Rust, no i32 ni Uuid)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Configuracion global de la app en formato clave-valor.
///
/// # PK de tipo String
/// A diferencia de las demas tablas que usan i32 o Uuid como PK,
/// aqui la PK es TEXT (String en Rust). Es un patron key-value
/// donde la clave es un nombre descriptivo como:
/// - "modo_mantenimiento" → "false"
/// - "version_minima_app" → "2.0.0"
/// - "max_ofertas_por_usuario" → "50"
///
/// Permite cambiar comportamiento de la app sin redesplegar el backend.
///
/// # Sin creado_en
/// Esta tabla solo tiene actualizado_en (no creado_en) porque
/// las configuraciones se insertan una vez y se actualizan muchas.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfiguracionAplicacion {
    /// TEXT PRIMARY KEY — nombre de la configuracion (NOT NULL por ser PK)
    pub clave: String,

    /// Valor de la configuracion (siempre texto, el backend lo parsea)
    pub valor: Option<String>,

    /// Explicacion de para que sirve esta clave
    pub descripcion: Option<String>,

    /// Se actualiza via trigger
    pub actualizado_en: Option<DateTime<Utc>>,
}
