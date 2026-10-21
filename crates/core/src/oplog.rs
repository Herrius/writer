//! Log de operaciones append-only con checksum por línea.
//!
//! Formato: `<json-canónico>\t<blake3-hex>\n`. El checksum se calcula sobre los
//! bytes exactos del JSON. Una última línea truncada se descarta; una línea
//! corrupta en medio del log es un error de recuperación.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::fsutil;
use crate::hash;
use crate::time;

/// Un cambio de contenido sobre una ruta relativa.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Efecto {
    /// Ruta relativa al proyecto con separadores `/`.
    pub ruta: String,
    /// Hash del contenido después del efecto; `null` si el archivo se elimina.
    pub hash_nuevo: Option<String>,
    /// Hash del contenido anterior; `null` si el archivo no existía.
    pub hash_anterior: Option<String>,
}

/// Entrada `PREPARE`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Prepare {
    /// Número de secuencia del log.
    pub seq: u64,
    /// Siempre `PREPARE`.
    pub tipo: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// ULID de la operación.
    pub op_id: String,
    /// Tarea de IA, si existe.
    #[serde(default)]
    pub task_id: Option<String>,
    /// Revisión base.
    pub base_rev: u64,
    /// Revisión objetivo (`base_rev + 1`).
    pub target_rev: u64,
    /// `humano`, `ia` o `externo`.
    pub autor: String,
    /// Alcance de la operación.
    pub scope: String,
    /// Modelo de IA, si existe.
    #[serde(default)]
    pub modelo: Option<String>,
    /// Lista de cambios.
    pub efectos: Vec<Efecto>,
}

/// Entrada `COMMIT`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    /// Número de secuencia.
    pub seq: u64,
    /// Siempre `COMMIT`.
    pub tipo: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Operación confirmada.
    pub op_id: String,
    /// Revisión confirmada.
    pub target_rev: u64,
}

/// Entrada `ABORT`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Abort {
    /// Número de secuencia.
    pub seq: u64,
    /// Siempre `ABORT`.
    pub tipo: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Operación revertida.
    pub op_id: String,
    /// Motivo legible.
    pub motivo: String,
}

/// Entrada del log.
#[derive(Clone, Debug)]
pub enum Entrada {
    /// Operación preparada.
    Prepare(Prepare),
    /// Operación confirmada.
    Commit(Commit),
    /// Operación revertida.
    Abort(Abort),
    /// Tipo desconocido con checksum válido (compatibilidad futura).
    Desconocida {
        /// Secuencia.
        seq: u64,
        /// Tipo original.
        tipo: String,
    },
}

impl Entrada {
    /// Secuencia de la entrada.
    pub fn seq(&self) -> u64 {
        match self {
            Entrada::Prepare(valor) => valor.seq,
            Entrada::Commit(valor) => valor.seq,
            Entrada::Abort(valor) => valor.seq,
            Entrada::Desconocida { seq, .. } => *seq,
        }
    }

    /// Tipo de la entrada.
    pub fn tipo(&self) -> &str {
        match self {
            Entrada::Prepare(_) => "PREPARE",
            Entrada::Commit(_) => "COMMIT",
            Entrada::Abort(_) => "ABORT",
            Entrada::Desconocida { tipo, .. } => tipo,
        }
    }
}

/// Operación confirmada: `PREPARE` + `COMMIT`.
#[derive(Clone, Debug)]
pub struct Operacion {
    /// Preparación.
    pub prepare: Prepare,
    /// Confirmación.
    pub commit: Commit,
}

/// Log abierto.
#[derive(Debug)]
pub struct Log {
    ruta: PathBuf,
    entradas: Vec<Entrada>,
    largo_valido: u64,
    siguiente_seq: u64,
    necesita_salto: bool,
}

