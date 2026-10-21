//! Lock del sistema operativo por transacción.
//!
//! El contenido de `.escritor/bloqueo` es informativo. La exclusión real la da
//! `File::try_lock` (`flock`/`LockFileEx`). Sin vencimiento: si el proceso
//! muere, el SO libera el lock.

use std::fs::{File, TryLockError};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::time;

/// Lock exclusivo del proyecto. Se libera al soltar el valor.
#[derive(Debug)]
pub struct ProjectLock {
    archivo: File,
    ruta: PathBuf,
}

impl ProjectLock {
    /// Toma el lock, esperando hasta el límite configurado.
    pub fn adquirir(escritor: &Path, op_id: Option<&str>) -> Result<Self> {
        let ruta = escritor.join("bloqueo");
        if let Some(padre) = ruta.parent() {
            std::fs::create_dir_all(padre).map_err(|error| Error::Io(error).with_path(padre))?;
        }
        let archivo = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&ruta)
            .map_err(|error| Error::Io(error).with_path(&ruta))?;

        let limite = std::env::var("WRITER_LOCK_TIMEOUT_MS")
            .ok()
            .and_then(|valor| valor.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_secs(5));
        let inicio = Instant::now();
        loop {
            match archivo.try_lock() {
                Ok(()) => break,
                Err(TryLockError::WouldBlock) if inicio.elapsed() < limite => {
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(TryLockError::WouldBlock) => {
                    return Err(Error::Busy(format!(
                        "otro proceso mantiene el lock de {}",
                        escritor.display()
                    )));
                }
                Err(TryLockError::Error(error)) => {
                    return Err(Error::Io(error).with_path(&ruta));
                }
            }
        }

        let mut lock = Self { archivo, ruta };
        lock.escribir_info(op_id)?;
        Ok(lock)
    }

    fn escribir_info(&mut self, op_id: Option<&str>) -> Result<()> {
        let info = serde_json::json!({
            "pid": std::process::id(),
            "host": std::env::var("HOSTNAME").unwrap_or_else(|_| "desconocido".to_string()),
            "inicio": time::ahora(),
            "op_id": op_id,
        });
        let texto = serde_json::to_string(&info)?;
        self.archivo
            .set_len(0)
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        self.archivo
            .seek(SeekFrom::Start(0))
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        self.archivo
            .write_all(texto.as_bytes())
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        self.archivo
            .write_all(b"\n")
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        self.archivo
            .sync_all()
            .map_err(|error| Error::Io(error).with_path(&self.ruta))?;
        Ok(())
    }

    /// Ruta del archivo de bloqueo.
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = self.archivo.unlock();
    }
}
