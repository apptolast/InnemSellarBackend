//! Servicio de notificacion por email para reportes de moderacion.
//!
//! Cuando un usuario reporta contenido (oferta/consejo/curso) via
//! `POST /api/v1/reportes`, este servicio envia un correo al admin con los
//! datos del reporte para que pueda revisarlo y aceptarlo o rechazarlo.
//!
//! # Por que SMTP y no HTTP API
//! Reutilizamos el SMTP de Gmail Workspace que ya esta configurado para otros
//! servicios de la organizacion (mismo host, mismo usuario autenticado, misma
//! app password). Asi evitamos crear una cuenta nueva en un proveedor de
//! terceros y mantenemos las credenciales centralizadas.
//!
//! # Por que Tokio + rustls (no native-tls)
//! El resto del crate (sea-orm, reqwest) usa rustls. Manteniendo coherencia
//! evitamos arrastrar `libssl-dev` al `Dockerfile` y simplificamos el build.
//!
//! # Por que un struct y no un trait + Arc<dyn ...>
//! El patron del proyecto para servicios (ver `auth_service.rs`) es un struct
//! `Clone` con metodos sincronos/async. Solo los repositorios usan traits
//! (porque tienen multiples implementaciones: `SeaXxxRepo` + futuros mocks).
//! Aqui solo hay una implementacion (Gmail SMTP), asi que un struct concreto
//! es mas simple. Los tests cubren la funcion pura `formatear_mensaje`.
//!
//! # Modo deshabilitado (dev local)
//! Si `SMTP_PASSWORD` o `REPORT_EMAIL_TO` estan vacios, `from_config`
//! construye un notifier "no-op": `enviar_notificacion_reporte` retorna
//! `Ok(())` inmediatamente. Permite arrancar el servidor sin SMTP en local.

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart, SinglePart, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::config::AppConfig;
use crate::models::enums::{MotivoReporte, TipoContenido};
use crate::models::reporte;

// ─── Errores ────────────────────────────────────────────────────────────────

/// Error que puede surgir al construir o enviar un mensaje.
///
/// # Por que un enum propio en lugar de `AppError::Internal(String)`
/// Las llamadas al notifier ocurren en un `tokio::spawn` fire-and-forget que
/// solo loguea el error con `tracing::warn!`. Tener variantes tipadas hace
/// que los logs sean mas utiles (sabemos si el fallo es de transporte SMTP,
/// de parseo de direccion, o de construccion del MIME) sin perder el detalle.
#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    /// Error construyendo el mensaje MIME (cabeceras, multipart, etc.).
    #[error("Error construyendo mensaje: {0}")]
    Build(#[from] lettre::error::Error),

    /// Error parseando una direccion de correo (formato invalido).
    #[error("Error parseando direccion de correo: {0}")]
    Address(#[from] lettre::address::AddressError),

    /// Error en el transporte SMTP (conexion, autenticacion, envio).
    #[error("Error de transporte SMTP: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

// ─── Servicio ───────────────────────────────────────────────────────────────

/// Servicio de envio de notificaciones por email.
///
/// # Por que `Option<...>` en cada campo
/// En modo deshabilitado (sin SMTP configurado) el notifier sigue siendo
/// inyectable en el `Depot` y `enviar_notificacion_reporte` no falla — solo
/// hace early-return. Asi los handlers no necesitan `if cfg.smtp_*` ni
/// ramas distintas para test/dev/prod.
///
/// # Por que `Clone`
/// Salvo necesita que los tipos inyectados con `affix_state::inject` sean
/// `Clone`. `AsyncSmtpTransport` implementa `Clone` internamente con un
/// `Arc`, asi que la copia es barata (incrementa un contador).
#[derive(Clone)]
pub struct EmailNotifier {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: Option<Mailbox>,
    to: Option<Mailbox>,
}

impl EmailNotifier {
    /// Construye el notifier a partir de la configuracion global.
    ///
    /// Si `smtp_password` o `report_email_to` estan vacios, devuelve un
    /// notifier deshabilitado y loguea un warning. Esto permite arrancar
    /// el servidor en entornos sin SMTP (tests, dev local).
    pub fn from_config(cfg: &AppConfig) -> Result<Self, EmailError> {
        if cfg.smtp_password.is_empty() || cfg.report_email_to.is_empty() {
            tracing::warn!("EmailNotifier deshabilitado: SMTP_PASSWORD o REPORT_EMAIL_TO vacios");
            return Ok(Self {
                transport: None,
                from: None,
                to: None,
            });
        }

        let from: Mailbox = cfg.report_email_from.parse()?;
        let to: Mailbox = cfg.report_email_to.parse()?;

        // STARTTLS sobre el puerto 587 (submission). Gmail rechaza autenticacion
        // sin TLS, asi que no usamos `builder_dangerous` ni `relay` en plano.
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)?
            .port(cfg.smtp_port)
            .credentials(Credentials::new(
                cfg.smtp_user.clone(),
                cfg.smtp_password.clone(),
            ))
            .build();

        tracing::info!(
            host = %cfg.smtp_host,
            port = cfg.smtp_port,
            user = %cfg.smtp_user,
            from = %from,
            to = %to,
            "EmailNotifier inicializado"
        );

        Ok(Self {
            transport: Some(transport),
            from: Some(from),
            to: Some(to),
        })
    }

    /// Envia el correo de notificacion al admin con los datos del reporte.
    ///
    /// Si el notifier esta deshabilitado (modo dev sin SMTP) retorna
    /// `Ok(())` inmediatamente sin hacer red.
    ///
    /// # Errores
    /// Devuelve `EmailError::Build` si el mensaje MIME no puede construirse,
    /// o `EmailError::Smtp` si el envio falla (DNS, TLS, autenticacion,
    /// rechazo del servidor). El llamador (handler) solo loguea el error.
    pub async fn enviar_notificacion_reporte(
        &self,
        reporte: &reporte::Model,
    ) -> Result<(), EmailError> {
        let (Some(transport), Some(from), Some(to)) = (
            self.transport.as_ref(),
            self.from.as_ref(),
            self.to.as_ref(),
        ) else {
            tracing::debug!("EmailNotifier deshabilitado, no se envia email");
            return Ok(());
        };

        let message = formatear_mensaje(reporte, from, to)?;
        transport.send(message).await?;

        tracing::info!(
            reporte_id = %reporte.id,
            "Email de notificacion enviado al admin"
        );
        Ok(())
    }
}

// ─── Helpers de presentacion ────────────────────────────────────────────────

/// Convierte el enum `TipoContenido` en su representacion string usada en la
/// API (lowercase). No usamos `Debug` (que daria PascalCase) ni dependemos
/// de implementar `Display` en el enum (lo que tocaria al modulo de modelos).
fn tipo_a_str(tipo: &TipoContenido) -> &'static str {
    match tipo {
        TipoContenido::Oferta => "oferta",
        TipoContenido::Consejo => "consejo",
        TipoContenido::Curso => "curso",
    }
}

