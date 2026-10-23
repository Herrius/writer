//! Modelo del proyecto: manifiesto, escenas y notas.
//!
//! El modelo conserva los campos desconocidos para no destruir datos al
//! reescribir. Los campos conocidos se emiten siempre en el orden del esquema.

use std::collections::BTreeMap;
use std::path::Path;

use unicode_normalization::UnicodeNormalization;

use crate::error::{Error, Result};
use crate::hash;
use crate::time;
use crate::ulid;
use crate::yaml::{self, Campos, Valor};

/// Versión del esquema en disco.
pub const ESQUEMA: i64 = 1;

/// Tipos de nota admitidos, que coinciden con sus carpetas.
pub const TIPOS_NOTA: [&str; 5] = ["tramas", "personajes", "lugares", "cronologia", "objetos"];

/// Estados de escena admitidos.
pub const ESTADOS_ESCENA: [&str; 3] = ["borrador", "revision", "final"];

/// Políticas de alcance admitidas.
pub const POLITICAS: [&str; 3] = ["auto", "proponer", "bloquear"];

/// Políticas de retención admitidas.
pub const RETENCIONES: [&str; 3] = ["completa", "diaria", "por_tamano"];

/// Resultado de leer un documento de texto con sus avisos de bytes.
#[derive(Clone, Debug)]
pub struct Lectura {
    /// Texto ya normalizado a LF y sin BOM.
    pub texto: String,
    /// Avisos de codificación o finales de línea.
    pub avisos: Vec<String>,
}

/// Manifiesto `proyecto.yml`.
#[derive(Clone, Debug)]
pub struct Proyecto {
    /// Versión del esquema; debe ser 1.
    pub esquema: i64,
    /// ULID estable del proyecto.
    pub id: String,
    /// Título no vacío.
    pub titulo: String,
    /// Autor opcional.
    pub autor: Option<String>,
    /// Código de idioma.
    pub idioma: Option<String>,
    /// Fecha de creación RFC 3339.
    pub creado: Option<String>,
    /// Política de retención.
    pub retencion: Option<String>,
    /// Si se guardan prompts de IA.
    pub guardar_prompts: Option<bool>,
    /// Política de estructura.
    pub politica_estructura: Option<String>,
    /// Política de metadatos.
    pub politica_metadatos: Option<String>,
    /// Política de notas.
    pub politica_notas: Option<String>,
    /// Política de prosa.
    pub politica_prosa: Option<String>,
    /// Política de exportación.
    pub politica_exportar: Option<String>,
    /// Campos desconocidos que se conservan.
    pub extra: Campos,
}

/// Escena de `manuscrito/libro/capitulo/`.
#[derive(Clone, Debug)]
pub struct Escena {
    /// ULID único.
    pub id: String,
    /// Título no vacío.
    pub titulo: String,
    /// Orden dentro de su carpeta, 1..99999.
    pub orden: i64,
    /// Estado; ausente significa borrador.
    pub estado: Option<String>,
    /// Personaje punto de vista.
    pub pov: Option<String>,
    /// Tramas referenciadas.
    pub tramas: Option<Vec<String>>,
    /// Personajes referenciados.
    pub personajes: Option<Vec<String>>,
    /// Fecha narrativa libre.
    pub fecha_narrativa: Option<String>,
    /// Etiquetas.
    pub etiquetas: Option<Vec<String>>,
    /// Campos desconocidos que se conservan.
    pub extra: Campos,
    /// Cuerpo Markdown.
    pub cuerpo: String,
}

/// Nota de `notas/<tipo>/`.
#[derive(Clone, Debug)]
pub struct Nota {
    /// ULID único.
    pub id: String,
    /// Tipo, que debe coincidir con la carpeta.
    pub tipo: String,
    /// Título no vacío.
    pub titulo: String,
    /// `true` si la IA no la sobrescribe.
    pub fija: Option<bool>,
    /// Etiquetas.
    pub etiquetas: Option<Vec<String>>,
    /// Referencias libres.
    pub refs: Option<Vec<String>>,
    /// Campos desconocidos que se conservan.
    pub extra: Campos,
    /// Cuerpo Markdown.
    pub cuerpo: String,
}

