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

/// Comprueba la extensión de un archivo sin depender de mayúsculas/minúsculas.
///
/// Windows conserva el caso pero no lo distingue; aceptar `.MD` evita que un
/// proyecto editado en Explorer aparezca vacío al abrirlo en Windows.
pub fn es_extension(ruta: &Path, extension: &str) -> bool {
    ruta.extension()
        .and_then(|valor| valor.to_str())
        .is_some_and(|valor| valor.eq_ignore_ascii_case(extension))
}

/// Valida una ruta relativa del proyecto sin tocar el sistema de archivos.
///
/// Rechaza separadores de Windows, prefijos de unidad, UNC, componentes `.` o
/// `..`, caracteres de control y nombres reservados de Windows. La comprobación
/// es deliberadamente estricta: una ruta de proyecto nunca necesita esas formas.
pub fn validar_ruta_relativa(relativa: &str) -> Result<()> {
    if relativa.is_empty()
        || relativa.starts_with('/')
        || relativa.starts_with('\\')
        || relativa.contains('\\')
        || relativa.contains(':')
        || relativa.contains('\0')
    {
        return Err(Error::Integrity(format!("ruta no válida: {relativa}")));
    }
    for componente in relativa.split('/') {
        if !componente_valido(componente) {
            return Err(Error::Integrity(format!("ruta no válida: {relativa}")));
        }
    }
    Ok(())
}

/// Comprueba si un componente de ruta es seguro y válido en Linux y Windows.
///
/// Se comparten las reglas más estrictas de ambos sistemas: nada de caracteres
/// reservados de Windows, nombres de dispositivo, puntos/espacios finales ni
/// componentes de más de 255 unidades UTF-16. Así un proyecto creado en Linux
/// también se puede abrir y sincronizar en Windows.
pub fn componente_valido(componente: &str) -> bool {
    !componente.is_empty()
        && componente != "."
        && componente != ".."
        && !componente.starts_with(' ')
        && !componente.ends_with('.')
        && !componente.ends_with(' ')
        && !componente.chars().any(char::is_control)
        && !componente.chars().any(|caracter| {
            matches!(
                caracter,
                '/' | '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*'
            )
        })
        && componente.encode_utf16().count() <= 255
        && !es_nombre_reservado(componente)
}

fn es_nombre_reservado(componente: &str) -> bool {
    let mayusculas = componente.to_ascii_uppercase();
    let base = mayusculas.split('.').next().unwrap_or("");
    let base = base.trim_end_matches([' ', '.']);
    matches!(
        base,
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$" | "CLOCK$"
    ) || (base.len() == 4
        && (base.starts_with("COM") || base.starts_with("LPT"))
        && base.as_bytes()[3].is_ascii_digit()
        && base.as_bytes()[3] != b'0')
        || matches!(
            base,
            "COM\u{00b9}"
                | "COM\u{00b2}"
                | "COM\u{00b3}"
                | "LPT\u{00b9}"
                | "LPT\u{00b2}"
                | "LPT\u{00b3}"
        )
}

/// Valida una ruta que debe ser un documento del proyecto.
pub fn validar_ruta_documento(relativa: &str) -> Result<()> {
    validar_ruta_relativa(relativa)?;
    if relativa.starts_with("manuscrito/") || relativa.starts_with("notas/") {
        Ok(())
    } else {
        Err(Error::Integrity(format!(
            "ruta fuera de manuscrito/ o notas/: {relativa}"
        )))
    }
}