/// Convierte `MotivoReporte` en su representacion string de la API.
fn motivo_a_str(motivo: &MotivoReporte) -> &'static str {
    match motivo {
        MotivoReporte::Spam => "spam",
        MotivoReporte::Inapropiado => "inapropiado",
        MotivoReporte::Desactualizado => "desactualizado",
        MotivoReporte::Incorrecto => "incorrecto",
        MotivoReporte::Duplicado => "duplicado",
        MotivoReporte::Otro => "otro",
    }
}

fn fmt_opt<T: std::fmt::Display>(v: &Option<T>) -> String {
    v.as_ref()
        .map(|x| x.to_string())
        .unwrap_or_else(|| "(sin dato)".into())
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Construye el `Message` MIME (texto plano + HTML) listo para enviar.
///
/// Es una funcion pura: no toca red ni estado global, solo lee del `Reporte`
/// y formatea. El test unitario al final del archivo cubre esta funcion.
fn formatear_mensaje(
    reporte: &reporte::Model,
    from: &Mailbox,
    to: &Mailbox,
) -> Result<Message, EmailError> {
    let tipo = reporte
        .tipo_contenido
        .as_ref()
        .map(tipo_a_str)
        .unwrap_or("(sin tipo)");
    let motivo = reporte
        .motivo
        .as_ref()
        .map(motivo_a_str)
        .unwrap_or("(sin motivo)");

    let subject = format!("[Inem Sellar] Nuevo reporte: {tipo} - {motivo}");

    let id_contenido = fmt_opt(&reporte.id_contenido);
    let detalle = reporte
        .detalle_motivo
        .clone()
        .unwrap_or_else(|| "(sin detalle)".into());
    let creado_en = fmt_opt(&reporte.creado_en);
    let estado = reporte
        .estado
        .as_ref()
        .map(|e| format!("{e:?}").to_lowercase())
        .unwrap_or_else(|| "(sin estado)".into());

    let texto = format!(
        "Nuevo reporte recibido en Inem Sellar\n\
         \n\
         Id reporte:    {id}\n\
         Tipo:          {tipo}\n\
         Id contenido:  {id_contenido}\n\
         Motivo:        {motivo}\n\
         Detalle:       {detalle}\n\
         Estado:        {estado}\n\
         Reportero:     {reportero}\n\
         Creado en:     {creado_en}\n\
         \n\
         Revisa el reporte desde el panel de moderacion para aceptarlo o rechazarlo.\n",
        id = reporte.id,
        reportero = reporte.id_reportero,
    );

    let html = format!(
        "<!DOCTYPE html><html><body style=\"font-family:Arial,sans-serif;color:#222;\">\
         <h2>Nuevo reporte recibido en Inem Sellar</h2>\
         <table style=\"border-collapse:collapse;\">\
           <tr><td><b>Id reporte</b></td><td><code>{id}</code></td></tr>\
           <tr><td><b>Tipo</b></td><td>{tipo}</td></tr>\
           <tr><td><b>Id contenido</b></td><td><code>{id_contenido}</code></td></tr>\
           <tr><td><b>Motivo</b></td><td>{motivo}</td></tr>\
           <tr><td><b>Detalle</b></td><td>{detalle}</td></tr>\
           <tr><td><b>Estado</b></td><td>{estado}</td></tr>\
           <tr><td><b>Reportero</b></td><td><code>{reportero}</code></td></tr>\
           <tr><td><b>Creado en</b></td><td>{creado_en}</td></tr>\
         </table>\
         <p>Revisa el reporte desde el panel de moderacion para aceptarlo o rechazarlo.</p>\
         </body></html>",
        id = reporte.id,
        reportero = reporte.id_reportero,
        detalle = html_escape(&detalle),
    );

    let message = Message::builder()
        .from(from.clone())
        .to(to.clone())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(texto),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html),
                ),
        )?;

    Ok(message)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    fn reporte_fake() -> reporte::Model {
        reporte::Model {
            id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
            tipo_contenido: Some(TipoContenido::Oferta),
            id_contenido: Some(Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()),
            id_reportero: Uuid::parse_str("33333333-3333-3333-3333-333333333333").unwrap(),
            motivo: Some(MotivoReporte::Spam),
            detalle_motivo: Some("Contenido sospechoso de spam <test>".into()),
            estado: None,
            id_procesador: None,
            procesado_en: None,
            creado_en: Some(Utc.with_ymd_and_hms(2026, 5, 7, 10, 0, 0).unwrap().into()),
            actualizado_en: None,
        }
    }

    #[test]
    fn formatea_subject_y_cuerpo_con_datos_del_reporte() {
        let from: Mailbox = "admin@apptolast.com".parse().unwrap();
        let to: Mailbox = "admin@apptolast.com".parse().unwrap();
        let r = reporte_fake();

        let msg = formatear_mensaje(&r, &from, &to).expect("debe construirse");
        let raw = String::from_utf8(msg.formatted()).expect("MIME debe ser UTF-8");

        // Subject con tipo y motivo
        assert!(
            raw.contains("[Inem Sellar] Nuevo reporte: oferta - spam"),
            "subject debe contener tipo y motivo, raw={raw}"
        );
        // Id del reporte y del contenido aparecen en el cuerpo
        assert!(raw.contains(&r.id.to_string()));
        assert!(raw.contains(&r.id_contenido.unwrap().to_string()));
        assert!(raw.contains(&r.id_reportero.to_string()));
        // Detalle escapado en HTML (los `<` y `>` deben aparecer codificados)
        assert!(raw.contains("&lt;test&gt;"));
        // Cabeceras From / To
        assert!(raw.contains("admin@apptolast.com"));
    }

    #[test]
    fn notifier_deshabilitado_si_falta_password() {
        let cfg = AppConfig {
            database_url: "x".into(),
            server_addr: "x".into(),
            port_addr: "x".into(),
            jwt_secret: "x".into(),
            jwt_expiracion_minutos: 15,
            firebase_project_id: "x".into(),
            smtp_host: "smtp.gmail.com".into(),
            smtp_port: 587,
            smtp_user: "admin@apptolast.com".into(),
            smtp_password: "".into(), // vacio -> deshabilitado
            report_email_from: "admin@apptolast.com".into(),
            report_email_to: "admin@apptolast.com".into(),
        };

        let notifier = EmailNotifier::from_config(&cfg).expect("no debe fallar");
        assert!(notifier.transport.is_none());
        assert!(notifier.from.is_none());
        assert!(notifier.to.is_none());
    }
}
