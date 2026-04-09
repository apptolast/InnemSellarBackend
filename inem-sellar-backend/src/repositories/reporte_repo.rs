use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::enums::{EstadoReporte, MotivoReporte, TipoContenido};
use crate::models::reporte;

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

    /// Procesar un reporte: aceptar o rechazar.
    /// Registra quien lo proceso y cuando.
    fn procesar_reporte(
        &self,
        id: Uuid,
        id_procesador: Uuid,
        aceptar: bool,
    ) -> impl std::future::Future<Output = Result<reporte::Model, AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaReporteRepo {
    db: DatabaseConnection,
}

impl SeaReporteRepo {
    pub fn new(db: DatabaseConnection) -> Self {
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
            .one(&self.db)
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

        nuevo.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn listar_reportes_pendientes(&self) -> Result<Vec<reporte::Model>, AppError> {
        reporte::Entity::find()
            .filter(reporte::Column::Estado.eq(Some(EstadoReporte::Pendiente)))
            .all(&self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn procesar_reporte(
        &self,
        id: Uuid,
        id_procesador: Uuid,
        aceptar: bool,
    ) -> Result<reporte::Model, AppError> {
        let reporte = reporte::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Reporte con id {id}")))?;

        let mut active: reporte::ActiveModel = reporte.into();
        active.estado = Set(Some(if aceptar {
            EstadoReporte::Aceptado
        } else {
            EstadoReporte::Rechazado
        }));
        active.id_procesador = Set(Some(id_procesador));
        active.procesado_en = Set(Some(Utc::now().fixed_offset()));

        active.update(&self.db).await.map_err(AppError::from_db)
    }
}
