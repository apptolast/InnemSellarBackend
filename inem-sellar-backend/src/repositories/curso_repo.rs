use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{curso, curso_provincia};

pub struct CrearCursoDto {
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub contenido: Option<String>,
    pub web: Option<String>,
    pub imagen_url: Option<String>,
    pub duracion_horas: Option<i32>,
    pub fecha_inicio: Option<sea_orm::prelude::Date>,
    pub fecha_fin: Option<sea_orm::prelude::Date>,
    pub curso_homologado: Option<bool>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    pub provincias: Vec<i32>,
}

pub struct ActualizarCursoDto {
    pub titulo: Option<String>,
    pub descripcion: Option<String>,
    pub contenido: Option<String>,
    pub web: Option<String>,
    pub imagen_url: Option<String>,
    pub duracion_horas: Option<i32>,
    pub fecha_inicio: Option<sea_orm::prelude::Date>,
    pub fecha_fin: Option<sea_orm::prelude::Date>,
    pub curso_homologado: Option<bool>,
    pub telefono_contacto: Option<String>,
    pub email_contacto: Option<String>,
    /// Si se envia, reemplaza las provincias asociadas.
    pub provincias: Option<Vec<i32>>,
}

pub trait CursoRepo: Send + Sync {
    fn listar_cursos(
        &self,
        id_provincia: Option<i32>,
        pagina: u64,
        por_pagina: u64,
    ) -> impl std::future::Future<Output = Result<(Vec<curso::Model>, u64), AppError>> + Send;

    fn obtener_curso(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<curso::Model, AppError>> + Send;

    fn crear_curso(
        &self,
        id_autor: Uuid,
        datos: CrearCursoDto,
    ) -> impl std::future::Future<Output = Result<curso::Model, AppError>> + Send;

    fn actualizar_curso(
        &self,
        id: Uuid,
        datos: ActualizarCursoDto,
    ) -> impl std::future::Future<Output = Result<curso::Model, AppError>> + Send;

    fn eliminar_curso(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

#[derive(Clone)]
pub struct SeaCursoRepo {
    db: DatabaseConnection,
}

impl SeaCursoRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl CursoRepo for SeaCursoRepo {
    async fn listar_cursos(
        &self,
        id_provincia: Option<i32>,
        pagina: u64,
        por_pagina: u64,
    ) -> Result<(Vec<curso::Model>, u64), AppError> {
        let mut query = curso::Entity::find()
            .filter(curso::Column::Activo.eq(Some(true)))
            .order_by_desc(curso::Column::CreadoEn);

        if let Some(id_prov) = id_provincia {
            let ids: Vec<Uuid> = curso_provincia::Entity::find()
                .filter(curso_provincia::Column::IdProvincia.eq(id_prov))
                .all(&self.db)
                .await
                .map_err(AppError::from_db)?
                .into_iter()
                .map(|cp| cp.id_curso)
                .collect();
            query = query.filter(curso::Column::Id.is_in(ids));
        }

        let paginator = query.paginate(&self.db, por_pagina);
        let total = paginator.num_items().await.map_err(AppError::from_db)?;
        let cursos = paginator
            .fetch_page(pagina.saturating_sub(1))
            .await
            .map_err(AppError::from_db)?;

        Ok((cursos, total))
    }

    async fn obtener_curso(&self, id: Uuid) -> Result<curso::Model, AppError> {
        curso::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Curso con id {id}")))
    }

    async fn crear_curso(
        &self,
        id_autor: Uuid,
        datos: CrearCursoDto,
    ) -> Result<curso::Model, AppError> {
        let id_curso = Uuid::new_v4();

        let nuevo = curso::ActiveModel {
            id: Set(id_curso),
            id_autor: Set(Some(id_autor)),
            titulo: Set(datos.titulo),
            descripcion: Set(datos.descripcion),
            contenido: Set(datos.contenido),
            web: Set(datos.web),
            imagen_url: Set(datos.imagen_url),
            duracion_horas: Set(datos.duracion_horas),
            fecha_inicio: Set(datos.fecha_inicio),
            fecha_fin: Set(datos.fecha_fin),
            curso_homologado: Set(datos.curso_homologado),
            telefono_contacto: Set(datos.telefono_contacto),
            email_contacto: Set(datos.email_contacto),
            activo: Set(Some(true)),
            ..Default::default()
        };

        let curso = nuevo.insert(&self.db).await.map_err(AppError::from_db)?;

        for id_prov in datos.provincias {
            let vinculo = curso_provincia::ActiveModel {
                id_curso: Set(id_curso),
                id_provincia: Set(id_prov),
            };
            vinculo.insert(&self.db).await.map_err(AppError::from_db)?;
        }

        Ok(curso)
    }

    async fn actualizar_curso(
        &self,
        id: Uuid,
        datos: ActualizarCursoDto,
    ) -> Result<curso::Model, AppError> {
        let curso = curso::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Curso con id {id}")))?;

        let mut active: curso::ActiveModel = curso.into();

        if datos.titulo.is_some() {
            active.titulo = Set(datos.titulo);
        }
        if datos.descripcion.is_some() {
            active.descripcion = Set(datos.descripcion);
        }
        if datos.contenido.is_some() {
            active.contenido = Set(datos.contenido);
        }
        if datos.web.is_some() {
            active.web = Set(datos.web);
        }
        if datos.imagen_url.is_some() {
            active.imagen_url = Set(datos.imagen_url);
        }
        if datos.duracion_horas.is_some() {
            active.duracion_horas = Set(datos.duracion_horas);
        }
        if datos.fecha_inicio.is_some() {
            active.fecha_inicio = Set(datos.fecha_inicio);
        }
        if datos.fecha_fin.is_some() {
            active.fecha_fin = Set(datos.fecha_fin);
        }
        if datos.curso_homologado.is_some() {
            active.curso_homologado = Set(datos.curso_homologado);
        }
        if datos.telefono_contacto.is_some() {
            active.telefono_contacto = Set(datos.telefono_contacto);
        }
        if datos.email_contacto.is_some() {
            active.email_contacto = Set(datos.email_contacto);
        }

        let resultado = active.update(&self.db).await.map_err(AppError::from_db)?;

        // Actualizar provincias si se enviaron
        if let Some(provincias) = datos.provincias {
            curso_provincia::Entity::delete_many()
                .filter(curso_provincia::Column::IdCurso.eq(id))
                .exec(&self.db)
                .await
                .map_err(AppError::from_db)?;

            for id_prov in provincias {
                let vinculo = curso_provincia::ActiveModel {
                    id_curso: Set(id),
                    id_provincia: Set(id_prov),
                };
                vinculo.insert(&self.db).await.map_err(AppError::from_db)?;
            }
        }

        Ok(resultado)
    }

    async fn eliminar_curso(&self, id: Uuid) -> Result<(), AppError> {
        let result = curso::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Curso con id {id}")));
        }
        Ok(())
    }
}
