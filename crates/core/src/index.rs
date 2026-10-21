//! Índice SQLite reconstruible con FTS5.
//!
//! Nunca es la verdad: se borra y se reconstruye desde los archivos y el log.
//! Si el archivo no se puede abrir o su versión no se entiende, se aparta y se
//! crea uno nuevo.

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::error::{Error, Result};
use crate::oplog::Operacion;
use crate::time;

const VERSION_INDICE: i64 = 1;

/// Fila de escena para el índice.
#[derive(Clone, Debug)]
pub struct EscenaFila {
    /// ULID.
    pub id: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Hash del archivo.
    pub hash: String,
    /// mtime en nanosegundos.
    pub mtime_ns: i64,
    /// Tamaño.
    pub tamano: i64,
    /// Título.
    pub titulo: String,
    /// Estado.
    pub estado: Option<String>,
    /// POV.
    pub pov: Option<String>,
    /// Orden.
    pub orden: i64,
    /// Fecha narrativa.
    pub fecha_narrativa: Option<String>,
    /// Palabras.
    pub palabras: i64,
    /// Tramas.
    pub tramas: Vec<String>,
    /// Personajes.
    pub personajes: Vec<String>,
    /// Etiquetas.
    pub etiquetas: Vec<String>,
    /// Cuerpo.
    pub cuerpo: String,
}

/// Fila de nota para el índice.
#[derive(Clone, Debug)]
pub struct NotaFila {
    /// ULID.
    pub id: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Tipo.
    pub tipo: String,
    /// Título.
    pub titulo: String,
    /// Fija.
    pub fija: bool,
    /// Hash del archivo.
    pub hash: String,
    /// mtime en nanosegundos.
    pub mtime_ns: i64,
    /// Tamaño.
    pub tamano: i64,
    /// Etiquetas.
    pub etiquetas: Vec<String>,
    /// Referencias.
    pub refs: Vec<String>,
    /// Cuerpo.
    pub cuerpo: String,
}

/// Resumen del índice.
#[derive(Clone, Debug, Default)]
pub struct ResumenIndice {
    /// Escenas indexadas.
    pub escenas: usize,
    /// Notas indexadas.
    pub notas: usize,
    /// Revisión registrada.
    pub revision: u64,
    /// Último commit registrado.
    pub ultimo_commit: Option<String>,
}

/// Conexión al índice.
#[derive(Debug)]
pub struct Indice {
    conexion: Connection,
    ruta: PathBuf,
}

impl Indice {
    /// Abre el índice, apartando una base corrupta o de versión desconocida.
    pub fn abrir(escritor: &Path) -> Result<Self> {
        std::fs::create_dir_all(escritor).map_err(|error| Error::Io(error).with_path(escritor))?;
        let ruta = escritor.join("indice.sqlite");
        match Self::abrir_una_vez(&ruta) {
            Ok(indice) => Ok(indice),
            Err(_) => {
                Self::apartar(&ruta)?;
                Self::abrir_una_vez(&ruta)
            }
        }
    }

    /// Crea un índice FTS5 efímero en memoria.
    ///
    /// Se usa como fallback para proyectos sin `.escritor`: garantiza la misma
    /// tokenización y semántica que el índice persistente sin crear archivos.
    pub fn en_memoria() -> Result<Self> {
        let conexion = Connection::open_in_memory().map_err(Error::Index)?;
        let mut indice = Self {
            conexion,
            ruta: PathBuf::new(),
        };
        indice
            .conexion
            .execute_batch(
                "PRAGMA foreign_keys=ON;\n\
                 PRAGMA busy_timeout=5000;\n",
            )
            .map_err(Error::Index)?;
        indice.crear_esquema()?;
        Ok(indice)
    }

    fn abrir_una_vez(ruta: &Path) -> Result<Self> {
        let mut indice = Self::intentar_abrir(ruta)?;
        indice.configurar()?;
        indice.crear_esquema()?;
        Ok(indice)
    }

