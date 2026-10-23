//! Metas, sesiones y temporizador local.
//!
//! Este módulo implementa la extensión `metas: 1` de `docs/esquema-v2.md` §4.
//! No participa en el log de operaciones del manuscrito: son métricas locales
//! del proyecto y se guardan en `.escritor/metas/` y `.escritor/sesiones/`.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::{fsutil, time, ulid};

/// Versión de la extensión de metas.
pub const VERSION: u32 = 1;

/// Presets de temporizador admitidos, en minutos.
pub const PRESETS: [u32; 5] = [5, 10, 15, 25, 30];

/// Metas locales del proyecto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Metas {
    /// Versión del documento de metas.
    pub version: u32,
    /// Meta diaria de palabras.
    pub diaria: Option<MetaDiaria>,
    /// Meta total del libro.
    pub libro: Option<MetaLibro>,
    /// Desafío con fecha límite.
    pub desafio: Option<Desafio>,
    /// Metas archivadas.
    pub archivadas: Vec<MetaArchivada>,
}

impl Default for Metas {
    fn default() -> Self {
        Self {
            version: VERSION,
            diaria: None,
            libro: None,
            desafio: None,
            archivadas: Vec::new(),
        }
    }
}

/// Meta diaria.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaDiaria {
    /// Palabras objetivo por día.
    pub objetivo: i64,
    /// Fecha de inicio (`YYYY-MM-DD`).
    pub desde: String,
}

/// Meta de libro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaLibro {
    /// Palabras objetivo del libro.
    pub objetivo: i64,
}

/// Desafío con fecha límite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Desafio {
    /// Palabras objetivo.
    pub objetivo: i64,
    /// Fecha límite (`YYYY-MM-DD`).
    pub limite: String,
}

/// Meta archivada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaArchivada {
    /// `diaria`, `libro` o `desafio`.
    pub tipo: String,
    /// Palabras objetivo.
    pub objetivo: i64,
    /// Inicio opcional (`YYYY-MM-DD`).
    pub desde: Option<String>,
    /// Fin opcional (`YYYY-MM-DD`).
    pub hasta: Option<String>,
    /// Motivo libre.
    pub motivo: Option<String>,
}

/// Pausa de una sesión.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pausa {
    /// Inicio RFC 3339.
    pub inicio: String,
    /// Fin RFC 3339.
    pub fin: String,
}

/// Sesión de escritura.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sesion {
    /// ULID de la sesión.
    pub id: String,
    /// Inicio RFC 3339.
    pub inicio: String,
    /// Fin RFC 3339.
    pub fin: String,
    /// Preset en minutos.
    pub preset: u32,
    /// Pausas.
    #[serde(default)]
    pub pausas: Vec<Pausa>,
    /// Palabras escritas.
    pub palabras: i64,
    /// Documento asociado.
    pub documento: String,
    /// Autor local.
    pub autor: String,
}

impl Metas {
    /// Valida el documento de metas antes de leerlo o escribirlo.
    pub fn validar(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(Error::Schema(format!(
                "versión de metas no soportada: {}",
                self.version
            )));
        }
        if let Some(meta) = &self.diaria {
            validar_objetivo(meta.objetivo, "diaria.objetivo")?;
            validar_fecha(&meta.desde, "diaria.desde")?;
        }
        if let Some(meta) = &self.libro {
            validar_objetivo(meta.objetivo, "libro.objetivo")?;
        }
        if let Some(meta) = &self.desafio {
            validar_objetivo(meta.objetivo, "desafio.objetivo")?;
            validar_fecha(&meta.limite, "desafio.limite")?;
        }
        for (indice, meta) in self.archivadas.iter().enumerate() {
            if !matches!(meta.tipo.as_str(), "diaria" | "libro" | "desafio") {
                return Err(Error::Schema(format!(
                    "archivadas[{indice}].tipo no válido: {}",
                    meta.tipo
                )));
            }
            validar_objetivo(meta.objetivo, "archivadas.objetivo")?;
            if let Some(desde) = &meta.desde {
                validar_fecha(desde, "archivadas.desde")?;
            }
            if let Some(hasta) = &meta.hasta {
                validar_fecha(hasta, "archivadas.hasta")?;
            }
        }
        Ok(())
    }
}

