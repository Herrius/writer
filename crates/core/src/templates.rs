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
        let total = self.estructura.libros as u64
            * self.estructura.capitulos_por_libro as u64
            * self.estructura.escenas_por_capitulo as u64;
        if total > 10_000 {
            return Err(Error::Schema(
                "la plantilla generaría más de 10.000 escenas".to_string(),
            ));
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

fn validar_slug(slug: &str) -> Result<()> {
    if slug.is_empty()
        || slug.contains(['/', '\\', '.', ':', '*', '?', '"', '<', '>', '|'])
        || slug.starts_with(' ')
        || slug.ends_with(' ')
        || slug.len() > 120
    {
        return Err(Error::Input(format!("slug de plantilla no válido: {slug}")));
    }
    const RESERVADOS: [&str; 9] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "LPT1", "LPT2", "LPT3",
    ];
    if RESERVADOS
        .iter()
        .any(|reservado| slug.eq_ignore_ascii_case(reservado))
    {
        return Err(Error::Input(format!(
            "slug de plantilla reservado en Windows: {slug}"
        )));
    }
    Ok(())
}

pub fn guardar_plantilla(directorio: &Path, slug: &str, plantilla: &Plantilla) -> Result<()> {
    validar_slug(slug)?;
    plantilla.validar()?;
    if !directorio.exists() {
        std::fs::create_dir_all(directorio)
            .map_err(|error| Error::Io(error).with_path(directorio))?;
    }
    let directorio = crate::fsutil::directorio_real(directorio)?;
    let ruta = directorio.join(slug);
    if ruta.exists() {
        return Err(Error::Conflict(format!(
            "ya existe una plantilla con ese slug: {slug}"
        )));
    }
    std::fs::create_dir(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
    let manifiesto = ruta.join("plantilla.json");
    let bytes = serde_json::to_vec_pretty(plantilla)?;
    if let Err(error) = crate::fsutil::escribir_atomico(&manifiesto, &bytes) {
        let _ = std::fs::remove_dir_all(&ruta);
        return Err(error);
    }
    Ok(())
}

pub fn eliminar_plantilla(directorio: &Path, slug: &str) -> Result<()> {
    validar_slug(slug)?;
    let directorio = crate::fsutil::directorio_real(directorio)?;
    let ruta = directorio.join(slug);
    let metadatos =
        std::fs::symlink_metadata(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
    if metadatos.file_type().is_symlink() || !metadatos.is_dir() {
        return Err(Error::Integrity(format!(
            "se rechaza una plantilla no regular: {}",
            ruta.display()
        )));
    }
    let manifiesto = ruta.join("plantilla.json");
    if !manifiesto.is_file() {
        return Err(Error::NotFound(format!("plantilla no encontrada: {slug}")));
    }
    let bytes =
        std::fs::read(&manifiesto).map_err(|error| Error::Io(error).with_path(&manifiesto))?;
    let plantilla: Plantilla = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Json(error).with_path(&manifiesto))?;
    plantilla.validar()?;
    std::fs::remove_dir_all(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))
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
        let metadatos =
            std::fs::symlink_metadata(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
        if metadatos.file_type().is_symlink() || !metadatos.is_dir() {
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
    validar_slug(slug)?;
    let ruta_plantilla = directorio_plantillas.join(slug);
    let metadatos = std::fs::symlink_metadata(&ruta_plantilla)
        .map_err(|error| Error::Io(error).with_path(&ruta_plantilla))?;
    if metadatos.file_type().is_symlink() {
        return Err(Error::Integrity(format!(
            "se rechaza una plantilla enlazada: {}",
            ruta_plantilla.display()
        )));
    }
    let manifiesto = ruta_plantilla.join("plantilla.json");
    if manifiesto.exists() {
        crate::fsutil::archivo_real(&manifiesto)?;
    }
    let bytes =
        std::fs::read(&manifiesto).map_err(|error| Error::Io(error).with_path(&manifiesto))?;
    let plantilla: Plantilla = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Json(error).with_path(&manifiesto))?;
    plantilla.validar()?;

    let proyecto = Project::new(destino);
    proyecto.init(titulo, autor, idioma)?;
    let resultado = (|| -> Result<()> {
        // La escena que crea `init` no forma parte de la plantilla.
        if let Some(primera) = proyecto.listar(false)?.into_iter().next() {
            proyecto.eliminar(primera.id())?;
        }
        let mut escenas_creadas = Vec::new();
        for libro in 1..=plantilla.estructura.libros {
            let nombre_libro = format!("{libro:02}-libro");
            for capitulo in 1..=plantilla.estructura.capitulos_por_libro {
                let nombre_capitulo = format!("{capitulo:02}-capitulo");
                for escena in 1..=plantilla.estructura.escenas_por_capitulo {
                    let titulo_escena = format!("Escena {escena}");
                    let (id, _) = proyecto.crear_escena(
                        &titulo_escena,
                        Some(&nombre_libro),
                        Some(&nombre_capitulo),
                        None,
                    )?;
                    escenas_creadas.push(id);
                }
            }
        }
        let notas_plantilla = ruta_plantilla.join("notas");
        if !notas_plantilla.is_dir() {
            crear_notas_iniciales(&proyecto, &plantilla.notas_iniciales)?;
        }
        copiar_manuscrito(
            &proyecto,
            &ruta_plantilla.join("manuscrito"),
            &mut escenas_creadas,
        )?;
        copiar_notas(&proyecto, &notas_plantilla)?;
        Ok(())
    })();
    if let Err(error) = resultado {
        let _ = std::fs::remove_dir_all(destino);
        return Err(error);
    }
    Ok(destino.to_path_buf())
}

fn crear_notas_iniciales(proyecto: &Project, tipos: &[String]) -> Result<()> {
    for tipo in tipos {
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
}

fn copiar_manuscrito(
    proyecto: &Project,
    directorio: &Path,
    escenas: &mut Vec<String>,
) -> Result<()> {
    if !directorio.is_dir() {
        return Ok(());
    }
    let mut archivos = Vec::new();
    recorrer_md(directorio, &mut archivos)?;
    archivos.sort();
    for (indice, archivo) in archivos.iter().enumerate() {
        let texto = std::fs::read_to_string(archivo)
            .map_err(|error| Error::Io(error).with_path(archivo))?;
        let cuerpo = crate::import::sin_frontmatter(&texto);
        if let Some(id) = escenas.get(indice) {
            proyecto.escribir_cuerpo(id, cuerpo)?;
        } else {
            let titulo = titulo_desde_archivo(archivo);
            let (id, _) = proyecto.crear_escena(&titulo, None, None, None)?;
            proyecto.escribir_cuerpo(&id, cuerpo)?;
            escenas.push(id);
        }
    }
    Ok(())
}

fn copiar_notas(proyecto: &Project, directorio: &Path) -> Result<()> {
    if !directorio.is_dir() {
        return Ok(());
    }
    let mut archivos = Vec::new();
    recorrer_md(directorio, &mut archivos)?;
    archivos.sort();
    for archivo in archivos {
        let Some(tipo) = archivo
            .parent()
            .and_then(|padre| padre.file_name())
            .and_then(|nombre| nombre.to_str())
        else {
            continue;
        };
        if !crate::model::TIPOS_NOTA.contains(&tipo) {
            continue;
        }
        let texto = std::fs::read_to_string(&archivo)
            .map_err(|error| Error::Io(error).with_path(&archivo))?;
        let cuerpo = crate::import::sin_frontmatter(&texto);
        let titulo = titulo_desde_archivo(&archivo);
        let (id, _) = proyecto.crear_nota(tipo, &titulo)?;
        proyecto.escribir_cuerpo(&id, cuerpo)?;
    }
    Ok(())
}

fn recorrer_md(directorio: &Path, archivos: &mut Vec<PathBuf>) -> Result<()> {
    for entrada in
        std::fs::read_dir(directorio).map_err(|error| Error::Io(error).with_path(directorio))?
    {
        let entrada = entrada.map_err(|error| Error::Io(error).with_path(directorio))?;
        let ruta = entrada.path();
        let metadatos =
            std::fs::symlink_metadata(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
        if metadatos.file_type().is_symlink() {
            return Err(Error::Integrity(format!(
                "se rechaza un enlace simbólico en la plantilla: {}",
                ruta.display()
            )));
        }
        if metadatos.is_dir() {
            recorrer_md(&ruta, archivos)?;
        } else if ruta.extension().and_then(|valor| valor.to_str()) == Some("md") {
            archivos.push(ruta);
        }
    }
    Ok(())
}

fn titulo_desde_archivo(archivo: &Path) -> String {
    let stem = archivo
        .file_stem()
        .and_then(|valor| valor.to_str())
        .unwrap_or("Nota");
    crate::model::titulo_humano(stem)
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
    fn guarda_lista_y_elimina_plantilla() {
        let temporal = tempfile::tempdir().expect("temporal");
        let directorio = temporal.path().join("plantillas");
        let plantilla = plantilla_valida();
        guardar_plantilla(&directorio, "tres-actos", &plantilla).expect("guardar");
        let lista = listar_plantillas(&directorio).expect("listar");
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0].0, "tres-actos");
        assert!(guardar_plantilla(&directorio, "tres-actos", &plantilla).is_err());
        eliminar_plantilla(&directorio, "tres-actos").expect("eliminar");
        assert!(listar_plantillas(&directorio).expect("listar").is_empty());
        assert!(eliminar_plantilla(&directorio, "tres-actos").is_err());
    }

    #[test]
    fn no_elimina_un_directorio_que_no_es_plantilla() {
        let temporal = tempfile::tempdir().expect("temporal");
        let directorio = temporal.path().join("plantillas");
        std::fs::create_dir_all(directorio.join("no-plantilla")).expect("carpeta");
        assert!(eliminar_plantilla(&directorio, "no-plantilla").is_err());
        assert!(directorio.join("no-plantilla").exists());
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
        assert_eq!(escaneo.escenas.len(), 2, "sin la escena inicial de init");
        assert_eq!(escaneo.notas.len(), 1);

        let contenido = temporal
            .path()
            .join("plantillas/tres-actos/manuscrito/01-libro/01-capitulo/001.md");
        std::fs::create_dir_all(contenido.parent().expect("padre")).expect("carpeta");
        std::fs::write(&contenido, "Contenido personalizado").expect("contenido");
        let segundo = temporal.path().join("novela2");
        crear_desde_plantilla(
            &plantillas,
            "tres-actos",
            &segundo,
            "Novela 2",
            None,
            Some("es"),
        )
        .expect("crear con contenido");
        let documentos = Project::new(&segundo).listar(false).expect("listar");
        assert!(documentos
            .iter()
            .any(|documento| documento.cuerpo().contains("Contenido personalizado")));
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
