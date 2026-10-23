//! Planificación local: Plot Grid y referencias.
//!
//! Implementa la extensión `planificacion: 1` de `docs/esquema-v2.md` §5. El
//! estado visual vive en `.escritor/planificacion/plot-grid.json`; las tramas,
//! personajes y notas siguen siendo Markdown en `notas/`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{fsutil, ulid};

/// Versión de la extensión de planificación.
pub const VERSION: u32 = 1;

/// Punto de una trama en una escena.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Punto {
    /// ULID de la escena.
    pub escena: String,
    /// Orden dentro de la trama.
    #[serde(default)]
    pub orden: i64,
    /// Etiqueta libre.
    #[serde(default)]
    pub etiqueta: Option<String>,
}

/// Trama del Plot Grid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TramaGrid {
    /// ULID de la nota de tipo `tramas`.
    pub id: String,
    /// Nombre visible.
    pub nombre: String,
    /// Color `#rrggbb`.
    pub color: String,
    /// Puntos enlazados a escenas.
    #[serde(default)]
    pub puntos: Vec<Punto>,
}

/// Plot Grid completo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlotGrid {
    /// Versión.
    pub version: u32,
    /// Tramas.
    #[serde(default)]
    pub tramas: Vec<TramaGrid>,
}

impl Default for PlotGrid {
    fn default() -> Self {
        Self {
            version: VERSION,
            tramas: Vec::new(),
        }
    }
}

impl PlotGrid {
    /// Valida el documento completo.
    pub fn validar(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(Error::Schema(format!(
                "versión de planificación no soportada: {}",
                self.version
            )));
        }
        let mut ids = BTreeSet::new();
        for trama in &self.tramas {
            if !ulid::es_valido(&trama.id) {
                return Err(Error::Schema(format!(
                    "trama con ULID no válido: {}",
                    trama.id
                )));
            }
            if !ids.insert(trama.id.clone()) {
                return Err(Error::Schema(format!(
                    "trama duplicada en el Plot Grid: {}",
                    trama.id
                )));
            }
            if trama.nombre.trim().is_empty() {
                return Err(Error::Schema("trama sin nombre".to_string()));
            }
            if !color_valido(&trama.color) {
                return Err(Error::Schema(format!(
                    "color de trama no válido: {}",
                    trama.color
                )));
            }
            let mut escenas = BTreeSet::new();
            for punto in &trama.puntos {
                if !ulid::es_valido(&punto.escena) {
                    return Err(Error::Schema(format!(
                        "punto con escena no válida: {}",
                        punto.escena
                    )));
                }
                if !escenas.insert(punto.escena.clone()) {
                    return Err(Error::Schema(format!(
                        "escena repetida en la trama {}: {}",
                        trama.nombre, punto.escena
                    )));
                }
                if let Some(etiqueta) = &punto.etiqueta {
                    if etiqueta.chars().count() > 100 {
                        return Err(Error::Schema(
                            "etiqueta de punto demasiado larga".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Lee el Plot Grid; un proyecto sin él devuelve el valor por defecto.
pub fn leer_plot_grid(raiz: &Path) -> Result<PlotGrid> {
    let Some(ruta) = ruta_plot_grid(raiz, false)? else {
        return Ok(PlotGrid::default());
    };
    if ruta.exists() {
        fsutil::archivo_real(&ruta)?;
    }
    let Some(bytes) = fsutil::leer_opcional(&ruta)? else {
        return Ok(PlotGrid::default());
    };
    let grid: PlotGrid =
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(error).with_path(&ruta))?;
    grid.validar()?;
    Ok(grid)
}

/// Guarda el Plot Grid con reemplazo atómico.
pub fn guardar_plot_grid(raiz: &Path, grid: &PlotGrid) -> Result<()> {
    grid.validar()?;
    let ruta = ruta_plot_grid(raiz, true)?
        .ok_or_else(|| Error::Project("no se pudo crear .escritor/planificacion".to_string()))?;
    let bytes = serde_json::to_vec_pretty(grid)?;
    fsutil::escribir_atomico(&ruta, &bytes)
}

fn ruta_plot_grid(raiz: &Path, crear: bool) -> Result<Option<PathBuf>> {
    let escritor = fsutil::validar_escritor(raiz)?
        .ok_or_else(|| Error::Project("el proyecto no tiene .escritor".to_string()))?;
    let directorio = escritor.join("planificacion");
    if !directorio.exists() {
        if !crear {
            return Ok(None);
        }
        std::fs::create_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?;
        fsutil::fsync_dir(&escritor)?;
    }
    let directorio = fsutil::directorio_real(&directorio)?;
    Ok(Some(directorio.join("plot-grid.json")))
}

fn color_valido(color: &str) -> bool {
    let bytes = color.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raiz() -> tempfile::TempDir {
        let temporal = tempfile::tempdir().expect("temporal");
        std::fs::create_dir(temporal.path().join(".escritor")).expect(".escritor");
        temporal
    }

    fn escena() -> String {
        ulid::desde_partes(1_700_000_000_000, [3; 10])
    }

    fn trama() -> String {
        ulid::desde_partes(1_700_000_000_001, [4; 10])
    }

    #[test]
    fn grid_por_defecto_si_no_existe() {
        let temporal = raiz();
        assert_eq!(
            leer_plot_grid(temporal.path()).expect("leer"),
            PlotGrid::default()
        );
    }

    #[test]
    fn grid_ida_y_vuelta() {
        let temporal = raiz();
        let grid = PlotGrid {
            version: VERSION,
            tramas: vec![TramaGrid {
                id: trama(),
                nombre: "El faro".to_string(),
                color: "#4d6b8a".to_string(),
                puntos: vec![Punto {
                    escena: escena(),
                    orden: 10,
                    etiqueta: Some("giro".to_string()),
                }],
            }],
        };
        guardar_plot_grid(temporal.path(), &grid).expect("guardar");
        assert_eq!(leer_plot_grid(temporal.path()).expect("leer"), grid);
    }

    #[test]
    fn color_invalido_se_rechaza() {
        let grid = PlotGrid {
            version: VERSION,
            tramas: vec![TramaGrid {
                id: trama(),
                nombre: "Trama".to_string(),
                color: "azul".to_string(),
                puntos: vec![],
            }],
        };
        assert!(grid.validar().is_err());
    }

    #[test]
    fn escena_duplicada_se_rechaza() {
        let grid = PlotGrid {
            version: VERSION,
            tramas: vec![TramaGrid {
                id: trama(),
                nombre: "Trama".to_string(),
                color: "#000000".to_string(),
                puntos: vec![
                    Punto {
                        escena: escena(),
                        orden: 1,
                        etiqueta: None,
                    },
                    Punto {
                        escena: escena(),
                        orden: 2,
                        etiqueta: None,
                    },
                ],
            }],
        };
        assert!(grid.validar().is_err());
    }

    #[cfg(unix)]
    #[test]
    fn directorio_enlazado_se_rechaza() {
        use std::os::unix::fs::symlink;
        let temporal = raiz();
        let fuera = tempfile::tempdir().expect("fuera");
        symlink(
            fuera.path(),
            temporal.path().join(".escritor/planificacion"),
        )
        .expect("symlink");
        assert!(leer_plot_grid(temporal.path()).is_err());
    }
}