impl Proyecto {
    /// Crea un manifiesto nuevo con los valores por defecto del esquema.
    pub fn nuevo(titulo: &str, autor: Option<&str>, idioma: Option<&str>) -> Result<Self> {
        let titulo = titulo.trim();
        if titulo.is_empty() {
            return Err(Error::Input(
                "el título del proyecto no puede estar vacío".into(),
            ));
        }
        validar_texto(titulo, "titulo", "proyecto")?;
        if let Some(autor) = autor {
            validar_texto(autor, "autor", "proyecto")?;
        }
        if let Some(idioma) = idioma {
            validar_texto(idioma, "idioma", "proyecto")?;
        }
        Ok(Self {
            esquema: ESQUEMA,
            id: ulid::nuevo()?,
            titulo: titulo.to_string(),
            autor: Some(autor.unwrap_or("").to_string()),
            idioma: Some(idioma.unwrap_or("es").to_string()),
            creado: Some(time::ahora()),
            retencion: Some("completa".to_string()),
            guardar_prompts: Some(false),
            politica_estructura: Some("auto".to_string()),
            politica_metadatos: Some("auto".to_string()),
            politica_notas: Some("auto".to_string()),
            politica_prosa: Some("proponer".to_string()),
            politica_exportar: Some("proponer".to_string()),
            extra: Campos::new(),
        })
    }

    /// Parsea y valida un `proyecto.yml`.
    pub fn desde_campos(campos: &Campos, contexto: &str) -> Result<(Self, Vec<String>)> {
        let mut avisos = Vec::new();
        let esquema = match campos.get("esquema") {
            Some(Valor::Entero(valor)) => *valor,
            Some(otro) => {
                return Err(Error::Schema(format!(
                    "{contexto}: 'esquema' debe ser un entero, no {}",
                    otro.tipo()
                )))
            }
            None => return Err(Error::Schema(format!("{contexto}: falta 'esquema'"))),
        };
        if esquema != ESQUEMA {
            return Err(Error::Schema(format!(
                "{contexto}: esquema no soportado: {esquema}"
            )));
        }
        let id = texto_obligatorio(campos, "id", contexto)?;
        if !ulid::es_valido(&id) {
            return Err(Error::Schema(format!(
                "{contexto}: 'id' no es un ULID válido: {id}"
            )));
        }
        let titulo = texto_obligatorio(campos, "titulo", contexto)?;
        if titulo.trim().is_empty() {
            return Err(Error::Schema(format!("{contexto}: 'titulo' está vacío")));
        }
        let autor = texto_opcional(campos, "autor", contexto)?;
        let idioma = texto_opcional(campos, "idioma", contexto)?;
        let creado = texto_opcional(campos, "creado", contexto)?;
        let retencion = texto_opcional(campos, "retencion", contexto)?;
        let guardar_prompts = booleano_opcional(campos, "guardar_prompts", contexto)?;
        let politica_estructura = texto_opcional(campos, "politica_estructura", contexto)?;
        let politica_metadatos = texto_opcional(campos, "politica_metadatos", contexto)?;
        let politica_notas = texto_opcional(campos, "politica_notas", contexto)?;
        let politica_prosa = texto_opcional(campos, "politica_prosa", contexto)?;
        let politica_exportar = texto_opcional(campos, "politica_exportar", contexto)?;

        if let Some(valor) = &retencion {
            if !RETENCIONES.contains(&valor.as_str()) {
                return Err(Error::Schema(format!(
                    "{contexto}: 'retencion' no válido: {valor:?}"
                )));
            }
        }
        for (clave, valor) in [
            ("politica_estructura", &politica_estructura),
            ("politica_metadatos", &politica_metadatos),
            ("politica_notas", &politica_notas),
            ("politica_prosa", &politica_prosa),
            ("politica_exportar", &politica_exportar),
        ] {
            if let Some(valor) = valor {
                if !POLITICAS.contains(&valor.as_str()) {
                    return Err(Error::Schema(format!(
                        "{contexto}: '{clave}' no válido: {valor:?}"
                    )));
                }
            }
        }
        if politica_prosa.as_deref() == Some("auto") {
            return Err(Error::Schema(format!(
                "{contexto}: 'politica_prosa' no puede ser 'auto'"
            )));
        }

        let mut extra = campos.clone();
        for clave in [
            "esquema",
            "id",
            "titulo",
            "autor",
            "idioma",
            "creado",
            "retencion",
            "guardar_prompts",
            "politica_estructura",
            "politica_metadatos",
            "politica_notas",
            "politica_prosa",
            "politica_exportar",
        ] {
            extra.remove(clave);
        }
        for clave in extra.keys() {
            avisos.push(format!(
                "{contexto}: campo desconocido '{clave}' (se conserva)"
            ));
        }

        Ok((
            Self {
                esquema,
                id,
                titulo,
                autor,
                idioma,
                creado,
                retencion,
                guardar_prompts,
                politica_estructura,
                politica_metadatos,
                politica_notas,
                politica_prosa,
                politica_exportar,
                extra,
            },
            avisos,
        ))
    }