impl Sesion {
    /// Valida una sesión antes de registrarla.
    pub fn validar(&self) -> Result<()> {
        if !ulid::es_valido(&self.id) {
            return Err(Error::Schema(format!(
                "sesión con ULID no válido: {}",
                self.id
            )));
        }
        let inicio = time::parse_rfc3339(&self.inicio)?;
        let fin = time::parse_rfc3339(&self.fin)?;
        if fin < inicio {
            return Err(Error::Schema(
                "la sesión termina antes de empezar".to_string(),
            ));
        }
        if !PRESETS.contains(&self.preset) {
            return Err(Error::Schema(format!(
                "preset de temporizador no válido: {}",
                self.preset
            )));
        }
        if self.palabras < 0 {
            return Err(Error::Schema(
                "las palabras de una sesión no pueden ser negativas".to_string(),
            ));
        }
        if self.documento.trim().is_empty() {
            return Err(Error::Schema(
                "la sesión debe indicar un documento".to_string(),
            ));
        }
        fsutil::validar_ruta_documento(&self.documento)?;
        if self.autor.trim().is_empty() || self.autor.len() > 200 {
            return Err(Error::Schema(
                "el autor de la sesión no es válido".to_string(),
            ));
        }
        let mut anterior = inicio;
        for pausa in &self.pausas {
            let desde = time::parse_rfc3339(&pausa.inicio)?;
            let hasta = time::parse_rfc3339(&pausa.fin)?;
            if desde < anterior || hasta < desde || hasta > fin {
                return Err(Error::Schema(
                    "las pausas de la sesión no están ordenadas".to_string(),
                ));
            }
            anterior = hasta;
        }
        Ok(())
    }
}

/// Lee las metas del proyecto. Un proyecto sin metas devuelve el valor por
/// defecto.
pub fn leer_metas(raiz: &Path) -> Result<Metas> {
    let ruta = ruta_metas(raiz)?;
    if ruta.exists() {
        fsutil::archivo_real(&ruta)?;
    }
    let Some(bytes) = fsutil::leer_opcional(&ruta)? else {
        return Ok(Metas::default());
    };
    let metas: Metas =
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(error).with_path(&ruta))?;
    metas.validar()?;
    Ok(metas)
}

/// Guarda las metas con reemplazo atómico.
pub fn guardar_metas(raiz: &Path, metas: &Metas) -> Result<()> {
    metas.validar()?;
    let ruta = ruta_metas(raiz)?;
    let bytes = serde_json::to_vec_pretty(metas)?;
    fsutil::escribir_atomico(&ruta, &bytes)
}

/// Registra una sesión al final del mes correspondiente.
pub fn agregar_sesion(raiz: &Path, sesion: &Sesion) -> Result<PathBuf> {
    sesion.validar()?;
    let mes = &sesion.inicio[..7];
    validar_mes(mes)?;
    let escritor = escritor(raiz)?;
    let directorio = escritor.join("sesiones");
    if !directorio.exists() {
        std::fs::create_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?;
        fsutil::fsync_dir(&escritor)?;
    }
    let directorio = fsutil::directorio_real(&directorio)?;
    let ruta = directorio.join(format!("{mes}.jsonl"));
    if ruta.exists() {
        fsutil::archivo_real(&ruta)?;
    }
    let mut archivo = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ruta)
        .map_err(|error| Error::Io(error).with_path(&ruta))?;
    let linea = serde_json::to_vec(sesion)?;
    archivo
        .write_all(&linea)
        .and_then(|()| archivo.write_all(b"\n"))
        .map_err(|error| Error::Io(error).with_path(&ruta))?;
    archivo
        .sync_all()
        .map_err(|error| Error::Io(error).with_path(&ruta))?;
    fsutil::fsync_dir(&directorio)?;
    Ok(ruta)
}