/// Entrada para crear un `PREPARE` con la secuencia y fecha actuales.
#[derive(Debug)]
pub struct PrepareInput<'a> {
    /// ULID de la operación.
    pub op_id: &'a str,
    /// Revisión base.
    pub base_rev: u64,
    /// Autor.
    pub autor: &'a str,
    /// Alcance.
    pub scope: &'a str,
    /// Tarea de IA.
    pub task_id: Option<&'a str>,
    /// Modelo de IA.
    pub modelo: Option<&'a str>,
    /// Efectos.
    pub efectos: Vec<Efecto>,
}

impl Log {
    /// Crea un log vacío.
    pub fn crear_vacio(ruta: &Path) -> Result<Self> {
        if let Some(padre) = ruta.parent() {
            std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
        }
        fsutil::escribir_atomico(ruta, b"")?;
        Ok(Self {
            ruta: ruta.to_path_buf(),
            entradas: Vec::new(),
            largo_valido: 0,
            siguiente_seq: 1,
            necesita_salto: false,
        })
    }

    /// Carga y valida un log. Si la última línea está truncada, la descarta.
    pub fn cargar(ruta: &Path) -> Result<Self> {
        let bytes = std::fs::read(ruta).map_err(|error| Error::Io(error).with_path(ruta))?;
        let mut entradas = Vec::new();
        let mut largo_valido = 0_u64;
        let mut offset = 0_usize;
        let mut siguiente_seq = 1_u64;
        let mut necesita_salto = false;
        let mut truncado = false;

        for linea in bytes.split_inclusive(|byte| *byte == b'\n') {
            let es_ultima = offset + linea.len() == bytes.len();
            let tiene_salto = linea.ends_with(b"\n");
            let sin_salto = linea.strip_suffix(b"\n").unwrap_or(linea);
            let truncada = es_ultima && !tiene_salto;
            if sin_salto.is_empty() {
                if es_ultima {
                    break;
                }
                return Err(corrupcion(ruta, "línea vacía en medio del log"));
            }
            let Some(posicion) = sin_salto.iter().rposition(|byte| *byte == b'\t') else {
                if truncada {
                    truncado = true;
                    break;
                }
                return Err(corrupcion(ruta, "línea del log sin tabulador"));
            };
            let json = &sin_salto[..posicion];
            let checksum = &sin_salto[posicion + 1..];
            let checksum_valido = checksum.len() == 64
                && checksum
                    .iter()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
            if !checksum_valido || hash::hash_hex(json) != String::from_utf8_lossy(checksum) {
                if truncada {
                    truncado = true;
                    break;
                }
                return Err(corrupcion(ruta, "checksum inválido en el log"));
            }
            let valor: Value = match serde_json::from_slice(json) {
                Ok(valor) => valor,
                Err(_) if truncada => {
                    truncado = true;
                    break;
                }
                Err(error) => {
                    return Err(corrupcion(
                        ruta,
                        &format!("JSON inválido en el log: {error}"),
                    ))
                }
            };
            let seq = valor
                .get("seq")
                .and_then(Value::as_u64)
                .ok_or_else(|| corrupcion(ruta, "entrada sin 'seq' entero"))?;
            if seq != siguiente_seq {
                return Err(corrupcion(
                    ruta,
                    &format!("salto de secuencia: se esperaba {siguiente_seq} y llegó {seq}"),
                ));
            }
            let tipo = valor
                .get("tipo")
                .and_then(Value::as_str)
                .ok_or_else(|| corrupcion(ruta, "entrada sin 'tipo' textual"))?
                .to_string();
            let entrada = match tipo.as_str() {
                "PREPARE" => {
                    let prepare: Prepare = serde_json::from_value(valor)
                        .map_err(|error| corrupcion(ruta, &format!("PREPARE inválido: {error}")))?;
                    Entrada::Prepare(prepare)
                }
                "COMMIT" => {
                    let commit: Commit = serde_json::from_value(valor)
                        .map_err(|error| corrupcion(ruta, &format!("COMMIT inválido: {error}")))?;
                    Entrada::Commit(commit)
                }
                "ABORT" => {
                    let abort: Abort = serde_json::from_value(valor)
                        .map_err(|error| corrupcion(ruta, &format!("ABORT inválido: {error}")))?;
                    Entrada::Abort(abort)
                }
                _ => Entrada::Desconocida { seq, tipo },
            };
            entradas.push(entrada);
            siguiente_seq = seq + 1;
            offset += linea.len();
            largo_valido = offset as u64;
        }

        if truncado {
            necesita_salto = false;
        } else if !bytes.is_empty() && !bytes.ends_with(b"\n") {
            necesita_salto = true;
        }

        let log = Self {
            ruta: ruta.to_path_buf(),
            entradas,
            largo_valido,
            siguiente_seq,
            necesita_salto,
        };
        log.validar()?;
        Ok(log)
    }