    /// Serializa el manifiesto en el orden normativo.
    pub fn a_campos(&self) -> Vec<(String, Valor)> {
        let mut campos = vec![
            ("esquema".to_string(), Valor::Entero(self.esquema)),
            ("id".to_string(), Valor::Texto(self.id.clone())),
            ("titulo".to_string(), Valor::Texto(self.titulo.clone())),
        ];
        for (clave, valor) in [
            ("autor", &self.autor),
            ("idioma", &self.idioma),
            ("creado", &self.creado),
            ("retencion", &self.retencion),
        ] {
            if let Some(valor) = valor {
                campos.push((clave.to_string(), Valor::Texto(valor.clone())));
            }
        }
        if let Some(valor) = self.guardar_prompts {
            campos.push(("guardar_prompts".to_string(), Valor::Booleano(valor)));
        }
        for (clave, valor) in [
            ("politica_estructura", &self.politica_estructura),
            ("politica_metadatos", &self.politica_metadatos),
            ("politica_notas", &self.politica_notas),
            ("politica_prosa", &self.politica_prosa),
            ("politica_exportar", &self.politica_exportar),
        ] {
            if let Some(valor) = valor {
                campos.push((clave.to_string(), Valor::Texto(valor.clone())));
            }
        }
        for (clave, valor) in &self.extra {
            campos.push((clave.clone(), valor.clone()));
        }
        campos
    }

    /// Serializa el manifiesto completo.
    pub fn a_bytes(&self) -> Vec<u8> {
        yaml::escribir_plano(&self.a_campos()).into_bytes()
    }
}

impl Escena {
    /// Crea una escena nueva.
    pub fn nueva(titulo: &str, orden: i64) -> Result<Self> {
        let titulo = titulo.trim();
        if titulo.is_empty() {
            return Err(Error::Input(
                "el título de la escena no puede estar vacío".into(),
            ));
        }
        validar_texto(titulo, "titulo", "escena")?;
        if !(1..=99_999).contains(&orden) {
            return Err(Error::Input("'orden' debe estar entre 1 y 99999".into()));
        }
        Ok(Self {
            id: ulid::nuevo()?,
            titulo: titulo.to_string(),
            orden,
            estado: Some("borrador".to_string()),
            pov: None,
            tramas: None,
            personajes: None,
            fecha_narrativa: None,
            etiquetas: None,
            extra: Campos::new(),
            cuerpo: String::new(),
        })
    }

