//! Papelera local de proyectos.
//!
//! Un proyecto no se borra: se mueve a `.writer-papelera/<ULID>/proyecto` en la
//! misma carpeta padre. Al estar en el mismo sistema de archivos, el movimiento
//! es un `rename` y no hay copia intermedia. La metadata vive en
//! `.writer-papelera/<ULID>/papelera.json`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::time;
use crate::ulid;

/// Versión de la entrada de papelera.
pub const VERSION: u32 = 1;
/// Nombre del directorio oculto de papelera.
pub const DIRECTORIO: &str = ".writer-papelera";
const MAX_TITULO: usize = 200;
const MAX_RUTA: usize = 4096;
const MAX_AUTOR: usize = 200;
const MAX_MANIFIESTO: u64 = 64 * 1024;

/// Entrada de un proyecto en la papelera local.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntradaPapelera {
    /// Versión.
    pub version: u32,
    /// ULID de la entrada.
    pub id: String,
    /// Título.
    pub titulo: String,
    /// Autor, si existía.
    #[serde(default)]
    pub autor: Option<String>,
    /// Color de portada, si existía.
    #[serde(default)]
    pub color: Option<String>,
    /// Palabras, si se conocían.
    #[serde(default)]
    pub palabras: u64,
    /// Ruta original absoluta.
    pub ruta_original: String,
    /// Ruta del contenedor en `.writer-papelera`.
    pub ruta_papelera: String,
    /// Fecha RFC 3339.
    pub eliminado: String,
}

impl EntradaPapelera {
    /// Valida una entrada leída de disco.
    pub fn validar(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(Error::Schema(format!(
                "versión de papelera no soportada: {}",
                self.version
            )));
        }
        if !ulid::es_valido(&self.id) {
            return Err(Error::Schema(format!(
                "papelera con ULID no válido: {}",
                self.id
            )));
        }
        if self.titulo.trim().is_empty() || self.titulo.chars().count() > MAX_TITULO {
            return Err(Error::Schema("título de papelera no válido".to_string()));
        }
        let original = Path::new(&self.ruta_original);
        let contenedor = Path::new(&self.ruta_papelera);
        if !original.is_absolute() || !contenedor.is_absolute() {
            return Err(Error::Schema(
                "las rutas de la papelera deben ser absolutas".to_string(),
            ));
        }
        if self.ruta_original.len() > MAX_RUTA || self.ruta_papelera.len() > MAX_RUTA {
            return Err(Error::Schema(
                "ruta de papelera demasiado larga".to_string(),
            ));
        }
        time::parse_rfc3339(&self.eliminado)?;
        if let Some(autor) = &self.autor {
            if autor.chars().count() > MAX_AUTOR {
                return Err(Error::Schema(
                    "autor de papelera demasiado largo".to_string(),
                ));
            }
        }
        if let Some(color) = &self.color {
            if color.len() != 7
                || !color.starts_with('#')
                || !color[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(Error::Schema("color de papelera no válido".to_string()));
            }
        }
        let original = Path::new(&self.ruta_original);
        let slot = Path::new(&self.ruta_papelera);
        let padre_slot = slot
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| Error::Schema("ruta de papelera sin padre original".to_string()))?;
        if original.parent() != Some(padre_slot) {
            return Err(Error::Schema(
                "la ruta original no es hermana de la papelera".to_string(),
            ));
        }
        Ok(())
    }

    fn manifiesto(&self) -> PathBuf {
        Path::new(&self.ruta_papelera).join("papelera.json")
    }
}