    fn validar(&self) -> Result<()> {
        let mut preparadas: BTreeMap<&str, &Prepare> = BTreeMap::new();
        let mut confirmadas: BTreeSet<&str> = BTreeSet::new();
        let mut abortadas: BTreeSet<&str> = BTreeSet::new();
        let mut revision = 0_u64;
        for entrada in &self.entradas {
            match entrada {
                Entrada::Prepare(prepare) => {
                    if prepare.tipo != "PREPARE" {
                        return Err(corrupcion(&self.ruta, "PREPARE con tipo incorrecto"));
                    }
                    if !matches!(prepare.autor.as_str(), "humano" | "ia" | "externo") {
                        return Err(corrupcion(&self.ruta, "autor de PREPARE no válido"));
                    }
                    if prepare.scope.trim().is_empty() {
                        return Err(corrupcion(&self.ruta, "PREPARE sin scope"));
                    }
                    if crate::time::parse_rfc3339(&prepare.ts).is_err() {
                        return Err(corrupcion(&self.ruta, "ts de PREPARE no es RFC 3339"));
                    }
                    if !crate::ulid::es_valido(&prepare.op_id) {
                        return Err(corrupcion(&self.ruta, "op_id de PREPARE no es ULID"));
                    }
                    if prepare.target_rev != prepare.base_rev + 1 {
                        return Err(corrupcion(&self.ruta, "target_rev no es base_rev + 1"));
                    }
                    if prepare.efectos.is_empty() {
                        return Err(corrupcion(&self.ruta, "PREPARE sin efectos"));
                    }
                    let mut rutas = BTreeSet::new();
                    for efecto in &prepare.efectos {
                        validar_efecto(&self.ruta, efecto)?;
                        if !rutas.insert(efecto.ruta.as_str()) {
                            return Err(corrupcion(
                                &self.ruta,
                                "PREPARE con ruta de efecto duplicada",
                            ));
                        }
                    }
                    if preparadas.insert(&prepare.op_id, prepare).is_some() {
                        return Err(corrupcion(&self.ruta, "PREPARE duplicado"));
                    }
                }
                Entrada::Commit(commit) => {
                    if commit.tipo != "COMMIT" {
                        return Err(corrupcion(&self.ruta, "COMMIT con tipo incorrecto"));
                    }
                    if crate::time::parse_rfc3339(&commit.ts).is_err() {
                        return Err(corrupcion(&self.ruta, "ts de COMMIT no es RFC 3339"));
                    }
                    let prepare = preparadas.get(commit.op_id.as_str()).ok_or_else(|| {
                        corrupcion(&self.ruta, "COMMIT sin PREPARE correspondiente")
                    })?;
                    if commit.target_rev != prepare.target_rev {
                        return Err(corrupcion(&self.ruta, "COMMIT con target_rev distinto"));
                    }
                    if prepare.base_rev != revision {
                        return Err(corrupcion(
                            &self.ruta,
                            &format!(
                                "revisión base {} no continúa la revisión {revision}",
                                prepare.base_rev
                            ),
                        ));
                    }
                    revision = prepare.target_rev;
                    if !confirmadas.insert(commit.op_id.as_str()) {
                        return Err(corrupcion(&self.ruta, "COMMIT duplicado"));
                    }
                }
                Entrada::Abort(abort) => {
                    if abort.tipo != "ABORT" {
                        return Err(corrupcion(&self.ruta, "ABORT con tipo incorrecto"));
                    }
                    if crate::time::parse_rfc3339(&abort.ts).is_err() {
                        return Err(corrupcion(&self.ruta, "ts de ABORT no es RFC 3339"));
                    }
                    if !preparadas.contains_key(abort.op_id.as_str()) {
                        return Err(corrupcion(&self.ruta, "ABORT sin PREPARE"));
                    }
                    abortadas.insert(&abort.op_id);
                }
                Entrada::Desconocida { .. } => {}
            }
        }
        for op_id in confirmadas {
            if abortadas.contains(op_id) {
                return Err(corrupcion(
                    &self.ruta,
                    "una operación no puede estar confirmada y abortada",
                ));
            }
        }
        Ok(())
    }

