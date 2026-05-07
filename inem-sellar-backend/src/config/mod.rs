//! Modulo de configuracion de la aplicacion.
//!
//! Carga la configuracion desde variables de entorno usando `dotenvy` (para `.env`)
//! y `envy` (para deserializar variables de entorno en un struct tipado).

use serde::Deserialize;

/// Configuracion global de la aplicacion, cargada desde variables de entorno.
///
/// # Por que un struct con Deserialize
/// En Rust, `struct` agrupa datos relacionados (similar a una clase en Dart pero sin herencia).
/// `#[derive(Deserialize)]` genera automaticamente el codigo para convertir datos externos
/// (variables de entorno, JSON) en este struct — como un `fromJson()` en Dart, pero resuelto
/// en tiempo de compilacion (coste cero en runtime).
///
/// # Por que Clone
/// `Clone` permite crear copias independientes del struct. Lo necesitamos porque la config
/// se comparte entre multiples handlers async, y cada uno necesita su propia copia.
/// En Dart todos los objetos viven en el heap y el garbage collector gestiona la memoria;
/// en Rust, nosotros decidimos explicitamente cuando copiar datos.
#[derive(Deserialize, Clone)]
pub struct AppConfig {
    /// URL de conexion a PostgreSQL. Ejemplo: `postgres://user:pass@host/db`.
    /// Se lee de la variable de entorno `DATABASE_URL`.
    pub database_url: String,

    /// Direccion IP y puerto donde escucha el servidor.
    /// Valor por defecto: `0.0.0.0:8080` (todas las interfaces, puerto 8080).
    ///
    /// # Por que String y no &str
    /// `String` es un tipo "owned" — el struct es dueno del dato y lo mantiene en memoria.
    /// `&str` es una referencia prestada (borrowed) que necesitaria un lifetime `'a`,
    /// complicando el codigo. Regla practica: si el dato debe vivir tanto como el struct, usa `String`.
    #[serde(default = "default_addr")]
    pub server_addr: String,

    /// Direccion de binding completa del servidor en formato `IP:puerto`.
    /// Valor por defecto: `0.0.0.0:8080`.
    ///
    /// Se lee de la variable de entorno `PORT_ADDR`. Se usa cuando se necesita
    /// distinguir entre la direccion de escucha del socket TCP (`port_addr`,
    /// que usa `0.0.0.0` para aceptar todas las interfaces) y la direccion
    /// publica que se incluye en los logs (`server_addr`, que muestra `localhost`
    /// para que sea clickable en el terminal durante desarrollo).
    #[serde(default = "port_addr")]
    pub port_addr: String,

    /// Secreto para firmar/verificar tokens JWT (HS256).
    /// Se lee de la variable de entorno `JWT_SECRET`.
    /// CRITICO: en produccion debe ser un valor aleatorio largo (>= 32 chars).
    pub jwt_secret: String,

    /// Duracion del access token JWT en minutos.
    /// Default: 15 minutos. El frontend debe refrescar antes de que expire.
    #[serde(default = "default_jwt_expiracion")]
    pub jwt_expiracion_minutos: u64,

    /// Identificador del proyecto de Firebase (`projectId` de la consola).
    ///
    /// # Por que es obligatorio y no tiene default
    /// El verificador de Firebase ID Tokens valida que el claim `aud` (audience)
    /// del token coincida EXACTAMENTE con este valor, y que el claim `iss` sea
    /// `https://securetoken.google.com/<project_id>`. Si arrancamos el servidor
    /// con un valor incorrecto o vacio, todos los logins con Firebase fallarian
    /// silenciosamente con 401. Es preferible un panic claro al arranque.
    ///
    /// # De donde sale
    /// Es el mismo `project_id` que figura en `google-services.json` (Android)
    /// y `GoogleService-Info.plist` (iOS) del cliente Flutter.
    /// Se lee de la variable de entorno `FIREBASE_PROJECT_ID`.
    pub firebase_project_id: String,

    // ── Configuracion SMTP (notificaciones de reportes al admin) ────────────
    //
    // Todos los campos son opcionales (default: cadena vacia / 587). Si
    // `smtp_password` o `report_email_to` quedan vacios, el `EmailNotifier`
    // se construye en modo deshabilitado y los handlers siguen funcionando
    // sin enviar email — util para arrancar el servicio en dev sin SMTP.
    /// Host del servidor SMTP. Se lee de `SMTP_HOST` (ej. `smtp.gmail.com`).
    #[serde(default)]
    pub smtp_host: String,