    fn intentar_abrir(ruta: &Path) -> Result<Self> {
        let conexion = Connection::open(ruta).map_err(Error::Index)?;
        let version: i64 = conexion
            .query_row("PRAGMA user_version", [], |fila| fila.get(0))
            .map_err(Error::Index)?;
        if version != 0 && version != VERSION_INDICE {
            return Err(Error::Other(format!(
                "versión de índice no soportada: {version}"
            )));
        }
        Ok(Self {
            conexion,
            ruta: ruta.to_path_buf(),
        })
    }

    pub(crate) fn apartar(ruta: &Path) -> Result<()> {
        if ruta.exists() {
            let destino = ruta.with_file_name(format!(
                "indice.sqlite.corrupto-{}",
                time::timestamp_slug(time::now_ms())
            ));
            std::fs::rename(ruta, &destino).map_err(|error| Error::Io(error).with_path(ruta))?;
        }
        for sufijo in ["-wal", "-shm"] {
            let auxiliar = PathBuf::from(format!("{}{sufijo}", ruta.display()));
            let _ = std::fs::remove_file(auxiliar);
        }
        Ok(())
    }

    fn configurar(&mut self) -> Result<()> {
        self.conexion
            .execute_batch(
                "PRAGMA journal_mode=WAL;\n\
                 PRAGMA synchronous=FULL;\n\
                 PRAGMA foreign_keys=ON;\n\
                 PRAGMA busy_timeout=5000;\n",
            )
            .map_err(Error::Index)?;
        Ok(())
    }

