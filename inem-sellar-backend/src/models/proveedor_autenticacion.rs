// src/models/proveedor_autenticacion.rs
//
// Tabla: proveedores_autenticacion
// CONCEPTO NUEVO: JSONB → serde_json::Value

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Proveedor OAuth externo (Google, Apple) vinculado a un usuario.
///
/// # Por que `serde_json::Value` para JSONB
/// PostgreSQL JSONB almacena JSON binario — puede tener cualquier estructura.
/// En Rust, `serde_json::Value` es un enum que representa cualquier valor JSON
/// valido (objeto, array, string, numero, bool, null). Es el equivalente a
/// `Map<String, dynamic>` en Dart.
///
/// Se usa porque cada proveedor OAuth devuelve datos distintos
/// (Google devuelve unos campos, Apple otros), asi que no podemos
/// definir un struct fijo.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProveedorAutenticacion {
    /// UUID PK
    pub id: Uuid,

    /// FK a usuarios — NOT NULL (siempre pertenece a un usuario)
    pub id_usuario: Uuid,

    /// Nombre del proveedor (ej: "google", "apple")
    pub proveedor: Option<String>,

    /// ID unico que el proveedor asigna al usuario
    pub identificador_proveedor: Option<String>,

    /// Email que el proveedor reporta para este usuario
    pub email_proveedor: Option<String>,

    /// Datos adicionales del proveedor en formato JSON libre.
    /// JSONB en PostgreSQL → serde_json::Value en Rust.
    pub datos_proveedor: Option<serde_json::Value>,

    pub creado_en: Option<DateTime<Utc>>,
    pub actualizado_en: Option<DateTime<Utc>>,
}
