//! Subconjunto YAML v1: plano, estricto y con ida y vuelta verificable.
//!
//! Es una implementación deliberadamente pequeña: no intenta ser YAML. Cada
//! construcción fuera del subconjunto se rechaza para que el motor nunca
//! reinterprete un archivo que no entiende.

use std::collections::BTreeMap;

use crate::error::{Error, Result};

/// Valor escalar o lista admitido por el esquema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Valor {
    /// Texto plano o entre comillas dobles.
    Texto(String),
    /// Entero decimal.
    Entero(i64),
    /// `true` o `false`.
    Booleano(bool),
    /// `null`.
    Nulo,
    /// Lista en línea de escalares.
    Lista(Vec<Valor>),
}

impl Valor {
    /// Devuelve el texto si el valor es textual.
    pub fn como_texto(&self) -> Option<&str> {
        match self {
            Valor::Texto(texto) => Some(texto),
            _ => None,
        }
    }

    /// Devuelve el entero si el valor es entero.
    pub fn como_entero(&self) -> Option<i64> {
        match self {
            Valor::Entero(valor) => Some(*valor),
            _ => None,
        }
    }

    /// Devuelve el booleano si el valor es booleano.
    pub fn como_booleano(&self) -> Option<bool> {
        match self {
            Valor::Booleano(valor) => Some(*valor),
            _ => None,
        }
    }

    /// Devuelve la lista si el valor es una lista.
    pub fn como_lista(&self) -> Option<&[Valor]> {
        match self {
            Valor::Lista(valores) => Some(valores),
            _ => None,
        }
    }

    /// Convierte el valor a JSON para `--json`.
    pub fn a_json(&self) -> serde_json::Value {
        match self {
            Valor::Texto(texto) => serde_json::Value::String(texto.clone()),
            Valor::Entero(valor) => serde_json::Value::from(*valor),
            Valor::Booleano(valor) => serde_json::Value::Bool(*valor),
            Valor::Nulo => serde_json::Value::Null,
            Valor::Lista(valores) => {
                serde_json::Value::Array(valores.iter().map(Valor::a_json).collect())
            }
        }
    }

    /// Nombre del tipo, para mensajes de error.
    pub fn tipo(&self) -> &'static str {
        match self {
            Valor::Texto(_) => "texto",
            Valor::Entero(_) => "entero",
            Valor::Booleano(_) => "booleano",
            Valor::Nulo => "null",
            Valor::Lista(_) => "lista",
        }
    }
}

/// Campos de un documento, ordenados por clave para conservarlos.
pub type Campos = BTreeMap<String, Valor>;

/// Parsea un documento con frontmatter `---` y devuelve campos y cuerpo.
pub fn parsear_documento(texto: &str, contexto: &str) -> Result<(Campos, String)> {
    // Los blobs históricos y los archivos importados pueden venir de Windows
    // con BOM o CRLF. `model::leer_texto` ya normaliza la lectura normal; esto
    // cubre las rutas que leen bytes directamente.
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let normalizado;
    let texto = if texto.contains('\r') {
        normalizado = texto.replace("\r\n", "\n").replace('\r', "\n");
        normalizado.as_str()
    } else {
        texto
    };
    let resto = texto
        .strip_prefix("---\n")
        .ok_or_else(|| Error::Schema(format!("{contexto}: el archivo no empieza por '---'")))?;
    let (cabecera, cuerpo) = separar_frontmatter(resto, contexto)?;
    let campos = parsear_lineas(cabecera, contexto, 2)?;
    Ok((campos, cuerpo))
}

fn separar_frontmatter<'a>(resto: &'a str, contexto: &str) -> Result<(&'a str, String)> {
    let mut inicio = 0;
    for linea in resto.split_inclusive('\n') {
        let contenido = linea.strip_suffix('\n').unwrap_or(linea);
        if contenido == "---" {
            let cabecera = &resto[..inicio];
            let mut cuerpo = resto[inicio + linea.len()..].to_string();
            if cuerpo.starts_with('\n') {
                cuerpo.remove(0);
            }
            return Ok((cabecera, cuerpo));
        }
        inicio += linea.len();
    }
    Err(Error::Schema(format!("{contexto}: frontmatter sin cerrar")))
}

/// Parsea un archivo plano (`proyecto.yml`), sin frontmatter.
pub fn parsear_plano(texto: &str, contexto: &str) -> Result<Campos> {
    parsear_lineas(texto, contexto, 1)
}