    /// Parsea y valida una escena.
    pub fn desde_campos(
        campos: &Campos,
        cuerpo: &str,
        contexto: &str,
    ) -> Result<(Self, Vec<String>)> {
        let mut avisos = Vec::new();
        let id = texto_obligatorio(campos, "id", contexto)?;
        if !ulid::es_valido(&id) {
            return Err(Error::Schema(format!(
                "{contexto}: 'id' no es un ULID válido: {id}"
            )));
        }
        let titulo = texto_obligatorio(campos, "titulo", contexto)?;
        if titulo.trim().is_empty() {
            return Err(Error::Schema(format!("{contexto}: 'titulo' está vacío")));
        }
        let orden = entero_obligatorio(campos, "orden", contexto)?;
        if !(1..=99_999).contains(&orden) {
            return Err(Error::Schema(format!(
                "{contexto}: 'orden' fuera de rango: {orden}"
            )));
        }
        let estado = texto_opcional(campos, "estado", contexto)?;
        if let Some(valor) = &estado {
            if !ESTADOS_ESCENA.contains(&valor.as_str()) {
                avisos.push(format!("{contexto}: estado desconocido {valor:?}"));
            }
        }
        let pov = texto_opcional(campos, "pov", contexto)?;
        if let Some(valor) = &pov {
            if !ulid::es_valido(valor) {
                return Err(Error::Schema(format!(
                    "{contexto}: 'pov' debe ser un ULID: {valor}"
                )));
            }
        }
        let tramas = lista_ulids(campos, "tramas", contexto)?;
        let personajes = lista_ulids(campos, "personajes", contexto)?;
        let fecha_narrativa = texto_opcional(campos, "fecha_narrativa", contexto)?;
        let etiquetas = lista_textos(campos, "etiquetas", contexto)?;
        if campos.contains_key("palabras") {
            avisos.push(format!("{contexto}: 'palabras' no debe escribirse a mano"));
        }
        let mut extra = campos.clone();
        for clave in [
            "id",
            "titulo",
            "orden",
            "estado",
            "pov",
            "tramas",
            "personajes",
            "fecha_narrativa",
            "etiquetas",
            "palabras",
        ] {
            extra.remove(clave);
        }
        for clave in extra.keys() {
            avisos.push(format!(
                "{contexto}: campo desconocido '{clave}' (se conserva)"
            ));
        }
        Ok((
            Self {
                id,
                titulo,
                orden,
                estado,
                pov,
                tramas,
                personajes,
                fecha_narrativa,
                etiquetas,
                extra,
                cuerpo: cuerpo.to_string(),
            },
            avisos,
        ))
    }

    /// Serializa la escena completa.
    pub fn a_bytes(&self) -> Vec<u8> {
        let mut campos = vec![
            ("id".to_string(), Valor::Texto(self.id.clone())),
            ("titulo".to_string(), Valor::Texto(self.titulo.clone())),
            ("orden".to_string(), Valor::Entero(self.orden)),
        ];
        if let Some(valor) = &self.estado {
            campos.push(("estado".to_string(), Valor::Texto(valor.clone())));
        }
        if let Some(valor) = &self.pov {
            campos.push(("pov".to_string(), Valor::Texto(valor.clone())));
        }
        for (clave, valor) in [("tramas", &self.tramas), ("personajes", &self.personajes)] {
            if let Some(valores) = valor {
                campos.push((
                    clave.to_string(),
                    Valor::Lista(valores.iter().cloned().map(Valor::Texto).collect()),
                ));
            }
        }
        if let Some(valor) = &self.fecha_narrativa {
            campos.push(("fecha_narrativa".to_string(), Valor::Texto(valor.clone())));
        }
        if let Some(valores) = &self.etiquetas {
            campos.push((
                "etiquetas".to_string(),
                Valor::Lista(valores.iter().cloned().map(Valor::Texto).collect()),
            ));
        }
        for (clave, valor) in &self.extra {
            campos.push((clave.clone(), valor.clone()));
        }
        yaml::escribir_documento(&campos, &self.cuerpo).into_bytes()
    }
}

impl Nota {
    /// Crea una nota nueva.
    pub fn nueva(tipo: &str, titulo: &str) -> Result<Self> {
        if !TIPOS_NOTA.contains(&tipo) {
            return Err(Error::Input(format!("tipo de nota no válido: {tipo}")));
        }
        let titulo = titulo.trim();
        if titulo.is_empty() {
            return Err(Error::Input(
                "el título de la nota no puede estar vacío".into(),
            ));
        }
        validar_texto(titulo, "titulo", "nota")?;
        Ok(Self {
            id: ulid::nuevo()?,
            tipo: tipo.to_string(),
            titulo: titulo.to_string(),
            fija: Some(false),
            etiquetas: None,
            refs: None,
            extra: Campos::new(),
            cuerpo: String::new(),
        })
    }