/// Lee todas las sesiones del proyecto, ordenadas por inicio.
pub fn listar_sesiones(raiz: &Path) -> Result<Vec<Sesion>> {
    let escritor = escritor(raiz)?;
    let directorio = escritor.join("sesiones");
    if !directorio.is_dir() {
        return Ok(Vec::new());
    }
    let directorio = fsutil::directorio_real(&directorio)?;
    let mut sesiones = Vec::new();
    for entrada in
        std::fs::read_dir(&directorio).map_err(|error| Error::Io(error).with_path(&directorio))?
    {
        let entrada = entrada.map_err(|error| Error::Io(error).with_path(&directorio))?;
        let ruta = entrada.path();
        if ruta.extension().and_then(|valor| valor.to_str()) != Some("jsonl") {
            continue;
        }
        let Some(mes) = ruta.file_stem().and_then(|valor| valor.to_str()) else {
            continue;
        };
        sesiones.extend(leer_sesiones(raiz, mes)?);
    }
    sesiones.sort_by(|a, b| a.inicio.cmp(&b.inicio));
    Ok(sesiones)
}

/// Lee las sesiones de un mes (`YYYY-MM`).
pub fn leer_sesiones(raiz: &Path, mes: &str) -> Result<Vec<Sesion>> {
    validar_mes(mes)?;
    let ruta = ruta_sesiones(raiz, mes)?;
    let Some(bytes) = fsutil::leer_opcional(&ruta)? else {
        return Ok(Vec::new());
    };
    let texto = String::from_utf8(bytes).map_err(|error| {
        Error::Format(format!("sesiones no son UTF-8: {error}")).with_path(&ruta)
    })?;
    let mut sesiones = Vec::new();
    for (numero, linea) in texto.lines().enumerate() {
        if linea.trim().is_empty() {
            continue;
        }
        let sesion: Sesion =
            serde_json::from_str(linea).map_err(|error| Error::Json(error).with_path(&ruta))?;
        sesion.validar().map_err(|error| {
            Error::Schema(format!("línea {}: {error}", numero + 1)).with_path(&ruta)
        })?;
        sesiones.push(sesion);
    }
    Ok(sesiones)
}

fn escritor(raiz: &Path) -> Result<PathBuf> {
    fsutil::validar_escritor(raiz)?
        .ok_or_else(|| Error::Project("el proyecto no tiene .escritor".to_string()))
}

fn ruta_metas(raiz: &Path) -> Result<PathBuf> {
    let escritor = escritor(raiz)?;
    let directorio = escritor.join("metas");
    if !directorio.exists() {
        std::fs::create_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?;
        fsutil::fsync_dir(&escritor)?;
    }
    let directorio = fsutil::directorio_real(&directorio)?;
    Ok(directorio.join("metas.json"))
}

fn ruta_sesiones(raiz: &Path, mes: &str) -> Result<PathBuf> {
    Ok(escritor(raiz)?
        .join("sesiones")
        .join(format!("{mes}.jsonl")))
}

fn validar_objetivo(valor: i64, campo: &str) -> Result<()> {
    if valor < 0 {
        return Err(Error::Schema(format!("{campo} no puede ser negativo")));
    }
    Ok(())
}

fn validar_fecha(texto: &str, campo: &str) -> Result<()> {
    let bytes = texto.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(Error::Schema(format!(
            "{campo} debe ser YYYY-MM-DD: {texto}"
        )));
    }
    let anio = texto[0..4].parse::<i32>().ok();
    let mes = texto[5..7].parse::<u32>().ok();
    let dia = texto[8..10].parse::<u32>().ok();
    let (Some(anio), Some(mes), Some(dia)) = (anio, mes, dia) else {
        return Err(Error::Schema(format!(
            "{campo} debe ser YYYY-MM-DD: {texto}"
        )));
    };
    if anio < 0 || !(1..=12).contains(&mes) || !(1..=31).contains(&dia) {
        return Err(Error::Schema(format!("{campo} fuera de rango: {texto}")));
    }
    Ok(())
}