    /// Entradas válidas del log.
    pub fn entradas(&self) -> &[Entrada] {
        &self.entradas
    }

    /// Secuencia que se usará en la próxima entrada.
    pub fn siguiente_seq(&self) -> u64 {
        self.siguiente_seq
    }

    /// Revisión confirmada actual.
    pub fn revision_actual(&self) -> u64 {
        self.operaciones()
            .last()
            .map(|operacion| operacion.prepare.target_rev)
            .unwrap_or(0)
    }

    /// Operaciones confirmadas en orden.
    pub fn operaciones(&self) -> Vec<Operacion> {
        let mut preparadas: BTreeMap<&str, &Prepare> = BTreeMap::new();
        let mut salida = Vec::new();
        for entrada in &self.entradas {
            match entrada {
                Entrada::Prepare(prepare) => {
                    preparadas.insert(&prepare.op_id, prepare);
                }
                Entrada::Commit(commit) => {
                    if let Some(prepare) = preparadas.get(commit.op_id.as_str()) {
                        salida.push(Operacion {
                            prepare: (*prepare).clone(),
                            commit: commit.clone(),
                        });
                    }
                }
                _ => {}
            }
        }
        salida.sort_by_key(|operacion| operacion.prepare.target_rev);
        salida
    }

    /// Operaciones con `PREPARE` pero sin `COMMIT` ni `ABORT`.
    ///
    /// Un `ABORT` válido es terminal: solo se anexa después de que la reversión
    /// terminó y quedó persistida. Si el proceso muere antes de escribirlo, el
    /// `PREPARE` sigue pendiente y se vuelve a revertir (idempotente).
    pub fn pendientes(&self) -> Vec<Prepare> {
        let confirmadas: BTreeSet<&str> = self
            .entradas
            .iter()
            .filter_map(|entrada| match entrada {
                Entrada::Commit(commit) => Some(commit.op_id.as_str()),
                _ => None,
            })
            .collect();
        let abortadas: BTreeSet<&str> = self
            .entradas
            .iter()
            .filter_map(|entrada| match entrada {
                Entrada::Abort(abort) => Some(abort.op_id.as_str()),
                _ => None,
            })
            .collect();
        self.entradas
            .iter()
            .filter_map(|entrada| match entrada {
                Entrada::Prepare(prepare)
                    if !confirmadas.contains(prepare.op_id.as_str())
                        && !abortadas.contains(prepare.op_id.as_str()) =>
                {
                    Some(prepare.clone())
                }
                _ => None,
            })
            .collect()
    }

    /// Indica si una operación ya tiene `ABORT`.
    pub fn abortada(&self, op_id: &str) -> bool {
        self.entradas.iter().any(|entrada| match entrada {
            Entrada::Abort(abort) => abort.op_id == op_id,
            _ => false,
        })
    }

    /// Entradas `ABORT` del log.
    pub fn abortadas(&self) -> Vec<&Abort> {
        self.entradas
            .iter()
            .filter_map(|entrada| match entrada {
                Entrada::Abort(abort) => Some(abort),
                _ => None,
            })
            .collect()
    }