    /// Parsea y valida una nota.
    pub fn desde_campos(
        campos: &Campos,
        cuerpo: &str,
        contexto: &str,
    ) -> Result<(Self, Vec<String>)> {
        let mut avisos = Vec::new();
        let id = texto_obligatorio(campos, "id", contexto)?;
        if !ulid::es_valido(&id) {
            return Err(Error::Schema(format!(
                "{contexto}: 'id' no es un ULID válido: {id}"
            )));
        }
        let tipo = texto_obligatorio(campos, "tipo", contexto)?;
        if !TIPOS_NOTA.contains(&tipo.as_str()) {
            return Err(Error::Schema(format!(
                "{contexto}: tipo de nota no válido: {tipo}"
            )));
        }
        let titulo = texto_obligatorio(campos, "titulo", contexto)?;
        if titulo.trim().is_empty() {
            return Err(Error::Schema(format!("{contexto}: 'titulo' está vacío")));
        }
        let fija = booleano_opcional(campos, "fija", contexto)?;
        let etiquetas = lista_textos(campos, "etiquetas", contexto)?;
        let refs = lista_ulids(campos, "refs", contexto)?;
        let mut extra = campos.clone();
        for clave in ["id", "tipo", "titulo", "fija", "etiquetas", "refs"] {
            extra.remove(clave);
        }
        for clave in extra.keys() {
            avisos.push(format!(
                "{contexto}: campo desconocido '{clave}' (se conserva)"
            ));
        }
        Ok((
            Self {
                id,
                tipo,
                titulo,
                fija,
                etiquetas,
                refs,
                extra,
                cuerpo: cuerpo.to_string(),
            },
            avisos,
        ))
    }

    /// Serializa la nota completa.
    pub fn a_bytes(&self) -> Vec<u8> {
        let mut campos = vec![
            ("id".to_string(), Valor::Texto(self.id.clone())),
            ("tipo".to_string(), Valor::Texto(self.tipo.clone())),
            ("titulo".to_string(), Valor::Texto(self.titulo.clone())),
        ];
        if let Some(valor) = self.fija {
            campos.push(("fija".to_string(), Valor::Booleano(valor)));
        }
        if let Some(valores) = &self.etiquetas {
            campos.push((
                "etiquetas".to_string(),
                Valor::Lista(valores.iter().cloned().map(Valor::Texto).collect()),
            ));
        }
        if let Some(valores) = &self.refs {
            campos.push((
                "refs".to_string(),
                Valor::Lista(valores.iter().cloned().map(Valor::Texto).collect()),
            ));
        }
        for (clave, valor) in &self.extra {
            campos.push((clave.clone(), valor.clone()));
        }
        yaml::escribir_documento(&campos, &self.cuerpo).into_bytes()
    }
}

/// Lee un archivo de texto y normaliza BOM/CRLF, con avisos.
pub fn leer_texto(ruta: &Path) -> Result<Lectura> {
    let crudo = std::fs::read(ruta).map_err(|error| Error::Io(error).with_path(ruta))?;
    let mut avisos = Vec::new();
    let mut inicio = 0;
    if crudo.starts_with(&[0xEF, 0xBB, 0xBF]) {
        inicio = 3;
        avisos.push("BOM UTF-8 (se tolera al leer, no al escribir)".to_string());
    }
    let sin_bom = &crudo[inicio..];
    if sin_bom.windows(2).any(|ventana| ventana == b"\r\n") {
        avisos.push("finales de línea CRLF".to_string());
    }
    if sin_bom
        .iter()
        .enumerate()
        .any(|(indice, byte)| *byte == b'\r' && sin_bom.get(indice + 1) != Some(&b'\n'))
    {
        return Err(Error::Schema(format!(
            "{}: retorno de carro aislado; se exige LF",
            ruta.display()
        )));
    }
    let texto = std::str::from_utf8(sin_bom)
        .map_err(|error| {
            Error::Schema(format!("{}: no es UTF-8 válido ({error})", ruta.display()))
        })?
        .replace("\r\n", "\n");
    for (numero, linea) in texto.split('\n').enumerate() {
        if linea.ends_with(' ') || linea.ends_with('\t') {
            avisos.push(format!("línea {}: espacios al final", numero + 1));
        }
    }
    Ok(Lectura { texto, avisos })
}

