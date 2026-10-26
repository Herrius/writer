//! Cambios locales propuestos (control de cambios sin tocar el documento).
//!
//! Un cambio es un sidecar en `.escritor/cambios/<ULID>.json` con el cuerpo
//! base y el cuerpo propuesto. El Markdown del manuscrito no se modifica hasta
//! que el usuario acepta el cambio. Aceptar escribe el cuerpo con el mismo
//! protocolo transaccional que un guardado normal; rechazar solo elimina el
//! sidecar.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diff::{self, Diferencias};
use crate::error::{Error, Result};
use crate::{fsutil, hash, time, ulid};

/// Versión de la extensión de control de cambios.
pub const VERSION: u32 = 1;

/// Tamaño máximo de cada cuerpo guardado en un cambio.
pub const MAX_CUERPO: usize = 4 * 1024 * 1024;

/// Tamaño máximo del JSON completo (dos cuerpos + metadatos).
const MAX_SIDECAR: u64 = (MAX_CUERPO as u64) * 2 + 64 * 1024;

/// Cambio local propuesto.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CambioLocal {
    /// Versión de la extensión.
    pub version: u32,
    /// ULID del cambio.
    pub id: String,
    /// ID del documento.
    pub documento: String,
    /// Ruta relativa del documento.
    pub ruta: String,
    /// Autor local.
    pub autor: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Revisión base, si se pudo determinar.
    #[serde(default)]
    pub base_revision: u64,
    /// Hash del archivo completo en el momento de crear el cambio.
    pub base_hash: String,
    /// Cuerpo del documento en la base.
    pub base: String,
    /// Cuerpo propuesto.
    pub propuesta: String,
}

impl CambioLocal {
    /// Valida el cambio antes de leerlo o escribirlo.
    pub fn validar(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(Error::Schema(format!(
                "versión de cambios no soportada: {}",
                self.version
            )));
        }
        if !ulid::es_valido(&self.id) {
            return Err(Error::Schema(format!(
                "cambio con ULID no válido: {}",
                self.id
            )));
        }
        if !ulid::es_valido(&self.documento) {
            return Err(Error::Schema(format!(
                "documento con ULID no válido: {}",
                self.documento
            )));
        }
        fsutil::validar_ruta_documento(&self.ruta)?;
        if self.autor.trim().is_empty() || self.autor.len() > 200 {
            return Err(Error::Schema("autor de cambio no válido".to_string()));
        }
        time::parse_rfc3339(&self.ts)?;
        if !hash::es_hash(&self.base_hash) {
            return Err(Error::Schema(format!(
                "hash base de cambio no válido: {}",
                self.base_hash
            )));
        }
        if self.base.len() > MAX_CUERPO || self.propuesta.len() > MAX_CUERPO {
            return Err(Error::Input(format!(
                "el cambio supera el máximo de {MAX_CUERPO} bytes por cuerpo"
            )));
        }
        Ok(())
    }

    /// Diferencia entre la base y la propuesta.
    pub fn diferencias(&self) -> Diferencias {
        diff::comparar(&self.base, &self.propuesta)
    }

    /// Crea un cambio con los valores obligatorios.
    pub fn nuevo(
        documento: &str,
        ruta: &str,
        autor: &str,
        base_revision: u64,
        base_hash: &str,
        base: &str,
        propuesta: &str,
    ) -> Result<Self> {
        let autor = autor.trim();
        if autor.is_empty() {
            return Err(Error::Input("el autor no puede estar vacío".to_string()));
        }
        let cambio = Self {
            version: VERSION,
            id: ulid::nuevo()?,
            documento: documento.to_string(),
            ruta: ruta.to_string(),
            autor: autor.to_string(),
            ts: time::ahora(),
            base_revision,
            base_hash: base_hash.to_string(),
            base: base.to_string(),
            propuesta: propuesta.to_string(),
        };
        cambio.validar()?;
        Ok(cambio)
    }
}

/// Lee un cambio por ULID.
pub fn leer(raiz: &Path, id: &str) -> Result<CambioLocal> {
    let ruta = ruta_cambio(raiz, id)?;
    let metadatos = match std::fs::symlink_metadata(&ruta) {
        Ok(metadatos) => metadatos,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::NotFound(format!("cambio no encontrado: {id}")));
        }
        Err(error) => return Err(Error::Io(error).with_path(&ruta)),
    };
    if !metadatos.file_type().is_file() || metadatos.file_type().is_symlink() {
        return Err(Error::Integrity(format!(
            "el cambio no es un archivo regular: {}",
            ruta.display()
        )));
    }
    if metadatos.len() > MAX_SIDECAR {
        return Err(Error::Input(format!(
            "el cambio supera el máximo de {MAX_SIDECAR} bytes"
        )));
    }
    let bytes = fsutil::leer_opcional(&ruta)?
        .ok_or_else(|| Error::NotFound(format!("cambio no encontrado: {id}")))?;
    let cambio: CambioLocal =
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(error).with_path(&ruta))?;
    cambio.validar()?;
    Ok(cambio)
}