    /// Hashes referenciados por efectos (hex, sin prefijo).
    pub fn blobs_referenciados(&self) -> BTreeSet<String> {
        let mut salida = BTreeSet::new();
        for entrada in &self.entradas {
            if let Entrada::Prepare(prepare) = entrada {
                for efecto in &prepare.efectos {
                    for hash in [&efecto.hash_anterior, &efecto.hash_nuevo]
                        .into_iter()
                        .flatten()
                    {
                        if let Ok(hex) = hash::hex(hash) {
                            salida.insert(hex.to_string());
                        }
                    }
                }
            }
        }
        salida
    }

    /// Añade un `PREPARE` con la siguiente secuencia.
    pub fn agregar_prepare(&mut self, entrada: PrepareInput<'_>) -> Result<Prepare> {
        let PrepareInput {
            op_id,
            base_rev,
            autor,
            scope,
            task_id,
            modelo,
            efectos,
        } = entrada;
        let prepare = Prepare {
            seq: self.siguiente_seq,
            tipo: "PREPARE".to_string(),
            ts: time::ahora(),
            op_id: op_id.to_string(),
            task_id: task_id.map(str::to_string),
            base_rev,
            target_rev: base_rev + 1,
            autor: autor.to_string(),
            scope: scope.to_string(),
            modelo: modelo.map(str::to_string),
            efectos,
        };
        self.anexar(serde_json::to_value(&prepare)?)?;
        Ok(prepare)
    }

    /// Añade un `COMMIT`.
    pub fn agregar_commit(&mut self, op_id: &str, target_rev: u64) -> Result<Commit> {
        let commit = Commit {
            seq: self.siguiente_seq,
            tipo: "COMMIT".to_string(),
            ts: time::ahora(),
            op_id: op_id.to_string(),
            target_rev,
        };
        self.anexar(serde_json::to_value(&commit)?)?;
        Ok(commit)
    }

    /// Añade un `ABORT`.
    pub fn agregar_abort(&mut self, op_id: &str, motivo: &str) -> Result<Abort> {
        let abort = Abort {
            seq: self.siguiente_seq,
            tipo: "ABORT".to_string(),
            ts: time::ahora(),
            op_id: op_id.to_string(),
            motivo: motivo.to_string(),
        };
        self.anexar(serde_json::to_value(&abort)?)?;
        Ok(abort)
    }

    fn anexar(&mut self, valor: Value) -> Result<()> {
        let json = json_canonico(&valor)?;
        let checksum = hash::hash_hex(json.as_bytes());
        let offset = self.largo_valido;
        let mut linea = Vec::with_capacity(json.len() + 66);
        if self.necesita_salto {
            linea.push(b'\n');
        }
        linea.extend_from_slice(json.as_bytes());
        linea.push(b'\t');
        linea.extend_from_slice(checksum.as_bytes());
        linea.push(b'\n');
        fsutil::escribir_en(&self.ruta, offset, &linea)?;
        self.largo_valido = offset + linea.len() as u64;
        self.necesita_salto = false;
        self.siguiente_seq += 1;
        let entrada = match valor.get("tipo").and_then(Value::as_str) {
            Some("PREPARE") => Entrada::Prepare(serde_json::from_value(valor)?),
            Some("COMMIT") => Entrada::Commit(serde_json::from_value(valor)?),
            Some("ABORT") => Entrada::Abort(serde_json::from_value(valor)?),
            _ => {
                return Err(Error::Integrity(
                    "se intentó anexar una entrada sin tipo conocido".into(),
                ))
            }
        };
        self.entradas.push(entrada);
        Ok(())
    }

    /// Fuerza el `fsync` del log.
    pub fn sincronizar(&self) -> Result<()> {
        let archivo = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.ruta)
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        archivo
            .sync_all()
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        Ok(())
    }

    /// Ruta del log.
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }
}