/// Lee y parsea un documento con frontmatter.
pub fn leer_documento(ruta: &Path) -> Result<(Campos, String, Vec<String>)> {
    let lectura = leer_texto(ruta)?;
    let (campos, cuerpo) = yaml::parsear_documento(&lectura.texto, &ruta.display().to_string())?;
    Ok((campos, cuerpo, lectura.avisos))
}

/// Lee y parsea un archivo plano.
pub fn leer_plano(ruta: &Path) -> Result<(Campos, Vec<String>)> {
    let lectura = leer_texto(ruta)?;
    let campos = yaml::parsear_plano(&lectura.texto, &ruta.display().to_string())?;
    Ok((campos, lectura.avisos))
}

/// Devuelve el texto visible de las marcas propias, sin su sintaxis.
///
/// `[[personaje:ULID|Marta]]` cuenta como «Marta»; una construcción que no sea
/// una marca válida se conserva literal.
fn texto_sin_marcas(cuerpo: &str) -> String {
    let mut salida = String::with_capacity(cuerpo.len());
    let mut resto = cuerpo;
    while let Some(inicio) = resto.find("[[") {
        salida.push_str(&resto[..inicio]);
        let tras = &resto[inicio + 2..];
        let Some(fin) = tras.find("]]") else {
            salida.push_str(&resto[inicio..]);
            return salida;
        };
        let interior = &tras[..fin];
        let es_marca = interior.split_once('|').is_some_and(|(prefijo, _)| {
            prefijo.starts_with("personaje:")
                || prefijo.starts_with("trama:")
                || prefijo.starts_with("comentario:")
        });
        if let Some((_, contenido)) = interior.split_once('|').filter(|_| es_marca) {
            salida.push_str(contenido);
        } else {
            salida.push_str(&resto[inicio..inicio + 2 + fin + 2]);
        }
        resto = &tras[fin + 2..];
    }
    salida.push_str(resto);
    salida
}

/// Cuenta palabras visibles como secuencias maximales de caracteres alfanuméricos.
pub fn contar_palabras(cuerpo: &str) -> i64 {
    let visible = texto_sin_marcas(cuerpo);
    let mut total = 0_i64;
    let mut dentro = false;
    for caracter in visible.chars() {
        if caracter.is_alphanumeric() {
            if !dentro {
                total += 1;
                dentro = true;
            }
        } else {
            dentro = false;
        }
    }
    total
}

/// Convierte un título en un slug ASCII en minúsculas.
pub fn slug(texto: &str) -> String {
    let normalizado: String = texto
        .nfkd()
        .filter(|caracter| !unicode_normalization::char::is_combining_mark(*caracter))
        .collect::<String>()
        .to_lowercase();
    let mut salida = String::new();
    let mut guion = false;
    for caracter in normalizado.chars() {
        if caracter.is_ascii_alphanumeric() {
            salida.push(caracter);
            guion = false;
        } else if !guion && !salida.is_empty() {
            salida.push('-');
            guion = true;
        }
    }
    while salida.ends_with('-') {
        salida.pop();
    }
    if salida.is_empty() {
        "sin-titulo".to_string()
    } else {
        salida
    }
}

