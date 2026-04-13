use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{consejo, consejo_provincia};

pub struct CrearConsejoDto {
    pub titulo: Option<String>,
    pub cuerpo: Option<String>,
    pub web: Option<String>,
    pub imagen_url: Option<String>,
    pub provincias: Vec<i32>,
}

pub struct ActualizarConsejoDto {
    pub titulo: Option<String>,
    pub cuerpo: Option<String>,
    pub web: Option<String>,
    pub imagen_url: Option<String>,
    /// Si se envia, reemplaza las provincias asociadas.
    pub provincias: Option<Vec<i32>>,
}

pub trait ConsejoRepo: Send + Sync {
    fn listar_consejos(
        &self,
        id_provincia: Option<i32>,
        pagina: u64,
        por_pagina: u64,
    ) -> impl std::future::Future<Output = Result<(Vec<consejo::Model>, u64), AppError>> + Send;

    fn obtener_consejo(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<consejo::Model, AppError>> + Send;

    fn crear_consejo(
        &self,
        id_autor: Uuid,
        datos: CrearConsejoDto,
    ) -> impl std::future::Future<Output = Result<consejo::Model, AppError>> + Send;

    fn actualizar_consejo(
        &self,
        id: Uuid,
        datos: ActualizarConsejoDto,
    ) -> impl std::future::Future<Output = Result<consejo::Model, AppError>> + Send;

    fn eliminar_consejo(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaConsejoRepo {
    db: DatabaseConnection,
}

impl SeaConsejoRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ConsejoRepo for SeaConsejoRepo {
    async fn listar_consejos(
        &self,
        id_provincia: Option<i32>,
        pagina: u64,
        por_pagina: u64,
    ) -> Result<(Vec<consejo::Model>, u64), AppError> {
        let mut query = consejo::Entity::find()
            .filter(consejo::Column::Activo.eq(Some(true)))
            .order_by_desc(consejo::Column::CreadoEn);

        if let Some(id_prov) = id_provincia {
            let ids: Vec<Uuid> = consejo_provincia::Entity::find()
                .filter(consejo_provincia::Column::IdProvincia.eq(id_prov))
                .all(&self.db)
                .await
                .map_err(AppError::from_db)?
                .into_iter()
                .map(|cp| cp.id_consejo)
                .collect();
            query = query.filter(consejo::Column::Id.is_in(ids));
        }

        let paginator = query.paginate(&self.db, por_pagina);
        let total = paginator.num_items().await.map_err(AppError::from_db)?;
        let consejos = paginator
            .fetch_page(pagina.saturating_sub(1))
            .await
            .map_err(AppError::from_db)?;

        Ok((consejos, total))
    }

    async fn obtener_consejo(&self, id: Uuid) -> Result<consejo::Model, AppError> {
        consejo::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Consejo con id {id}")))
    }

    async fn crear_consejo(
        &self,
        id_autor: Uuid,
        datos: CrearConsejoDto,
    ) -> Result<consejo::Model, AppError> {
        let id_consejo = Uuid::new_v4();

        let nuevo = consejo::ActiveModel {
            id: Set(id_consejo),
            id_autor: Set(Some(id_autor)),
            titulo: Set(datos.titulo),
            cuerpo: Set(datos.cuerpo),
            web: Set(datos.web),
            imagen_url: Set(datos.imagen_url),
            activo: Set(Some(true)),
            ..Default::default()
        };

        let consejo = nuevo.insert(&self.db).await.map_err(AppError::from_db)?;

        for id_prov in datos.provincias {
            let vinculo = consejo_provincia::ActiveModel {
                id_consejo: Set(id_consejo),
                id_provincia: Set(id_prov),
            };
            vinculo.insert(&self.db).await.map_err(AppError::from_db)?;
        }

        Ok(consejo)
    }

    async fn actualizar_consejo(
        &self,
        id: Uuid,
        datos: ActualizarConsejoDto,
    ) -> Result<consejo::Model, AppError> {
        let consejo = consejo::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Consejo con id {id}")))?;

        let mut active: consejo::ActiveModel = consejo.into();

        if datos.titulo.is_some() {
            active.titulo = Set(datos.titulo);
        }
        if datos.cuerpo.is_some() {
            active.cuerpo = Set(datos.cuerpo);
        }
        if datos.web.is_some() {
            active.web = Set(datos.web);
        }
        if datos.imagen_url.is_some() {
            active.imagen_url = Set(datos.imagen_url);
        }

        let resultado = active.update(&self.db).await.map_err(AppError::from_db)?;

        // Actualizar provincias si se enviaron
        if let Some(provincias) = datos.provincias {
            consejo_provincia::Entity::delete_many()
                .filter(consejo_provincia::Column::IdConsejo.eq(id))
                .exec(&self.db)
                .await
                .map_err(AppError::from_db)?;

            for id_prov in provincias {
                let vinculo = consejo_provincia::ActiveModel {
                    id_consejo: Set(id),
                    id_provincia: Set(id_prov),
                };
                vinculo.insert(&self.db).await.map_err(AppError::from_db)?;
            }
        }

        Ok(resultado)
    }

    async fn eliminar_consejo(&self, id: Uuid) -> Result<(), AppError> {
        let result = consejo::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Consejo con id {id}")));
        }
        Ok(())
    }
}
