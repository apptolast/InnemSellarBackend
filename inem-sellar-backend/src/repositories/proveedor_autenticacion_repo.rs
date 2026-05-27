// src/repositories/proveedor_autenticacion_repo.rs
//
// Repositorio de proveedores de autenticacion externos (OAuth + anonimo).
//
// Cada fila representa una IDENTIDAD de un usuario en un proveedor concreto.
// Un mismo `usuarios.id` puede tener varias filas (account linking):
//   - una con `proveedor='firebase'`, `identificador_proveedor=<firebase_uid>`
//   - otra con `proveedor='anonymous'`, `identificador_proveedor=NULL`
//   - en el futuro, una con `proveedor='apple'`, `identificador_proveedor=<sub>`
//
// El indice unico parcial `idx_proveedores_autenticacion_unico` impone
// `(proveedor, identificador_proveedor)` UNIQUE solo cuando ambos estan
// presentes — los anonimos (con identificador NULL) no entran en la
// restriccion, asi que pueden coexistir varios.

use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::proveedor_autenticacion;

/// Contrato de acceso a la tabla `proveedores_autenticacion`.
///
/// # Por que un trait y no llamadas directas a SeaORM
/// Mantiene consistencia con el resto del proyecto: cada tabla se accede
/// a traves de su trait. Esto permite:
///   - Inyectar mocks en tests (impl alternativa del trait).
///   - Cambiar el ORM en el futuro sin tocar handlers.
///   - Ver de un vistazo todas las operaciones sobre la tabla.
pub trait ProveedorAutenticacionRepo: Send + Sync {
    /// Busca una identidad por `(proveedor, identificador_proveedor)`.
    /// Devuelve `None` si no existe — significa que es un usuario nuevo.
    fn buscar_por_proveedor_e_identificador(
        &self,
        proveedor: &str,
        identificador: &str,
    ) -> impl std::future::Future<Output = Result<Option<proveedor_autenticacion::Model>, AppError>> + Send;

    /// Busca una identidad SOLO por `firebase_uid`, ignorando el provider.
    ///
    /// # Por que existe este metodo
    /// Si el cliente usa `FirebaseAuth.linkWithCredential(...)` para subir
    /// un usuario anonimo a Google/email-pwd, Firebase conserva el mismo
    /// `firebase_uid` pero cambia el `sign_in_provider` del siguiente token.
    /// Sin este lookup defensivo, el handler crearia un usuario duplicado.
    /// Con el, detectamos que el `sub` ya existe en otro provider y reusamos
    /// el `id_usuario`, anadiendo solo la nueva fila de identidad.
    ///
    /// Devuelve la primera fila encontrada (PostgreSQL no garantiza orden,
    /// pero todas las filas con el mismo `firebase_uid` apuntan al mismo
    /// `id_usuario` por construccion, asi que cualquiera vale).
    fn buscar_por_firebase_uid_cualquier_provider(
        &self,
        firebase_uid: &str,
    ) -> impl std::future::Future<Output = Result<Option<proveedor_autenticacion::Model>, AppError>> + Send;

    /// Crea una nueva identidad enlazada a un usuario existente.
    ///
    /// `identificador` es `Option<&str>` porque los anonimos no tienen
    /// `firebase_uid` (no hay aun token de Firebase asociado al user backend).
    fn crear(
        &self,
        id_usuario: Uuid,
        proveedor: &str,
        identificador: Option<&str>,
        email: Option<&str>,
        datos: Option<serde_json::Value>,
    ) -> impl std::future::Future<Output = Result<proveedor_autenticacion::Model, AppError>> + Send;

    /// Actualiza el JSONB `datos_proveedor` y opcionalmente `email_proveedor`.
    ///
    /// Se llama en cada login posterior con Firebase para refrescar la foto
    /// del perfil, el flag `email_verified`, los `identities`, etc., con la
    /// version mas reciente que envia Google.
    fn actualizar_datos(
        &self,
        id: Uuid,
        email: Option<&str>,
        datos: Option<serde_json::Value>,
    ) -> impl std::future::Future<Output = Result<(), AppError>> + Send;

    /// `true` si el usuario tiene una fila en `proveedores_autenticacion`
    /// con `proveedor='anonymous'`.
    ///
    /// # Por que existe este metodo
    /// Lo necesita el handler `refrescar` para preservar el flag `anonimo`
    /// del JWT al rotar tokens. Si el usuario era anonimo cuando obtuvo el
    /// refresh_token, el nuevo access_token tambien debe tener `anonimo=true`.
    fn es_anonimo(
        &self,
        id_usuario: Uuid,
    ) -> impl std::future::Future<Output = Result<bool, AppError>> + Send;

    /// `true` si el usuario tiene alguna identidad Firebase con ese email
    /// y `datos_proveedor.email_verified=true`.
    ///
    /// Disponible para auditorias o futuras politicas que necesiten exigir
    /// email verificado. La elevacion admin actual depende solo de
    /// `ADMIN_EMAIL_ALLOWLIST`.
    fn tiene_email_verificado(
        &self,
        id_usuario: Uuid,
        email: &str,
    ) -> impl std::future::Future<Output = Result<bool, AppError>> + Send;
}

/// Implementacion con SeaORM.
#[derive(Clone)]
pub struct SeaProveedorAutenticacionRepo {
    db: Arc<DatabaseConnection>,
}