/// Convierte `01-libro` en `Libro` para la exportación.
pub fn titulo_humano(carpeta: &str) -> String {
    let sin_prefijo = match carpeta.split_once('-') {
        Some((prefijo, resto))
            if prefijo.bytes().all(|byte| byte.is_ascii_digit()) && !resto.is_empty() =>
        {
            resto
        }
        _ => carpeta,
    };
    let texto = sin_prefijo.replace(['-', '_'], " ");
    let mut caracteres = texto.chars();
    match caracteres.next() {
        Some(primero) => primero.to_uppercase().collect::<String>() + caracteres.as_str(),
        None => String::new(),
    }
}

/// Comprueba que un valor es un hash del esquema.
pub fn es_hash(texto: &str) -> bool {
    hash::es_hash(texto)
}

fn texto_obligatorio(campos: &Campos, clave: &str, contexto: &str) -> Result<String> {
    match campos.get(clave) {
        Some(Valor::Texto(texto)) => {
            validar_texto(texto, clave, contexto)?;
            Ok(texto.clone())
        }
        Some(otro) => Err(Error::Schema(format!(
            "{contexto}: '{clave}' debe ser texto, no {}",
            otro.tipo()
        ))),
        None => Err(Error::Schema(format!("{contexto}: falta '{clave}'"))),
    }
}

fn texto_opcional(campos: &Campos, clave: &str, contexto: &str) -> Result<Option<String>> {
    match campos.get(clave) {
        Some(Valor::Texto(texto)) => {
            validar_texto(texto, clave, contexto)?;
            Ok(Some(texto.clone()))
        }
        Some(otro) => Err(Error::Schema(format!(
            "{contexto}: '{clave}' debe ser texto, no {}",
            otro.tipo()
        ))),
        None => Ok(None),
    }
}

pub(crate) fn validar_texto(texto: &str, clave: &str, contexto: &str) -> Result<()> {
    if texto.chars().any(|caracter| caracter < ' ') {
        return Err(Error::Schema(format!(
            "{contexto}: '{clave}' contiene un carácter de control"
        )));
    }
    Ok(())
}

fn entero_obligatorio(campos: &Campos, clave: &str, contexto: &str) -> Result<i64> {
    match campos.get(clave) {
        Some(Valor::Entero(valor)) => Ok(*valor),
        Some(otro) => Err(Error::Schema(format!(
            "{contexto}: '{clave}' debe ser entero, no {}",
            otro.tipo()
        ))),
        None => Err(Error::Schema(format!("{contexto}: falta '{clave}'"))),
    }
}

fn booleano_opcional(campos: &Campos, clave: &str, contexto: &str) -> Result<Option<bool>> {
    match campos.get(clave) {
        Some(Valor::Booleano(valor)) => Ok(Some(*valor)),
        Some(otro) => Err(Error::Schema(format!(
            "{contexto}: '{clave}' debe ser booleano, no {}",
            otro.tipo()
        ))),
        None => Ok(None),
    }
}

fn lista_textos(campos: &Campos, clave: &str, contexto: &str) -> Result<Option<Vec<String>>> {
    match campos.get(clave) {
        Some(Valor::Lista(valores)) => {
            let mut salida = Vec::with_capacity(valores.len());
            for valor in valores {
                match valor {
                    Valor::Texto(texto) if !texto.trim().is_empty() => {
                        validar_texto(texto, clave, contexto)?;
                        salida.push(texto.clone());
                    }
                    Valor::Texto(_) => {
                        return Err(Error::Schema(format!(
                            "{contexto}: '{clave}' contiene un texto vacío"
                        )))
                    }
                    otro => {
                        return Err(Error::Schema(format!(
                            "{contexto}: '{clave}' contiene un valor no textual: {}",
                            otro.tipo()
                        )))
                    }
                }
            }
            Ok(Some(salida))
        }
        Some(otro) => Err(Error::Schema(format!(
            "{contexto}: '{clave}' debe ser lista, no {}",
            otro.tipo()
        ))),
        None => Ok(None),
    }
}

fn lista_ulids(campos: &Campos, clave: &str, contexto: &str) -> Result<Option<Vec<String>>> {
    let valores = lista_textos(campos, clave, contexto)?;
    if let Some(valores) = &valores {
        for valor in valores {
            if !ulid::es_valido(valor) {
                return Err(Error::Schema(format!(
                    "{contexto}: '{clave}' contiene un id que no es ULID: {valor}"
                )));
            }
        }
    }
    Ok(valores)
}

