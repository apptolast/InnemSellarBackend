use std::sync::Arc;

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::enums::{EstadoModeracion, EstadoReporte, MotivoReporte, TipoContenido};
use crate::models::{consejo, curso, oferta_empleo, reporte};

pub struct CrearReporteDto {
    pub tipo_contenido: TipoContenido,
    pub id_contenido: Uuid,
    pub motivo: MotivoReporte,
    pub detalle_motivo: Option<String>,
}

pub trait ReporteRepo: Send + Sync {
    fn crear_reporte(
        &self,
        id_reportero: Uuid,
        datos: CrearReporteDto,
    ) -> impl std::future::Future<Output = Result<reporte::Model, AppError>> + Send;

    fn listar_reportes_pendientes(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<reporte::Model>, AppError>> + Send;

    /// Obtener un reporte por su UUID.
    fn obtener_reporte(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<reporte::Model, AppError>> + Send;

    /// Eliminar un reporte (admin).
    fn eliminar_reporte(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    /// Procesar un reporte: aceptar o rechazar.
    /// Registra quien lo proceso y cuando.
    fn procesar_reporte(
        &self,
        id: Uuid,
        id_procesador: Uuid,
        aceptar: bool,
        ocultar_contenido: bool,
    ) -> impl std::future::Future<Output = Result<reporte::Model, AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaReporteRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaReporteRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl ReporteRepo for SeaReporteRepo {
    async fn crear_reporte(
        &self,
        id_reportero: Uuid,
        datos: CrearReporteDto,
    ) -> Result<reporte::Model, AppError> {
        // Verificar que no haya un reporte duplicado del mismo usuario
        let existente = reporte::Entity::find()
            .filter(reporte::Column::TipoContenido.eq(Some(datos.tipo_contenido.clone())))
            .filter(reporte::Column::IdContenido.eq(Some(datos.id_contenido)))
            .filter(reporte::Column::IdReportero.eq(id_reportero))
            .one(&*self.db)
            .await
            .map_err(AppError::from_db)?;

        if existente.is_some() {
            return Err(AppError::Conflict("Ya has reportado este contenido".into()));
        }

        let nuevo = reporte::ActiveModel {
            id: Set(Uuid::new_v4()),
            tipo_contenido: Set(Some(datos.tipo_contenido)),
            id_contenido: Set(Some(datos.id_contenido)),
            id_reportero: Set(id_reportero),
            motivo: Set(Some(datos.motivo)),
            detalle_motivo: Set(datos.detalle_motivo),
            ..Default::default()
        };

        nuevo.insert(&*self.db).await.map_err(AppError::from_db)
    }

    async fn listar_reportes_pendientes(&self) -> Result<Vec<reporte::Model>, AppError> {
        reporte::Entity::find()
            .filter(reporte::Column::Estado.eq(Some(EstadoReporte::Pendiente)))
            .all(&*self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn obtener_reporte(&self, id: Uuid) -> Result<reporte::Model, AppError> {
        reporte::Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Reporte con id {id}")))
    }

    async fn eliminar_reporte(&self, id: Uuid) -> Result<(), AppError> {
        let result = reporte::Entity::delete_by_id(id)
            .exec(&*self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Reporte con id {id}")));
        }
        Ok(())
    }

    async fn procesar_reporte(
        &self,
        id: Uuid,
        id_procesador: Uuid,
        aceptar: bool,
        ocultar_contenido: bool,
    ) -> Result<reporte::Model, AppError> {
        let txn = self.db.begin().await.map_err(AppError::from_db)?;
        let reporte = reporte::Entity::find_by_id(id)
            .one(&txn)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Reporte con id {id}")))?;

        if aceptar && ocultar_contenido {
            ocultar_contenido_reportado(&txn, &reporte).await?;
        }

        let mut active: reporte::ActiveModel = reporte.into();
        active.estado = Set(Some(if aceptar {
            EstadoReporte::Aceptado
        } else {
            EstadoReporte::Rechazado
        }));
        active.id_procesador = Set(Some(id_procesador));
        active.procesado_en = Set(Some(Utc::now().fixed_offset()));

        let reporte_actualizado = active.update(&txn).await.map_err(AppError::from_db)?;
        txn.commit().await.map_err(AppError::from_db)?;

        Ok(reporte_actualizado)
    }
}

async fn ocultar_contenido_reportado<C>(db: &C, reporte: &reporte::Model) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    let tipo = reporte
        .tipo_contenido
        .as_ref()
        .ok_or_else(|| AppError::Internal("Reporte sin tipo_contenido".into()))?;
    let id_contenido = reporte
        .id_contenido
        .ok_or_else(|| AppError::Internal("Reporte sin id_contenido".into()))?;

    match tipo {
        TipoContenido::Oferta => {
            let contenido = oferta_empleo::Entity::find_by_id(id_contenido)
                .one(db)
                .await
                .map_err(AppError::from_db)?
                .ok_or_else(|| AppError::NotFound(format!("Oferta con id {id_contenido}")))?;
            let mut active: oferta_empleo::ActiveModel = contenido.into();
            active.activo = Set(Some(false));
            active.estado_moderacion = Set(Some(EstadoModeracion::Rechazado));
            active.update(db).await.map_err(AppError::from_db)?;
        }
        TipoContenido::Consejo => {
            let contenido = consejo::Entity::find_by_id(id_contenido)
                .one(db)
                .await
                .map_err(AppError::from_db)?
                .ok_or_else(|| AppError::NotFound(format!("Consejo con id {id_contenido}")))?;
            let mut active: consejo::ActiveModel = contenido.into();
            active.activo = Set(Some(false));
            active.estado_moderacion = Set(Some(EstadoModeracion::Rechazado));
            active.update(db).await.map_err(AppError::from_db)?;
        }
        TipoContenido::Curso => {
            let contenido = curso::Entity::find_by_id(id_contenido)
                .one(db)
                .await
                .map_err(AppError::from_db)?
                .ok_or_else(|| AppError::NotFound(format!("Curso con id {id_contenido}")))?;
            let mut active: curso::ActiveModel = contenido.into();
            active.activo = Set(Some(false));
            active.estado_moderacion = Set(Some(EstadoModeracion::Rechazado));
            active.update(db).await.map_err(AppError::from_db)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use sea_orm::{DatabaseBackend, MockDatabase};

    use super::*;

    fn reporte_fake(id: Uuid, id_contenido: Uuid, id_reportero: Uuid) -> reporte::Model {
        let now = Some(Utc::now().fixed_offset());
        reporte::Model {
            id,
            tipo_contenido: Some(TipoContenido::Oferta),
            id_contenido: Some(id_contenido),
            id_reportero,
            motivo: Some(MotivoReporte::Spam),
            detalle_motivo: None,
            estado: Some(EstadoReporte::Pendiente),
            id_procesador: None,
            procesado_en: None,
            creado_en: now,
            actualizado_en: now,
        }
    }

    fn oferta_fake(id: Uuid, id_autor: Uuid) -> oferta_empleo::Model {
        let now = Some(Utc::now().fixed_offset());
        oferta_empleo::Model {
            id,
            id_autor,
            titulo_puesto: Some("Oferta reportada".into()),
            empresa: None,
            ubicacion: None,
            descripcion: None,
            telefono_contacto: None,
            email_contacto: None,
            web_contacto: None,
            activo: Some(true),
            caduca_en: None,
            cantidad_upvotes: Some(0),
            cantidad_downvotes: Some(0),
            cantidad_reportes: Some(1),
            estado_moderacion: Some(EstadoModeracion::Aprobado),
            creado_en: now,
            actualizado_en: now,
        }
    }

    #[tokio::test]
    async fn procesar_reporte_aceptar_ocultar_actualiza_contenido_y_reporte() {
        let id_reporte = Uuid::new_v4();
        let id_contenido = Uuid::new_v4();
        let id_reportero = Uuid::new_v4();
        let id_procesador = Uuid::new_v4();
        let reporte = reporte_fake(id_reporte, id_contenido, id_reportero);
        let oferta = oferta_fake(id_contenido, id_reportero);
        let mut oferta_oculta = oferta.clone();
        oferta_oculta.activo = Some(false);
        oferta_oculta.estado_moderacion = Some(EstadoModeracion::Rechazado);
        let mut reporte_procesado = reporte.clone();
        reporte_procesado.estado = Some(EstadoReporte::Aceptado);
        reporte_procesado.id_procesador = Some(id_procesador);
        reporte_procesado.procesado_en = Some(Utc::now().fixed_offset());

        let db = Arc::new(
            MockDatabase::new(DatabaseBackend::Postgres)
                .append_query_results(vec![vec![reporte.clone()]])
                .append_query_results(vec![vec![oferta.clone()]])
                .append_query_results(vec![vec![oferta_oculta]])
                .append_query_results(vec![vec![reporte_procesado]])
                .into_connection(),
        );
        let repo = SeaReporteRepo::new(Arc::clone(&db));

        let resultado = repo
            .procesar_reporte(id_reporte, id_procesador, true, true)
            .await
            .expect("debe procesar y ocultar");
        assert_eq!(resultado.estado, Some(EstadoReporte::Aceptado));
        drop(repo);

        let logs = Arc::try_unwrap(db).ok().unwrap().into_transaction_log();
        let logs = format!("{logs:?}");
        assert!(logs.contains(r#"UPDATE \"ofertas_empleo\" SET \"activo\" = $1"#));
        assert!(logs.contains(r#"\"estado_moderacion\" = CAST($2 AS \"estado_moderacion\")"#));
        assert!(
            logs.contains(r#"UPDATE \"reportes\" SET \"estado\" = CAST($1 AS \"estado_reporte\")"#)
        );
        assert!(logs.contains(r#"\"id_procesador\" = $2, \"procesado_en\" = $3"#));
    }
}