fn validar_efecto(ruta: &Path, efecto: &Efecto) -> Result<()> {
    if efecto.ruta.is_empty()
        || efecto.ruta.starts_with('/')
        || efecto
            .ruta
            .split('/')
            .any(|parte| parte.is_empty() || parte == "." || parte == "..")
    {
        return Err(corrupcion(
            ruta,
            &format!("ruta de efecto no válida: {}", efecto.ruta),
        ));
    }
    if efecto.hash_nuevo.is_none() && efecto.hash_anterior.is_none() {
        return Err(corrupcion(ruta, "efecto sin hashes"));
    }
    for hash in [&efecto.hash_anterior, &efecto.hash_nuevo]
        .into_iter()
        .flatten()
    {
        if !crate::hash::es_hash(hash) {
            return Err(corrupcion(
                ruta,
                &format!("hash de efecto no válido: {hash}"),
            ));
        }
    }
    Ok(())
}

fn corrupcion(ruta: &Path, mensaje: &str) -> Error {
    Error::Recovery(format!("{}: {mensaje}", ruta.display()))
}

/// Serializa un valor a JSON canónico: claves ordenadas, sin espacios.
pub fn json_canonico(valor: &Value) -> Result<String> {
    let canonico = canonicalizar(valor.clone());
    Ok(serde_json::to_string(&canonico)?)
}