/// Mapa de campos de un documento, útil para la CLI.
pub fn campos_a_json(campos: &Campos) -> serde_json::Value {
    let mut mapa = serde_json::Map::new();
    for (clave, valor) in campos {
        mapa.insert(clave.clone(), valor.a_json());
    }
    serde_json::Value::Object(mapa)
}

/// Convierte un mapa de campos a `BTreeMap` para pruebas.
pub fn campos_desde_pares(pares: &[(&str, Valor)]) -> Campos {
    let mut campos = BTreeMap::new();
    for (clave, valor) in pares {
        campos.insert((*clave).to_string(), valor.clone());
    }
    campos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proyecto_ida_y_vuelta() {
        let proyecto = Proyecto::nuevo("El umbral", Some("Autora"), Some("es")).expect("nuevo");
        let bytes = proyecto.a_bytes();
        let texto = String::from_utf8(bytes).expect("utf8");
        let campos = yaml::parsear_plano(&texto, "proyecto.yml").expect("parsear");
        let (de_vuelta, avisos) = Proyecto::desde_campos(&campos, "proyecto.yml").expect("modelo");
        assert!(avisos.is_empty());
        assert_eq!(de_vuelta.titulo, "El umbral");
        assert_eq!(de_vuelta.id, proyecto.id);
        assert_eq!(de_vuelta.politica_prosa.as_deref(), Some("proponer"));
    }

    #[test]
    fn escena_ida_y_vuelta() {
        let mut escena = Escena::nueva("Escena # 1", 10).expect("nueva");
        escena.cuerpo = "Hola\n".to_string();
        escena.etiquetas = Some(vec!["uno, dos".to_string()]);
        let texto = String::from_utf8(escena.a_bytes()).expect("utf8");
        let (campos, cuerpo) = yaml::parsear_documento(&texto, "escena").expect("parsear");
        let (de_vuelta, avisos) = Escena::desde_campos(&campos, &cuerpo, "escena").expect("modelo");
        assert!(avisos.is_empty());
        assert_eq!(de_vuelta.titulo, "Escena # 1");
        assert_eq!(de_vuelta.orden, 10);
        assert_eq!(de_vuelta.etiquetas, Some(vec!["uno, dos".to_string()]));
        assert_eq!(de_vuelta.cuerpo, "Hola\n");
    }

    #[test]
    fn nota_ida_y_vuelta() {
        let mut nota = Nota::nueva("personajes", "Marta").expect("nueva");
        nota.refs = Some(vec!["01KDVDNAZ84TFF59TDWH9EDD1R".to_string()]);
        let texto = String::from_utf8(nota.a_bytes()).expect("utf8");
        let (campos, cuerpo) = yaml::parsear_documento(&texto, "nota").expect("parsear");
        let (de_vuelta, avisos) = Nota::desde_campos(&campos, &cuerpo, "nota").expect("modelo");
        assert!(avisos.is_empty());
        assert_eq!(de_vuelta.tipo, "personajes");
        assert_eq!(de_vuelta.refs.as_ref().map(Vec::len), Some(1));
    }

    #[test]
    fn slug_y_titulo_humano() {
        assert_eq!(slug("El año de la nieve"), "el-ano-de-la-nieve");
        assert_eq!(slug("  "), "sin-titulo");
        assert_eq!(titulo_humano("01-libro"), "Libro");
        assert_eq!(titulo_humano("02-capitulo"), "Capitulo");
    }

    #[test]
    fn palabras() {
        assert_eq!(contar_palabras("Hola, mundo.  ¿Qué tal?"), 4);
        assert_eq!(contar_palabras(""), 0);
        assert_eq!(
            contar_palabras("[[personaje:01KDVDNBYG20KW71NVRK6P2FHG|Marta]] llega."),
            2
        );
        assert_eq!(
            contar_palabras("[[comentario:revisar|Marta]] y [[otra|cosa]]"),
            4
        );
    }
}
