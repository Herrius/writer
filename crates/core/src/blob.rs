//! Almacén de blobs inmutable y deduplicado en `.escritor/objetos/`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::fsutil;
use crate::hash;

/// Almacén de contenidos históricos.
#[derive(Clone, Debug)]
pub struct BlobStore {
    raiz: PathBuf,
}

impl BlobStore {
    /// Abre (y crea si hace falta) el almacén dentro de `.escritor`.
    pub fn abrir(escritor: &Path) -> Result<Self> {
        let raiz = escritor.join("objetos");
        std::fs::create_dir_all(&raiz).map_err(|error| Error::Io(error).with_path(&raiz))?;
        fsutil::fsync_dir(escritor)?;
        Ok(Self { raiz })
    }

    /// Ruta del blob de un hash.
    pub fn ruta(&self, hash_con_prefijo: &str) -> Result<PathBuf> {
        let hex = hash::hex(hash_con_prefijo)?;
        Ok(self.raiz.join(hex))
    }

    /// Guarda unos bytes y devuelve su hash. Si ya existe, no reescribe.
    pub fn poner(&self, bytes: &[u8]) -> Result<String> {
        let valor = hash::hash(bytes);
        let ruta = self.ruta(&valor)?;
        if ruta.is_file() {
            let existente =
                std::fs::read(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
            if !hash::coincide(&existente, &valor) {
                return Err(Error::Integrity(format!(
                    "blob corrupto en {}",
                    ruta.display()
                )));
            }
            return Ok(valor);
        }
        fsutil::escribir_atomico(&ruta, bytes)?;
        Ok(valor)
    }

    /// Lee y verifica un blob.
    pub fn obtener(&self, hash_con_prefijo: &str) -> Result<Vec<u8>> {
        let ruta = self.ruta(hash_con_prefijo)?;
        let bytes = fsutil::leer_opcional(&ruta)?
            .ok_or_else(|| Error::NotFound(format!("blob inexistente: {hash_con_prefijo}")))?;
        if !hash::coincide(&bytes, hash_con_prefijo) {
            return Err(Error::Integrity(format!(
                "blob corrupto: {hash_con_prefijo}"
            )));
        }
        Ok(bytes)
    }

    /// Devuelve el blob si existe, sin fallar cuando falta.
    pub fn obtener_opcional(&self, hash_con_prefijo: &str) -> Result<Option<Vec<u8>>> {
        let ruta = self.ruta(hash_con_prefijo)?;
        let Some(bytes) = fsutil::leer_opcional(&ruta)? else {
            return Ok(None);
        };
        if !hash::coincide(&bytes, hash_con_prefijo) {
            return Err(Error::Integrity(format!(
                "blob corrupto: {hash_con_prefijo}"
            )));
        }
        Ok(Some(bytes))
    }

    /// Comprueba si existe un blob.
    pub fn tiene(&self, hash_con_prefijo: &str) -> Result<bool> {
        let ruta = self.ruta(hash_con_prefijo)?;
        Ok(ruta.is_file())
    }

    /// Lista los hashes hexadecimales almacenados.
    pub fn listar(&self) -> Result<Vec<String>> {
        let mut salida = Vec::new();
        let entradas = std::fs::read_dir(&self.raiz)
            .map_err(|error| Error::Io(error).with_path(&self.raiz))?;
        for entrada in entradas {
            let entrada = entrada.map_err(|error| Error::Io(error).with_path(&self.raiz))?;
            let ruta = entrada.path();
            if !ruta.is_file() {
                continue;
            }
            if let Some(nombre) = ruta.file_name().and_then(|nombre| nombre.to_str()) {
                if nombre.len() == 64
                    && nombre
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                {
                    salida.push(nombre.to_string());
                }
            }
        }
        salida.sort();
        Ok(salida)
    }

    /// Elimina blobs que no aparecen en el conjunto de hashes hex referenciados.
    pub fn limpiar(&self, referenciados: &BTreeSet<String>) -> Result<usize> {
        let mut borrados = 0;
        for hex in self.listar()? {
            if !referenciados.contains(&hex) {
                fsutil::eliminar_archivo(&self.raiz.join(&hex))?;
                borrados += 1;
            }
        }
        Ok(borrados)
    }

    /// Ruta del directorio de blobs.
    pub fn raiz(&self) -> &Path {
        &self.raiz
    }
}
