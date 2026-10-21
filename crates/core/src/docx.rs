//! Exportación básica a DOCX.
//!
//! El Markdown se interpreta con `pulldown-cmark` y se traduce a un subconjunto
//! de WordprocessingML: títulos, párrafos, énfasis, listas, citas, código,
//! enlaces, separadores y notas opcionales. No intenta reproducir tablas, HTML
//! ni numeración nativa de Word.

use std::io::{Cursor, Write};

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::error::{Error, Result};
use crate::model;
use crate::verify::{Documento, Escaneo};

/// Genera el DOCX en memoria.
pub(crate) fn exportar(escaneo: &Escaneo, incluir_notas: bool) -> Result<Vec<u8>> {
    let mut render = Render::nuevo();
    if let Some(proyecto) = &escaneo.proyecto {
        render.portada(&proyecto.titulo, proyecto.autor.as_deref());
    }

    let documentos = escaneo.ordenados();
    let mut libro_actual = String::new();
    let mut capitulo_actual = String::new();
    for documento in &documentos {
        let Documento::Escena(encontrada) = documento else {
            continue;
        };
        let partes: Vec<&str> = encontrada.relativa.split('/').collect();
        if partes.len() != 4 {
            continue;
        }
        let libro = model::titulo_humano(partes[1]);
        let capitulo = model::titulo_humano(partes[2]);
        if libro != libro_actual {
            render.titulo(&libro, 1);
            libro_actual = libro;
            capitulo_actual.clear();
        }
        if capitulo != capitulo_actual {
            render.titulo(&capitulo, 2);
            capitulo_actual = capitulo;
        }
        render.titulo(&encontrada.escena.titulo, 3);
        render.markdown(&encontrada.escena.cuerpo);
    }

    if incluir_notas {
        let notas: Vec<_> = documentos
            .iter()
            .filter_map(|documento| match documento {
                Documento::Nota(nota) => Some(nota),
                Documento::Escena(_) => None,
            })
            .collect();
        if !notas.is_empty() {
            render.salto_pagina();
            render.titulo("Notas", 1);
            let mut tipo_actual = String::new();
            for nota in notas {
                if nota.nota.tipo != tipo_actual {
                    render.titulo(&model::titulo_humano(&nota.nota.tipo), 2);
                    tipo_actual = nota.nota.tipo.clone();
                }
                render.titulo(&nota.nota.titulo, 3);
                render.markdown(&nota.nota.cuerpo);
            }
        }
    }

    render.terminar()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Estilo {
    Normal,
    Titulo(u8),
}

#[derive(Clone, Debug)]
struct Fragmento {
    texto: String,
    negrita: bool,
    cursiva: bool,
    tachado: bool,
    codigo: bool,
    enlace: Option<String>,
}

#[derive(Clone, Debug)]
struct Lista {
    ordenada: bool,
    siguiente: u64,
}

#[derive(Debug)]
struct Render {
    cuerpo: String,
    titulo: String,
    autor: Option<String>,
    relaciones: Vec<String>,
    runs: Vec<Fragmento>,
    estilo: Estilo,
    prefijo: Option<String>,
    nivel_lista: usize,
    cita: usize,
    negrita: usize,
    cursiva: usize,
    tachado: usize,
    enlace: Option<String>,
    codigo_bloque: Option<String>,
    listas: Vec<Lista>,
    imagen_alt: Option<String>,
}

impl Render {
    fn nuevo() -> Self {
        Self {
            cuerpo: String::new(),
            titulo: String::new(),
            autor: None,
            relaciones: Vec::new(),
            runs: Vec::new(),
            estilo: Estilo::Normal,
            prefijo: None,
            nivel_lista: 0,
            cita: 0,
            negrita: 0,
            cursiva: 0,
            tachado: 0,
            enlace: None,
            codigo_bloque: None,
            listas: Vec::new(),
            imagen_alt: None,
        }
    }

    fn portada(&mut self, titulo: &str, autor: Option<&str>) {
        self.titulo = titulo.to_string();
        self.autor = autor.map(str::to_string);
        let run = Self::run_xml(titulo, true, false, false, false, false, Some(48), None);
        self.cuerpo.push_str(&format!(
            r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="240"/></w:pPr>{run}</w:p>"#
        ));
        if let Some(autor) = autor {
            let run = Self::run_xml(autor, false, true, false, false, false, Some(26), None);
            self.cuerpo.push_str(&format!(
                r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr>{run}</w:p>"#
            ));
        }
        self.cuerpo.push_str("<w:p/>");
    }

    fn titulo(&mut self, texto: &str, nivel: u8) {
        self.flush();
        self.estilo = Estilo::Titulo(nivel);
        self.push_texto(texto);
        self.flush();
        self.estilo = Estilo::Normal;
    }

    fn salto_pagina(&mut self) {
        self.cuerpo
            .push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
    }

    fn markdown(&mut self, markdown: &str) {
        let mut opciones = Options::empty();
        opciones.insert(Options::ENABLE_STRIKETHROUGH);
        opciones.insert(Options::ENABLE_TASKLISTS);
        let parser = Parser::new_ext(markdown, opciones);
        for evento in parser {
            self.evento(evento);
        }
        self.flush();
    }

    fn evento(&mut self, evento: Event<'_>) {
        match evento {
            Event::Start(Tag::Paragraph) => {
                if !self.runs.is_empty() {
                    self.flush();
                }
            }
            Event::End(TagEnd::Paragraph) => self.flush(),
            Event::Start(Tag::Heading { level, .. }) => {
                self.flush();
                self.estilo = Estilo::Titulo(level as u8);
            }
            Event::End(TagEnd::Heading(_)) => {
                self.flush();
                self.estilo = Estilo::Normal;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                self.flush();
                self.cita += 1;
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                self.flush();
                self.cita = self.cita.saturating_sub(1);
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
            | Event::Start(Tag::CodeBlock(CodeBlockKind::Indented)) => {
                self.flush();
                self.codigo_bloque = Some(String::new());
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(codigo) = self.codigo_bloque.take() {
                    for linea in codigo.lines() {
                        let run =
                            Self::run_xml(linea, false, false, false, true, false, Some(20), None);
                        self.cuerpo.push_str(&format!(
                            r#"<w:p><w:pPr><w:spacing w:after="0"/><w:ind w:left="360"/></w:pPr>{run}</w:p>"#
                        ));
                    }
                }
            }
            Event::Start(Tag::List(inicio)) => {
                self.flush();
                self.listas.push(Lista {
                    ordenada: inicio.is_some(),
                    siguiente: inicio.unwrap_or(1),
                });
            }
            Event::End(TagEnd::List(_)) => {
                self.flush();
                self.listas.pop();
            }
            Event::Start(Tag::Item) => {
                self.flush();
                self.nivel_lista = self.listas.len().saturating_sub(1);
                if let Some(lista) = self.listas.last_mut() {
                    if lista.ordenada {
                        self.prefijo = Some(format!("{}. ", lista.siguiente));
                        lista.siguiente += 1;
                    } else {
                        self.prefijo = Some("• ".to_string());
                    }
                }
            }
            Event::End(TagEnd::Item) => {
                self.flush();
                self.prefijo = None;
                self.nivel_lista = 0;
            }
            Event::Start(Tag::Emphasis) => self.cursiva += 1,
            Event::End(TagEnd::Emphasis) => self.cursiva = self.cursiva.saturating_sub(1),
            Event::Start(Tag::Strong) => self.negrita += 1,
            Event::End(TagEnd::Strong) => self.negrita = self.negrita.saturating_sub(1),
            Event::Start(Tag::Strikethrough) => self.tachado += 1,
            Event::End(TagEnd::Strikethrough) => self.tachado = self.tachado.saturating_sub(1),
            Event::Start(Tag::Link { dest_url, .. }) => {
                self.enlace = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => self.enlace = None,
            Event::Start(Tag::Image { .. }) => {
                self.imagen_alt = Some(String::new());
            }
            Event::End(TagEnd::Image) => {
                if let Some(alt) = self.imagen_alt.take() {
                    if !alt.is_empty() {
                        self.push_texto(&format!("[imagen: {alt}]"));
                    }
                }
            }
            Event::Text(texto) => {
                if let Some(codigo) = self.codigo_bloque.as_mut() {
                    codigo.push_str(&texto);
                } else if let Some(alt) = self.imagen_alt.as_mut() {
                    alt.push_str(&texto);
                } else {
                    self.push_texto(&texto);
                }
            }
            Event::Code(texto) => {
                self.runs.push(Fragmento {
                    texto: texto.to_string(),
                    negrita: self.negrita > 0,
                    cursiva: self.cursiva > 0,
                    tachado: self.tachado > 0,
                    codigo: true,
                    enlace: self.enlace.clone(),
                });
            }
            Event::SoftBreak => self.push_texto(" "),
            Event::HardBreak => self.push_texto("\n"),
            Event::Rule => {
                self.flush();
                self.cuerpo.push_str(
                    r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:color w:val="888888"/></w:rPr><w:t>* * *</w:t></w:r></w:p>"#,
                );
            }
            Event::TaskListMarker(marcada) => {
                self.push_texto(if marcada { "[x] " } else { "[ ] " });
            }
            Event::FootnoteReference(etiqueta) => {
                self.push_texto(&format!("[^{etiqueta}]"));
            }
            Event::Html(_) | Event::InlineHtml(_) => {}
            Event::InlineMath(texto) | Event::DisplayMath(texto) => self.push_texto(&texto),
            Event::Start(_) | Event::End(_) => {}
        }
    }

    fn push_texto(&mut self, texto: &str) {
        if texto.is_empty() {
            return;
        }
        self.runs.push(Fragmento {
            texto: texto.to_string(),
            negrita: self.negrita > 0,
            cursiva: self.cursiva > 0,
            tachado: self.tachado > 0,
            codigo: false,
            enlace: self.enlace.clone(),
        });
    }

    fn flush(&mut self) {
        if self.runs.is_empty() {
            self.prefijo = None;
            return;
        }
        let estilo = self.estilo;
        let cita = self.cita > 0;
        let nivel_lista = self.nivel_lista;
        let prefijo = self.prefijo.take();
        let runs = std::mem::take(&mut self.runs);

        let mut ppr = String::new();
        match estilo {
            Estilo::Titulo(nivel) => {
                ppr.push_str(&format!(
                    r#"<w:spacing w:before="240" w:after="{}"/>"#,
                    if nivel <= 2 { 160 } else { 100 }
                ));
            }
            Estilo::Normal => ppr.push_str(r#"<w:spacing w:after="120"/>"#),
        }
        if cita {
            ppr.push_str(r#"<w:ind w:left="720"/>"#);
        } else if nivel_lista > 0 {
            ppr.push_str(&format!(r#"<w:ind w:left="{}"/>"#, 360 * nivel_lista));
        }

        let mut contenido = String::new();
        if let Some(prefijo) = prefijo {
            contenido.push_str(&Self::run_xml(
                &prefijo, false, false, false, false, false, None, None,
            ));
        }
        for fragmento in runs {
            let heading = matches!(estilo, Estilo::Titulo(_));
            let tamano = match estilo {
                Estilo::Titulo(nivel) => Some(match nivel {
                    1 => 36,
                    2 => 30,
                    3 => 26,
                    _ => 24,
                }),
                Estilo::Normal => Some(24),
            };
            let run = Self::run_xml(
                &fragmento.texto,
                fragmento.negrita || heading,
                fragmento.cursiva || cita,
                fragmento.tachado,
                fragmento.codigo,
                fragmento.enlace.is_some(),
                tamano,
                None,
            );
            if let Some(enlace) = fragmento.enlace {
                let id = self.relacion(&enlace);
                contenido.push_str(&format!(r#"<w:hyperlink r:id="{id}">{run}</w:hyperlink>"#));
            } else {
                contenido.push_str(&run);
            }
        }
        self.cuerpo
            .push_str(&format!("<w:p><w:pPr>{ppr}</w:pPr>{contenido}</w:p>"));
    }

    #[allow(clippy::too_many_arguments)]
    fn run_xml(
        texto: &str,
        negrita: bool,
        cursiva: bool,
        tachado: bool,
        codigo: bool,
        enlace: bool,
        tamano: Option<usize>,
        color: Option<&str>,
    ) -> String {
        let fuente = if codigo { "Consolas" } else { "Georgia" };
        let mut props = format!(r#"<w:rFonts w:ascii="{fuente}" w:hAnsi="{fuente}"/>"#);
        if negrita {
            props.push_str("<w:b/>");
        }
        if cursiva {
            props.push_str("<w:i/>");
        }
        if tachado {
            props.push_str("<w:strike/>");
        }
        if let Some(tamano) = tamano {
            props.push_str(&format!(
                r#"<w:sz w:val="{tamano}"/><w:szCs w:val="{tamano}"/>"#
            ));
        }
        if enlace {
            props.push_str(r#"<w:color w:val="1F4E79"/><w:u w:val="single"/>"#);
        }
        if let Some(color) = color {
            props.push_str(&format!(r#"<w:color w:val="{color}"/>"#));
        }

        let mut run = format!("<w:r><w:rPr>{props}</w:rPr>");
        for (indice, linea) in texto.split('\n').enumerate() {
            if indice > 0 {
                run.push_str("<w:br/>");
            }
            if !linea.is_empty() {
                run.push_str(&format!(
                    r#"<w:t xml:space="preserve">{}</w:t>"#,
                    escapar_xml(linea)
                ));
            }
        }
        run.push_str("</w:r>");
        run
    }

    fn relacion(&mut self, url: &str) -> String {
        if let Some(posicion) = self.relaciones.iter().position(|actual| actual == url) {
            return format!("rId{}", posicion + 1);
        }
        self.relaciones.push(url.to_string());
        format!("rId{}", self.relaciones.len())
    }

    fn terminar(self) -> Result<Vec<u8>> {
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/><Override PartName="/docProps/app.xml" ContentType="application/vnd.openxmlformats-officedocument.extended-properties+xml"/></Types>"#;
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties" Target="docProps/app.xml"/></Relationships>"#;
        let document = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>{}<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr></w:body></w:document>"#,
            self.cuerpo
        );
        let document_rels = if self.relaciones.is_empty() {
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#.to_string()
        } else {
            let enlaces: String = self
                .relaciones
                .iter()
                .enumerate()
                .map(|(indice, url)| {
                    format!(
                        r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{}" TargetMode="External"/>"#,
                        indice + 1,
                        escapar_xml(url)
                    )
                })
                .collect();
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{enlaces}</Relationships>"#
            )
        };
        let core = format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><dc:title>{}</dc:title><dc:creator>{}</dc:creator></cp:coreProperties>"#,
            escapar_xml(&self.titulo),
            escapar_xml(self.autor.as_deref().unwrap_or(""))
        );
        let app = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties" xmlns:vt="http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"><Application>writer</Application></Properties>"#;

        let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
        let opciones = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (nombre, contenido) in [
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", rels),
            ("word/document.xml", &document),
            ("word/_rels/document.xml.rels", &document_rels),
            ("docProps/core.xml", &core),
            ("docProps/app.xml", app),
        ] {
            zip.start_file(nombre, opciones)
                .map_err(|error| Error::Other(format!("DOCX: {error}")))?;
            zip.write_all(contenido.as_bytes())?;
        }
        let cursor = zip
            .finish()
            .map_err(|error| Error::Other(format!("DOCX: {error}")))?;
        Ok(cursor.into_inner())
    }
}

fn escapar_xml(texto: &str) -> String {
    texto
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