/// Une una ruta relativa del proyecto con su raíz y rechaza enlaces simbólicos
/// en cualquier componente existente. Devuelve la ruta bajo la raíz canónica.
pub fn ruta_del_proyecto(raiz: &Path, relativa: &str) -> Result<PathBuf> {
    validar_ruta_relativa(relativa)?;
    let raiz_real = dunce::canonicalize(raiz).map_err(|error| Error::Io(error).with_path(raiz))?;
    let componentes: Vec<&str> = relativa.split('/').collect();
    let mut salida = raiz_real;
    for (indice, componente) in componentes.iter().enumerate() {
        salida.push(componente);
        match std::fs::symlink_metadata(&salida) {
            Ok(metadatos) => {
                if metadatos.file_type().is_symlink() {
                    return Err(Error::Integrity(format!(
                        "la ruta del proyecto contiene un enlace simbólico: {}",
                        salida.display()
                    )));
                }
                if indice + 1 < componentes.len() {
                    if !metadatos.is_dir() {
                        return Err(Error::Integrity(format!(
                            "un componente de la ruta no es un directorio: {}",
                            salida.display()
                        )));
                    }
                } else if !metadatos.is_file() {
                    return Err(Error::Integrity(format!(
                        "la ruta no es un archivo regular: {}",
                        salida.display()
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                for resto in &componentes[indice + 1..] {
                    salida.push(resto);
                }
                break;
            }
            Err(error) => return Err(Error::Io(error).with_path(&salida)),
        }
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
    dunce::canonicalize(ruta).map_err(|error| Error::Io(error).with_path(ruta))
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
    dunce::canonicalize(ruta).map_err(|error| Error::Io(error).with_path(ruta))
}

#[cfg(unix)]
fn es_enlace_duro(metadatos: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    metadatos.nlink() > 1
}

#[cfg(not(unix))]
fn es_enlace_duro(_metadatos: &std::fs::Metadata) -> bool {
    // En Windows los enlaces duros también existen, pero `nlink` no está
    // disponible de forma estable en todas las versiones objetivo. La defensa
    // principal sigue siendo el rechazo de symlinks/junctions.
    false
}

/// Valida `.escritor` y sus rutas operativas antes de abrirlas o crearlas.
///
/// Devuelve `None` si `.escritor` no existe. La raíz puede ser un alias, pero
/// `.escritor`, el log, el lock, los blobs, el índice y la cuarentena deben ser
/// reales: un enlace simbólico permitiría escribir o borrar fuera del proyecto.
pub fn validar_escritor(raiz: &Path) -> Result<Option<PathBuf>> {
    let ruta = raiz.join(".escritor");
    match std::fs::symlink_metadata(&ruta) {
        Ok(metadatos) => {
            if metadatos.file_type().is_symlink() {
                return Err(Error::Integrity(format!(
                    "se rechaza .escritor enlazado: {}",
                    ruta.display()
                )));
            }
            if !metadatos.is_dir() {
                return Err(Error::Integrity(format!(
                    "se esperaba un directorio .escritor real: {}",
                    ruta.display()
                )));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(Error::Io(error).with_path(&ruta)),
    }
    let real = directorio_real(&ruta)?;
    for (nombre, es_directorio) in [
        ("operaciones.log", false),
        ("bloqueo", false),
        ("objetos", true),
        ("cuarentena", true),
        ("metas", true),
        ("sesiones", true),
        ("comentarios", true),
        ("planificacion", true),
        ("indice.sqlite", false),
        ("indice.sqlite-wal", false),
        ("indice.sqlite-shm", false),
    ] {
        let operativa = real.join(nombre);
        match std::fs::symlink_metadata(&operativa) {
            Ok(metadatos) => {
                if metadatos.file_type().is_symlink() {
                    return Err(Error::Integrity(format!(
                        "se rechaza una ruta operativa enlazada: {}",
                        operativa.display()
                    )));
                }
                if !es_directorio && es_enlace_duro(&metadatos) {
                    return Err(Error::Integrity(format!(
                        "se rechaza una ruta operativa con enlaces duros: {}",
                        operativa.display()
                    )));
                }
                let correcto = if es_directorio {
                    metadatos.is_dir()
                } else {
                    metadatos.is_file()
                };
                if !correcto {
                    return Err(Error::Integrity(format!(
                        "tipo incorrecto en la ruta operativa {}",
                        operativa.display()
                    )));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(Error::Io(error).with_path(&operativa)),
        }
    }
    Ok(Some(real))
}

/// Valida los dos directorios operativos y el informe antes de acceder a la
/// cuarentena. La raíz puede ser un alias, pero sus directorios internos no.
/// Devuelve `None` si falta un directorio y no se pidió crearlo.
pub(crate) fn directorio_cuarentena(raiz: &Path, crear: bool) -> Result<Option<PathBuf>> {
    let mut ruta = dunce::canonicalize(raiz).map_err(|error| Error::Io(error).with_path(raiz))?;
    for nombre in [".escritor", "cuarentena"] {
        ruta.push(nombre);
        match std::fs::symlink_metadata(&ruta) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if !crear {
                    return Ok(None);
                }
                std::fs::create_dir(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
                if let Some(padre) = ruta.parent() {
                    fsync_dir(padre)?;
                }
            }
            Err(error) => return Err(Error::Io(error).with_path(&ruta)),
        }
        ruta = directorio_real(&ruta)?;
    }
    let informe = ruta.join("informe.jsonl");
    match std::fs::symlink_metadata(&informe) {
        Ok(_) => {
            archivo_real(&informe)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(Error::Io(error).with_path(&informe)),
    }
    Ok(Some(ruta))
}

/// Aparta un sidecar inválido a la cuarentena del proyecto.
pub(crate) fn apartar_sidecar(raiz: &Path, ruta: &Path) -> Result<PathBuf> {
    let directorio = directorio_cuarentena(raiz, true)?
        .ok_or_else(|| Error::Project("no se pudo abrir la cuarentena".to_string()))?;
    let nombre = ruta
        .file_name()
        .and_then(|valor| valor.to_str())
        .unwrap_or("sidecar.json");
    let marca = crate::time::timestamp_slug(crate::time::now_ms());
    let destino = directorio.join(format!("{marca}-{nombre}"));
    std::fs::rename(ruta, &destino).map_err(|error| Error::Io(error).with_path(ruta))?;
    fsync_dir(&directorio)?;
    Ok(destino)
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

    #[test]
    fn valida_rutas_sin_formas_de_windows() {
        for ruta in [
            "..\\fuera",
            "C:\\Windows\\x.md",
            "\\\\servidor\\recurso\\x.md",
            "manuscrito\\x.md",
            "manuscrito/01/../../fuera.md",
            "manuscrito//x.md",
            "manuscrito/CON.md",
            "manuscrito/CONIN$.md",
            "manuscrito/CLOCK$.md",
            "manuscrito/COM\u{00b9}.md",
            "manuscrito/a?b.md",
            "manuscrito/a*b.md",
            "manuscrito/a\"b.md",
            "manuscrito/a<b.md",
            "manuscrito/a>b.md",
            "manuscrito/a|b.md",
            "manuscrito/ archivo.md",
            "manuscrito/archivo.",
            "manuscrito/archivo ",
            "manuscrito/archivo\u{1}.md",
        ] {
            assert!(
                validar_ruta_relativa(ruta).is_err(),
                "debió rechazar {ruta:?}"
            );
        }
        for ruta in ["manuscrito/01/x.md", "notas/personajes/marta.md"] {
            assert!(
                validar_ruta_documento(ruta).is_ok(),
                "debió aceptar {ruta:?}"
            );
        }
        assert!(validar_ruta_documento("proyecto.yml").is_err());
        assert!(es_extension(Path::new("manuscrito/001.MD"), "md"));
        assert!(!es_extension(Path::new("manuscrito/001.md.bak"), "md"));
    }
}