    fn crear_esquema(&mut self) -> Result<()> {
        self.conexion
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS meta (
                     clave TEXT PRIMARY KEY,
                     valor TEXT NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS escenas (
                     id TEXT PRIMARY KEY,
                     ruta TEXT NOT NULL UNIQUE,
                     hash TEXT NOT NULL,
                     mtime_ns INTEGER NOT NULL,
                     tamano INTEGER NOT NULL,
                     titulo TEXT NOT NULL,
                     estado TEXT,
                     pov TEXT,
                     orden INTEGER NOT NULL,
                     fecha_narrativa TEXT,
                     palabras INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS escena_tramas (
                     escena_id TEXT NOT NULL,
                     trama_id TEXT NOT NULL,
                     PRIMARY KEY (escena_id, trama_id)
                 );
                 CREATE TABLE IF NOT EXISTS escena_personajes (
                     escena_id TEXT NOT NULL,
                     personaje_id TEXT NOT NULL,
                     PRIMARY KEY (escena_id, personaje_id)
                 );
                 CREATE TABLE IF NOT EXISTS notas (
                     id TEXT PRIMARY KEY,
                     ruta TEXT NOT NULL UNIQUE,
                     tipo TEXT NOT NULL,
                     titulo TEXT NOT NULL,
                     fija INTEGER NOT NULL,
                     hash TEXT NOT NULL,
                     mtime_ns INTEGER NOT NULL,
                     tamano INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS notas_refs (
                     origen_id TEXT NOT NULL,
                     destino_id TEXT NOT NULL,
                     PRIMARY KEY (origen_id, destino_id)
                 );
                 CREATE TABLE IF NOT EXISTS etiquetas (
                     objeto_id TEXT NOT NULL,
                     etiqueta TEXT NOT NULL,
                     PRIMARY KEY (objeto_id, etiqueta)
                 );
                 CREATE TABLE IF NOT EXISTS historial (
                     rev INTEGER PRIMARY KEY,
                     op_id TEXT NOT NULL,
                     task_id TEXT,
                     autor TEXT NOT NULL,
                     scope TEXT NOT NULL,
                     ts TEXT NOT NULL
                 );
                 CREATE VIRTUAL TABLE IF NOT EXISTS fts_escenas USING fts5(
                     id UNINDEXED,
                     titulo,
                     cuerpo
                 );
                 CREATE VIRTUAL TABLE IF NOT EXISTS fts_notas USING fts5(
                     id UNINDEXED,
                     titulo,
                     cuerpo
                 );
                 PRAGMA user_version=1;",
            )
            .map_err(Error::Index)?;
        Ok(())
    }

    /// Reconstruye el índice completo en una transacción.
    pub fn reconstruir(
        &mut self,
        escenas: &[EscenaFila],
        notas: &[NotaFila],
        historial: &[Operacion],
        revision: u64,
    ) -> Result<()> {
        let transaccion = self.conexion.transaction().map_err(Error::Index)?;
        transaccion
            .execute_batch(
                "DELETE FROM fts_escenas;
                 DELETE FROM fts_notas;
                 DELETE FROM etiquetas;
                 DELETE FROM notas_refs;
                 DELETE FROM escena_tramas;
                 DELETE FROM escena_personajes;
                 DELETE FROM escenas;
                 DELETE FROM notas;
                 DELETE FROM historial;",
            )
            .map_err(Error::Index)?;
        {
            let mut insertar = transaccion
                .prepare(
                    "INSERT INTO escenas
                     (id, ruta, hash, mtime_ns, tamano, titulo, estado, pov, orden,
                      fecha_narrativa, palabras)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                )
                .map_err(Error::Index)?;
            for escena in escenas {
                insertar
                    .execute(params![
                        escena.id,
                        escena.ruta,
                        escena.hash,
                        escena.mtime_ns,
                        escena.tamano,
                        escena.titulo,
                        escena.estado,
                        escena.pov,
                        escena.orden,
                        escena.fecha_narrativa,
                        escena.palabras,
                    ])
                    .map_err(Error::Index)?;
            }
        }
        {
            let mut insertar = transaccion
                .prepare(
                    "INSERT INTO notas
                     (id, ruta, tipo, titulo, fija, hash, mtime_ns, tamano)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                )
                .map_err(Error::Index)?;
            for nota in notas {
                insertar
                    .execute(params![
                        nota.id,
                        nota.ruta,
                        nota.tipo,
                        nota.titulo,
                        i64::from(nota.fija),
                        nota.hash,
                        nota.mtime_ns,
                        nota.tamano,
                    ])
                    .map_err(Error::Index)?;
            }
        }
        {
            let mut insertar = transaccion
                .prepare("INSERT INTO escena_tramas (escena_id, trama_id) VALUES (?1, ?2)")
                .map_err(Error::Index)?;
            for escena in escenas {
                for trama in &escena.tramas {
                    insertar
                        .execute(params![escena.id, trama])
                        .map_err(Error::Index)?;
                }
            }
        }
        {
            let mut insertar = transaccion
                .prepare("INSERT INTO escena_personajes (escena_id, personaje_id) VALUES (?1, ?2)")
                .map_err(Error::Index)?;
            for escena in escenas {
                for personaje in &escena.personajes {
                    insertar
                        .execute(params![escena.id, personaje])
                        .map_err(Error::Index)?;
                }
            }
        }
        {
            let mut insertar = transaccion
                .prepare("INSERT INTO notas_refs (origen_id, destino_id) VALUES (?1, ?2)")
                .map_err(Error::Index)?;
            for nota in notas {
                for referencia in &nota.refs {
                    insertar
                        .execute(params![nota.id, referencia])
                        .map_err(Error::Index)?;
                }
            }
        }
        {
            let mut insertar = transaccion
                .prepare("INSERT INTO etiquetas (objeto_id, etiqueta) VALUES (?1, ?2)")
                .map_err(Error::Index)?;
            for escena in escenas {
                for etiqueta in &escena.etiquetas {
                    insertar
                        .execute(params![escena.id, etiqueta])
                        .map_err(Error::Index)?;
                }
            }
            for nota in notas {
                for etiqueta in &nota.etiquetas {
                    insertar
                        .execute(params![nota.id, etiqueta])
                        .map_err(Error::Index)?;
                }
            }
        }
        {
            let mut insertar = transaccion
                .prepare(
                    "INSERT INTO historial (rev, op_id, task_id, autor, scope, ts)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .map_err(Error::Index)?;
            for operacion in historial {
                insertar
                    .execute(params![
                        operacion.prepare.target_rev,
                        operacion.prepare.op_id,
                        operacion.prepare.task_id,
                        operacion.prepare.autor,
                        operacion.prepare.scope,
                        operacion.prepare.ts,
                    ])
                    .map_err(Error::Index)?;
            }
        }
        {
            let mut insertar = transaccion
                .prepare("INSERT INTO fts_escenas (id, titulo, cuerpo) VALUES (?1, ?2, ?3)")
                .map_err(Error::Index)?;
            for escena in escenas {
                insertar
                    .execute(params![escena.id, escena.titulo, escena.cuerpo])
                    .map_err(Error::Index)?;
            }
        }
        {
            let mut insertar = transaccion
                .prepare("INSERT INTO fts_notas (id, titulo, cuerpo) VALUES (?1, ?2, ?3)")
                .map_err(Error::Index)?;
            for nota in notas {
                insertar
                    .execute(params![nota.id, nota.titulo, nota.cuerpo])
                    .map_err(Error::Index)?;
            }
        }
        {
            let mut insertar = transaccion
                .prepare(
                    "INSERT INTO meta (clave, valor) VALUES (?1, ?2)
                     ON CONFLICT(clave) DO UPDATE SET valor=excluded.valor",
                )
                .map_err(Error::Index)?;
            insertar
                .execute(params!["esquema", VERSION_INDICE.to_string()])
                .map_err(Error::Index)?;
            insertar
                .execute(params!["revision", revision.to_string()])
                .map_err(Error::Index)?;
            let ultimo = historial
                .last()
                .map(|operacion| operacion.prepare.op_id.clone())
                .unwrap_or_default();
            insertar
                .execute(params!["ultimo_commit", ultimo])
                .map_err(Error::Index)?;
        }
        transaccion.commit().map_err(Error::Index)?;
        Ok(())
    }

    /// Resumen del índice.
    pub fn resumen(&self) -> Result<ResumenIndice> {
        let escenas: i64 = self
            .conexion
            .query_row("SELECT COUNT(*) FROM escenas", [], |fila| fila.get(0))
            .map_err(Error::Index)?;
        let notas: i64 = self
            .conexion
            .query_row("SELECT COUNT(*) FROM notas", [], |fila| fila.get(0))
            .map_err(Error::Index)?;
        let revision = self
            .conexion
            .query_row(
                "SELECT valor FROM meta WHERE clave='revision'",
                [],
                |fila| fila.get::<_, String>(0),
            )
            .ok()
            .and_then(|valor| valor.parse().ok())
            .unwrap_or(0_u64);
        let ultimo_commit = self
            .conexion
            .query_row(
                "SELECT valor FROM meta WHERE clave='ultimo_commit'",
                [],
                |fila| fila.get::<_, String>(0),
            )
            .ok()
            .filter(|valor| !valor.is_empty());
        Ok(ResumenIndice {
            escenas: usize::try_from(escenas).unwrap_or(0),
            notas: usize::try_from(notas).unwrap_or(0),
            revision,
            ultimo_commit,
        })
    }

    /// Busca en escenas y notas con FTS5.
    pub fn buscar(&self, consulta: &str, limite: usize) -> Result<Vec<(String, String, String)>> {
        let mut salida = Vec::new();
        let limite = i64::try_from(limite).unwrap_or(20);
        let mut sentencia = self
            .conexion
            .prepare(
                "SELECT e.id, e.ruta, snippet(fts_escenas, 2, '[', ']', '…', 12)
                 FROM fts_escenas JOIN escenas e ON e.id = fts_escenas.id
                 WHERE fts_escenas MATCH ?1
                 UNION ALL
                 SELECT n.id, n.ruta, snippet(fts_notas, 2, '[', ']', '…', 12)
                 FROM fts_notas JOIN notas n ON n.id = fts_notas.id
                 WHERE fts_notas MATCH ?1
                 LIMIT ?2",
            )
            .map_err(Error::Index)?;
        let filas = sentencia
            .query_map(params![consulta, limite], |fila| {
                Ok((
                    fila.get::<_, String>(0)?,
                    fila.get::<_, String>(1)?,
                    fila.get::<_, String>(2)?,
                ))
            })
            .map_err(Error::Index)?;
        for fila in filas {
            salida.push(fila.map_err(Error::Index)?);
        }
        Ok(salida)
    }

    /// Ruta del archivo SQLite.
    pub fn ruta(&self) -> &Path {
        &self.ruta
    }
}
