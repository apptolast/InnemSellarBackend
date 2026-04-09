// src/models/mod.rs
//
// En SeaORM, cada modulo exporta Model, Entity, Column, ActiveModel, Relation.
// El patron de uso es:
//   use crate::models::comunidad_autonoma;
//   comunidad_autonoma::Entity::find().all(&db).await
//   let ca: comunidad_autonoma::Model = ...;

pub mod enums;

// Geografia
pub mod comunidad_autonoma;
pub mod oficina_sepe;
pub mod provincia;

// Autenticacion
pub mod proveedor_autenticacion;
pub mod token_refresco;
pub mod usuario;

// Contenido
pub mod consejo;
pub mod curso;
pub mod oferta_empleo;

// Relaciones N:M
pub mod consejo_provincia;
pub mod curso_provincia;
pub mod oferta_provincia;

// Interacciones
pub mod reporte;
pub mod voto;

// Sistema
pub mod configuracion_aplicacion;
pub mod prestacion;