/// Mueve un proyecto a la papelera local.
pub fn mover_a_papelera(
    ruta_proyecto: &Path,
    titulo: &str,
    autor: Option<&str>,
    color: Option<&str>,
    palabras: u64,
) -> Result<EntradaPapelera> {
    let titulo = titulo.trim();
    if titulo.is_empty() || titulo.chars().count() > MAX_TITULO {
        return Err(Error::Input("título de proyecto no válido".to_string()));
    }
    let metadatos = std::fs::symlink_metadata(ruta_proyecto)
        .map_err(|error| Error::Io(error).with_path(ruta_proyecto))?;
    if metadatos.file_type().is_symlink() || !metadatos.is_dir() {
        return Err(Error::Integrity(format!(
            "se rechaza mover un proyecto no regular: {}",
            ruta_proyecto.display()
        )));
    }
    let proyecto = dunce::canonicalize(ruta_proyecto)
        .map_err(|error| Error::Io(error).with_path(ruta_proyecto))?;
    if !proyecto.join("proyecto.yml").is_file() {
        return Err(Error::Input(format!(
            "no parece un proyecto writer: {}",
            ruta_proyecto.display()
        )));
    }
    let padre = proyecto
        .parent()
        .ok_or_else(|| Error::Input("el proyecto no tiene carpeta padre".to_string()))?;
    let raiz_papelera = padre.join(DIRECTORIO);
    if raiz_papelera.exists() {
        crate::fsutil::directorio_real(&raiz_papelera)?;
    } else {
        std::fs::create_dir(&raiz_papelera)
            .map_err(|error| Error::Io(error).with_path(&raiz_papelera))?;
    }
    let id = ulid::nuevo()?;
    let slot = raiz_papelera.join(&id);
    if slot.exists() {
        return Err(Error::Conflict(
            "la entrada de papelera ya existe".to_string(),
        ));
    }
    std::fs::create_dir(&slot).map_err(|error| Error::Io(error).with_path(&slot))?;
    let entrada = EntradaPapelera {
        version: VERSION,
        id,
        titulo: titulo.to_string(),
        autor: autor.map(str::to_string),
        color: color.map(str::to_string),
        palabras,
        ruta_original: proyecto.to_string_lossy().to_string(),
        ruta_papelera: slot.to_string_lossy().to_string(),
        eliminado: time::ahora(),
    };
    // La metadata se escribe antes de tocar el proyecto. Si falla, el slot
    // contiene solo el manifiesto y se puede eliminar sin ningún riesgo.
    if let Err(error) = escribir_manifiesto(&entrada) {
        let _ = std::fs::remove_dir_all(&slot);
        return Err(error);
    }
    let proyecto_en_papelera = slot.join("proyecto");
    if let Err(error) = std::fs::rename(&proyecto, &proyecto_en_papelera) {
        let _ = std::fs::remove_dir_all(&slot);
        return Err(Error::Io(error).with_path(&proyecto));
    }
    Ok(entrada)
}

/// Restaura un proyecto desde su carpeta de papelera.
pub fn restaurar_de_papelera(ruta_papelera: &Path) -> Result<PathBuf> {
    let entrada = leer_entrada(ruta_papelera)?;
    let proyecto_en_papelera = ruta_papelera.join("proyecto");
    let metadatos = std::fs::symlink_metadata(&proyecto_en_papelera)
        .map_err(|error| Error::Io(error).with_path(&proyecto_en_papelera))?;
    if metadatos.file_type().is_symlink() || !metadatos.is_dir() {
        return Err(Error::Integrity(
            "el proyecto de la papelera no es un directorio real".to_string(),
        ));
    }
    let destino = PathBuf::from(&entrada.ruta_original);
    if destino.exists() {
        return Err(Error::Conflict(format!(
            "el destino original ya existe: {}",
            destino.display()
        )));
    }
    let padre = destino
        .parent()
        .ok_or_else(|| Error::Input("la ruta original no tiene carpeta padre".to_string()))?;
    crate::fsutil::directorio_real(padre)?;
    std::fs::rename(&proyecto_en_papelera, &destino)
        .map_err(|error| Error::Io(error).with_path(&destino))?;
    let _ = std::fs::remove_dir_all(ruta_papelera);
    Ok(destino)
}

/// Elimina definitivamente una entrada de papelera.
pub fn purgar_papelera(ruta_papelera: &Path) -> Result<()> {
    let _entrada = leer_entrada(ruta_papelera)?;
    std::fs::remove_dir_all(ruta_papelera)
        .map_err(|error| Error::Io(error).with_path(ruta_papelera))
}

