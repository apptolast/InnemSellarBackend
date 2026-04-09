// src/repositories/geografia_repo.rs
//
// Repositorio de datos geograficos con SeaORM.
//
// CAMBIO CLAVE vs version SQLx:
//   Antes: sqlx::query_as::<_, Struct>("SELECT * FROM tabla").fetch_all(&pool)
//   Ahora: tabla::Entity::find().all(&db)
//
// El SQL ya NO se escribe a mano. SeaORM genera las queries
// automaticamente a partir de las entidades.

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::errors::AppError;
use crate::models::{comunidad_autonoma, oficina_sepe, provincia};

// ─── DTOs de escritura ────────────────────────────────────────────────────────

/// Datos para crear una nueva comunidad autonoma.
///
/// # Por que un DTO separado y no el Model directamente
/// El `Model` de SeaORM incluye campos auto-gestionados por la base de datos
/// (`id` SERIAL, `creado_en`, `actualizado_en`) que NO deben venir del cliente.
/// El DTO solo expone los campos que el usuario puede controlar, garantizando
/// que jamas se sobreescriba la PK o los timestamps desde fuera.
///
/// Es el mismo patron que `fromJson()` en Dart: solo deserializas los campos
/// que el cliente puede enviar, ignorando los internos.
pub struct CrearComunidadDto {
    /// Nombre oficial de la comunidad. Ej: "Andalucia", "Cataluna".
    pub nombre: Option<String>,
    /// Nombre del servicio de empleo regional. Ej: "SAE", "SOC", "Lanbide".
    pub nombre_servicio_empleo: Option<String>,
    /// URL del portal web del servicio de empleo regional.
    pub web_servicio_empleo: Option<String>,
    /// URL de la pagina de sellado/renovacion de demanda de empleo.
    pub url_sellado: Option<String>,
}

/// Datos para actualizar una comunidad autonoma existente.
///
/// # Por que todos los campos son Option
/// La actualizacion es parcial (PATCH semantics aunque usemos PUT).
/// Solo se actualizan los campos que vienen con valor.
/// Si un campo es `None`, no se toca en base de datos.
/// En el repositorio lo gestionamos con `if datos.campo.is_some()`.
pub struct ActualizarComunidadDto {
    /// Nuevo nombre. `None` = no modificar.
    pub nombre: Option<String>,
    /// Nuevo nombre del servicio de empleo. `None` = no modificar.
    pub nombre_servicio_empleo: Option<String>,
    /// Nueva URL web del servicio. `None` = no modificar.
    pub web_servicio_empleo: Option<String>,
    /// Nueva URL de sellado. `None` = no modificar.
    pub url_sellado: Option<String>,
}

/// Datos para crear una nueva provincia.
///
/// # Por que `id: i32` es obligatorio aqui (y no en comunidad)
/// La tabla `provincias` usa `id INTEGER PRIMARY KEY` con codigo INE manual
/// (no SERIAL). El codigo INE es un estandar oficial espanol (01-52) y
/// NO debe ser auto-generado por la base de datos. Por eso `id` viene
/// en el DTO de creacion y se pasa explicitamente con `Set(datos.id)`.
///
/// En cambio, `comunidades_autonomas` usa `SERIAL` (auto-increment),
/// por lo que el campo `id` NO aparece en `CrearComunidadDto`.
pub struct CrearProvinciaDto {
    /// Codigo INE de la provincia (1-52). Es la PK manual.
    pub id: i32,
    /// Nombre de la provincia. Ej: "Sevilla", "Barcelona".
    pub nombre: Option<String>,
    /// ID de la comunidad autonoma a la que pertenece (FK).
    pub id_comunidad: i32,
    /// Ruta al asset de logo en la app Flutter. Ej: "assets/logos/sevilla.png".
    pub logo_asset: Option<String>,
}

/// Datos para actualizar una provincia existente.
pub struct ActualizarProvinciaDto {
    /// Nuevo nombre. `None` = no modificar.
    pub nombre: Option<String>,
    /// Nueva comunidad autonoma (reasignacion). `None` = no modificar.
    pub id_comunidad: Option<i32>,
    /// Nuevo logo asset. `None` = no modificar.
    pub logo_asset: Option<String>,
}