/// Guarda un cambio con reemplazo atómico.
pub fn guardar(raiz: &Path, cambio: &CambioLocal) -> Result<()> {
    cambio.validar()?;
    let ruta = ruta_cambio(raiz, &cambio.id)?;
    let bytes = serde_json::to_vec_pretty(cambio)?;
    if bytes.len() as u64 > MAX_SIDECAR {
        return Err(Error::Input(format!(
            "el JSON del cambio supera el máximo de {MAX_SIDECAR} bytes"
        )));
    }
    fsutil::escribir_atomico(&ruta, &bytes)
}

/// Elimina un cambio. No falla si ya no existe.
pub fn eliminar(raiz: &Path, id: &str) -> Result<()> {
    let ruta = ruta_cambio(raiz, id)?;
    fsutil::eliminar_archivo(&ruta)
}

/// Lista los cambios válidos; los sidecars corruptos se apartan a cuarentena.
pub fn listar(raiz: &Path) -> Result<Vec<CambioLocal>> {
    let Some(directorio) = directorio(raiz, false)? else {
        return Ok(Vec::new());
    };
    let mut cambios = Vec::new();
    for entrada in
        std::fs::read_dir(&directorio).map_err(|error| Error::Io(error).with_path(&directorio))?
    {
        let entrada = entrada.map_err(|error| Error::Io(error).with_path(&directorio))?;
        let ruta = entrada.path();
        if !fsutil::es_extension(&ruta, "json") {
            continue;
        }
        let Ok(metadatos) = std::fs::symlink_metadata(&ruta) else {
            continue;
        };
        if !metadatos.file_type().is_file()
            || metadatos.file_type().is_symlink()
            || metadatos.len() > MAX_SIDECAR
        {
            let _ = fsutil::apartar_sidecar(raiz, &ruta);
            continue;
        }
        let Ok(bytes) = std::fs::read(&ruta) else {
            let _ = fsutil::apartar_sidecar(raiz, &ruta);
            continue;
        };
        let Ok(cambio) = serde_json::from_slice::<CambioLocal>(&bytes) else {
            let _ = fsutil::apartar_sidecar(raiz, &ruta);
            continue;
        };
        if cambio.validar().is_err() {
            let _ = fsutil::apartar_sidecar(raiz, &ruta);
            continue;
        }
        cambios.push(cambio);
    }
    cambios.sort_by(|a, b| a.ts.cmp(&b.ts).then(a.id.cmp(&b.id)));
    Ok(cambios)
}

/// Busca el cambio pendiente de un documento, si existe.
pub fn para_documento(
    raiz: &Path,
    documento: &str,
    ruta_documento: &str,
) -> Result<Option<CambioLocal>> {
    Ok(listar(raiz)?
        .into_iter()
        .filter(|cambio| cambio.documento == documento && cambio.ruta == ruta_documento)
        .max_by(|a, b| a.ts.cmp(&b.ts).then(a.id.cmp(&b.id))))
}

/// Elimina todos los cambios pendientes de un documento.
pub fn eliminar_para_documento(raiz: &Path, documento: &str, ruta_documento: &str) -> Result<()> {
    for cambio in listar(raiz)? {
        if cambio.documento == documento && cambio.ruta == ruta_documento {
            eliminar(raiz, &cambio.id)?;
        }
    }
    Ok(())
}

fn directorio(raiz: &Path, crear: bool) -> Result<Option<PathBuf>> {
    let Some(escritor) = fsutil::validar_escritor(raiz)? else {
        if !crear {
            return Ok(None);
        }
        return Err(Error::Project("el proyecto no tiene .escritor".to_string()));
    };
    let directorio = escritor.join("cambios");
    if !directorio.exists() {
        if !crear {
            return Ok(None);
        }
        std::fs::create_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?;
    }
    Ok(Some(fsutil::directorio_real(&directorio)?))
}

fn ruta_cambio(raiz: &Path, id: &str) -> Result<PathBuf> {
    if !ulid::es_valido(id) {
        return Err(Error::Input(format!("ULID de cambio no válido: {id}")));
    }
    let directorio = directorio(raiz, true)?
        .ok_or_else(|| Error::Project("no se pudo abrir .escritor/cambios".to_string()))?;
    Ok(directorio.join(format!("{id}.json")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valida_ulid_y_hash() {
        let cambio = CambioLocal {
            version: VERSION,
            id: "no-es-ulid".to_string(),
            documento: "01KDVDNAZ84TFF59TDWH9EDD1R".to_string(),
            ruta: "manuscrito/01-libro/01-capitulo/001.md".to_string(),
            autor: "local".to_string(),
            ts: time::ahora(),
            base_revision: 1,
            base_hash: hash::hash(b"base"),
            base: "base".to_string(),
            propuesta: "propuesta".to_string(),
        };
        assert!(cambio.validar().is_err());
    }
}
