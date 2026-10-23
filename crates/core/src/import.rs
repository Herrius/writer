//! Importación local y backup.
//!
//! Implementa `docs/importacion-v1.md`: Markdown y DOCX con límites, y backup
//! ZIP del proyecto sin tocar el original. Todo se ejecuta sin red.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::project::Project;

/// Tamaño máximo de un archivo de importación (20 MB).
pub const MAX_ARCHIVO: u64 = 20 * 1024 * 1024;
/// Tamaño máximo total de una importación (200 MB).
pub const MAX_TOTAL: u64 = 200 * 1024 * 1024;
/// Número máximo de entradas de un ZIP de backup.
pub const MAX_ENTRADAS: usize = 100_000;
/// Tamaño máximo descomprimido de un backup (1 GB).
pub const MAX_DESCOMPRIMIDO: u64 = 1024 * 1024 * 1024;

/// Importa un Markdown como escena o nota nueva.
///
/// Devuelve el ID y la ruta del documento creado.
pub fn importar_markdown(
    proyecto: &Project,
    archivo: &Path,
    titulo: &str,
    tipo_nota: Option<&str>,
) -> Result<(String, String)> {
    let bytes = leer_limitado(archivo)?;
    let texto = String::from_utf8(bytes)
        .map_err(|error| Error::Format(format!("Markdown no es UTF-8: {error}")))?;
    let cuerpo = sin_frontmatter(&texto);
    let (id, ruta) = match tipo_nota {
        Some(tipo) => proyecto.crear_nota(tipo, titulo)?,
        None => proyecto.crear_escena(titulo, None, None, None)?,
    };
    proyecto.escribir_cuerpo(&id, cuerpo)?;
    Ok((id, ruta))
}

/// Importa un DOCX básico como escena o nota nueva.
///
/// Solo extrae texto de `word/document.xml`; imágenes, tablas y macros se
/// ignoran con aviso. El archivo nunca se ejecuta.
pub fn importar_docx(
    proyecto: &Project,
    archivo: &Path,
    titulo: &str,
    tipo_nota: Option<&str>,
) -> Result<(String, String)> {
    let bytes = leer_limitado(archivo)?;
    let cursor = std::io::Cursor::new(bytes);
    let mut zip = zip::ZipArchive::new(cursor)
        .map_err(|error| Error::Format(format!("DOCX no es un ZIP válido: {error}")))?;
    let mut total = 0_u64;
    let mut documento = None;
    for indice in 0..zip.len() {
        let mut entrada = zip
            .by_index(indice)
            .map_err(|error| Error::Format(format!("DOCX dañado: {error}")))?;
        total = total.saturating_add(entrada.size());
        if total > MAX_TOTAL {
            return Err(Error::Format(
                "DOCX supera el tamaño descomprimido permitido".to_string(),
            ));
        }
        if entrada
            .name()
            .map_err(|error| Error::Format(error.to_string()))?
            == "word/document.xml"
        {
            let mut xml = String::new();
            entrada.read_to_string(&mut xml).map_err(Error::Io)?;
            documento = Some(xml);
        }
    }
    let xml = documento.ok_or_else(|| Error::Format("DOCX sin word/document.xml".to_string()))?;
    let cuerpo = texto_docx(&xml)?;
    let (id, ruta) = match tipo_nota {
        Some(tipo) => proyecto.crear_nota(tipo, titulo)?,
        None => proyecto.crear_escena(titulo, None, None, None)?,
    };
    proyecto.escribir_cuerpo(&id, &cuerpo)?;
    Ok((id, ruta))
}

/// Importa un HTML básico como escena o nota nueva.
pub fn importar_html(
    proyecto: &Project,
    archivo: &Path,
    titulo: &str,
    tipo_nota: Option<&str>,
) -> Result<(String, String)> {
    let bytes = leer_limitado(archivo)?;
    let texto = String::from_utf8(bytes)
        .map_err(|error| Error::Format(format!("HTML no es UTF-8: {error}")))?;
    let cuerpo = texto_html(&texto);
    let (id, ruta) = match tipo_nota {
        Some(tipo) => proyecto.crear_nota(tipo, titulo)?,
        None => proyecto.crear_escena(titulo, None, None, None)?,
    };
    proyecto.escribir_cuerpo(&id, &cuerpo)?;
    Ok((id, ruta))
}

