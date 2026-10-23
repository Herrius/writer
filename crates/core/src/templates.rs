//! Plantillas locales para crear proyectos.
//!
//! Implementa `docs/plantillas-v1.md`: las plantillas viven en el directorio de
//! datos de la app, no en el proyecto. Crear desde plantilla genera ULID,
//! fechas y estructura nuevos, sin copiar historial ni sidecars.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::project::Project;
use crate::ulid;

/// Versión de la extensión de plantillas.
pub const VERSION: u32 = 1;

/// Manifiesto de plantilla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plantilla {
    /// Versión.
    pub version: u32,
    /// Nombre visible.
    pub nombre: String,
    /// Descripción.
    #[serde(default)]
    pub descripcion: String,
    /// Idioma.
    #[serde(default)]
    pub idioma: Option<String>,
    /// Estructura a generar.
    pub estructura: Estructura,
    /// Tipos de nota iniciales.
    #[serde(default)]
    pub notas_iniciales: Vec<String>,
}

/// Estructura de una plantilla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Estructura {
    /// Libros.
    pub libros: u32,
    /// Capítulos por libro.
    pub capitulos_por_libro: u32,
    /// Escenas por capítulo.
    pub escenas_por_capitulo: u32,
}

impl Plantilla {
    /// Valida el manifiesto.
    pub fn validar(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(Error::Schema(format!(
                "versión de plantilla no soportada: {}",
                self.version
            )));
        }
        if self.nombre.trim().is_empty() || self.nombre.chars().count() > 200 {
            return Err(Error::Schema("nombre de plantilla no válido".to_string()));
        }
        if let Some(idioma) = &self.idioma {
            if idioma.trim().is_empty() || idioma.len() > 20 {
                return Err(Error::Schema("idioma de plantilla no válido".to_string()));
            }
        }
        for (valor, campo) in [
            (self.estructura.libros, "libros"),
            (self.estructura.capitulos_por_libro, "capitulos_por_libro"),
            (self.estructura.escenas_por_capitulo, "escenas_por_capitulo"),
        ] {
            if !(1..=999).contains(&valor) {
                return Err(Error::Schema(format!("{campo} debe estar entre 1 y 999")));
            }
        }
        for tipo in &self.notas_iniciales {
            if !crate::model::TIPOS_NOTA.contains(&tipo.as_str()) {
                return Err(Error::Schema(format!(
                    "tipo de nota inicial no válido: {tipo}"
                )));
            }
        }
        Ok(())
    }
}

/// Lista las plantillas válidas de un directorio.
pub fn listar_plantillas(directorio: &Path) -> Result<Vec<(String, Plantilla)>> {
    if !directorio.exists() {
        return Ok(Vec::new());
    }
    let directorio = crate::fsutil::directorio_real(directorio)?;
    let mut plantillas = Vec::new();
    for entrada in
        std::fs::read_dir(&directorio).map_err(|error| Error::Io(error).with_path(&directorio))?
    {
        let entrada = entrada.map_err(|error| Error::Io(error).with_path(&directorio))?;
        let ruta = entrada.path();
        if !ruta.is_dir() {
            continue;
        }
        let manifiesto = ruta.join("plantilla.json");
        if !manifiesto.is_file() {
            continue;
        }
        let bytes =
            std::fs::read(&manifiesto).map_err(|error| Error::Io(error).with_path(&manifiesto))?;
        let plantilla: Plantilla = match serde_json::from_slice(&bytes) {
            Ok(plantilla) => plantilla,
            Err(_) => continue,
        };
        if plantilla.validar().is_err() {
            continue;
        }
        let slug = ruta
            .file_name()
            .and_then(|valor| valor.to_str())
            .unwrap_or_default()
            .to_string();
        plantillas.push((slug, plantilla));
    }
    plantillas.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(plantillas)
}

