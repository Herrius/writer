//! Exportación PDF local con fuente estándar y formato de manuscrito básico.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::fsutil;
use crate::verify::Escaneo;

/// Formato de manuscrito para PDF.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Formato {
    /// Tamaño de fuente del cuerpo.
    pub fuente: f32,
    /// Interlineado en puntos.
    pub interlineado: f32,
    /// Margen en puntos.
    pub margen: f32,
}

impl Default for Formato {
    fn default() -> Self {
        Self {
            fuente: 12.0,
            interlineado: 18.0,
            margen: 72.0,
        }
    }
}

impl Formato {
    /// Valida el formato antes de usarlo.
    pub fn validar(&self) -> Result<()> {
        if !(6.0..=36.0).contains(&self.fuente) {
            return Err(Error::Input(
                "la fuente del PDF debe estar entre 6 y 36 pt".to_string(),
            ));
        }
        if !(self.fuente..=72.0).contains(&self.interlineado) {
            return Err(Error::Input(
                "el interlineado del PDF debe estar entre la fuente y 72 pt".to_string(),
            ));
        }
        if !(36.0..=144.0).contains(&self.margen) {
            return Err(Error::Input(
                "el margen del PDF debe estar entre 36 y 144 pt".to_string(),
            ));
        }
        Ok(())
    }
}

/// Exporta el manuscrito visible a PDF.
pub fn exportar(escaneo: &Escaneo, destino: &Path, forzar: bool) -> Result<PathBuf> {
    let destino = preparar_destino(&escaneo.raiz, destino, forzar)?;
    let mut escenas: Vec<_> = escaneo.escenas.iter().collect();
    escenas.sort_by(|a, b| comparar(a.relativa.as_str(), b.relativa.as_str()));
    let formato = leer_formato(&escaneo.raiz)?;
    let lineas = maquetar(escaneo, &escenas, formato);
    let bytes = generar_pdf(&lineas, formato);
    fsutil::escribir_atomico(&destino, &bytes)?;
    Ok(destino)
}

/// Lee el formato guardado o el valor por defecto.
pub fn leer_formato(raiz: &Path) -> Result<Formato> {
    let ruta = raiz.join(".escritor/formato.json");
    let Some(bytes) = fsutil::leer_opcional(&ruta)? else {
        return Ok(Formato::default());
    };
    let formato: Formato =
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(error).with_path(&ruta))?;
    formato.validar()?;
    Ok(formato)
}

/// Guarda el formato de manuscrito con reemplazo atómico.
pub fn guardar_formato(raiz: &Path, formato: &Formato) -> Result<()> {
    formato.validar()?;
    let escritor = fsutil::validar_escritor(raiz)?
        .ok_or_else(|| Error::Project("el proyecto no tiene .escritor".to_string()))?;
    let ruta = escritor.join("formato.json");
    let bytes = serde_json::to_vec_pretty(formato)?;
    fsutil::escribir_atomico(&ruta, &bytes)
}

fn comparar(a: &str, b: &str) -> std::cmp::Ordering {
    a.split('/').cmp(b.split('/'))
}

fn maquetar(
    escaneo: &Escaneo,
    escenas: &[&crate::verify::EscenaEncontrada],
    formato: Formato,
) -> Vec<String> {
    let mut lineas = Vec::new();
    if let Some(proyecto) = &escaneo.proyecto {
        lineas.push(proyecto.titulo.clone());
        if let Some(autor) = &proyecto.autor {
            if !autor.trim().is_empty() {
                lineas.push(autor.clone());
            }
        }
        lineas.push(String::new());
    }
    let ancho = ((595.0 - formato.margen * 2.0) / (formato.fuente * 0.52)) as usize;
    for escena in escenas {
        lineas.push(escena.escena.titulo.clone());
        lineas.push(String::new());
        let texto = crate::docx::quitar_marcas(&escena.escena.cuerpo);
        for parrafo in texto.split('\n') {
            let limpio = parrafo.trim();
            if limpio.is_empty() {
                lineas.push(String::new());
                continue;
            }
            let mut actual = String::new();
            for palabra in limpio.split_whitespace() {
                if !actual.is_empty()
                    && actual.chars().count() + palabra.chars().count() + 1 > ancho
                {
                    lineas.push(std::mem::take(&mut actual));
                }
                if !actual.is_empty() {
                    actual.push(' ');
                }
                actual.push_str(palabra);
            }
            if !actual.is_empty() {
                lineas.push(actual);
            }
        }
        lineas.push(String::new());
    }
    lineas
}