/// Importa un RTF básico como escena o nota nueva.
pub fn importar_rtf(
    proyecto: &Project,
    archivo: &Path,
    titulo: &str,
    tipo_nota: Option<&str>,
) -> Result<(String, String)> {
    let bytes = leer_limitado(archivo)?;
    let texto = String::from_utf8_lossy(&bytes).to_string();
    let cuerpo = texto_rtf(&texto);
    let (id, ruta) = match tipo_nota {
        Some(tipo) => proyecto.crear_nota(tipo, titulo)?,
        None => proyecto.crear_escena(titulo, None, None, None)?,
    };
    proyecto.escribir_cuerpo(&id, &cuerpo)?;
    Ok((id, ruta))
}

/// Crea un backup ZIP del proyecto.
pub fn crear_backup(raiz: &Path, destino: &Path) -> Result<PathBuf> {
    let raiz = std::fs::canonicalize(raiz).map_err(|error| Error::Io(error).with_path(raiz))?;
    if destino.starts_with(&raiz) {
        return Err(Error::Input(
            "el backup no puede escribirse dentro del proyecto".to_string(),
        ));
    }
    if let Some(padre) = destino.parent() {
        std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    }
    let temporal = destino.with_extension("zip.tmp");
    let archivo =
        std::fs::File::create(&temporal).map_err(|error| Error::Io(error).with_path(&temporal))?;
    let mut zip = zip::ZipWriter::new(archivo);
    let opciones = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut pendientes = vec![raiz.clone()];
    while let Some(directorio) = pendientes.pop() {
        for entrada in std::fs::read_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?
        {
            let entrada = entrada.map_err(|error| Error::Io(error).with_path(&directorio))?;
            let ruta = entrada.path();
            let relativa = ruta
                .strip_prefix(&raiz)
                .map_err(|error| Error::Other(error.to_string()))?;
            let texto = relativa.to_string_lossy().replace('\\', "/");
            if texto == "bloqueo"
                || texto == "indice.sqlite"
                || texto == "indice.sqlite-wal"
                || texto == "indice.sqlite-shm"
                || texto.starts_with("cuarentena/")
            {
                continue;
            }
            let metadatos = std::fs::symlink_metadata(&ruta)
                .map_err(|error| Error::Io(error).with_path(&ruta))?;
            if metadatos.file_type().is_symlink() {
                return Err(Error::Integrity(format!(
                    "se rechaza un enlace simbólico en el backup: {}",
                    ruta.display()
                )));
            }
            if metadatos.is_dir() {
                pendientes.push(ruta);
            } else if metadatos.is_file() {
                zip.start_file(texto, opciones)
                    .map_err(|error| Error::Format(error.to_string()))?;
                let bytes =
                    std::fs::read(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
                zip.write_all(&bytes)
                    .map_err(|error| Error::Io(error).with_path(&ruta))?;
            }
        }
    }
    zip.finish()
        .map_err(|error| Error::Format(error.to_string()))?;
    std::fs::rename(&temporal, destino).map_err(|error| Error::Io(error).with_path(destino))?;
    Ok(destino.to_path_buf())
}

/// Restaura un backup ZIP en un destino nuevo.
pub fn restaurar_backup(archivo: &Path, destino: &Path) -> Result<PathBuf> {
    if destino.exists() {
        let vacio = std::fs::read_dir(destino)
            .map(|mut entradas| entradas.next().is_none())
            .unwrap_or(false);
        if !vacio {
            return Err(Error::Conflict(format!(
                "el destino no está vacío: {}",
                destino.display()
            )));
        }
    }
    std::fs::create_dir_all(destino).map_err(|error| Error::Io(error).with_path(destino))?;
    let archivo_abierto =
        std::fs::File::open(archivo).map_err(|error| Error::Io(error).with_path(archivo))?;
    let mut zip = zip::ZipArchive::new(archivo_abierto)
        .map_err(|error| Error::Format(format!("backup no es un ZIP válido: {error}")))?;
    if zip.len() > MAX_ENTRADAS {
        return Err(Error::Format("backup con demasiadas entradas".to_string()));
    }
    let mut total = 0_u64;
    for indice in 0..zip.len() {
        let mut entrada = zip
            .by_index(indice)
            .map_err(|error| Error::Format(format!("backup dañado: {error}")))?;
        let nombre = entrada
            .name()
            .map_err(|error| Error::Format(error.to_string()))?
            .to_string();
        crate::fsutil::validar_ruta_relativa(&nombre)?;
        if nombre.starts_with("cuarentena/") || nombre == "bloqueo" {
            continue;
        }
        total = total.saturating_add(entrada.size());
        if total > MAX_DESCOMPRIMIDO {
            return Err(Error::Format(
                "backup supera el tamaño descomprimido permitido".to_string(),
            ));
        }
        let salida = destino.join(&nombre);
        if let Some(padre) = salida.parent() {
            std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
        }
        let mut archivo_salida =
            std::fs::File::create(&salida).map_err(|error| Error::Io(error).with_path(&salida))?;
        std::io::copy(&mut entrada, &mut archivo_salida)
            .map_err(|error| Error::Io(error).with_path(&salida))?;
    }
    Project::new(destino).verificar()?;
    Ok(destino.to_path_buf())
}

fn leer_limitado(archivo: &Path) -> Result<Vec<u8>> {
    let metadatos =
        std::fs::metadata(archivo).map_err(|error| Error::Io(error).with_path(archivo))?;
    if metadatos.len() > MAX_ARCHIVO {
        return Err(Error::Format(format!(
            "el archivo supera el límite de {} MB",
            MAX_ARCHIVO / (1024 * 1024)
        )));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(archivo)
        .map_err(|error| Error::Io(error).with_path(archivo))?
        .take(MAX_ARCHIVO + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| Error::Io(error).with_path(archivo))?;
    if bytes.len() as u64 > MAX_ARCHIVO {
        return Err(Error::Format("el archivo supera el límite".to_string()));
    }
    Ok(bytes)
}

fn sin_frontmatter(texto: &str) -> &str {
    if let Some(resto) = texto.strip_prefix("---\n") {
        if let Some(fin) = resto.find("\n---\n") {
            return texto[3 + fin + 5..].trim_start_matches('\n');
        }
    }
    texto
}

fn texto_docx(xml: &str) -> Result<String> {
    let mut salida = String::new();
    let mut resto = xml;
    while let Some(inicio) = resto.find("<w:t") {
        let tras = &resto[inicio..];
        let Some(cierre_inicio) = tras.find('>') else {
            break;
        };
        let contenido = &tras[cierre_inicio + 1..];
        let Some(fin) = contenido.find("</w:t>") else {
            break;
        };
        salida.push_str(&desescapar_xml(&contenido[..fin]));
        resto = &contenido[fin + 6..];
        if resto.contains("<w:p") {
            salida.push('\n');
        }
    }
    let limpio = salida
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{}\n", limpio.trim()))
}

fn desescapar_xml(texto: &str) -> String {
    texto
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn texto_html(html: &str) -> String {
    let mut sin_scripts = String::with_capacity(html.len());
    let mut resto = html;
    while let Some(inicio) = resto.find("<script") {
        sin_scripts.push_str(&resto[..inicio]);
        let tras = &resto[inicio..];
        let Some(fin) = tras.find("</script>") else {
            resto = "";
            break;
        };
        resto = &tras[fin + 9..];
    }
    sin_scripts.push_str(resto);
    let mut sin_estilos = String::with_capacity(sin_scripts.len());
    let mut resto = sin_scripts.as_str();
    while let Some(inicio) = resto.find("<style") {
        sin_estilos.push_str(&resto[..inicio]);
        let tras = &resto[inicio..];
        let Some(fin) = tras.find("</style>") else {
            resto = "";
            break;
        };
        resto = &tras[fin + 8..];
    }
    sin_estilos.push_str(resto);

    let mut salida = String::with_capacity(sin_estilos.len());
    let mut dentro = false;
    for caracter in sin_estilos.chars() {
        match caracter {
            '<' => {
                dentro = true;
                salida.push(' ');
            }
            '>' => {
                dentro = false;
                salida.push(' ');
            }
            _ if !dentro => salida.push(caracter),
            _ => {}
        }
    }
    let texto = desescapar_xml(&salida)
        .replace('\u{a0}', " ")
        .lines()
        .map(str::trim)
        .filter(|linea| !linea.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n", texto)
}

fn texto_rtf(rtf: &str) -> String {
    let mut salida = String::with_capacity(rtf.len());
    let mut profundidad = 0_i32;
    let mut iter = rtf.chars().peekable();
    while let Some(caracter) = iter.next() {
        match caracter {
            '{' => profundidad += 1,
            '}' => profundidad -= 1,
            '\\' => {
                let mut control = String::new();
                while let Some(siguiente) = iter.peek().copied() {
                    if siguiente.is_ascii_alphabetic() {
                        control.push(siguiente);
                        iter.next();
                    } else {
                        break;
                    }
                }
                match control.as_str() {
                    "par" | "line" => {
                        if profundidad > 0 {
                            salida.push('\n');
                        }
                    }
                    "tab" => salida.push('\t'),
                    "emdash" => salida.push('—'),
                    "endash" => salida.push('–'),
                    "lquote" => salida.push('‘'),
                    "rquote" => salida.push('’'),
                    "ldquo" => salida.push('“'),
                    "rdquo" => salida.push('”'),
                    _ => {}
                }
            }
            _ if profundidad > 0 => salida.push(caracter),
            _ => {}
        }
    }
    let limpio = salida
        .lines()
        .map(str::trim)
        .filter(|linea| !linea.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    format!("{}\n", limpio)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proyecto() -> (tempfile::TempDir, Project) {
        let temporal = tempfile::tempdir().expect("temporal");
        let proyecto = Project::new(temporal.path().join("novela"));
        proyecto.init("Novela", None, None).expect("init");
        (temporal, proyecto)
    }

    #[test]
    fn importa_markdown_y_quita_frontmatter() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("entrada.md");
        std::fs::write(&archivo, "---\ntitulo: X\n---\n\nTexto importado.\n").expect("escribir");
        let (id, _) = importar_markdown(&proyecto, &archivo, "Importada", None).expect("importar");
        assert_eq!(
            proyecto.leer(&id).expect("leer").cuerpo(),
            "Texto importado.\n"
        );
    }

    #[test]
    fn rechaza_archivo_demasiado_grande() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("grande.md");
        let fichero = std::fs::File::create(&archivo).expect("crear");
        fichero.set_len(MAX_ARCHIVO + 1).expect("tamaño");
        assert!(importar_markdown(&proyecto, &archivo, "Grande", None).is_err());
    }

    #[test]
    fn backup_y_restauracion() {
        let (temporal, proyecto) = proyecto();
        let (id, _) = proyecto.crear_nota("ideas", "Idea").expect("nota");
        proyecto
            .escribir_cuerpo(&id, "contenido")
            .expect("escribir");
        let archivo = temporal.path().join("backup.zip");
        crear_backup(proyecto.raiz(), &archivo).expect("backup");
        let destino = temporal.path().join("restaurado");
        restaurar_backup(&archivo, &destino).expect("restaurar");
        let escaneo = Project::new(&destino).verificar().expect("verificar");
        assert!(escaneo.errores.is_empty(), "errores: {:?}", escaneo.errores);
    }

    #[test]
    fn importa_html_sin_scripts() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("entrada.html");
        std::fs::write(
            &archivo,
            "<html><body><script>alert(1)</script><p>Hola <b>mundo</b></p></body></html>",
        )
        .expect("escribir");
        let (id, _) = importar_html(&proyecto, &archivo, "HTML", None).expect("importar");
        let documento = proyecto.leer(&id).expect("leer");
        let cuerpo = documento.cuerpo();
        assert!(cuerpo.contains("Hola"));
        assert!(!cuerpo.contains("alert"));
    }

    #[test]
    fn importa_rtf_basico() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("entrada.rtf");
        std::fs::write(&archivo, "{\\rtf1\\ansi Hola\\par mundo}").expect("escribir");
        let (id, _) = importar_rtf(&proyecto, &archivo, "RTF", None).expect("importar");
        let documento = proyecto.leer(&id).expect("leer");
        let cuerpo = documento.cuerpo();
        assert!(cuerpo.contains("Hola"));
        assert!(cuerpo.contains("mundo"));
    }

    #[test]
    fn restauracion_rechaza_destino_ocupado() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("backup.zip");
        crear_backup(proyecto.raiz(), &archivo).expect("backup");
        let destino = temporal.path().join("ocupado");
        std::fs::create_dir(&destino).expect("carpeta");
        std::fs::write(destino.join("x"), b"x").expect("archivo");
        assert!(restaurar_backup(&archivo, &destino).is_err());
    }
}