/// Crea un proyecto desde una plantilla.
///
/// `slug` identifica la carpeta de la plantilla; `destino` es la carpeta del
/// proyecto nuevo. El proyecto debe poder inicializarse y la plantilla debe ser
/// válida.
pub fn crear_desde_plantilla(
    directorio_plantillas: &Path,
    slug: &str,
    destino: &Path,
    titulo: &str,
    autor: Option<&str>,
    idioma: Option<&str>,
) -> Result<PathBuf> {
    if !ulid::es_valido(&ulid::nuevo()?) {
        return Err(Error::Other("ULID no válido".to_string()));
    }
    if slug.is_empty() || slug.contains(['/', '\\', '.']) {
        return Err(Error::Input(format!("slug de plantilla no válido: {slug}")));
    }
    let ruta_plantilla = directorio_plantillas.join(slug);
    let manifiesto = ruta_plantilla.join("plantilla.json");
    let bytes =
        std::fs::read(&manifiesto).map_err(|error| Error::Io(error).with_path(&manifiesto))?;
    let plantilla: Plantilla = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Json(error).with_path(&manifiesto))?;
    plantilla.validar()?;

    let proyecto = Project::new(destino);
    proyecto.init(titulo, autor, idioma)?;
    let resultado = (|| -> Result<()> {
        for libro in 1..=plantilla.estructura.libros {
            let nombre_libro = format!("{libro:02}-libro");
            for capitulo in 1..=plantilla.estructura.capitulos_por_libro {
                let nombre_capitulo = format!("{capitulo:02}-capitulo");
                for escena in 1..=plantilla.estructura.escenas_por_capitulo {
                    let titulo_escena = format!("Escena {escena}");
                    proyecto.crear_escena(
                        &titulo_escena,
                        Some(&nombre_libro),
                        Some(&nombre_capitulo),
                        None,
                    )?;
                }
            }
        }
        for tipo in &plantilla.notas_iniciales {
            let etiqueta = match tipo.as_str() {
                "personajes" => "Personajes",
                "tramas" => "Tramas",
                "lugares" => "Lugares",
                "cronologia" => "Cronología",
                "objetos" => "Objetos",
                "cuaderno" => "Cuaderno",
                "ideas" => "Ideas",
                _ => "Nota",
            };
            proyecto.crear_nota(tipo, etiqueta)?;
        }
        Ok(())
    })();
    if let Err(error) = resultado {
        let _ = std::fs::remove_dir_all(destino);
        return Err(error);
    }
    Ok(destino.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plantilla_valida() -> Plantilla {
        Plantilla {
            version: VERSION,
            nombre: "Tres actos".to_string(),
            descripcion: "Base".to_string(),
            idioma: Some("es".to_string()),
            estructura: Estructura {
                libros: 1,
                capitulos_por_libro: 2,
                escenas_por_capitulo: 1,
            },
            notas_iniciales: vec!["personajes".to_string()],
        }
    }

    #[test]
    fn plantilla_invalida_se_rechaza() {
        let mut plantilla = plantilla_valida();
        plantilla.estructura.libros = 0;
        assert!(plantilla.validar().is_err());
    }

    #[test]
    fn lista_omite_plantillas_invalidas() {
        let temporal = tempfile::tempdir().expect("temporal");
        let buena = temporal.path().join("buena");
        std::fs::create_dir(&buena).expect("carpeta");
        std::fs::write(
            buena.join("plantilla.json"),
            serde_json::to_vec(&plantilla_valida()).expect("json"),
        )
        .expect("escribir");
        let mala = temporal.path().join("mala");
        std::fs::create_dir(&mala).expect("carpeta");
        std::fs::write(mala.join("plantilla.json"), b"{no es json").expect("escribir");
        let lista = listar_plantillas(temporal.path()).expect("listar");
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].0, "buena");
    }

    #[test]
    fn crea_proyecto_desde_plantilla() {
        let temporal = tempfile::tempdir().expect("temporal");
        let plantillas = temporal.path().join("plantillas");
        let buena = plantillas.join("tres-actos");
        std::fs::create_dir_all(&buena).expect("carpeta");
        std::fs::write(
            buena.join("plantilla.json"),
            serde_json::to_vec(&plantilla_valida()).expect("json"),
        )
        .expect("escribir");

        let destino = temporal.path().join("novela");
        crear_desde_plantilla(
            &plantillas,
            "tres-actos",
            &destino,
            "Novela",
            Some("Autora"),
            Some("es"),
        )
        .expect("crear");
        let escaneo = Project::new(&destino).verificar().expect("verificar");
        assert!(escaneo.errores.is_empty(), "errores: {:?}", escaneo.errores);
        assert_eq!(escaneo.escenas.len(), 3, "1 inicial + 2 de la plantilla");
        assert_eq!(escaneo.notas.len(), 1);
    }

    #[test]
    fn slug_con_ruta_se_rechaza() {
        let temporal = tempfile::tempdir().expect("temporal");
        assert!(crear_desde_plantilla(
            temporal.path(),
            "../fuera",
            &temporal.path().join("x"),
            "X",
            None,
            None
        )
        .is_err());
    }
}