fn parsear_lineas(texto: &str, contexto: &str, primer_numero: usize) -> Result<Campos> {
    let mut datos = Campos::new();
    for (indice, linea) in texto.split('\n').enumerate() {
        let numero = primer_numero + indice;
        let donde = format!("{contexto}: línea {numero}");
        if linea.trim().is_empty() {
            continue;
        }
        if linea.starts_with(' ') || linea.starts_with('\t') {
            return Err(Error::Schema(format!(
                "{donde}: indentación fuera del subconjunto YAML v1"
            )));
        }
        if linea.trim_start().starts_with('#') {
            return Err(Error::Schema(format!(
                "{donde}: comentarios no admitidos en el frontmatter"
            )));
        }
        if linea.starts_with("- ") {
            return Err(Error::Schema(format!(
                "{donde}: listas en bloque fuera del subconjunto YAML v1"
            )));
        }
        let (clave, valor) = linea
            .split_once(':')
            .ok_or_else(|| Error::Schema(format!("{donde}: falta ':' tras la clave")))?;
        let clave = clave.trim();
        if !clave_valida(clave) {
            return Err(Error::Schema(format!(
                "{donde}: clave no válida: {clave:?}"
            )));
        }
        if datos.contains_key(clave) {
            return Err(Error::Schema(format!("{donde}: clave duplicada: {clave}")));
        }
        datos.insert(clave.to_string(), parsear_escalar(valor, &donde)?);
    }
    Ok(datos)
}

