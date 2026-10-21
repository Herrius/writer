//! Recuperación al abrir el proyecto: todo `PREPARE` sin `COMMIT` se revierte.
//!
//! La reversión es idempotente y solo toca archivos que siguen coincidiendo con
//! el `hash_nuevo`. Un cambio externo se conserva y se registra como conflicto.

use std::path::Path;

use crate::blob::BlobStore;
use crate::error::{Error, Result};
use crate::fsutil;
use crate::hash;
use crate::oplog::{Efecto, Log};

/// Resultado de una pasada de recuperación.
#[derive(Clone, Debug, Default)]
pub struct Recuperacion {
    /// Operaciones revertidas.
    pub revertidas: usize,
    /// Archivos restaurados.
    pub restaurados: usize,
    /// Archivos eliminados por ser creaciones no confirmadas.
    pub eliminados: usize,
    /// Conflictos con cambios externos.
    pub conflictos: Vec<String>,
    /// Avisos informativos.
    pub avisos: Vec<String>,
}

/// Revierte las operaciones pendientes del log.
pub fn recuperar(raiz: &Path, log: &mut Log, blobs: &BlobStore) -> Result<Recuperacion> {
    let pendientes = log.pendientes();
    let mut informe = Recuperacion::default();
    if pendientes.is_empty() {
        return Ok(informe);
    }

    // Preflight: si falta o está corrupto un blob necesario, no se toca nada.
    // El proyecto abre en modo recuperación y no escribe.
    for prepare in &pendientes {
        for efecto in prepare.efectos.iter().rev() {
            let ruta = fsutil::ruta_del_proyecto(raiz, &efecto.ruta)?;
            let actual = hash::hash_archivo(&ruta)?;
            if actual == efecto.hash_nuevo {
                if let Some(anterior) = &efecto.hash_anterior {
                    blobs.obtener(anterior).map_err(|error| {
                        Error::Recovery(format!(
                            "no se puede revertir {} (operación {}): {error}",
                            efecto.ruta, prepare.op_id
                        ))
                    })?;
                }
            }
        }
    }

    for prepare in pendientes {
        let mut conflicto = false;
        for efecto in prepare.efectos.iter().rev() {
            let resultado = revertir_efecto(raiz, blobs, efecto)?;
            match resultado {
                ResultadoEfecto::Restaurado => informe.restaurados += 1,
                ResultadoEfecto::Eliminado => informe.eliminados += 1,
                ResultadoEfecto::SinCambios => {}
                ResultadoEfecto::Conflicto(mensaje) => {
                    conflicto = true;
                    informe.conflictos.push(mensaje);
                }
            }
        }
        if !log.abortada(&prepare.op_id) {
            let motivo = if conflicto {
                "conflicto"
            } else {
                "recuperacion"
            };
            log.agregar_abort(&prepare.op_id, motivo)?;
        }
        informe.revertidas += 1;
        if conflicto {
            informe.avisos.push(format!(
                "operación {}: se conservaron cambios externos",
                prepare.op_id
            ));
        }
    }
    log.sincronizar()?;
    Ok(informe)
}

enum ResultadoEfecto {
    Restaurado,
    Eliminado,
    SinCambios,
    Conflicto(String),
}

fn revertir_efecto(raiz: &Path, blobs: &BlobStore, efecto: &Efecto) -> Result<ResultadoEfecto> {
    let ruta = fsutil::ruta_del_proyecto(raiz, &efecto.ruta)?;
    let actual = hash::hash_archivo(&ruta)?;
    if actual == efecto.hash_nuevo {
        match &efecto.hash_anterior {
            Some(anterior) => {
                let bytes = blobs.obtener(anterior)?;
                fsutil::escribir_atomico(&ruta, &bytes)?;
                Ok(ResultadoEfecto::Restaurado)
            }
            None => {
                fsutil::eliminar_archivo(&ruta)?;
                Ok(ResultadoEfecto::Eliminado)
            }
        }
    } else if actual == efecto.hash_anterior {
        // Ya está en el estado anterior. Se sincroniza antes de permitir que se
        // anexe ABORT: una reversión anterior pudo escribir los bytes y fallar
        // en el fsync del directorio.
        if efecto.hash_anterior.is_some() {
            fsutil::sincronizar_archivo(&ruta)?;
        } else if let Some(padre) = ruta.parent() {
            fsutil::fsync_dir(padre)?;
        }
        Ok(ResultadoEfecto::SinCambios)
    } else {
        Ok(ResultadoEfecto::Conflicto(format!(
            "{}: el archivo cambió por fuera; no se revierte",
            efecto.ruta
        )))
    }
}