fn generar_pdf(lineas: &[String], formato: Formato) -> Vec<u8> {
    let alto_pagina = 842.0;
    let lineas_por_pagina = ((alto_pagina - formato.margen * 2.0) / formato.interlineado)
        .floor()
        .max(1.0) as usize;
    let paginas: Vec<&[String]> = lineas.chunks(lineas_por_pagina).collect();
    let total = paginas.len().max(1);
    let mut objetos: Vec<String> = Vec::new();
    objetos.push("<< /Type /Catalog /Pages 2 0 R >>".to_string());
    let kids: Vec<String> = (0..total)
        .map(|indice| format!("{} 0 R", 5 + indice * 2))
        .collect();
    objetos.push(format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        kids.join(" "),
        total
    ));
    objetos.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_string(),
    );
    objetos.push(
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>"
            .to_string(),
    );
    for (indice, pagina) in paginas.iter().enumerate() {
        let contenido_id = 6 + indice * 2;
        objetos.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {contenido_id} 0 R >>"
        ));
        let mut flujo = String::new();
        flujo.push_str("BT\n");
        let _ = writeln!(flujo, "/F1 {} Tf", formato.fuente);
        let _ = writeln!(flujo, "{} TL", formato.interlineado);
        let _ = writeln!(
            flujo,
            "{} {} Td",
            formato.margen,
            alto_pagina - formato.margen
        );
        for linea in *pagina {
            let _ = writeln!(flujo, "({}) Tj", escapar_pdf(linea));
            flujo.push_str("T*\n");
        }
        flujo.push_str("ET\n");
        let _ = writeln!(
            flujo,
            "BT /F1 9 Tf 500 30 Td ({} / {}) Tj ET",
            indice + 1,
            total
        );
        objetos.push(format!(
            "<< /Length {} >>\nstream\n{}endstream",
            flujo.len(),
            flujo
        ));
    }
    let mut salida = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (indice, objeto) in objetos.iter().enumerate() {
        offsets.push(salida.len());
        let _ = writeln!(salida, "{} 0 obj", indice + 1);
        let _ = writeln!(salida, "{objeto}");
        salida.push_str("endobj\n");
    }
    let xref = salida.len();
    let _ = writeln!(salida, "xref\n0 {}", objetos.len() + 1);
    salida.push_str("0000000000 65535 f \n");
    for offset in offsets {
        let _ = writeln!(salida, "{offset:010} 00000 n ");
    }
    let _ = writeln!(
        salida,
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF",
        objetos.len() + 1
    );
    salida.into_bytes()
}

fn escapar_pdf(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len());
    for caracter in texto.chars() {
        match caracter {
            '(' => salida.push_str("\\("),
            ')' => salida.push_str("\\)"),
            '\\' => salida.push_str("\\\\"),
            '\n' | '\r' => salida.push(' '),
            caracter if caracter.is_ascii() => salida.push(caracter),
            caracter => {
                let codigo = caracter as u32;
                if codigo <= 255 {
                    let _ = write!(salida, "\\{codigo:03o}");
                } else {
                    salida.push('?');
                }
            }
        }
    }
    salida
}

fn preparar_destino(raiz: &Path, destino: &Path, forzar: bool) -> Result<PathBuf> {
    let raiz = std::fs::canonicalize(raiz).map_err(|error| Error::Io(error).with_path(raiz))?;
    let padre = destino
        .parent()
        .filter(|padre| !padre.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let padre = std::fs::canonicalize(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    let nombre = destino
        .file_name()
        .ok_or_else(|| Error::Input("el destino PDF no tiene nombre".to_string()))?;
    let salida = padre.join(nombre);
    if salida.starts_with(&raiz) {
        return Err(Error::Input(
            "el PDF no puede escribirse dentro del proyecto".to_string(),
        ));
    }
    if salida.exists() && !forzar {
        return Err(Error::Conflict(format!(
            "el destino ya existe: {}",
            salida.display()
        )));
    }
    Ok(salida)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_minimo_tiene_cabecera_y_xref() {
        let bytes = generar_pdf(&["Hola".to_string()], Formato::default());
        let texto = String::from_utf8_lossy(&bytes);
        assert!(texto.starts_with("%PDF-1.4"));
        assert!(texto.contains("startxref"));
        assert!(texto.contains("(Hola) Tj"));
        assert!(texto.trim_end().ends_with("%%EOF"));
    }

    #[test]
    fn escapa_parentesis_y_acentos() {
        assert_eq!(escapar_pdf("a(b)c"), "a\\(b\\)c");
        assert_eq!(escapar_pdf("café"), "caf\\351");
    }
}