/// Datos para crear una oficina SEPE.
///
/// # Por que `id_provincia` esta aqui y no es la PK
/// La oficina tiene su propia PK SERIAL (`id`), pero `id_provincia` es
/// una FK unica — cada provincia tiene exactamente una oficina (1:1).
/// Viene en el DTO porque el cliente decide a que provincia asociar la oficina.
pub struct CrearOficinaDto {
    /// Provincia a la que pertenece esta oficina SEPE.
    pub id_provincia: i32,
    /// Telefono de atencion al ciudadano.
    pub telefono: Option<String>,
    /// URL del portal web de la oficina.
    pub web: Option<String>,
    /// URL del catalogo de cursos de formacion provincial.
    pub url_cursos: Option<String>,
    /// URL del servicio de orientacion laboral provincial.
    pub url_orientacion: Option<String>,
}

/// Datos para actualizar una oficina SEPE existente.
pub struct ActualizarOficinaDto {
    /// Nuevo telefono. `None` = no modificar.
    pub telefono: Option<String>,
    /// Nueva URL web. `None` = no modificar.
    pub web: Option<String>,
    /// Nueva URL de cursos. `None` = no modificar.
    pub url_cursos: Option<String>,
    /// Nueva URL de orientacion. `None` = no modificar.
    pub url_orientacion: Option<String>,
}

// ─── Trait (contrato / interfaz) ─────────────────────────────────────────────