impl SeaProveedorAutenticacionRepo {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

impl ProveedorAutenticacionRepo for SeaProveedorAutenticacionRepo {
    async fn buscar_por_proveedor_e_identificador(
        &self,
        proveedor: &str,
        identificador: &str,
    ) -> Result<Option<proveedor_autenticacion::Model>, AppError> {
        // Filtramos por los DOS campos a la vez. El indice unico parcial de
        // PostgreSQL convierte esta consulta en un index lookup de O(log n).
        proveedor_autenticacion::Entity::find()
            .filter(proveedor_autenticacion::Column::Proveedor.eq(Some(proveedor.to_string())))
            .filter(
                proveedor_autenticacion::Column::IdentificadorProveedor
                    .eq(Some(identificador.to_string())),
            )
            .one(&*self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn buscar_por_firebase_uid_cualquier_provider(
        &self,
        firebase_uid: &str,
    ) -> Result<Option<proveedor_autenticacion::Model>, AppError> {
        // Sin filtro por proveedor: solo el `identificador_proveedor`. La
        // consulta no usa el indice unico parcial (que necesita ambos campos)
        // y por tanto es un seq scan en el peor caso, pero el camino caliente
        // (login normal) NO entra aqui — solo lo dispara el caso `linkWithCredential`,
        // que es raro. El volumen de filas en `proveedores_autenticacion` es
        // bajo (1-3 por usuario), asi que el coste es aceptable.
        proveedor_autenticacion::Entity::find()
            .filter(
                proveedor_autenticacion::Column::IdentificadorProveedor
                    .eq(Some(firebase_uid.to_string())),
            )
            .one(&*self.db)
            .await
            .map_err(AppError::from_db)
    }

    async fn crear(
        &self,
        id_usuario: Uuid,
        proveedor: &str,
        identificador: Option<&str>,
        email: Option<&str>,
        datos: Option<serde_json::Value>,
    ) -> Result<proveedor_autenticacion::Model, AppError> {
        // `..Default::default()` deja `creado_en` y `actualizado_en` como
        // `NotSet`, dejando que PostgreSQL aplique el `DEFAULT now()` definido
        // en el schema. Mejor que setearlos en el cliente — evita drift de reloj.
        let nuevo = proveedor_autenticacion::ActiveModel {
            id: Set(Uuid::new_v4()),
            id_usuario: Set(id_usuario),
            proveedor: Set(Some(proveedor.to_string())),
            identificador_proveedor: Set(identificador.map(String::from)),
            email_proveedor: Set(email.map(String::from)),
            datos_proveedor: Set(datos),
            ..Default::default()
        };

        nuevo.insert(&*self.db).await.map_err(AppError::from_db)
    }

    async fn actualizar_datos(
        &self,
        id: Uuid,
        email: Option<&str>,
        datos: Option<serde_json::Value>,
    ) -> Result<(), AppError> {
        // Cargamos la fila para convertirla en ActiveModel y actualizarla.
        // SeaORM exige este patron (load → mutate → save) cuando hay triggers
        // que dependen de OLD/NEW (en este caso, `set_actualizado_en`).
        let fila = proveedor_autenticacion::Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .map_err(AppError::from_db)?
            .ok_or_else(|| {
                AppError::NotFound(format!("Proveedor autenticacion {id} no encontrado"))
            })?;

        let mut active: proveedor_autenticacion::ActiveModel = fila.into();
        if let Some(em) = email {
            active.email_proveedor = Set(Some(em.to_string()));
        }
        if let Some(d) = datos {
            active.datos_proveedor = Set(Some(d));
        }
        active.update(&*self.db).await.map_err(AppError::from_db)?;
        Ok(())
    }

    async fn es_anonimo(&self, id_usuario: Uuid) -> Result<bool, AppError> {
        // `count` evita cargar la fila completa: solo nos interesa la
        // existencia. PostgreSQL puede responderlo con un index lookup.
        let cuantos = proveedor_autenticacion::Entity::find()
            .filter(proveedor_autenticacion::Column::IdUsuario.eq(id_usuario))
            .filter(proveedor_autenticacion::Column::Proveedor.eq(Some("anonymous".to_string())))
            .count(&*self.db)
            .await
            .map_err(AppError::from_db)?;
        Ok(cuantos > 0)
    }

    async fn tiene_email_verificado(
        &self,
        id_usuario: Uuid,
        email: &str,
    ) -> Result<bool, AppError> {
        let email_normalizado = email.trim().to_ascii_lowercase();
        let proveedores = proveedor_autenticacion::Entity::find()
            .filter(proveedor_autenticacion::Column::IdUsuario.eq(id_usuario))
            .all(&*self.db)
            .await
            .map_err(AppError::from_db)?;

        Ok(proveedores.into_iter().any(|proveedor| {
            let mismo_email = proveedor
                .email_proveedor
                .as_deref()
                .map(|email| email.trim().eq_ignore_ascii_case(&email_normalizado))
                .unwrap_or(false);
            let email_verificado = proveedor
                .datos_proveedor
                .as_ref()
                .and_then(|datos| datos.get("email_verified"))
                .and_then(|valor| valor.as_bool())
                == Some(true);

            mismo_email && email_verificado
        }))
    }
}