fn canonicalizar(valor: Value) -> Value {
    match valor {
        Value::Object(mapa) => {
            let mut ordenado = BTreeMap::new();
            for (clave, valor) in mapa {
                ordenado.insert(clave, canonicalizar(valor));
            }
            Value::Object(ordenado.into_iter().collect())
        }
        Value::Array(valores) => Value::Array(valores.into_iter().map(canonicalizar).collect()),
        otro => otro,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn efecto(ruta: &str, anterior: Option<&str>, nuevo: Option<&str>) -> Efecto {
        Efecto {
            ruta: ruta.to_string(),
            hash_nuevo: nuevo.map(str::to_string),
            hash_anterior: anterior.map(str::to_string),
        }
    }

    fn entrada<'a>(op_id: &'a str, base_rev: u64) -> PrepareInput<'a> {
        PrepareInput {
            op_id,
            base_rev,
            autor: "humano",
            scope: "prosa",
            task_id: None,
            modelo: None,
            efectos: vec![efecto(
                "manuscrito/01-libro/01-capitulo/001.md",
                None,
                Some("blake3:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
            )],
        }
    }

    #[test]
    fn json_canonico_ordena_claves() {
        let valor = serde_json::json!({"b": 2, "a": 1, "c": {"z": true, "y": null}});
        assert_eq!(
            json_canonico(&valor).expect("json"),
            r#"{"a":1,"b":2,"c":{"y":null,"z":true}}"#
        );
    }

    #[test]
    fn log_ida_y_vuelta_y_commit() {
        let directorio = tempfile::tempdir().expect("temporal");
        let ruta = directorio.path().join("operaciones.log");
        let mut log = Log::crear_vacio(&ruta).expect("crear");
        let op_id = "01KDVDNGTR27H6P3AMKDQG6STT";
        log.agregar_prepare(entrada(op_id, 0)).expect("prepare");
        log.agregar_commit(op_id, 1).expect("commit");
        drop(log);

        let log = Log::cargar(&ruta).expect("cargar");
        assert_eq!(log.revision_actual(), 1);
        assert_eq!(log.operaciones().len(), 1);
        assert_eq!(log.siguiente_seq(), 3);
    }

    #[test]
    fn ultima_linea_truncada_se_descarta_y_se_reescribe() {
        let directorio = tempfile::tempdir().expect("temporal");
        let ruta = directorio.path().join("operaciones.log");
        let mut log = Log::crear_vacio(&ruta).expect("crear");
        let op_id = "01KDVDNGTR27H6P3AMKDQG6STT";
        log.agregar_prepare(entrada(op_id, 0)).expect("prepare");
        let largo = std::fs::metadata(&ruta).expect("meta").len();
        let mut bytes = std::fs::read(&ruta).expect("leer");
        bytes.push(b'x');
        std::fs::write(&ruta, bytes).expect("truncar");
        let mut log = Log::cargar(&ruta).expect("cargar truncado");
        assert_eq!(log.largo_valido, largo);
        log.agregar_abort(op_id, "recuperacion").expect("abort");
        drop(log);
        let log = Log::cargar(&ruta).expect("recargar");
        assert_eq!(log.pendientes().len(), 0);
    }

    #[test]
    fn ultima_linea_completa_con_checksum_corrupto_es_error() {
        let directorio = tempfile::tempdir().expect("temporal");
        let ruta = directorio.path().join("operaciones.log");
        let mut log = Log::crear_vacio(&ruta).expect("crear");
        let op_id = "01KDVDNGTR27H6P3AMKDQG6STT";
        log.agregar_prepare(entrada(op_id, 0)).expect("prepare");
        log.agregar_commit(op_id, 1).expect("commit");
        drop(log);

        let mut texto = std::fs::read_to_string(&ruta).expect("leer");
        assert!(texto.ends_with('\n'));
        let inicio = texto
            .trim_end_matches('\n')
            .rfind('\n')
            .map(|posicion| posicion + 1)
            .unwrap_or(0);
        let tabulador = texto[inicio..].rfind('\t').expect("tab") + inicio;
        let original = texto.as_bytes()[tabulador + 1] as char;
        let reemplazo = if original == '0' { "1" } else { "0" };
        texto.replace_range(tabulador + 1..tabulador + 2, reemplazo);
        std::fs::write(&ruta, texto).expect("corromper");

        let error = Log::cargar(&ruta).expect_err("una línea completa corrupta no es truncamiento");
        assert!(
            error.to_string().contains("checksum"),
            "error inesperado: {error}"
        );
    }

    #[test]
    fn anexar_sin_salto_final_no_introduce_nul() {
        let directorio = tempfile::tempdir().expect("temporal");
        let ruta = directorio.path().join("operaciones.log");
        let mut log = Log::crear_vacio(&ruta).expect("crear");
        let primero = "01KDVDNGTR27H6P3AMKDQG6STT";
        log.agregar_prepare(entrada(primero, 0)).expect("prepare");
        log.agregar_commit(primero, 1).expect("commit");
        drop(log);

        let texto = std::fs::read_to_string(&ruta).expect("leer");
        std::fs::write(&ruta, texto.trim_end_matches('\n')).expect("quitar salto final");

        let mut log = Log::cargar(&ruta).expect("cargar sin salto");
        let segundo = "01KDVDNGTR27H6P3AMKDQG6STV";
        log.agregar_prepare(entrada(segundo, 1))
            .expect("segundo prepare");
        drop(log);

        let bytes = std::fs::read(&ruta).expect("leer bytes");
        assert!(!bytes.contains(&0), "no debe aparecer un byte NUL");
        let log = Log::cargar(&ruta).expect("recargar");
        assert_eq!(log.pendientes().len(), 1);
        assert_eq!(log.siguiente_seq(), 4);
    }

    #[test]
    fn checksum_en_medio_es_corrupcion() {
        let directorio = tempfile::tempdir().expect("temporal");
        let ruta = directorio.path().join("operaciones.log");
        let mut log = Log::crear_vacio(&ruta).expect("crear");
        for (indice, op_id) in ["01KDVDNGTR27H6P3AMKDQG6STT", "01KDVDNGTR27H6P3AMKDQG6STV"]
            .iter()
            .enumerate()
        {
            log.agregar_prepare(entrada(op_id, indice as u64))
                .expect("prepare");
            log.agregar_commit(op_id, indice as u64 + 1)
                .expect("commit");
        }
        drop(log);
        let mut texto = std::fs::read_to_string(&ruta).expect("leer");
        let posicion = texto.find('\t').expect("tab");
        let original = texto.as_bytes()[posicion + 1] as char;
        let reemplazo = if original == '0' { "1" } else { "0" };
        texto.replace_range(posicion + 1..posicion + 2, reemplazo);
        std::fs::write(&ruta, texto).expect("corromper");
        assert!(Log::cargar(&ruta).is_err());
    }
}