    /// Puerto del servidor SMTP. Default: `587` (STARTTLS).
    /// Se lee de `SMTP_PORT`.
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,

    /// Usuario para autenticarse en el servidor SMTP.
    /// En Gmail Workspace coincide con la direccion de correo del remitente.
    /// Se lee de `SMTP_USER`.
    #[serde(default)]
    pub smtp_user: String,

    /// Contrasena (o app password en Gmail) del usuario SMTP.
    /// CRITICO: en produccion debe inyectarse via `Secret` de Kubernetes,
    /// nunca commitearse al repositorio. Se lee de `SMTP_PASSWORD`.
    #[serde(default)]
    pub smtp_password: String,

    /// Direccion `From:` que aparecera en el correo al admin.
    /// Debe coincidir con `smtp_user` (o ser un alias send-as configurado)
    /// para que Gmail no rechace el envio. Se lee de `REPORT_EMAIL_FROM`.
    #[serde(default)]
    pub report_email_from: String,

    /// Direccion `To:` del admin que recibe la notificacion.
    /// Si esta vacio, las notificaciones de reportes quedan deshabilitadas.
    /// Se lee de `REPORT_EMAIL_TO`.
    #[serde(default)]
    pub report_email_to: String,
}

/// Puerto SMTP por defecto: `587` (submission con STARTTLS).
///
/// # Por que 587 y no 465 ni 25
/// `587` es el puerto de "submission" (RFC 6409): autenticado y con STARTTLS.
/// `465` es SMTPS (TLS implicito) — Gmail tambien lo soporta, pero `587` es
/// mas estandar. `25` es relay entre servidores y casi todos los proveedores
/// lo bloquean para clientes (anti-spam).
fn default_smtp_port() -> u16 {
    587
}

/// Duracion por defecto del access token: 15 minutos.
fn default_jwt_expiracion() -> u64 {
    15
}

/// Devuelve la direccion por defecto del servidor.
///
/// # Por que una funcion separada
/// `serde` requiere que el valor por defecto de un campo sea una funcion (no un literal).
/// `#[serde(default = "default_addr")]` llama a esta funcion si la variable de entorno
/// `SERVER_ADDR` no esta definida. Es una limitacion del sistema de macros de serde.
///
/// # Por que `.to_string()`
/// `"0.0.0.0:8080"` es un `&str` (referencia a texto en el binario), pero necesitamos
/// un `String` (texto en el heap, owned). `.to_string()` crea esa copia en el heap.
fn default_addr() -> String {
    "localhost:8080".to_string()
}

/// Devuelve la direccion de binding por defecto del servidor.
///
/// Misma logica que `default_addr`: serde necesita una funcion, no un literal.
/// `0.0.0.0` significa "escucha en todas las interfaces de red disponibles",
/// lo que es necesario dentro de un contenedor Docker para que el trafico
/// externo pueda llegar al proceso.
fn port_addr() -> String {
    "0.0.0.0:8080".to_string()
}

impl AppConfig {
    /// Carga la configuracion desde variables de entorno.
    ///
    /// Primero carga el archivo `.env` si existe (via `dotenvy`), luego deserializa
    /// todas las variables de entorno en un `AppConfig` (via `envy`).
    ///
    /// # Por que `dotenvy::dotenv().ok()`
    /// `.ok()` convierte un `Result` en `Option`, descartando el error silenciosamente.
    /// Esto es intencionado: en produccion no habra archivo `.env` (las variables vienen
    /// del entorno del contenedor Docker), asi que no queremos que falle.
    ///
    /// # Por que `.expect()` y no `?`
    /// `expect()` hace panic con un mensaje claro si falla. Aqui es aceptable porque
    /// la configuracion es critica: sin ella, el servidor no puede arrancar. Es mejor
    /// fallar rapido con un mensaje claro que intentar continuar sin config.
    /// En handlers de API, usaremos `?` para propagar errores sin panic.
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        envy::from_env::<Self>().expect("Error cargando configuracion desde variables de entorno")
    }
}
