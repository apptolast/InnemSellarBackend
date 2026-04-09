// src/repositories/oferta_repo.rs
//
// Repositorio de ofertas de empleo — CRUD completo con paginacion y filtros.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{oferta_empleo, oferta_provincia};

/// DTO para crear una oferta (datos que vienen del handler, no del HTTP).
#[derive(Clone)]
pub struct CrearOfertaDto {
    pub titulo_puesto: Option<String>,
    pub empresa: Option<String>,
    pub ubicacion: Option<String>,
    pub descripcion: Option<String>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub web_contacto: Option<String>,
    pub provincias: Vec<i32>,
}

/// DTO para actualizar una oferta (todos los campos opcionales).
#[derive(Clone)]
pub struct ActualizarOfertaDto {
    pub titulo_puesto: Option<String>,
    pub empresa: Option<String>,
    pub ubicacion: Option<String>,
    pub descripcion: Option<String>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub web_contacto: Option<String>,
}

/// Contrato de acceso a datos de ofertas.
pub trait OfertaRepo: Send + Sync {
    fn listar_ofertas(
        &self,
        id_provincia: Option<i32>,
        pagina: u64,
        por_pagina: u64,
    ) -> impl std::future::Future<Output = Result<(Vec<oferta_empleo::Model>, u64), AppError>> + Send;

    fn obtener_oferta(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<oferta_empleo::Model, AppError>> + Send;

    fn crear_oferta(
        &self,
        id_autor: Uuid,
        datos: CrearOfertaDto,
    ) -> impl std::future::Future<Output = Result<oferta_empleo::Model, AppError>> + Send;

    fn actualizar_oferta(
        &self,
        id: Uuid,
        datos: ActualizarOfertaDto,
    ) -> impl std::future::Future<Output = Result<oferta_empleo::Model, AppError>> + Send;

    fn eliminar_oferta(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaOfertaRepo {
    db: DatabaseConnection,
}

impl SeaOfertaRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl OfertaRepo for SeaOfertaRepo {
    async fn listar_ofertas(
        &self,
        id_provincia: Option<i32>,
        pagina: u64,
        por_pagina: u64,
    ) -> Result<(Vec<oferta_empleo::Model>, u64), AppError> {
        let mut query = oferta_empleo::Entity::find()
            .filter(oferta_empleo::Column::Activo.eq(Some(true)))
            .order_by_desc(oferta_empleo::Column::CreadoEn);

        // Filtro por provincia: buscamos ofertas vinculadas a esa provincia
        if let Some(id_prov) = id_provincia {
            // Subquery: IDs de ofertas que estan en esa provincia
            let ids_ofertas: Vec<Uuid> = oferta_provincia::Entity::find()
                .filter(oferta_provincia::Column::IdProvincia.eq(id_prov))
                .all(&self.db)
                .await
                .map_err(AppError::from_db)?
                .into_iter()
                .map(|op| op.id_oferta)
                .collect();

            query = query.filter(oferta_empleo::Column::Id.is_in(ids_ofertas));
        }

        // Paginacion con SeaORM
        let paginator = query.paginate(&self.db, por_pagina);
        let total = paginator.num_items().await.map_err(AppError::from_db)?;
        let ofertas = paginator
            .fetch_page(pagina.saturating_sub(1)) // pagina 1-based → 0-based
            .await
            .map_err(AppError::from_db)?;

        Ok((ofertas, total))
    }

    async fn obtener_oferta(&self, id: Uuid) -> Result<oferta_empleo::Model, AppError> {
        oferta_empleo::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Oferta con id {id}")))
    }

    async fn crear_oferta(
        &self,
        id_autor: Uuid,
        datos: CrearOfertaDto,
    ) -> Result<oferta_empleo::Model, AppError> {
        let id_oferta = Uuid::new_v4();

        let nueva = oferta_empleo::ActiveModel {
            id: Set(id_oferta),
            id_autor: Set(id_autor),
            titulo_puesto: Set(datos.titulo_puesto),
            empresa: Set(datos.empresa),
            ubicacion: Set(datos.ubicacion),
            descripcion: Set(datos.descripcion),
            telefono_contacto: Set(datos.telefono_contacto),
            email_contacto: Set(datos.email_contacto),
            web_contacto: Set(datos.web_contacto),
            activo: Set(Some(true)),
            ..Default::default()
        };

        let oferta = nueva.insert(&self.db).await.map_err(AppError::from_db)?;

        // Vincular provincias (N:M)
        for id_prov in datos.provincias {
            let vinculo = oferta_provincia::ActiveModel {
                id_oferta: Set(id_oferta),
                id_provincia: Set(id_prov),
            };
            vinculo.insert(&self.db).await.map_err(AppError::from_db)?;
        }

        Ok(oferta)
    }

    async fn actualizar_oferta(
        &self,
        id: Uuid,
        datos: ActualizarOfertaDto,
    ) -> Result<oferta_empleo::Model, AppError> {
        let oferta = oferta_empleo::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Oferta con id {id}")))?;

        let mut active: oferta_empleo::ActiveModel = oferta.into();

        // Solo actualizamos los campos que vienen con valor
        if datos.titulo_puesto.is_some() {
            active.titulo_puesto = Set(datos.titulo_puesto);
        }
        if datos.empresa.is_some() {
            active.empresa = Set(datos.empresa);
        }
        if datos.ubicacion.is_some() {
            active.ubicacion = Set(datos.ubicacion);
        }
        if datos.descripcion.is_some() {
            active.descripcion = Set(datos.descripcion);
        }
        if datos.telefono_contacto.is_some() {
            active.telefono_contacto = Set(datos.telefono_contacto);
        }
        if datos.email_contacto.is_some() {
            active.email_contacto = Set(datos.email_contacto);
        }
        if datos.web_contacto.is_some() {
            active.web_contacto = Set(datos.web_contacto);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }

    async fn eliminar_oferta(&self, id: Uuid) -> Result<(), AppError> {
        let result = oferta_empleo::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Oferta con id {id}")));
        }

        Ok(())
    }
}
