//! Motor del proyecto de escritura.
//!
//! Esta crate es la única fuente de reglas: la CLI, la aplicación de escritorio y
//! el servidor MCP no reimplementan nada, solo llaman a esta API.
//!
//! La Fase 1 implementa el modelo, la escritura atómica, los blobs, el log de
//! operaciones, la recuperación, el lock del SO, el índice SQLite/FTS5 y las
//! operaciones de proyecto. La documentación normativa está en
//! `docs/esquema-v1.md` y `docs/protocolo-commit.md`.

#![forbid(unsafe_code)]

pub mod blob;
pub mod comments;
pub mod docx;
pub mod error;
pub mod fault;
pub mod fsutil;
pub mod goals;
pub mod hash;
pub mod import;
pub mod index;
pub mod lock;
pub mod model;
pub mod oplog;
pub mod planning;
pub mod project;
pub mod recovery;
pub mod templates;
pub mod time;
pub mod ulid;
pub mod verify;
pub mod yaml;

pub use error::{Error, Result};
pub use project::{
    Cambio, CuarentenaItem, Project, ResultadoBusqueda, ResultadoGuardado, ResultadoRevertir,
    RevisionDetalle, SesionEscritura, SesionLectura,
};

/// Versión de la crate, inyectada por Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Versión del esquema en disco que entiende este motor.
///
/// Debe coincidir con el campo `esquema` de `proyecto.yml` y con
/// `docs/esquema-v1.md`.
pub const ESQUEMA_VERSION: u32 = 1;

/// Directorio operativo dentro de un proyecto (`<proyecto>/.escritor`).
pub const DIR_ESCRITOR: &str = ".escritor";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn esquema_soportado_es_el_uno() {
        assert_eq!(ESQUEMA_VERSION, 1);
    }

    #[test]
    fn version_esta_definida() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.starts_with("0."));
    }
}
