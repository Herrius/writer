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
    if cuerpo.trim().is_empty() {
        return Err(Error::Format("el Markdown está vacío".to_string()));
    }
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
    let cuerpo = cuerpo_docx(archivo)?;
    if cuerpo.trim().is_empty() {
        return Err(Error::Format("el DOCX no contiene texto".to_string()));
    }
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
    if cuerpo.trim().is_empty() {
        return Err(Error::Format("el HTML no contiene texto".to_string()));
    }
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
    if cuerpo.trim().is_empty() {
        return Err(Error::Format("el RTF no contiene texto".to_string()));
    }
    let (id, ruta) = match tipo_nota {
        Some(tipo) => proyecto.crear_nota(tipo, titulo)?,
        None => proyecto.crear_escena(titulo, None, None, None)?,
    };
    proyecto.escribir_cuerpo(&id, &cuerpo)?;
    Ok((id, ruta))
}

/// Previsualiza un archivo de importación sin escribir en el proyecto.
///
/// Devuelve el formato detectado y el texto convertido (limitado para la UI).
pub fn previsualizar(archivo: &Path) -> Result<(String, String)> {
    let extension = archivo
        .extension()
        .and_then(|valor| valor.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let (formato, cuerpo) = match extension.as_str() {
        "docx" => ("DOCX", cuerpo_docx(archivo)?),
        "html" | "htm" => {
            let bytes = leer_limitado(archivo)?;
            let texto = String::from_utf8(bytes)
                .map_err(|error| Error::Format(format!("HTML no es UTF-8: {error}")))?;
            ("HTML", texto_html(&texto))
        }
        "rtf" => {
            let bytes = leer_limitado(archivo)?;
            ("RTF", texto_rtf(&String::from_utf8_lossy(&bytes)))
        }
        _ => {
            let bytes = leer_limitado(archivo)?;
            let texto = String::from_utf8(bytes)
                .map_err(|error| Error::Format(format!("Markdown no es UTF-8: {error}")))?;
            ("Markdown", sin_frontmatter(&texto).to_string())
        }
    };
    let recortado: String = cuerpo.chars().take(2000).collect();
    Ok((formato.to_string(), recortado))
}

fn cuerpo_docx(archivo: &Path) -> Result<String> {
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
    texto_docx(&xml)
}

/// Crea un backup ZIP del proyecto.
pub fn crear_backup(raiz: &Path, destino: &Path) -> Result<PathBuf> {
    let raiz = std::fs::canonicalize(raiz).map_err(|error| Error::Io(error).with_path(raiz))?;
    if !destino.is_absolute() {
        return Err(Error::Input(
            "el destino del backup debe ser una ruta absoluta".to_string(),
        ));
    }
    let destino = destino.to_path_buf();
    let padre = destino
        .parent()
        .ok_or_else(|| Error::Input("el destino del backup no tiene padre".to_string()))?;
    std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    let padre = std::fs::canonicalize(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    let destino = padre.join(
        destino
            .file_name()
            .ok_or_else(|| Error::Input("el destino del backup no tiene nombre".to_string()))?,
    );
    if destino.starts_with(&raiz) {
        return Err(Error::Input(
            "el backup no puede escribirse dentro del proyecto".to_string(),
        ));
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
            if es_excluido(&texto) {
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
    std::fs::rename(&temporal, &destino)
        .map_err(|error| Error::Io(error).with_path(destino.as_path()))?;
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
    let temporal = destino.with_extension("restaurando");
    if temporal.exists() {
        std::fs::remove_dir_all(&temporal)
            .map_err(|error| Error::Io(error).with_path(&temporal))?;
    }
    std::fs::create_dir_all(&temporal).map_err(|error| Error::Io(error).with_path(&temporal))?;
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
        if es_excluido(&nombre) {
            continue;
        }
        total = total.saturating_add(entrada.size());
        if total > MAX_DESCOMPRIMIDO {
            return Err(Error::Format(
                "backup supera el tamaño descomprimido permitido".to_string(),
            ));
        }
        let salida = temporal.join(&nombre);
        if let Some(padre) = salida.parent() {
            std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
        }
        let mut archivo_salida =
            std::fs::File::create(&salida).map_err(|error| Error::Io(error).with_path(&salida))?;
        std::io::copy(&mut entrada, &mut archivo_salida)
            .map_err(|error| Error::Io(error).with_path(&salida))?;
    }
    Project::new(&temporal).verificar()?;
    if destino.exists() {
        std::fs::remove_dir(destino).map_err(|error| Error::Io(error).with_path(destino))?;
    }
    std::fs::rename(&temporal, destino).map_err(|error| Error::Io(error).with_path(destino))?;
    Ok(destino.to_path_buf())
}

fn es_excluido(texto: &str) -> bool {
    matches!(
        texto,
        ".escritor/bloqueo"
            | ".escritor/indice.sqlite"
            | ".escritor/indice.sqlite-wal"
            | ".escritor/indice.sqlite-shm"
    ) || texto.starts_with(".escritor/cuarentena/")
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

pub(crate) fn sin_frontmatter(texto: &str) -> &str {
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
    while let Some(inicio) = resto.find('<') {
        resto = &resto[inicio..];
        if resto.starts_with("</w:p>") {
            salida.push('\n');
            resto = &resto[6..];
            continue;
        }
        // Solo el elemento de texto exacto `<w:t>` o `<w:t ...>`; así no se
        // confunden `<w:tbl>`, `<w:tr>`, `<w:tc>` ni `<w:tab/>`.
        if resto.starts_with("<w:t>") || resto.starts_with("<w:t ") {
            let Some(cierre) = resto.find('>') else {
                break;
            };
            let contenido = &resto[cierre + 1..];
            let Some(fin) = contenido.find("</w:t>") else {
                break;
            };
            salida.push_str(&desescapar_xml(&contenido[..fin]));
            resto = &contenido[fin + 6..];
            continue;
        }
        if let Some(fin) = resto.find('>') {
            resto = &resto[fin + 1..];
        } else {
            break;
        }
    }
    let limpio = salida
        .lines()
        .map(str::trim)
        .filter(|linea| !linea.is_empty() || !salida.is_empty())
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
    let sin_scripts = quitar_bloque(html, "script");
    let sin_estilos = quitar_bloque(&sin_scripts, "style");
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

fn quitar_bloque(html: &str, etiqueta: &str) -> String {
    let minusculas = html.to_ascii_lowercase();
    let apertura = format!("<{etiqueta}");
    let cierre = format!("</{etiqueta}>");
    let mut salida = String::with_capacity(html.len());
    let mut cursor = 0;
    while let Some(relativo) = minusculas[cursor..].find(&apertura) {
        let inicio = cursor + relativo;
        salida.push_str(&html[cursor..inicio]);
        let Some(rel_cierre) = minusculas[inicio..].find(&cierre) else {
            cursor = html.len();
            break;
        };
        cursor = inicio + rel_cierre + cierre.len();
    }
    salida.push_str(&html[cursor..]);
    salida
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
                let Some(siguiente) = iter.next() else { break };
                match siguiente {
                    '\\' | '{' | '}' => {
                        if profundidad > 0 {
                            salida.push(siguiente);
                        }
                    }
                    '\'' => {
                        let mut hex = String::new();
                        for _ in 0..2 {
                            if let Some(digito) = iter.next() {
                                hex.push(digito);
                            }
                        }
                        if profundidad > 0 {
                            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                                salida.push(byte as char);
                            }
                        }
                    }
                    'u' => {
                        let mut digitos = String::new();
                        while let Some(digito) = iter.peek().copied() {
                            if digito.is_ascii_digit() || digito == '-' {
                                digitos.push(digito);
                                iter.next();
                            } else {
                                break;
                            }
                        }
                        if let Some(reemplazo) = iter.peek().copied() {
                            if reemplazo == '?' {
                                iter.next();
                            }
                        }
                        if profundidad > 0 {
                            if let Ok(codigo) = digitos.parse::<i64>() {
                                if let Some(caracter) = char::from_u32(codigo.max(0) as u32) {
                                    salida.push(caracter);
                                }
                            }
                        }
                    }
                    _ if siguiente.is_ascii_alphabetic() => {
                        let mut control = String::from(siguiente);
                        while let Some(digito) = iter.peek().copied() {
                            if digito.is_ascii_alphabetic() {
                                control.push(digito);
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
                            "tab" => {
                                if profundidad > 0 {
                                    salida.push('\t');
                                }
                            }
                            "emdash" => salida.push('—'),
                            "endash" => salida.push('–'),
                            "lquote" => salida.push('‘'),
                            "rquote" => salida.push('’'),
                            "ldquo" => salida.push('“'),
                            "rdquo" => salida.push('”'),
                            _ => {}
                        }
                    }
                    _ => {
                        if profundidad > 0 {
                            salida.push(siguiente);
                        }
                    }
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
    fn docx_con_tabla_y_runs_no_inyecta_xml() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("tabla.docx");
        let cursor = std::io::Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(cursor);
        let opciones = zip::write::SimpleFileOptions::default();
        zip.start_file("word/document.xml", opciones)
            .expect("entrada");
        zip.write_all(b"<w:document><w:body><w:p><w:r><w:t>Hola</w:t></w:r><w:r><w:t> mundo</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Celda</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>").expect("xml");
        let cursor = zip.finish().expect("zip");
        std::fs::write(&archivo, cursor.into_inner()).expect("escribir");
        let (id, _) = importar_docx(&proyecto, &archivo, "DOCX", None).expect("importar");
        let documento = proyecto.leer(&id).expect("leer");
        let cuerpo = documento.cuerpo();
        assert!(cuerpo.contains("Hola mundo"), "cuerpo: {cuerpo}");
        assert!(cuerpo.contains("Celda"));
        assert!(!cuerpo.contains("<w:"));
    }

    #[test]
    fn rtf_con_acentos_y_unicode() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("acentos.rtf");
        std::fs::write(
            &archivo,
            "{\\rtf1\\ansi Caf\\'e9 y ni\\'f1o. \\u233?xito\\par}",
        )
        .expect("escribir");
        let (id, _) = importar_rtf(&proyecto, &archivo, "RTF", None).expect("importar");
        let documento = proyecto.leer(&id).expect("leer");
        let cuerpo = documento.cuerpo();
        assert!(cuerpo.contains("Café y niño"), "cuerpo: {cuerpo}");
        assert!(cuerpo.contains("éxito"), "cuerpo: {cuerpo}");
    }

    #[test]
    fn html_con_script_mayusculas() {
        let (temporal, proyecto) = proyecto();
        let archivo = temporal.path().join("mayusculas.html");
        std::fs::write(&archivo, "<SCRIPT>alert('x')</SCRIPT><p>Texto</p>").expect("escribir");
        let (id, _) = importar_html(&proyecto, &archivo, "HTML", None).expect("importar");
        let documento = proyecto.leer(&id).expect("leer");
        let cuerpo = documento.cuerpo();
        assert!(cuerpo.contains("Texto"));
        assert!(!cuerpo.contains("alert"));
    }

    #[test]
    fn backup_excluye_lock_indice_y_cuarentena() {
        let (temporal, proyecto) = proyecto();
        let escritor = proyecto.raiz().join(".escritor");
        std::fs::write(escritor.join("bloqueo"), b"x").expect("lock");
        std::fs::write(escritor.join("indice.sqlite"), b"x").expect("indice");
        std::fs::create_dir_all(escritor.join("cuarentena")).expect("cuarentena");
        std::fs::write(escritor.join("cuarentena/copia.md"), b"x").expect("copia");
        let archivo = temporal.path().join("backup.zip");
        crear_backup(proyecto.raiz(), &archivo).expect("backup");
        let mut zip =
            zip::ZipArchive::new(std::fs::File::open(&archivo).expect("abrir")).expect("zip");
        let nombres: Vec<String> = (0..zip.len())
            .map(|indice| {
                zip.by_index(indice)
                    .expect("entrada")
                    .name()
                    .expect("nombre")
                    .to_string()
            })
            .collect();
        assert!(!nombres.iter().any(|n| n == ".escritor/bloqueo"));
        assert!(!nombres.iter().any(|n| n == ".escritor/indice.sqlite"));
        assert!(!nombres
            .iter()
            .any(|n| n.starts_with(".escritor/cuarentena/")));
        assert!(nombres.iter().any(|n| n == "proyecto.yml"));
    }

    #[test]
    fn backup_relativo_dentro_del_proyecto_se_rechaza() {
        let (_temporal, proyecto) = proyecto();
        let resultado = crear_backup(proyecto.raiz(), Path::new("proyecto.yml"));
        assert!(resultado.is_err());
        assert!(std::fs::read(proyecto.raiz().join("proyecto.yml"))
            .expect("manifiesto")
            .starts_with(b"esquema:"));
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