fn clave_valida(clave: &str) -> bool {
    let mut bytes = clave.bytes();
    match bytes.next() {
        Some(byte) if byte.is_ascii_lowercase() || byte == b'_' => {}
        _ => return false,
    }
    bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// Parsea un escalar del subconjunto.
pub fn parsear_escalar(texto: &str, contexto: &str) -> Result<Valor> {
    let valor = texto.trim();
    if valor.is_empty() {
        return Err(Error::Schema(format!("{contexto}: valor vacío")));
    }
    let primero = valor.chars().next().unwrap_or('\0');
    match primero {
        '\'' => Err(Error::Schema(format!(
            "{contexto}: comillas simples no admitidas; usa comillas dobles"
        ))),
        '"' => Ok(Valor::Texto(texto_entre_comillas(valor, contexto)?)),
        '[' => parsear_lista(valor, contexto),
        '{' => Err(Error::Schema(format!(
            "{contexto}: mapas anidados fuera del subconjunto YAML v1"
        ))),
        ']' | '}' | '&' | '*' | '!' | '|' | '>' | '@' | '`' | '%' | '#' => Err(Error::Schema(
            format!("{contexto}: indicador YAML no admitido: {valor:?}"),
        )),
        _ => {
            if valor == "true" {
                return Ok(Valor::Booleano(true));
            }
            if valor == "false" {
                return Ok(Valor::Booleano(false));
            }
            if valor == "null" {
                return Ok(Valor::Nulo);
            }
            if es_entero(valor) {
                return valor.parse::<i64>().map(Valor::Entero).map_err(|_| {
                    Error::Schema(format!("{contexto}: entero fuera de rango: {valor:?}"))
                });
            }
            if primero == '-' || primero == '?' {
                return Err(Error::Schema(format!(
                    "{contexto}: construcción YAML no admitida: {valor:?}"
                )));
            }
            if valor.contains('#') {
                return Err(Error::Schema(format!(
                    "{contexto}: '#' en texto plano inicia un comentario; usa comillas dobles"
                )));
            }
            if valor.contains('"') || valor.contains('\'') {
                return Err(Error::Schema(format!(
                    "{contexto}: comillas sin escapar en un texto plano; usa comillas dobles"
                )));
            }
            if valor.chars().any(|caracter| caracter < ' ') {
                return Err(Error::Schema(format!(
                    "{contexto}: carácter de control en el texto"
                )));
            }
            Ok(Valor::Texto(valor.to_string()))
        }
    }
}

fn texto_entre_comillas(valor: &str, contexto: &str) -> Result<String> {
    let mut salida = String::new();
    let mut caracteres = valor.char_indices();
    let _ = caracteres.next();
    while let Some((indice, caracter)) = caracteres.next() {
        match caracter {
            '\\' => {
                let (_, siguiente) = caracteres.next().ok_or_else(|| {
                    Error::Schema(format!("{contexto}: escape incompleto al final del texto"))
                })?;
                match siguiente {
                    '\\' => salida.push('\\'),
                    '"' => salida.push('"'),
                    'n' => salida.push('\n'),
                    't' => salida.push('\t'),
                    otro => {
                        return Err(Error::Schema(format!(
                            "{contexto}: escape no admitido \\{otro}"
                        )))
                    }
                }
            }
            '"' => {
                if indice != valor.len() - 1 {
                    return Err(Error::Schema(format!(
                        "{contexto}: comilla interior sin escapar o texto tras el cierre"
                    )));
                }
                return Ok(salida);
            }
            caracter if caracter < ' ' => {
                return Err(Error::Schema(format!(
                    "{contexto}: carácter de control en el texto"
                )));
            }
            caracter => salida.push(caracter),
        }
    }
    Err(Error::Schema(format!(
        "{contexto}: texto entre comillas sin cerrar"
    )))
}

fn parsear_lista(texto: &str, contexto: &str) -> Result<Valor> {
    let interior = texto
        .strip_prefix('[')
        .and_then(|resto| resto.strip_suffix(']'))
        .ok_or_else(|| Error::Schema(format!("{contexto}: lista sin cerrar")))?;
    let interior = interior.trim();
    if interior.is_empty() {
        return Ok(Valor::Lista(Vec::new()));
    }
    let mut valores = Vec::new();
    for elemento in dividir_lista(interior, contexto)? {
        let limpio = elemento.trim();
        if limpio.starts_with('[') || limpio.starts_with('{') {
            return Err(Error::Schema(format!(
                "{contexto}: listas o mapas anidados fuera del subconjunto YAML v1"
            )));
        }
        valores.push(parsear_escalar(&elemento, contexto)?);
    }
    Ok(Valor::Lista(valores))
}

fn dividir_lista(interior: &str, contexto: &str) -> Result<Vec<String>> {
    let mut elementos = Vec::new();
    let mut actual = String::new();
    let mut entre_comillas = false;
    let mut escapado = false;
    for caracter in interior.chars() {
        if escapado {
            actual.push(caracter);
            escapado = false;
            continue;
        }
        if entre_comillas {
            match caracter {
                '\\' => {
                    actual.push(caracter);
                    escapado = true;
                }
                '"' => {
                    actual.push(caracter);
                    entre_comillas = false;
                }
                _ => actual.push(caracter),
            }
            continue;
        }
        match caracter {
            '"' => {
                entre_comillas = true;
                actual.push(caracter);
            }
            ',' => {
                elementos.push(std::mem::take(&mut actual));
            }
            _ => actual.push(caracter),
        }
    }
    if entre_comillas {
        return Err(Error::Schema(format!(
            "{contexto}: comillas sin cerrar en lista"
        )));
    }
    elementos.push(actual);
    Ok(elementos)
}

fn es_entero(valor: &str) -> bool {
    let resto = valor.strip_prefix('-').unwrap_or(valor);
    !resto.is_empty() && resto.bytes().all(|byte| byte.is_ascii_digit())
}

/// Serializa un valor escalar para el subconjunto, citando cuando hace falta.
pub fn serializar_escalar(valor: &Valor) -> String {
    match valor {
        Valor::Texto(texto) => serializar_texto(texto),
        Valor::Entero(valor) => valor.to_string(),
        Valor::Booleano(valor) => valor.to_string(),
        Valor::Nulo => "null".to_string(),
        Valor::Lista(valores) => {
            let elementos: Vec<String> = valores
                .iter()
                .map(|valor| match valor {
                    Valor::Texto(texto) => serializar_elemento(texto),
                    otro => serializar_escalar(otro),
                })
                .collect();
            format!("[{}]", elementos.join(", "))
        }
    }
}

fn serializar_elemento(texto: &str) -> String {
    if necesita_comillas(texto) {
        citar(texto)
    } else {
        texto.to_string()
    }
}

fn serializar_texto(texto: &str) -> String {
    if necesita_comillas(texto) {
        citar(texto)
    } else {
        texto.to_string()
    }
}

fn necesita_comillas(texto: &str) -> bool {
    if texto.is_empty() || texto != texto.trim() {
        return true;
    }
    if texto.chars().any(|caracter| caracter < ' ') {
        return true;
    }
    if texto
        .chars()
        .any(|caracter| matches!(caracter, ',' | '"' | '\'' | '[' | ']' | '{' | '}' | '#'))
    {
        return true;
    }
    let primero = texto.chars().next().unwrap_or('\0');
    if matches!(
        primero,
        '-' | '?' | '&' | '*' | '!' | '|' | '>' | '@' | '`' | '%' | '#' | '['
    ) {
        return true;
    }
    if es_entero(texto) || matches!(texto, "true" | "false" | "null") {
        return true;
    }
    false
}

fn citar(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len() + 2);
    salida.push('"');
    for caracter in texto.chars() {
        match caracter {
            '\\' => salida.push_str("\\\\"),
            '"' => salida.push_str("\\\""),
            '\n' => salida.push_str("\\n"),
            '\t' => salida.push_str("\\t"),
            otro => salida.push(otro),
        }
    }
    salida.push('"');
    salida
}