/// Lee y valida la metadata de una entrada de papelera.
pub fn leer_entrada(ruta_papelera: &Path) -> Result<EntradaPapelera> {
    let nombre = ruta_papelera
        .file_name()
        .and_then(|valor| valor.to_str())
        .ok_or_else(|| Error::Input("ruta de papelera no válida".to_string()))?;
    if !ulid::es_valido(nombre) {
        return Err(Error::Input("entrada de papelera no válida".to_string()));
    }
    let padre = ruta_papelera
        .parent()
        .and_then(|valor| valor.file_name())
        .and_then(|valor| valor.to_str());
    if padre != Some(DIRECTORIO) {
        return Err(Error::Input(
            "la ruta no está dentro de .writer-papelera".to_string(),
        ));
    }
    let metadatos = std::fs::symlink_metadata(ruta_papelera)
        .map_err(|error| Error::Io(error).with_path(ruta_papelera))?;
    if metadatos.file_type().is_symlink() || !metadatos.is_dir() {
        return Err(Error::Integrity(
            "la entrada de papelera no es un directorio real".to_string(),
        ));
    }
    let padre_papelera = ruta_papelera
        .parent()
        .ok_or_else(|| Error::Input("la entrada no tiene padre".to_string()))?;
    crate::fsutil::directorio_real(padre_papelera)?;
    let canonica = crate::fsutil::directorio_real(ruta_papelera)?;
    if canonica != ruta_papelera {
        return Err(Error::Integrity(
            "la ruta de papelera no es canónica".to_string(),
        ));
    }
    let manifiesto = ruta_papelera.join("papelera.json");
    let canonico = crate::fsutil::archivo_real(&manifiesto)?;
    let tamano = std::fs::metadata(&canonico)
        .map_err(|error| Error::Io(error).with_path(&canonico))?
        .len();
    if tamano > MAX_MANIFIESTO {
        return Err(Error::Input(
            "metadata de papelera demasiado grande".to_string(),
        ));
    }
    let bytes = std::fs::read(&canonico).map_err(|error| Error::Io(error).with_path(&canonico))?;
    let entrada: EntradaPapelera = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Json(error).with_path(&manifiesto))?;
    entrada.validar()?;
    if entrada.id != nombre {
        return Err(Error::Integrity(
            "el id de la papelera no coincide con su carpeta".to_string(),
        ));
    }
    let esperado = ruta_papelera.to_string_lossy();
    if entrada.ruta_papelera != esperado {
        return Err(Error::Integrity(
            "la metadata de la papelera apunta a otra carpeta".to_string(),
        ));
    }
    Ok(entrada)
}

