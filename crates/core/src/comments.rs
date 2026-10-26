//! Comentarios locales v2: anclas, hilos y estado.
//!
//! Implementa la extensión `comentarios: 2` de `docs/esquema-v2.md` §3. El
//! ancla vive en el Markdown; el hilo y sus mensajes viven en
//! `.escritor/comentarios/<ULID>.json`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{fsutil, hash, time, ulid};

/// Versión de la extensión de comentarios.
pub const VERSION: u32 = 2;

/// Estado de un hilo.
pub const ESTADOS: [&str; 2] = ["abierto", "resuelto"];

/// Tipos de mensaje append-only.
pub const TIPOS_MENSAJE: [&str; 5] = [
    "comentario",
    "respuesta",
    "edicion",
    "resolucion",
    "borrado",
];

/// Ancla de un comentario al texto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ancla {
    /// Texto marcado.
    pub texto: String,
    /// Hash del texto marcado.
    pub hash: String,
    /// Offset orientativo de inicio.
    #[serde(default)]
    pub inicio: usize,
    /// Offset orientativo de fin.
    #[serde(default)]
    pub fin: usize,
}

/// Mensaje de un hilo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mensaje {
    /// ULID del mensaje.
    pub id: String,
    /// Tipo de mensaje.
    pub tipo: String,
    /// Autor local.
    pub autor: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Texto.
    pub texto: String,
    /// Mensaje reemplazado, si aplica.
    #[serde(default)]
    pub reemplaza: Option<String>,
}

/// Hilo de comentarios.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comentario {
    /// Versión de la extensión.
    pub version: u32,
    /// ULID del hilo.
    pub id: String,
    /// `comentario` o `sticky`.
    #[serde(default = "tipo_comentario")]
    pub tipo: String,
    /// Documento relativo.
    pub documento: String,
    /// Ancla; `None` para sticky.
    pub ancla: Option<Ancla>,
    /// `abierto` o `resuelto`.
    pub estado: String,
    /// Creación.
    pub creado: String,
    /// Última actualización.
    pub actualizado: String,
    /// Autor local.
    pub autor: String,
    /// Marcado como huérfano.
    #[serde(default)]
    pub huerfano: bool,
    /// Mensajes append-only.
    pub mensajes: Vec<Mensaje>,
}

fn tipo_comentario() -> String {
    "comentario".to_string()
}

impl Comentario {
    /// Valida el hilo antes de leerlo o escribirlo.
    pub fn validar(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(Error::Schema(format!(
                "versión de comentarios no soportada: {}",
                self.version
            )));
        }
        if !ulid::es_valido(&self.id) {
            return Err(Error::Schema(format!(
                "comentario con ULID no válido: {}",
                self.id
            )));
        }
        if !matches!(self.tipo.as_str(), "comentario" | "sticky") {
            return Err(Error::Schema(format!(
                "tipo de comentario no válido: {}",
                self.tipo
            )));
        }
        fsutil::validar_ruta_documento(&self.documento)?;
        if !ESTADOS.contains(&self.estado.as_str()) {
            return Err(Error::Schema(format!(
                "estado de comentario no válido: {}",
                self.estado
            )));
        }
        if self.autor.trim().is_empty() || self.autor.len() > 200 {
            return Err(Error::Schema("autor de comentario no válido".to_string()));
        }
        time::parse_rfc3339(&self.creado)?;
        time::parse_rfc3339(&self.actualizado)?;
        for mensaje in &self.mensajes {
            if !ulid::es_valido(&mensaje.id) {
                return Err(Error::Schema(format!(
                    "mensaje con ULID no válido: {}",
                    mensaje.id
                )));
            }
            if !TIPOS_MENSAJE.contains(&mensaje.tipo.as_str()) {
                return Err(Error::Schema(format!(
                    "tipo de mensaje no válido: {}",
                    mensaje.tipo
                )));
            }
            if mensaje.autor.trim().is_empty() || mensaje.texto.trim().is_empty() {
                return Err(Error::Schema(
                    "mensaje de comentario vacío o sin autor".to_string(),
                ));
            }
            if mensaje.texto.contains('|') || mensaje.texto.contains("]]") {
                return Err(Error::Schema(
                    "el mensaje de comentario contiene delimitadores reservados".to_string(),
                ));
            }
            time::parse_rfc3339(&mensaje.ts)?;
        }
        Ok(())
    }
}

