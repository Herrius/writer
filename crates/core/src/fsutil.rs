//! Utilidades de sistema de archivos: escritura atómica, fsync y metadatos.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;

use crate::error::{Error, Result};

/// Escribe bytes de forma atómica: temporal en el mismo directorio, `fsync` y
/// reemplazo. En POSIX se sincroniza además el directorio.
pub fn escribir_atomico(ruta: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    }
    let mut archivo = AtomicWriteFile::options()
        .open(ruta)
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    archivo
        .write_all(bytes)
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    archivo
        .commit()
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    if let Some(padre) = ruta.parent() {
        fsync_dir(padre)?;
    }
    Ok(())
}

/// Fuerza la persistencia del directorio (POSIX; en Windows no aplica).
pub fn fsync_dir(ruta: &Path) -> Result<()> {
    crate::fault::fallar_fsync_dir(ruta)?;
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let archivo = match File::open(ruta) {
            Ok(archivo) => archivo,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(Error::Io(error).with_path(ruta)),
        };
        archivo
            .sync_all()
            .map_err(|error| Error::Io(error).with_path(ruta))?;
    }
    #[cfg(target_os = "macos")]
    {
        // APFS no garantiza `fsync` sobre directorios; el archivo ya se ha
        // sincronizado y el reemplazo es atómico. No se convierte en error.
        if let Ok(archivo) = File::open(ruta) {
            let _ = archivo.sync_all();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = ruta;
    }
    Ok(())
}

/// Elimina un archivo si existe y sincroniza el directorio.
pub fn eliminar_archivo(ruta: &Path) -> Result<()> {
    match std::fs::remove_file(ruta) {
        Ok(()) => {
            if let Some(padre) = ruta.parent() {
                fsync_dir(padre)?;
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(Error::Io(error).with_path(ruta)),
    }
}

/// Fuerza la persistencia de un archivo que ya está en el estado deseado.
pub fn sincronizar_archivo(ruta: &Path) -> Result<()> {
    let archivo = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(ruta)
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    archivo
        .sync_all()
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    if let Some(padre) = ruta.parent() {
        fsync_dir(padre)?;
    }
    Ok(())
}

/// Lee un archivo, o `None` si no existe.
pub fn leer_opcional(ruta: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::read(ruta) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(Error::Io(error).with_path(ruta)),
    }
}

/// Metadatos útiles para el índice.
#[derive(Clone, Copy, Debug)]
pub struct Metadatos {
    /// Modificación en nanosegundos desde la época.
    pub mtime_ns: i64,
    /// Tamaño en bytes.
    pub tamano: i64,
}

/// Obtiene `mtime_ns` y tamaño de un archivo.
pub fn metadatos(ruta: &Path) -> Result<Metadatos> {
    let meta = std::fs::metadata(ruta).map_err(|error| Error::Io(error).with_path(ruta))?;
    let mtime_ns = meta
        .modified()
        .ok()
        .and_then(|momento| momento.duration_since(std::time::UNIX_EPOCH).ok())
        .and_then(|duracion| i64::try_from(duracion.as_nanos()).ok())
        .unwrap_or(0);
    let tamano = i64::try_from(meta.len()).unwrap_or(i64::MAX);
    Ok(Metadatos { mtime_ns, tamano })
}

/// Abre (o crea) un archivo de escritura sin truncar.
pub fn abrir_lectura_escritura(ruta: &Path) -> Result<File> {
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    }
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(ruta)
        .map_err(|error| Error::Io(error).with_path(ruta))
}

/// Escribe un archivo de log de forma append-only tras `set_len(offset)`.
pub fn escribir_en(ruta: &Path, offset: u64, bytes: &[u8]) -> Result<()> {
    let mut archivo = abrir_lectura_escritura(ruta)?;
    archivo
        .set_len(offset)
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    use std::io::{Seek, SeekFrom};
    archivo
        .seek(SeekFrom::Start(offset))
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    archivo
        .write_all(bytes)
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    archivo
        .sync_all()
        .map_err(|error| Error::Io(error).with_path(ruta))?;
    Ok(())
}

/// Ruta relativa con separadores `/`, independiente de la plataforma.
pub fn ruta_relativa(ruta: &Path, raiz: &Path) -> Result<String> {
    let relativa = ruta.strip_prefix(raiz).map_err(|_| {
        Error::Other(format!(
            "{} no está dentro de {}",
            ruta.display(),
            raiz.display()
        ))
    })?;
    Ok(relativa
        .components()
        .map(|componente| componente.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/"))
}

/// Une una ruta relativa del proyecto con su raíz, rechazando `..`.
pub fn ruta_del_proyecto(raiz: &Path, relativa: &str) -> Result<PathBuf> {
    let mut salida = raiz.to_path_buf();
    for componente in relativa.split('/') {
        if componente.is_empty() || componente == "." || componente == ".." {
            return Err(Error::Integrity(format!("ruta no válida: {relativa}")));
        }
        salida.push(componente);
    }
    Ok(salida)
}

/// Comprueba que `ruta` es un directorio real (no un enlace simbólico) y
/// devuelve su ruta canónica.
pub fn directorio_real(ruta: &Path) -> Result<PathBuf> {
    let metadatos =
        std::fs::symlink_metadata(ruta).map_err(|error| Error::Io(error).with_path(ruta))?;
    if !metadatos.file_type().is_dir() {
        return Err(Error::Integrity(format!(
            "se esperaba un directorio real: {}",
            ruta.display()
        )));
    }
    std::fs::canonicalize(ruta).map_err(|error| Error::Io(error).with_path(ruta))
}

/// Comprueba que `ruta` es un archivo real (no un enlace simbólico) y devuelve
/// su ruta canónica.
pub fn archivo_real(ruta: &Path) -> Result<PathBuf> {
    let metadatos =
        std::fs::symlink_metadata(ruta).map_err(|error| Error::Io(error).with_path(ruta))?;
    if !metadatos.file_type().is_file() {
        return Err(Error::Integrity(format!(
            "se esperaba un archivo real: {}",
            ruta.display()
        )));
    }
    std::fs::canonicalize(ruta).map_err(|error| Error::Io(error).with_path(ruta))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escritura_atomica_reemplaza() {
        let directorio = tempfile::tempdir().expect("temporal");
        let ruta = directorio.path().join("archivo.txt");
        escribir_atomico(&ruta, b"uno").expect("escribir");
        assert_eq!(std::fs::read(&ruta).expect("leer"), b"uno");
        escribir_atomico(&ruta, b"dos").expect("reescribir");
        assert_eq!(std::fs::read(&ruta).expect("leer"), b"dos");
    }

    #[test]
    fn rutas_relativas() {
        let raiz = Path::new("/proyecto");
        let ruta = raiz.join("manuscrito/01-libro/01-capitulo/001.md");
        assert_eq!(
            ruta_relativa(&ruta, raiz).expect("relativa"),
            "manuscrito/01-libro/01-capitulo/001.md"
        );
        assert!(ruta_del_proyecto(raiz, "../fuera").is_err());
    }
}