/// Serializa un documento completo con frontmatter y cuerpo.
pub fn escribir_documento(campos: &[(String, Valor)], cuerpo: &str) -> String {
    let mut salida = String::from("---\n");
    for (clave, valor) in campos {
        salida.push_str(clave);
        salida.push_str(": ");
        salida.push_str(&serializar_escalar(valor));
        salida.push('\n');
    }
    salida.push_str("---\n\n");
    salida.push_str(cuerpo);
    if !salida.ends_with('\n') {
        salida.push('\n');
    }
    salida
}

/// Serializa un archivo plano (`proyecto.yml`).
pub fn escribir_plano(campos: &[(String, Valor)]) -> String {
    let mut salida = String::new();
    for (clave, valor) in campos {
        salida.push_str(clave);
        salida.push_str(": ");
        salida.push_str(&serializar_escalar(valor));
        salida.push('\n');
    }
    salida
}

/// Añade el cuerpo a un documento ya serializado sin duplicar el salto final.
pub fn con_cuerpo(campos: &[(String, Valor)], cuerpo: &str) -> String {
    escribir_documento(campos, cuerpo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documento_basico() {
        let texto = "---\nid: 01KDVDNAZ84TFF59TDWH9EDD1R\ntitulo: \"Escena # 1\"\norden: 10\nrefs: [a, \"b, c\"]\n---\n\nCuerpo\n";
        let (campos, cuerpo) = parsear_documento(texto, "prueba").expect("parsear");
        assert_eq!(
            campos["id"].como_texto(),
            Some("01KDVDNAZ84TFF59TDWH9EDD1R")
        );
        assert_eq!(campos["titulo"].como_texto(), Some("Escena # 1"));
        assert_eq!(campos["orden"].como_entero(), Some(10));
        assert_eq!(cuerpo, "Cuerpo\n");
    }

    #[test]
    fn documento_con_bom_y_crlf() {
        let texto = "\u{feff}---\r\ntitulo: Windows\r\n---\r\n\r\nCuerpo\r\n";
        let (campos, cuerpo) = parsear_documento(texto, "prueba").expect("parsear");
        assert_eq!(campos["titulo"].como_texto(), Some("Windows"));
        assert_eq!(cuerpo, "Cuerpo\n");
    }

    #[test]
    fn rechaza_comillas_simples() {
        let error = parsear_plano("titulo: 'Hola'\n", "prueba").expect_err("debe fallar");
        assert!(error.to_string().contains("comillas simples"));
    }

    #[test]
    fn rechaza_comentarios_y_anidamiento() {
        assert!(parsear_plano("titulo: Hola # comentario\n", "prueba").is_err());
        assert!(parsear_plano("  titulo: Hola\n", "prueba").is_err());
        assert!(parsear_plano("titulo: {a: b}\n", "prueba").is_err());
    }

    #[test]
    fn ida_y_vuelta() {
        let campos = vec![
            ("titulo".to_string(), Valor::Texto("Escena # 1".to_string())),
            ("orden".to_string(), Valor::Entero(10)),
            (
                "refs".to_string(),
                Valor::Lista(vec![
                    Valor::Texto("uno".to_string()),
                    Valor::Texto("dos, tres".to_string()),
                ]),
            ),
        ];
        let texto = escribir_documento(&campos, "Hola\n");
        let (de_vuelta, cuerpo) = parsear_documento(&texto, "prueba").expect("parsear");
        assert_eq!(de_vuelta["titulo"].como_texto(), Some("Escena # 1"));
        assert_eq!(de_vuelta["orden"].como_entero(), Some(10));
        assert_eq!(de_vuelta["refs"].como_lista().map(<[Valor]>::len), Some(2));
        assert_eq!(cuerpo, "Hola\n");
    }

    #[test]
    fn tolera_lista_vacia_y_nulos() {
        let (campos, _) =
            parsear_documento("---\nrefs: []\nnota: null\n---\n", "x").expect("parsear");
        assert_eq!(campos["refs"].como_lista().map(<[Valor]>::len), Some(0));
        assert_eq!(campos["nota"], Valor::Nulo);
    }
}