impl Mensaje {
    /// Crea un mensaje nuevo.
    pub fn nuevo(tipo: &str, autor: &str, texto: &str) -> Result<Self> {
        if !TIPOS_MENSAJE.contains(&tipo) {
            return Err(Error::Input(format!("tipo de mensaje no válido: {tipo}")));
        }
        let autor = autor.trim();
        let texto = texto.trim();
        if autor.is_empty() || texto.is_empty() {
            return Err(Error::Input(
                "el mensaje y el autor no pueden estar vacíos".to_string(),
            ));
        }
        Ok(Self {
            id: ulid::nuevo()?,
            tipo: tipo.to_string(),
            autor: autor.to_string(),
            ts: time::ahora(),
            texto: texto.to_string(),
            reemplaza: None,
        })
    }
}

/// Crea un hilo anclado a un documento.
pub fn crear_comentario(
    raiz: &Path,
    documento: &str,
    texto: &str,
    texto_marcado: &str,
    autor: &str,
) -> Result<Comentario> {
    let texto = texto.trim();
    let texto_marcado = texto_marcado.trim();
    if texto.is_empty() || texto_marcado.is_empty() {
        return Err(Error::Input(
            "el comentario y el texto marcado no pueden estar vacíos".to_string(),
        ));
    }
    let autor = autor.trim();
    if autor.is_empty() {
        return Err(Error::Input("el autor no puede estar vacío".to_string()));
    }
    fsutil::validar_ruta_documento(documento)?;
    let ruta_documento = raiz.join(documento);
    if !ruta_documento.is_file() {
        return Err(Error::NotFound(format!(
            "documento del comentario no encontrado: {documento}"
        )));
    }
    let ahora = time::ahora();
    let id = ulid::nuevo()?;
    let comentario = Comentario {
        version: VERSION,
        id: id.clone(),
        tipo: "comentario".to_string(),
        documento: documento.to_string(),
        ancla: Some(Ancla {
            texto: texto_marcado.to_string(),
            hash: hash::hash(texto_marcado.as_bytes()),
            inicio: 0,
            fin: 0,
        }),
        estado: "abierto".to_string(),
        creado: ahora.clone(),
        actualizado: ahora,
        autor: autor.to_string(),
        huerfano: false,
        mensajes: vec![Mensaje::nuevo("comentario", autor, texto)?],
    };
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Crea una sticky note sin ancla.
pub fn crear_sticky(raiz: &Path, documento: &str, texto: &str, autor: &str) -> Result<Comentario> {
    let texto = texto.trim();
    let autor = autor.trim();
    if texto.is_empty() || autor.is_empty() {
        return Err(Error::Input(
            "la nota y el autor no pueden estar vacíos".to_string(),
        ));
    }
    let ahora = time::ahora();
    let comentario = Comentario {
        version: VERSION,
        id: ulid::nuevo()?,
        tipo: "sticky".to_string(),
        documento: documento.to_string(),
        ancla: None,
        estado: "abierto".to_string(),
        creado: ahora.clone(),
        actualizado: ahora,
        autor: autor.to_string(),
        huerfano: false,
        mensajes: vec![Mensaje::nuevo("comentario", autor, texto)?],
    };
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Lee un hilo por ULID.
pub fn leer_comentario(raiz: &Path, id: &str) -> Result<Comentario> {
    if !ulid::es_valido(id) {
        return Err(Error::Input(format!("ULID de comentario no válido: {id}")));
    }
    let ruta = ruta_comentario(raiz, id)?;
    if ruta.exists() {
        fsutil::archivo_real(&ruta)?;
    }
    let bytes = fsutil::leer_opcional(&ruta)?
        .ok_or_else(|| Error::NotFound(format!("comentario no encontrado: {id}")))?;
    let comentario: Comentario =
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(error).with_path(&ruta))?;
    comentario.validar()?;
    Ok(comentario)
}

/// Guarda un hilo con reemplazo atómico.
pub fn guardar_comentario(raiz: &Path, comentario: &Comentario) -> Result<()> {
    comentario.validar()?;
    let ruta = ruta_comentario(raiz, &comentario.id)?;
    let bytes = serde_json::to_vec_pretty(comentario)?;
    fsutil::escribir_atomico(&ruta, &bytes)
}

/// Lista los hilos de un proyecto, opcionalmente filtrados por documento.
pub fn listar_comentarios(raiz: &Path, documento: Option<&str>) -> Result<Vec<Comentario>> {
    let directorio = directorio_comentarios(raiz, false)?;
    let Some(directorio) = directorio else {
        return Ok(Vec::new());
    };
    let mut comentarios = Vec::new();
    for entrada in
        std::fs::read_dir(&directorio).map_err(|error| Error::Io(error).with_path(&directorio))?
    {
        let entrada = entrada.map_err(|error| Error::Io(error).with_path(&directorio))?;
        let ruta = entrada.path();
        if !fsutil::es_extension(&ruta, "json") {
            continue;
        }
        let bytes = std::fs::read(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
        let Ok(comentario) = serde_json::from_slice::<Comentario>(&bytes) else {
            let _ = fsutil::apartar_sidecar(raiz, &ruta);
            continue;
        };
        if comentario.validar().is_err() {
            let _ = fsutil::apartar_sidecar(raiz, &ruta);
            continue;
        }
        if documento.is_none_or(|valor| valor == comentario.documento) {
            comentarios.push(comentario);
        }
    }
    comentarios.sort_by(|a, b| a.creado.cmp(&b.creado).then(a.id.cmp(&b.id)));
    Ok(comentarios)
}

/// Añade una respuesta o mensaje al hilo.
pub fn agregar_mensaje(
    raiz: &Path,
    id: &str,
    tipo: &str,
    autor: &str,
    texto: &str,
) -> Result<Comentario> {
    let mut comentario = leer_comentario(raiz, id)?;
    let mensaje = Mensaje::nuevo(tipo, autor, texto)?;
    comentario.mensajes.push(mensaje);
    comentario.actualizado = time::ahora();
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Cambia el estado del hilo y deja un mensaje de resolución.
pub fn resolver_comentario(raiz: &Path, id: &str, autor: &str) -> Result<Comentario> {
    let mut comentario = leer_comentario(raiz, id)?;
    if comentario.estado == "resuelto" {
        return Ok(comentario);
    }
    comentario
        .mensajes
        .push(Mensaje::nuevo("resolucion", autor, "Hilo resuelto.")?);
    comentario.estado = "resuelto".to_string();
    comentario.actualizado = time::ahora();
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Marca o desmarca un hilo como huérfano.
pub fn marcar_huerfano(raiz: &Path, id: &str, huerfano: bool) -> Result<Comentario> {
    let mut comentario = leer_comentario(raiz, id)?;
    comentario.huerfano = huerfano;
    comentario.actualizado = time::ahora();
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Recalcula los huérfanos de un documento a partir de su texto actual.
///
/// Un ancla se considera presente si su texto marcado aparece en el documento.
/// No se borra ningún hilo: solo se actualiza `huerfano`.
pub fn sincronizar_huerfanos(raiz: &Path, documento: &str, texto: &str) -> Result<Vec<Comentario>> {
    let mut comentarios = listar_comentarios(raiz, Some(documento))?;
    for comentario in &mut comentarios {
        let Some(ancla) = &comentario.ancla else {
            continue;
        };
        let encontrado = buscar_ancla(texto, &ancla.texto);
        let huerfano = encontrado.is_none();
        if let Some((inicio, fin)) = encontrado {
            comentario.ancla = Some(Ancla {
                texto: ancla.texto.clone(),
                hash: hash::hash(ancla.texto.as_bytes()),
                inicio,
                fin,
            });
        }
        if huerfano != comentario.huerfano || encontrado.is_some() {
            comentario.huerfano = huerfano;
            comentario.actualizado = time::ahora();
            comentario.validar()?;
            guardar_comentario(raiz, comentario)?;
        }
    }
    Ok(comentarios)
}

fn buscar_ancla(texto: &str, ancla: &str) -> Option<(usize, usize)> {
    let empieza_alfanumerico = ancla
        .chars()
        .next()
        .is_some_and(|caracter| caracter.is_alphanumeric());
    let termina_alfanumerico = ancla
        .chars()
        .last()
        .is_some_and(|caracter| caracter.is_alphanumeric());
    for (inicio, _) in texto.match_indices(ancla) {
        let antes_ok = !empieza_alfanumerico
            || texto[..inicio]
                .chars()
                .next_back()
                .is_none_or(|caracter| !caracter.is_alphanumeric());
        let fin = inicio + ancla.len();
        let despues_ok = !termina_alfanumerico
            || texto[fin..]
                .chars()
                .next()
                .is_none_or(|caracter| !caracter.is_alphanumeric());
        if antes_ok && despues_ok {
            return Some((inicio, fin));
        }
    }
    None
}

fn directorio_comentarios(raiz: &Path, crear: bool) -> Result<Option<PathBuf>> {
    let escritor = fsutil::validar_escritor(raiz)?
        .ok_or_else(|| Error::Project("el proyecto no tiene .escritor".to_string()))?;
    let directorio = escritor.join("comentarios");
    if !directorio.exists() {
        if !crear {
            return Ok(None);
        }
        std::fs::create_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?;
        fsutil::fsync_dir(&escritor)?;
    }
    fsutil::directorio_real(&directorio).map(Some)
}

fn ruta_comentario(raiz: &Path, id: &str) -> Result<PathBuf> {
    if !ulid::es_valido(id) {
        return Err(Error::Input(format!("ULID de comentario no válido: {id}")));
    }
    let directorio = directorio_comentarios(raiz, true)?
        .ok_or_else(|| Error::Project("no se pudo crear .escritor/comentarios".to_string()))?;
    Ok(directorio.join(format!("{id}.json")))
}

/// Reabre un hilo resuelto.
pub fn reabrir_comentario(raiz: &Path, id: &str, autor: &str) -> Result<Comentario> {
    let mut comentario = leer_comentario(raiz, id)?;
    if comentario.estado == "abierto" {
        return Ok(comentario);
    }
    comentario
        .mensajes
        .push(Mensaje::nuevo("resolucion", autor, "Hilo reabierto.")?);
    comentario.estado = "abierto".to_string();
    comentario.actualizado = time::ahora();
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Edita un mensaje añadiendo un evento `edicion` con el texto nuevo.
pub fn editar_mensaje(
    raiz: &Path,
    id: &str,
    mensaje_id: &str,
    autor: &str,
    texto: &str,
) -> Result<Comentario> {
    let mut comentario = leer_comentario(raiz, id)?;
    if !comentario
        .mensajes
        .iter()
        .any(|mensaje| mensaje.id == mensaje_id)
    {
        return Err(Error::NotFound(format!(
            "mensaje no encontrado: {mensaje_id}"
        )));
    }
    let mut mensaje = Mensaje::nuevo("edicion", autor, texto)?;
    mensaje.reemplaza = Some(mensaje_id.to_string());
    comentario.mensajes.push(mensaje);
    comentario.actualizado = time::ahora();
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Borra un mensaje añadiendo un evento `borrado` (no destruye el historial).
pub fn borrar_mensaje(raiz: &Path, id: &str, mensaje_id: &str, autor: &str) -> Result<Comentario> {
    let mut comentario = leer_comentario(raiz, id)?;
    if !comentario
        .mensajes
        .iter()
        .any(|mensaje| mensaje.id == mensaje_id)
    {
        return Err(Error::NotFound(format!(
            "mensaje no encontrado: {mensaje_id}"
        )));
    }
    let mut mensaje = Mensaje::nuevo("borrado", autor, "Mensaje borrado.")?;
    mensaje.reemplaza = Some(mensaje_id.to_string());
    comentario.mensajes.push(mensaje);
    comentario.actualizado = time::ahora();
    comentario.validar()?;
    guardar_comentario(raiz, &comentario)?;
    Ok(comentario)
}

/// Archiva un hilo: mueve el sidecar a `comentarios/archivados/`.
///
/// No borra el ancla del Markdown; la UI debe quitarla antes si corresponde.
pub fn archivar_comentario(raiz: &Path, id: &str) -> Result<()> {
    leer_comentario(raiz, id)?;
    let directorio = directorio_comentarios(raiz, true)?
        .ok_or_else(|| Error::Project("no se pudo abrir .escritor/comentarios".to_string()))?;
    let archivados = directorio.join("archivados");
    if !archivados.exists() {
        std::fs::create_dir(&archivados)
            .map_err(|error| Error::Io(error).with_path(&archivados))?;
        fsutil::fsync_dir(&directorio)?;
    }
    let archivados = fsutil::directorio_real(&archivados)?;
    let origen = directorio.join(format!("{id}.json"));
    let destino = archivados.join(format!("{id}.json"));
    std::fs::rename(&origen, &destino).map_err(|error| Error::Io(error).with_path(&origen))?;
    fsutil::fsync_dir(&directorio)?;
    fsutil::fsync_dir(&archivados)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raiz() -> tempfile::TempDir {
        let temporal = tempfile::tempdir().expect("temporal");
        std::fs::create_dir(temporal.path().join(".escritor")).expect(".escritor");
        std::fs::create_dir(temporal.path().join("manuscrito")).expect("manuscrito");
        std::fs::create_dir(temporal.path().join("manuscrito/01-libro")).expect("libro");
        std::fs::create_dir(temporal.path().join("manuscrito/01-libro/01-capitulo"))
            .expect("capítulo");
        std::fs::write(
            temporal
                .path()
                .join("manuscrito/01-libro/01-capitulo/001-escena.md"),
            b"texto",
        )
        .expect("escena");
        temporal
    }

    #[test]
    fn crea_y_lee_comentario() {
        let temporal = raiz();
        let comentario = crear_comentario(
            temporal.path(),
            "manuscrito/01-libro/01-capitulo/001-escena.md",
            "revisar",
            "texto marcado",
            "local",
        )
        .expect("crear");
        let leido = leer_comentario(temporal.path(), &comentario.id).expect("leer");
        assert_eq!(leido.mensajes.len(), 1);
        assert_eq!(leido.estado, "abierto");
    }

    #[test]
    fn respuesta_y_resolucion() {
        let temporal = raiz();
        let comentario = crear_comentario(
            temporal.path(),
            "manuscrito/01-libro/01-capitulo/001-escena.md",
            "revisar",
            "texto",
            "local",
        )
        .expect("crear");
        let con_respuesta = agregar_mensaje(
            temporal.path(),
            &comentario.id,
            "respuesta",
            "local",
            "de acuerdo",
        )
        .expect("responder");
        assert_eq!(con_respuesta.mensajes.len(), 2);
        let resuelto =
            resolver_comentario(temporal.path(), &comentario.id, "local").expect("resolver");
        assert_eq!(resuelto.estado, "resuelto");
        assert_eq!(resuelto.mensajes.len(), 3);
    }

    #[test]
    fn sticky_sin_ancla() {
        let temporal = raiz();
        let sticky = crear_sticky(
            temporal.path(),
            "manuscrito/01-libro/01-capitulo/001-escena.md",
            "idea suelta",
            "local",
        )
        .expect("sticky");
        assert!(sticky.ancla.is_none());
        assert_eq!(sticky.tipo, "sticky");
    }

    #[test]
    fn huerfano_se_marca_y_se_recupera() {
        let temporal = raiz();
        let documento = "manuscrito/01-libro/01-capitulo/001-escena.md";
        let comentario = crear_comentario(temporal.path(), documento, "revisar", "texto", "local")
            .expect("crear");
        let huerfanos =
            sincronizar_huerfanos(temporal.path(), documento, "otra cosa").expect("sync");
        assert!(huerfanos[0].huerfano);
        let recuperados =
            sincronizar_huerfanos(temporal.path(), documento, "el texto vuelve").expect("sync");
        assert!(!recuperados[0].huerfano);
        assert_eq!(recuperados[0].id, comentario.id);
    }

    #[test]
    fn lista_por_documento() {
        let temporal = raiz();
        let documento = "manuscrito/01-libro/01-capitulo/001-escena.md";
        crear_comentario(temporal.path(), documento, "uno", "texto", "local").expect("uno");
        crear_comentario(temporal.path(), documento, "dos", "texto", "local").expect("dos");
        let lista = listar_comentarios(temporal.path(), Some(documento)).expect("listar");
        assert_eq!(lista.len(), 2);
    }
}
