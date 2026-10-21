//! Errores públicos del motor.
//!
//! La CLI no reinterpreta errores: los muestra. Cada variante indica qué capa
//! falló y si el proyecto puede seguir escribiéndose.

use std::path::PathBuf;

/// Resultado del motor.
pub type Result<T> = std::result::Result<T, Error>;

/// Error del motor de escritura.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error de entrada/salida del sistema de archivos.
    #[error("E/S: {0}")]
    Io(#[from] std::io::Error),

    /// Un documento no cumple el esquema v1.
    #[error("esquema: {0}")]
    Schema(String),

    /// El proyecto no es válido o no se puede abrir.
    #[error("proyecto: {0}")]
    Project(String),

    /// La operación choca con un cambio externo.
    #[error("conflicto: {0}")]
    Conflict(String),

    /// Otro proceso mantiene el lock del proyecto.
    #[error("proyecto ocupado: {0}")]
    Busy(String),

    /// El proyecto necesita recuperación y no se puede escribir todavía.
    #[error("modo recuperación: {0}")]
    Recovery(String),

    /// Error del índice SQLite.
    #[error("índice: {0}")]
    Index(#[from] rusqlite::Error),

    /// Error de serialización del log.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    /// Un documento o revisión no existe.
    #[error("no encontrado: {0}")]
    NotFound(String),

    /// El dato de entrada no es válido (título vacío, orden fuera de rango…).
    #[error("entrada: {0}")]
    Input(String),

    /// Un archivo no tiene el formato esperado.
    #[error("formato: {0}")]
    Format(String),

    /// Un requisito de seguridad o integridad no se cumple.
    #[error("integridad: {0}")]
    Integrity(String),

    /// Error genérico con contexto.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Añade la ruta del archivo al mensaje sin perder la variante.
    pub fn with_path(self, ruta: impl Into<PathBuf>) -> Self {
        let ruta = ruta.into();
        let ruta = ruta.display();
        match self {
            Error::Io(error) => Error::Io(std::io::Error::new(
                error.kind(),
                format!("{ruta}: {error}"),
            )),
            Error::Schema(mensaje) => Error::Schema(format!("{ruta}: {mensaje}")),
            Error::Project(mensaje) => Error::Project(format!("{ruta}: {mensaje}")),
            Error::Conflict(mensaje) => Error::Conflict(format!("{ruta}: {mensaje}")),
            Error::Busy(mensaje) => Error::Busy(format!("{ruta}: {mensaje}")),
            Error::Recovery(mensaje) => Error::Recovery(format!("{ruta}: {mensaje}")),
            Error::Index(error) => Error::Other(format!("{ruta}: índice: {error}")),
            Error::Json(error) => Error::Other(format!("{ruta}: json: {error}")),
            Error::NotFound(mensaje) => Error::NotFound(format!("{ruta}: {mensaje}")),
            Error::Input(mensaje) => Error::Input(format!("{ruta}: {mensaje}")),
            Error::Format(mensaje) => Error::Format(format!("{ruta}: {mensaje}")),
            Error::Integrity(mensaje) => Error::Integrity(format!("{ruta}: {mensaje}")),
            Error::Other(mensaje) => Error::Other(format!("{ruta}: {mensaje}")),
        }
    }
}