/// Contrato (interfaz) para acceso a datos geograficos.
///
/// # Por que un trait en vez de una struct directamente
/// En Rust, los `trait` son equivalentes a las interfaces en Dart o Java.
/// Definen un contrato de comportamiento sin implementacion concreta.
///
/// Ventajas:
/// - Los handlers dependen de la abstraccion `GeografiaRepo`, no de `SeaGeografiaRepo`.
///   Esto permite reemplazar la implementacion en tests (mock) sin tocar los handlers.
/// - El compilador garantiza que `SeaGeografiaRepo` implementa todos los metodos.
pub trait GeografiaRepo: Send + Sync {
    fn listar_comunidades(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<comunidad_autonoma::Model>, AppError>> + Send;

    fn obtener_comunidad(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<comunidad_autonoma::Model, AppError>> + Send;

    fn crear_comunidad(
        &self,
        datos: CrearComunidadDto,
    ) -> impl std::future::Future<Output = Result<comunidad_autonoma::Model, AppError>> + Send;

    fn actualizar_comunidad(
        &self,
        id: i32,
        datos: ActualizarComunidadDto,
    ) -> impl std::future::Future<Output = Result<comunidad_autonoma::Model, AppError>> + Send;

    fn eliminar_comunidad(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn listar_provincias(
        &self,
        id_comunidad: Option<i32>,
    ) -> impl std::future::Future<Output = Result<Vec<provincia::Model>, AppError>> + Send;

    fn obtener_provincia(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<provincia::Model, AppError>> + Send;

    fn crear_provincia(
        &self,
        datos: CrearProvinciaDto,
    ) -> impl std::future::Future<Output = Result<provincia::Model, AppError>> + Send;

    fn actualizar_provincia(
        &self,
        id: i32,
        datos: ActualizarProvinciaDto,
    ) -> impl std::future::Future<Output = Result<provincia::Model, AppError>> + Send;

    fn eliminar_provincia(
        &self,
        id: i32,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    fn obtener_oficina_por_provincia(
        &self,
        id_provincia: i32,
    ) -> impl std::future::Future<Output = Result<oficina_sepe::Model, AppError>> + Send;

    fn crear_oficina(
        &self,
        datos: CrearOficinaDto,
    ) -> impl std::future::Future<Output = Result<oficina_sepe::Model, AppError>> + Send;

    fn actualizar_oficina(
        &self,
        id_provincia: i32,
        datos: ActualizarOficinaDto,
    ) -> impl std::future::Future<Output = Result<oficina_sepe::Model, AppError>> + Send;

    fn eliminar_oficina(
        &self,
        id_provincia: i32,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;
}

// ─── Implementacion con SeaORM ────────────────────────────────────────────────

/// Implementacion con SeaORM + PostgreSQL.
#[derive(Clone)]
pub struct SeaGeografiaRepo {
    db: DatabaseConnection,
}

impl SeaGeografiaRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

/// # Por que Entity::find() en vez de SQL crudo
/// SeaORM genera el SQL por ti. Cuando escribes:
///   `comunidad_autonoma::Entity::find().all(&self.db).await`
/// SeaORM genera internamente:
///   `SELECT * FROM comunidades_autonomas`
/// Y mapea el resultado a `comunidad_autonoma::Model`.
///
/// Para filtrar:
///   `.filter(provincia::Column::IdComunidad.eq(5))`
/// Genera:
///   `WHERE id_comunidad = 5`
///
/// Para ordenar:
///   `.order_by_asc(comunidad_autonoma::Column::Id)`
/// Genera:
///   `ORDER BY id ASC`
///
/// # Por que `From<sea_orm::DbErr>` en AppError
/// SeaORM usa `DbErr` como tipo de error. Necesitamos convertirlo
/// a nuestro `AppError`. Lo hacemos con un impl From mas abajo.
///
/// # Por que `ActiveModel` y `Set` para inserts/updates
/// SeaORM distingue entre `Model` (solo lectura, resultado de SELECT)
/// y `ActiveModel` (escritura, para INSERT/UPDATE).
/// `Set(valor)` marca un campo como "debe escribirse" en la query.
/// `..Default::default()` deja el resto en `NotSet` (campos ignorados).
/// Esto equivale a un INSERT parcial o UPDATE parcial en SQL.
impl GeografiaRepo for SeaGeografiaRepo {
    async fn listar_comunidades(&self) -> Result<Vec<comunidad_autonoma::Model>, AppError> {
        let comunidades = comunidad_autonoma::Entity::find()
            .order_by_asc(comunidad_autonoma::Column::Id)
            .all(&self.db)
            .await
            .map_err(AppError::from_db)?;

        Ok(comunidades)
    }

    async fn obtener_comunidad(&self, id: i32) -> Result<comunidad_autonoma::Model, AppError> {
        comunidad_autonoma::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Comunidad autonoma con id {id}")))
    }

    async fn crear_comunidad(
        &self,
        datos: CrearComunidadDto,
    ) -> Result<comunidad_autonoma::Model, AppError> {
        // `..Default::default()` deja `id` en NotSet porque es SERIAL:
        // PostgreSQL lo asigna automaticamente. Nunca pasamos el id en
        // un INSERT a tablas con primary key auto-increment.
        let nueva = comunidad_autonoma::ActiveModel {
            nombre: Set(datos.nombre),
            nombre_servicio_empleo: Set(datos.nombre_servicio_empleo),
            web_servicio_empleo: Set(datos.web_servicio_empleo),
            url_sellado: Set(datos.url_sellado),
            ..Default::default()
        };
        nueva.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn actualizar_comunidad(
        &self,
        id: i32,
        datos: ActualizarComunidadDto,
    ) -> Result<comunidad_autonoma::Model, AppError> {
        // Primero buscamos el registro existente. Si no existe, devolvemos NotFound.
        // Luego convertimos el Model en ActiveModel con `.into()` para poder mutar
        // solo los campos que queremos actualizar.
        let comunidad = comunidad_autonoma::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Comunidad autonoma con id {id}")))?;

        let mut active: comunidad_autonoma::ActiveModel = comunidad.into();

        if datos.nombre.is_some() {
            active.nombre = Set(datos.nombre);
        }
        if datos.nombre_servicio_empleo.is_some() {
            active.nombre_servicio_empleo = Set(datos.nombre_servicio_empleo);
        }
        if datos.web_servicio_empleo.is_some() {
            active.web_servicio_empleo = Set(datos.web_servicio_empleo);
        }
        if datos.url_sellado.is_some() {
            active.url_sellado = Set(datos.url_sellado);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }

    async fn eliminar_comunidad(&self, id: i32) -> Result<(), AppError> {
        // `delete_by_id` genera: DELETE FROM comunidades_autonomas WHERE id = $1
        // Comprobamos `rows_affected` para detectar si el registro existia o no.
        let result = comunidad_autonoma::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Comunidad autonoma con id {id}"
            )));
        }
        Ok(())
    }

    async fn listar_provincias(
        &self,
        id_comunidad: Option<i32>,
    ) -> Result<Vec<provincia::Model>, AppError> {
        let mut query = provincia::Entity::find().order_by_asc(provincia::Column::Id);

        // Si viene filtro, anadimos WHERE. Si no, devolvemos todas.
        if let Some(id_com) = id_comunidad {
            query = query.filter(provincia::Column::IdComunidad.eq(id_com));
        }

        let provincias = query.all(&self.db).await.map_err(AppError::from_db)?;

        Ok(provincias)
    }

    async fn obtener_provincia(&self, id: i32) -> Result<provincia::Model, AppError> {
        provincia::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Provincia con id {id}")))
    }

    async fn crear_provincia(
        &self,
        datos: CrearProvinciaDto,
    ) -> Result<provincia::Model, AppError> {
        // A diferencia de comunidad, aqui SI pasamos el id con Set() porque
        // la tabla usa `auto_increment = false` — el codigo INE (1-52) es manual.
        let nueva = provincia::ActiveModel {
            id: Set(datos.id),
            nombre: Set(datos.nombre),
            id_comunidad: Set(datos.id_comunidad),
            logo_asset: Set(datos.logo_asset),
            ..Default::default()
        };
        nueva.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn actualizar_provincia(
        &self,
        id: i32,
        datos: ActualizarProvinciaDto,
    ) -> Result<provincia::Model, AppError> {
        let provincia = provincia::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| AppError::NotFound(format!("Provincia con id {id}")))?;

        let mut active: provincia::ActiveModel = provincia.into();

        if datos.nombre.is_some() {
            active.nombre = Set(datos.nombre);
        }
        if let Some(id_com) = datos.id_comunidad {
            active.id_comunidad = Set(id_com);
        }
        if datos.logo_asset.is_some() {
            active.logo_asset = Set(datos.logo_asset);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }

    async fn eliminar_provincia(&self, id: i32) -> Result<(), AppError> {
        let result = provincia::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Provincia con id {id}")));
        }
        Ok(())
    }

    async fn obtener_oficina_por_provincia(
        &self,
        id_provincia: i32,
    ) -> Result<oficina_sepe::Model, AppError> {
        oficina_sepe::Entity::find()
            .filter(oficina_sepe::Column::IdProvincia.eq(id_provincia))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Oficina SEPE para provincia con id {id_provincia}"))
            })
    }

    async fn crear_oficina(&self, datos: CrearOficinaDto) -> Result<oficina_sepe::Model, AppError> {
        // Verificar que no exista ya una oficina para esta provincia
        // (la columna id_provincia tiene UNIQUE constraint en el schema).
        let existente = oficina_sepe::Entity::find()
            .filter(oficina_sepe::Column::IdProvincia.eq(datos.id_provincia))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if existente.is_some() {
            return Err(AppError::Conflict(format!(
                "Ya existe una oficina SEPE para la provincia con id {}",
                datos.id_provincia
            )));
        }

        // `id` es SERIAL: lo dejamos en NotSet con `..Default::default()`.
        let nueva = oficina_sepe::ActiveModel {
            id_provincia: Set(datos.id_provincia),
            telefono: Set(datos.telefono),
            web: Set(datos.web),
            url_cursos: Set(datos.url_cursos),
            url_orientacion: Set(datos.url_orientacion),
            ..Default::default()
        };
        nueva.insert(&self.db).await.map_err(AppError::from_db)
    }

    async fn actualizar_oficina(
        &self,
        id_provincia: i32,
        datos: ActualizarOficinaDto,
    ) -> Result<oficina_sepe::Model, AppError> {
        // Las oficinas se buscan por id_provincia (clave natural), no por su PK interna.
        let oficina = oficina_sepe::Entity::find()
            .filter(oficina_sepe::Column::IdProvincia.eq(id_provincia))
            .one(&self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Oficina SEPE para provincia con id {id_provincia}"))
            })?;

        let mut active: oficina_sepe::ActiveModel = oficina.into();

        if datos.telefono.is_some() {
            active.telefono = Set(datos.telefono);
        }
        if datos.web.is_some() {
            active.web = Set(datos.web);
        }
        if datos.url_cursos.is_some() {
            active.url_cursos = Set(datos.url_cursos);
        }
        if datos.url_orientacion.is_some() {
            active.url_orientacion = Set(datos.url_orientacion);
        }

        active.update(&self.db).await.map_err(AppError::from_db)
    }

    async fn eliminar_oficina(&self, id_provincia: i32) -> Result<(), AppError> {
        // No tenemos `delete_by_id` directo con id_provincia (no es PK),
        // por lo que usamos `delete_many` con filtro.
        let result = oficina_sepe::Entity::delete_many()
            .filter(oficina_sepe::Column::IdProvincia.eq(id_provincia))
            .exec(&self.db)
            .await
            .map_err(AppError::from_db)?;

        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!(
                "Oficina SEPE para provincia con id {id_provincia}"
            )));
        }
        Ok(())
    }
}
