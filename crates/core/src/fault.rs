//! Inyección de fallos para probar durabilidad.
//!
//! `WRITER_FAULT` acepta los valores del plan:
//! `prepare`, `after_write:N`, `commit` y `before_replace`. Se añade
//! `before_check` para probar el conflicto del CAS. El proceso termina con
//! código 70 sin ejecutar destructores, como un corte real.
//!
//! Para simular un `EIO` en la sincronización del directorio se usan
//! `WRITER_FAULT_FSYNC_DIR` (ruta exacta) y, opcionalmente,
//! `WRITER_FAULT_FSYNC_ONCE=1` para fallar solo la primera vez.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{Error, Result};

static UNA_VEZ: AtomicBool = AtomicBool::new(true);
static FSYNC_UNA_VEZ: AtomicBool = AtomicBool::new(true);

fn configurado() -> Option<String> {
    std::env::var("WRITER_FAULT")
        .ok()
        .filter(|valor| !valor.is_empty())
}

/// Indica si el fallo pedido está activo.
pub fn activo(nombre: &str) -> bool {
    configurado().as_deref() == Some(nombre)
}

/// Corta el proceso en el punto indicado.
pub fn punto(nombre: &str) -> Result<()> {
    if activo(nombre) {
        eprintln!("WRITER_FAULT={nombre}: corte simulado");
        std::process::exit(70);
    }
    Ok(())
}

/// Corta después de aplicar `aplicados` efectos.
pub fn despues_de_escribir(aplicados: usize) -> Result<()> {
    let nombre = format!("after_write:{aplicados}");
    if activo(&nombre) {
        eprintln!("WRITER_FAULT={nombre}: corte simulado");
        std::process::exit(70);
    }
    Ok(())
}

/// Escribe un contenido externo antes de la comprobación del CAS.
pub fn antes_de_comprobar(ruta: &Path) -> Result<()> {
    if activo("before_check") && primera_vez() {
        escribir_externo(ruta)?;
    }
    Ok(())
}

/// Escribe un contenido externo entre la comprobación y el reemplazo.
pub fn antes_de_reemplazar(ruta: &Path) -> Result<()> {
    if activo("before_replace") && primera_vez() {
        escribir_externo(ruta)?;
    }
    Ok(())
}

fn primera_vez() -> bool {
    UNA_VEZ.swap(false, Ordering::SeqCst)
}

/// Falla el `fsync` del directorio indicado para probar errores de E/S.
pub fn fallar_fsync_dir(ruta: &Path) -> Result<()> {
    let Ok(objetivo) = std::env::var("WRITER_FAULT_FSYNC_DIR") else {
        return Ok(());
    };
    if objetivo.is_empty() || !misma_ruta(Path::new(&objetivo), ruta) {
        return Ok(());
    }
    let una_vez = std::env::var("WRITER_FAULT_FSYNC_ONCE")
        .map(|valor| !valor.is_empty())
        .unwrap_or(false);
    if una_vez && !FSYNC_UNA_VEZ.swap(false, Ordering::SeqCst) {
        return Ok(());
    }
    Err(Error::Other(format!(
        "EIO inyectado en fsync del directorio: {}",
        ruta.display()
    )))
}

fn misma_ruta(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn escribir_externo(ruta: &Path) -> Result<()> {
    let contenido = std::env::var("WRITER_FAULT_EXTERNAL")
        .unwrap_or_else(|_| "contenido externo\n".to_string());
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
    }
    std::fs::write(ruta, contenido.as_bytes()).map_err(|error| Error::Io(error).with_path(ruta))?;
    eprintln!(
        "WRITER_FAULT: escritura externa simulada en {}",
        ruta.display()
    );
    Ok(())
}