fn escribir_manifiesto(entrada: &EntradaPapelera) -> Result<()> {
    entrada.validar()?;
    let bytes = serde_json::to_vec_pretty(entrada)?;
    crate::fsutil::escribir_atomico(&entrada.manifiesto(), &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Project;

    fn proyecto_temporal() -> (tempfile::TempDir, PathBuf) {
        let temporal = tempfile::tempdir().expect("temporal");
        let ruta = temporal.path().join("novela");
        Project::new(&ruta)
            .init("Novela", None, None)
            .expect("init");
        (temporal, ruta)
    }

    #[test]
    fn mueve_y_restaura_un_proyecto() {
        let (temporal, ruta) = proyecto_temporal();
        let entrada = mover_a_papelera(&ruta, "Novela", None, Some("#6b5747"), 10).expect("mover");
        assert!(!ruta.exists());
        assert!(Path::new(&entrada.ruta_papelera).join("proyecto").is_dir());
        let restaurada =
            restaurar_de_papelera(Path::new(&entrada.ruta_papelera)).expect("restaurar");
        assert_eq!(restaurada, ruta);
        assert!(restaurada.join("proyecto.yml").is_file());
        assert!(!Path::new(&entrada.ruta_papelera).exists());
        let _ = temporal;
    }

    #[test]
    fn purga_la_papelera() {
        let (_temporal, ruta) = proyecto_temporal();
        let entrada = mover_a_papelera(&ruta, "Novela", None, None, 0).expect("mover");
        purgar_papelera(Path::new(&entrada.ruta_papelera)).expect("purgar");
        assert!(!Path::new(&entrada.ruta_papelera).exists());
        assert!(!ruta.exists());
    }

    #[test]
    fn rechaza_un_directorio_que_no_es_proyecto() {
        let temporal = tempfile::tempdir().expect("temporal");
        let ruta = temporal.path().join("no-proyecto");
        std::fs::create_dir(&ruta).expect("carpeta");
        assert!(mover_a_papelera(&ruta, "X", None, None, 0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn rechaza_un_proyecto_enlazado() {
        use std::os::unix::fs::symlink;

        let (temporal, ruta) = proyecto_temporal();
        let enlace = temporal.path().join("enlace");
        symlink(&ruta, &enlace).expect("symlink");
        assert!(mover_a_papelera(&enlace, "Novela", None, None, 0).is_err());
    }

    #[test]
    fn no_lee_una_papelera_fuera_del_directorio_esperado() {
        let temporal = tempfile::tempdir().expect("temporal");
        let fuera = temporal.path().join("fuera");
        std::fs::create_dir(&fuera).expect("fuera");
        std::fs::write(fuera.join("papelera.json"), b"{}").expect("json");
        assert!(leer_entrada(&fuera).is_err());
    }

    #[test]
    fn no_purga_un_directorio_arbitrario() {
        let temporal = tempfile::tempdir().expect("temporal");
        let fuera = temporal.path().join("no-papelera");
        std::fs::create_dir(&fuera).expect("fuera");
        assert!(purgar_papelera(&fuera).is_err());
        assert!(fuera.exists());
    }

    #[test]
    fn rechaza_metadata_que_restaura_dentro_del_slot() {
        let (_temporal, ruta) = proyecto_temporal();
        let entrada = mover_a_papelera(&ruta, "Novela", None, None, 0).expect("mover");
        let slot = Path::new(&entrada.ruta_papelera);
        let manifiesto = slot.join("papelera.json");
        let mut valor: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifiesto).expect("leer")).expect("json");
        valor["ruta_original"] =
            serde_json::Value::String(slot.join("recuperado").to_string_lossy().to_string());
        std::fs::write(&manifiesto, serde_json::to_vec(&valor).expect("json")).expect("escribir");
        assert!(leer_entrada(slot).is_err());
        assert!(restaurar_de_papelera(slot).is_err());
        assert!(
            slot.join("proyecto").is_dir(),
            "el proyecto sigue en la papelera"
        );
    }

    #[test]
    fn rechaza_metadata_que_restaura_fuera_del_padre() {
        let (temporal, ruta) = proyecto_temporal();
        let entrada = mover_a_papelera(&ruta, "Novela", None, None, 0).expect("mover");
        let slot = Path::new(&entrada.ruta_papelera);
        let fuera = temporal.path().join("sub").join("ajeno");
        let manifiesto = slot.join("papelera.json");
        let mut valor: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifiesto).expect("leer")).expect("json");
        valor["ruta_original"] = serde_json::Value::String(fuera.to_string_lossy().to_string());
        std::fs::write(&manifiesto, serde_json::to_vec(&valor).expect("json")).expect("escribir");
        assert!(restaurar_de_papelera(slot).is_err());
        assert!(!fuera.exists());
        assert!(slot.join("proyecto").is_dir());
    }

    #[test]
    fn no_restaura_si_el_destino_existe() {
        let (_temporal, ruta) = proyecto_temporal();
        let entrada = mover_a_papelera(&ruta, "Novela", None, None, 0).expect("mover");
        std::fs::create_dir(&ruta).expect("destino ocupado");
        assert!(restaurar_de_papelera(Path::new(&entrada.ruta_papelera)).is_err());
    }
}