fn validar_mes(mes: &str) -> Result<()> {
    let bytes = mes.as_bytes();
    if bytes.len() != 7 || bytes[4] != b'-' {
        return Err(Error::Schema(format!("mes no válido: {mes}")));
    }
    let Some(anio) = mes[0..4].parse::<i32>().ok() else {
        return Err(Error::Schema(format!("mes no válido: {mes}")));
    };
    let Some(mes_numero) = mes[5..7].parse::<u32>().ok() else {
        return Err(Error::Schema(format!("mes no válido: {mes}")));
    };
    if anio < 0 || !(1..=12).contains(&mes_numero) {
        return Err(Error::Schema(format!("mes fuera de rango: {mes}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raiz() -> tempfile::TempDir {
        let temporal = tempfile::tempdir().expect("temporal");
        std::fs::create_dir(temporal.path().join(".escritor")).expect(".escritor");
        temporal
    }

    fn sesion_ejemplo() -> Sesion {
        Sesion {
            id: ulid::desde_partes(1_700_000_000_000, [7; 10]),
            inicio: "2026-10-23T10:00:00.000Z".to_string(),
            fin: "2026-10-23T10:25:00.000Z".to_string(),
            preset: 25,
            pausas: vec![],
            palabras: 412,
            documento: "manuscrito/01-libro/01-capitulo/001-escena.md".to_string(),
            autor: "local".to_string(),
        }
    }

    #[test]
    fn metas_por_defecto_si_no_existen() {
        let temporal = raiz();
        let metas = leer_metas(temporal.path()).expect("metas");
        assert_eq!(metas, Metas::default());
    }

    #[test]
    fn metas_ida_y_vuelta() {
        let temporal = raiz();
        let metas = Metas {
            diaria: Some(MetaDiaria {
                objetivo: 500,
                desde: "2026-10-23".to_string(),
            }),
            libro: Some(MetaLibro { objetivo: 80_000 }),
            desafio: Some(Desafio {
                objetivo: 50_000,
                limite: "2026-12-31".to_string(),
            }),
            ..Metas::default()
        };
        guardar_metas(temporal.path(), &metas).expect("guardar");
        assert_eq!(leer_metas(temporal.path()).expect("leer"), metas);
    }

    #[test]
    fn objetivo_negativo_se_rechaza() {
        let metas = Metas {
            diaria: Some(MetaDiaria {
                objetivo: -1,
                desde: "2026-10-23".to_string(),
            }),
            ..Metas::default()
        };
        assert!(metas.validar().is_err());
    }

    #[test]
    fn sesion_ida_y_vuelta() {
        let temporal = raiz();
        let sesion = sesion_ejemplo();
        let ruta = agregar_sesion(temporal.path(), &sesion).expect("sesión");
        assert!(ruta.ends_with("sesiones/2026-10.jsonl"));
        assert_eq!(
            leer_sesiones(temporal.path(), "2026-10").expect("leer"),
            vec![sesion]
        );
    }

    #[test]
    fn sesion_invalida_se_rechaza() {
        let mut sesion = sesion_ejemplo();
        sesion.preset = 7;
        assert!(sesion.validar().is_err());

        let mut sesion = sesion_ejemplo();
        sesion.fin = "2026-10-23T09:00:00.000Z".to_string();
        assert!(sesion.validar().is_err());
    }

    #[test]
    fn sidecar_corrupto_es_error() {
        let temporal = raiz();
        let ruta = ruta_metas(temporal.path()).expect("ruta");
        std::fs::write(&ruta, b"{no es json").expect("escribir");
        assert!(leer_metas(temporal.path()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn directorio_de_metas_enlazado_se_rechaza() {
        use std::os::unix::fs::symlink;
        let temporal = raiz();
        let fuera = tempfile::tempdir().expect("fuera");
        symlink(fuera.path(), temporal.path().join(".escritor/metas")).expect("symlink");
        assert!(leer_metas(temporal.path()).is_err());
    }
}
