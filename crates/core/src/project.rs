//! Operaciones de proyecto: apertura, transacciones, creación, edición,
//! movimiento, borrado, historial y exportación.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::blob::BlobStore;
use crate::error::{Error, Result};
use crate::fault;
use crate::fsutil;
use crate::hash;
use crate::index::{EscenaFila, Indice, NotaFila};
use crate::lock::ProjectLock;
use crate::model::{self, Escena, Nota, Proyecto, TIPOS_NOTA};
use crate::oplog::{Efecto, Log, Operacion};
use crate::recovery;
use crate::ulid;
use crate::verify::{self, Documento, Escaneo};

/// Proyecto abierto por ruta.
#[derive(Clone, Debug)]
pub struct Project {
    raiz: PathBuf,
}

/// Sesión de lectura: mantiene el lock mientras dure la operación.
#[derive(Debug)]
pub struct SesionLectura {
    _lock: Option<ProjectLock>,
    /// Log, si el proyecto ya tiene `.escritor`.
    pub log: Option<Log>,
    /// Escaneo posterior a cualquier recuperación.
    pub escaneo: Escaneo,
}

/// Sesión de escritura con lock, log, blobs e índice.
#[derive(Debug)]
pub struct SesionEscritura {
    _lock: ProjectLock,
    raiz: PathBuf,
    /// Log cargado.
    pub log: Log,
    /// Almacén de blobs.
    pub blobs: BlobStore,
    /// Índice reconstruido.
    pub indice: Indice,
    /// Escaneo actual.
    pub escaneo: Escaneo,
}

struct Preparado {
    efecto: Efecto,
    contenido: Option<Vec<u8>>,
}

/// Cambio que se aplicará en una transacción.
#[derive(Clone, Debug)]
pub struct Cambio {
    /// Ruta relativa.
    pub ruta: String,
    /// Contenido nuevo; `None` elimina el archivo.
    pub contenido: Option<Vec<u8>>,
    /// Si es `true`, el archivo no debe existir antes.
    pub crear_solo: bool,
}

impl Cambio {
    /// Escribe o reemplaza un archivo existente.
    pub fn existente(ruta: impl Into<String>, contenido: Vec<u8>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: Some(contenido),
            crear_solo: false,
        }
    }

    /// Crea un archivo nuevo; falla si ya existe.
    pub fn nuevo(ruta: impl Into<String>, contenido: Vec<u8>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: Some(contenido),
            crear_solo: true,
        }
    }

    /// Elimina un archivo.
    pub fn eliminar(ruta: impl Into<String>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: None,
            crear_solo: false,
        }
    }
}

/// Detalle de una revisión.
#[derive(Clone, Debug)]
pub struct RevisionDetalle {
    /// Operación.
    pub operacion: Operacion,
    /// Contenido de cada efecto: ruta y bytes después de la revisión.
    pub contenidos: Vec<(String, Option<Vec<u8>>)>,
}

/// Resultado de `reindex`.
#[derive(Clone, Debug)]
pub struct ResultadoReindex {
    /// Escenas indexadas.
    pub escenas: usize,
    /// Notas indexadas.
    pub notas: usize,
    /// Revisión.
    pub revision: u64,
    /// Blobs huérfanos eliminados.
    pub blobs_eliminados: usize,
}

/// Resultado de `revert`.
#[derive(Clone, Debug)]
pub struct ResultadoRevertir {
    /// Revisión nueva creada.
    pub revision: u64,
    /// Errores que quedan en el proyecto después de la reversión.
    pub errores_pendientes: Vec<String>,
}

/// Resultado de guardar el cuerpo de un documento.
#[derive(Clone, Debug)]
pub struct ResultadoGuardado {
    /// Revisión confirmada.
    pub revision: u64,
    /// Hash del archivo después del guardado.
    pub hash_archivo: String,
}

/// Resultado de una búsqueda de texto.
#[derive(Clone, Debug)]
pub struct ResultadoBusqueda {
    /// ULID del documento.
    pub id: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Título.
    pub titulo: String,
    /// `escena` o `nota`.
    pub tipo: String,
    /// Fragmento con la coincidencia marcada entre `[` y `]` cuando hay índice.
    pub fragmento: String,
}

/// Copia de un archivo inválido en cuarentena.
#[derive(Clone, Debug)]
pub struct CuarentenaItem {
    /// Nombre del archivo dentro de `.escritor/cuarentena/`.
    pub archivo: String,
    /// Ruta original, si el informe la conserva.
    pub ruta_original: Option<String>,
    /// Motivo del apartado, si el informe lo conserva.
    pub motivo: Option<String>,
    /// Fecha del apartado, si el informe la conserva.
    pub ts: Option<String>,
    /// Tamaño de la copia en bytes.
    pub tamano: u64,
}

#[derive(Clone, Debug, Default)]
struct InformeCuarentena {
    ts: Option<String>,
    ruta_original: Option<String>,
    motivo: Option<String>,
}

impl Project {
    /// Abre un proyecto por su raíz.
    pub fn new(raiz: impl Into<PathBuf>) -> Self {
        Self { raiz: raiz.into() }
    }

    /// Raíz del proyecto.
    pub fn raiz(&self) -> &Path {
        &self.raiz
    }

    /// Crea un proyecto nuevo en un directorio vacío o inexistente.
    pub fn init(&self, titulo: &str, autor: Option<&str>, idioma: Option<&str>) -> Result<()> {
        if self.raiz.exists() {
            let mut entradas = std::fs::read_dir(&self.raiz)
                .map_err(|error| Error::Io(error).with_path(&self.raiz))?;
            if entradas.next().is_some() {
                return Err(Error::Project(format!(
                    "el directorio no está vacío: {}",
                    self.raiz.display()
                )));
            }
        } else {
            std::fs::create_dir_all(&self.raiz)
                .map_err(|error| Error::Io(error).with_path(&self.raiz))?;
        }
        let proyecto = Proyecto::nuevo(titulo, autor, idioma)?;
        fsutil::escribir_atomico(&self.raiz.join("proyecto.yml"), &proyecto.a_bytes())?;
        for directorio in [
            "manuscrito",
            "notas",
            ".escritor/objetos",
            ".escritor/cuarentena",
        ] {
            let ruta = self.raiz.join(directorio);
            std::fs::create_dir_all(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
        }
        Log::crear_vacio(&self.ruta_log())?;

        let mut sesion = self.escritura()?;
        let mut escena = Escena::nueva("Primera escena", 10)?;
        escena.cuerpo = String::new();
        let ruta = "manuscrito/01-libro/01-capitulo/001-primera-escena.md".to_string();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::nuevo(ruta, escena.a_bytes())],
            "humano",
            "estructura",
            None,
            None,
        )?;
        Ok(())
    }

    /// Abre una sesión de lectura; si existe `.escritor`, recupera pendientes.
    pub fn lectura(&self) -> Result<SesionLectura> {
        let escritor = self.ruta_escritor();
        if !escritor.exists() {
            let escaneo = verify::escanear(&self.raiz)?;
            return Ok(SesionLectura {
                _lock: None,
                log: None,
                escaneo,
            });
        }
        let lock = ProjectLock::adquirir(&escritor, None)?;
        let ruta_log = self.ruta_log();
        if !ruta_log.is_file() {
            let mut escaneo = verify::escanear(&self.raiz)?;
            escaneo.errores.push(format!(
                "modo recuperación: falta {}; el historial no está disponible",
                ruta_log.display()
            ));
            return Ok(SesionLectura {
                _lock: Some(lock),
                log: None,
                escaneo,
            });
        }
        let mut log = Log::cargar(&ruta_log)?;
        let blobs = BlobStore::abrir(&escritor)?;
        let recuperacion = recovery::recuperar(&self.raiz, &mut log, &blobs)?;
        let mut escaneo = verify::escanear(&self.raiz)?;
        for conflicto in recuperacion.conflictos {
            escaneo.avisos.push(format!("recuperación: {conflicto}"));
        }
        escaneo.avisos.sort();
        escaneo.avisos.dedup();
        Ok(SesionLectura {
            _lock: Some(lock),
            log: Some(log),
            escaneo,
        })
    }

    /// Abre una sesión de escritura, creando `.escritor` si el proyecto es limpio.
    pub fn escritura(&self) -> Result<SesionEscritura> {
        self.escritura_con(false)
    }

    fn escritura_con(&self, permitir_errores: bool) -> Result<SesionEscritura> {
        if !self.raiz.join("proyecto.yml").is_file() {
            return Err(Error::Project(format!(
                "falta proyecto.yml en {}",
                self.raiz.display()
            )));
        }
        let escritor = self.ruta_escritor();
        if !escritor.exists() {
            std::fs::create_dir_all(escritor.join("objetos"))
                .map_err(|error| Error::Io(error).with_path(&escritor))?;
            std::fs::create_dir_all(escritor.join("cuarentena"))
                .map_err(|error| Error::Io(error).with_path(&escritor))?;
            Log::crear_vacio(&self.ruta_log())?;
        } else if !self.ruta_log().is_file() {
            return Err(Error::Recovery(format!(
                "falta {}; el proyecto necesita recuperación",
                self.ruta_log().display()
            )));
        }
        let lock = ProjectLock::adquirir(&escritor, None)?;
        let mut log = Log::cargar(&self.ruta_log())?;
        let blobs = BlobStore::abrir(&escritor)?;
        let recuperacion = recovery::recuperar(&self.raiz, &mut log, &blobs)?;
        let mut escaneo = verify::escanear(&self.raiz)?;
        for conflicto in recuperacion.conflictos {
            escaneo.avisos.push(format!("recuperación: {conflicto}"));
        }
        escaneo.avisos.sort();
        escaneo.avisos.dedup();
        let bloqueantes: Vec<&str> = escaneo
            .errores
            .iter()
            .map(String::as_str)
            .filter(|error| *error != "el proyecto no tiene escenas")
            .collect();
        if escaneo.proyecto.is_none() || (!permitir_errores && !bloqueantes.is_empty()) {
            return Err(Error::Project(format!(
                "el proyecto tiene errores y no se puede escribir: {}",
                bloqueantes.join("; ")
            )));
        }
        let (escenas, notas) = escaneo.filas_indice();
        let operaciones = log.operaciones();
        let indice = abrir_indice(
            &escritor,
            &escenas,
            &notas,
            &operaciones,
            log.revision_actual(),
        )?;
        Ok(SesionEscritura {
            _lock: lock,
            raiz: self.raiz.clone(),
            log,
            blobs,
            indice,
            escaneo,
        })
    }

    /// Reindexa y, opcionalmente, limpia blobs huérfanos.
    pub fn reindexar(&self, limpiar: bool) -> Result<ResultadoReindex> {
        let mut sesion = match self.escritura_con(true) {
            Ok(sesion) => sesion,
            Err(error) if parece_corrupto(&error) => {
                Indice::apartar(&self.ruta_escritor().join("indice.sqlite"))?;
                self.escritura_con(true)?
            }
            Err(error) => return Err(error),
        };
        if limpiar {
            let mut referenciados = sesion.log.blobs_referenciados();
            for documento in sesion.escaneo.ordenados() {
                if let Ok(hex) = hash::hex(documento.hash()) {
                    referenciados.insert(hex.to_string());
                }
            }
            let eliminados = sesion.blobs.limpiar(&referenciados)?;
            sesion.refrescar()?;
            let resumen = sesion.indice.resumen()?;
            return Ok(ResultadoReindex {
                escenas: resumen.escenas,
                notas: resumen.notas,
                revision: sesion.log.revision_actual(),
                blobs_eliminados: eliminados,
            });
        }
        sesion.refrescar()?;
        let resumen = sesion.indice.resumen()?;
        Ok(ResultadoReindex {
            escenas: resumen.escenas,
            notas: resumen.notas,
            revision: sesion.log.revision_actual(),
            blobs_eliminados: 0,
        })
    }

    /// Busca un documento por ID o ruta relativa.
    pub fn leer(&self, id_o_ruta: &str) -> Result<Documento> {
        let sesion = self.lectura()?;
        sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))
    }

    /// Lista los documentos en orden de lectura.
    pub fn listar(&self, incluir_notas: bool) -> Result<Vec<Documento>> {
        let sesion = self.lectura()?;
        let mut documentos = sesion.escaneo.ordenados();
        if !incluir_notas {
            documentos.retain(|documento| matches!(documento, Documento::Escena(_)));
        }
        Ok(documentos)
    }

    /// Busca texto en escenas y notas.
    ///
    /// Usa el índice FTS5 cuando el proyecto ya tiene historial. Si el proyecto
    /// todavía no tiene `.escritor` (por ejemplo, un fixture de solo lectura), no
    /// crea el directorio operativo: busca en memoria sobre los documentos
    /// escaneados.
    pub fn buscar(&self, consulta: &str, limite: usize) -> Result<Vec<ResultadoBusqueda>> {
        let consulta = consulta.trim();
        if consulta.is_empty() {
            return Err(Error::Input("la consulta no puede estar vacía".into()));
        }
        let terminos = terminos_busqueda(consulta)?;
        let limite = limite.clamp(1, 100);
        let sesion = self.lectura()?;
        let (escenas, notas) = sesion.escaneo.filas_indice();

        let indice = if sesion.log.is_some() {
            let operaciones = sesion
                .log
                .as_ref()
                .map(|log| log.operaciones())
                .unwrap_or_default();
            let revision = sesion
                .log
                .as_ref()
                .map(|log| log.revision_actual())
                .unwrap_or(0);
            abrir_indice(
                &self.ruta_escritor(),
                &escenas,
                &notas,
                &operaciones,
                revision,
            )?
        } else {
            let mut indice = crate::index::Indice::en_memoria()?;
            indice.reconstruir(&escenas, &notas, &[], 0)?;
            indice
        };

        let filas = indice.buscar(&consulta_fts(&terminos), limite)?;
        let mut resultados = Vec::new();
        for (id, ruta, fragmento) in filas {
            let documento = sesion
                .escaneo
                .por_id(&id)
                .or_else(|| sesion.escaneo.por_ruta(&ruta));
            if let Some(documento) = documento {
                resultados.push(resultado_busqueda(documento, fragmento));
            }
        }
        resultados.truncate(limite);
        Ok(resultados)
    }

    /// Lista las copias de cuarentena con los metadatos del informe.
    pub fn cuarentena(&self) -> Result<Vec<CuarentenaItem>> {
        let directorio = self.ruta_escritor().join("cuarentena");
        if !directorio.exists() {
            return Ok(Vec::new());
        }
        let directorio = fsutil::directorio_real(&directorio)?;
        let mut informes: BTreeMap<String, InformeCuarentena> = BTreeMap::new();
        let informe = directorio.join("informe.jsonl");
        if std::fs::symlink_metadata(&informe).is_ok() {
            let informe = fsutil::archivo_real(&informe)?;
            if let Ok(texto) = std::fs::read_to_string(&informe) {
                for linea in texto.lines() {
                    if linea.trim().is_empty() {
                        continue;
                    }
                    let Ok(valor) = serde_json::from_str::<serde_json::Value>(linea) else {
                        continue;
                    };
                    let Some(archivo) = valor.get("archivo").and_then(|campo| campo.as_str())
                    else {
                        continue;
                    };
                    let ts = valor
                        .get("ts")
                        .and_then(|campo| campo.as_str())
                        .map(str::to_string);
                    let ruta = valor
                        .get("ruta_original")
                        .and_then(|campo| campo.as_str())
                        .map(str::to_string);
                    let motivo = valor
                        .get("motivo")
                        .and_then(|campo| campo.as_str())
                        .map(str::to_string);
                    informes.insert(
                        archivo.to_string(),
                        InformeCuarentena {
                            ts,
                            ruta_original: ruta,
                            motivo,
                        },
                    );
                }
            }
        }

        let entradas = std::fs::read_dir(&directorio)
            .map_err(|error| Error::Io(error).with_path(&directorio))?;
        let mut archivos = Vec::new();
        for entrada in entradas {
            let entrada = entrada.map_err(|error| Error::Io(error).with_path(&directorio))?;
            let nombre = entrada.file_name().to_string_lossy().to_string();
            if nombre == "informe.jsonl" {
                continue;
            }
            let metadatos = entrada
                .metadata()
                .map_err(|error| Error::Io(error).with_path(entrada.path()))?;
            if !metadatos.is_file() {
                continue;
            }
            let informe = informes.remove(&nombre).unwrap_or_default();
            archivos.push(CuarentenaItem {
                archivo: nombre,
                ruta_original: informe.ruta_original,
                motivo: informe.motivo,
                ts: informe.ts,
                tamano: metadatos.len(),
            });
        }
        archivos.sort_by(|a, b| a.archivo.cmp(&b.archivo));
        Ok(archivos)
    }

    /// Elimina una copia de cuarentena. Nunca toca el archivo original.
    pub fn descartar_cuarentena(&self, archivo: &str) -> Result<()> {
        let nombre = archivo.trim();
        if nombre.is_empty()
            || nombre == "."
            || nombre == ".."
            || nombre == "informe.jsonl"
            || nombre.contains('/')
            || nombre.contains('\\')
        {
            return Err(Error::Input(format!(
                "nombre de cuarentena no válido: {archivo}"
            )));
        }
        let directorio = fsutil::directorio_real(&self.ruta_escritor().join("cuarentena"))?;
        let ruta = directorio.join(nombre);
        match std::fs::symlink_metadata(&ruta) {
            Ok(metadatos) if metadatos.file_type().is_file() => {
                fsutil::eliminar_archivo(&ruta)?;
                Ok(())
            }
            Ok(_) => Err(Error::Input(format!(
                "no es un archivo de cuarentena: {archivo}"
            ))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(Error::NotFound(
                format!("archivo de cuarentena no encontrado: {archivo}"),
            )),
            Err(error) => Err(Error::Io(error).with_path(&ruta)),
        }
    }

    /// Historial confirmado.
    pub fn historial(&self) -> Result<Vec<Operacion>> {
        let sesion = self.lectura()?;
        if sesion.log.is_none() && self.ruta_escritor().exists() {
            return Err(Error::Recovery(
                "falta operaciones.log; el historial no está disponible".to_string(),
            ));
        }
        Ok(sesion.log.map(|log| log.operaciones()).unwrap_or_default())
    }

    /// Detalle de una revisión y los bytes posteriores.
    pub fn mostrar(&self, revision: u64) -> Result<RevisionDetalle> {
        let sesion = self.lectura()?;
        let log = sesion
            .log
            .as_ref()
            .ok_or_else(|| Error::NotFound("el proyecto no tiene historial".to_string()))?;
        let operacion = log
            .operaciones()
            .into_iter()
            .find(|operacion| operacion.prepare.target_rev == revision)
            .ok_or_else(|| Error::NotFound(format!("revisión no encontrada: {revision}")))?;
        let blobs = BlobStore::abrir(&self.ruta_escritor())?;
        let mut contenidos = Vec::new();
        for efecto in &operacion.prepare.efectos {
            let contenido = match &efecto.hash_nuevo {
                Some(hash) => Some(blobs.obtener(hash)?),
                None => None,
            };
            contenidos.push((efecto.ruta.clone(), contenido));
        }
        Ok(RevisionDetalle {
            operacion,
            contenidos,
        })
    }

    /// Crea una escena nueva y devuelve su ID y ruta.
    pub fn crear_escena(
        &self,
        titulo: &str,
        libro: Option<&str>,
        capitulo: Option<&str>,
        orden: Option<i64>,
    ) -> Result<(String, String)> {
        let mut sesion = self.escritura()?;
        let libro = libro.map(str::to_string).unwrap_or_else(|| {
            primer_hijo(&self.raiz.join("manuscrito")).unwrap_or_else(|| "01-libro".to_string())
        });
        let capitulo = capitulo.map(str::to_string).unwrap_or_else(|| {
            primer_hijo(&self.raiz.join("manuscrito").join(&libro))
                .unwrap_or_else(|| "01-capitulo".to_string())
        });
        validar_componente(&libro)?;
        validar_componente(&capitulo)?;
        let orden = match orden {
            Some(valor) => {
                if !(1..=99_999).contains(&valor) {
                    return Err(Error::Input("'orden' debe estar entre 1 y 99999".into()));
                }
                if sesion.escaneo.escenas.iter().any(|escena| {
                    escena
                        .relativa
                        .starts_with(&format!("manuscrito/{libro}/{capitulo}/"))
                        && escena.escena.orden == valor
                }) {
                    return Err(Error::Conflict(format!(
                        "'orden' {valor} ya está usado en {libro}/{capitulo}"
                    )));
                }
                valor
            }
            None => siguiente_orden(&sesion.escaneo, &libro, &capitulo),
        };
        let relativa = nombre_escena_libre(&self.raiz, &libro, &capitulo, titulo)?;
        let escena = Escena::nueva(titulo, orden)?;
        let id = escena.id.clone();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::nuevo(relativa.clone(), escena.a_bytes())],
            "humano",
            "estructura",
            None,
            None,
        )?;
        Ok((id, relativa))
    }

    /// Crea una nota nueva.
    pub fn crear_nota(&self, tipo: &str, titulo: &str) -> Result<(String, String)> {
        if !TIPOS_NOTA.contains(&tipo) {
            return Err(Error::Input(format!("tipo de nota no válido: {tipo}")));
        }
        let mut sesion = self.escritura()?;
        let relativa = nombre_nota_libre(&self.raiz, tipo, titulo)?;
        let nota = Nota::nueva(tipo, titulo)?;
        let id = nota.id.clone();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::nuevo(relativa.clone(), nota.a_bytes())],
            "humano",
            "notas",
            None,
            None,
        )?;
        Ok((id, relativa))
    }

    /// Reemplaza el cuerpo de una escena o nota.
    pub fn escribir_cuerpo(&self, id_o_ruta: &str, cuerpo: &str) -> Result<u64> {
        Ok(self
            .escribir_cuerpo_con_base(id_o_ruta, cuerpo, None)?
            .revision)
    }

    /// Reemplaza el cuerpo con control optimista de concurrencia.
    ///
    /// `hash_base` es el hash del archivo que la interfaz leyó. Si el archivo
    /// cambió desde entonces, se devuelve [`Error::Conflict`] y no se toca nada.
    /// Un `None` mantiene el comportamiento de la CLI (sin comprobación).
    pub fn escribir_cuerpo_con_base(
        &self,
        id_o_ruta: &str,
        cuerpo: &str,
        hash_base: Option<&str>,
    ) -> Result<ResultadoGuardado> {
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        if let Some(esperado) = hash_base {
            if documento.hash() != esperado {
                return Err(Error::Conflict(format!(
                    "el documento cambió fuera del editor: {id_o_ruta}"
                )));
            }
        }
        let cuerpo = normalizar_cuerpo(cuerpo);
        let (ruta, bytes, scope) = match documento {
            Documento::Escena(mut encontrada) => {
                encontrada.escena.cuerpo = cuerpo;
                (
                    encontrada.relativa.clone(),
                    encontrada.escena.a_bytes(),
                    "prosa",
                )
            }
            Documento::Nota(mut encontrada) => {
                encontrada.nota.cuerpo = cuerpo;
                (
                    encontrada.relativa.clone(),
                    encontrada.nota.a_bytes(),
                    "notas",
                )
            }
        };
        let hash_archivo = crate::hash::hash(&bytes);
        let revision = self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(ruta, bytes)],
            "humano",
            scope,
            None,
            None,
        )?;
        Ok(ResultadoGuardado {
            revision,
            hash_archivo,
        })
    }

    /// Mueve una escena o nota sin cambiar su ID.
    pub fn mover(
        &self,
        id_o_ruta: &str,
        libro: Option<&str>,
        capitulo: Option<&str>,
        tipo: Option<&str>,
    ) -> Result<String> {
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        match documento {
            Documento::Escena(encontrada) => {
                let partes: Vec<&str> = encontrada.relativa.split('/').collect();
                let libro = libro.unwrap_or(partes[1]);
                let capitulo = capitulo.unwrap_or(partes[2]);
                validar_componente(libro)?;
                validar_componente(capitulo)?;
                let nombre = partes[3];
                let destino = format!("manuscrito/{libro}/{capitulo}/{nombre}");
                if destino == encontrada.relativa {
                    return Ok(destino);
                }
                if self.raiz.join(&destino).exists() {
                    return Err(Error::Conflict(format!("ya existe {destino}")));
                }
                if sesion.escaneo.escenas.iter().any(|escena| {
                    escena
                        .relativa
                        .starts_with(&format!("manuscrito/{libro}/{capitulo}/"))
                        && escena.escena.orden == encontrada.escena.orden
                        && escena.escena.id != encontrada.escena.id
                }) {
                    return Err(Error::Conflict(format!(
                        "'orden' {} ya está usado en {libro}/{capitulo}",
                        encontrada.escena.orden
                    )));
                }
                let bytes = std::fs::read(&encontrada.ruta)
                    .map_err(|error| Error::Io(error).with_path(&encontrada.ruta))?;
                self.commit_efectos(
                    &mut sesion,
                    vec![
                        Cambio::eliminar(encontrada.relativa.clone()),
                        Cambio::nuevo(destino.clone(), bytes),
                    ],
                    "humano",
                    "estructura",
                    None,
                    None,
                )?;
                Ok(destino)
            }
            Documento::Nota(mut encontrada) => {
                let partes: Vec<&str> = encontrada.relativa.split('/').collect();
                let tipo = tipo.unwrap_or(partes[1]);
                if !TIPOS_NOTA.contains(&tipo) {
                    return Err(Error::Input(format!("tipo de nota no válido: {tipo}")));
                }
                let nombre = partes[2];
                let destino = format!("notas/{tipo}/{nombre}");
                if destino == encontrada.relativa {
                    return Ok(destino);
                }
                if self.raiz.join(&destino).exists() {
                    return Err(Error::Conflict(format!("ya existe {destino}")));
                }
                let bytes = if tipo == encontrada.nota.tipo {
                    std::fs::read(&encontrada.ruta)
                        .map_err(|error| Error::Io(error).with_path(&encontrada.ruta))?
                } else {
                    encontrada.nota.tipo = tipo.to_string();
                    encontrada.nota.a_bytes()
                };
                self.commit_efectos(
                    &mut sesion,
                    vec![
                        Cambio::eliminar(encontrada.relativa.clone()),
                        Cambio::nuevo(destino.clone(), bytes),
                    ],
                    "humano",
                    "estructura",
                    None,
                    None,
                )?;
                Ok(destino)
            }
        }
    }

    /// Elimina un documento. El contenido queda en el historial y se puede revertir.
    pub fn eliminar(&self, id_o_ruta: &str) -> Result<String> {
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        let relativa = documento.relativa().to_string();
        let scope = match documento {
            Documento::Escena(_) => "estructura",
            Documento::Nota(_) => "notas",
        };
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::eliminar(relativa.clone())],
            "humano",
            scope,
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Revierte una revisión concreta creando una revisión nueva.
    ///
    /// A diferencia de una escritura normal, una reversión puede partir de un
    /// proyecto con errores: su objetivo puede ser precisamente repararlos (por
    /// ejemplo, restaurar una nota referenciada). El resultado informa de los
    /// errores que queden después.
    pub fn revertir(&self, revision: u64) -> Result<ResultadoRevertir> {
        let mut sesion = self.escritura_con(true)?;
        let operacion = sesion
            .log
            .operaciones()
            .into_iter()
            .find(|operacion| operacion.prepare.target_rev == revision)
            .ok_or_else(|| Error::NotFound(format!("revisión no encontrada: {revision}")))?;
        let mut cambios = Vec::new();
        let mut conflictos = Vec::new();
        for efecto in &operacion.prepare.efectos {
            let ruta = fsutil::ruta_del_proyecto(&self.raiz, &efecto.ruta)?;
            let actual = hash::hash_archivo(&ruta)?;
            if actual != efecto.hash_nuevo {
                conflictos.push(format!(
                    "{}: el archivo cambió después de la revisión {revision}",
                    efecto.ruta
                ));
                continue;
            }
            let contenido = match &efecto.hash_anterior {
                Some(hash_anterior) => Some(sesion.blobs.obtener(hash_anterior)?),
                None => None,
            };
            cambios.push(match contenido {
                Some(bytes) => Cambio::existente(efecto.ruta.clone(), bytes),
                None => Cambio::eliminar(efecto.ruta.clone()),
            });
        }
        if !conflictos.is_empty() {
            return Err(Error::Conflict(conflictos.join("; ")));
        }
        let nueva = self.commit_efectos(&mut sesion, cambios, "humano", "revert", None, None)?;
        let escaneo = verify::escanear(&self.raiz)?;
        Ok(ResultadoRevertir {
            revision: nueva,
            errores_pendientes: escaneo.errores,
        })
    }

    /// Exporta el manuscrito a un archivo Markdown.
    pub fn exportar(&self, destino: &Path, incluir_notas: bool, forzar: bool) -> Result<PathBuf> {
        let destino_abs = self.preparar_destino(destino, forzar)?;
        let sesion = self.lectura()?;
        if !sesion.escaneo.errores.is_empty() {
            return Err(Error::Project(format!(
                "no se exporta un proyecto con errores: {}",
                sesion.escaneo.errores.join("; ")
            )));
        }
        let contenido = exportar_markdown(&sesion.escaneo, incluir_notas);
        fsutil::escribir_atomico(&destino_abs, contenido.as_bytes())?;
        Ok(destino_abs)
    }

    /// Exporta el manuscrito a un archivo DOCX.
    pub fn exportar_docx(
        &self,
        destino: &Path,
        incluir_notas: bool,
        forzar: bool,
    ) -> Result<PathBuf> {
        let destino_abs = self.preparar_destino(destino, forzar)?;
        let sesion = self.lectura()?;
        if !sesion.escaneo.errores.is_empty() {
            return Err(Error::Project(format!(
                "no se exporta un proyecto con errores: {}",
                sesion.escaneo.errores.join("; ")
            )));
        }
        let contenido = crate::docx::exportar(&sesion.escaneo, incluir_notas)?;
        fsutil::escribir_atomico(&destino_abs, &contenido)?;
        Ok(destino_abs)
    }

    fn preparar_destino(&self, destino: &Path, forzar: bool) -> Result<PathBuf> {
        let raiz_lexica = normalizar_lexica(&absoluta(&self.raiz)?);
        let raiz_real = std::fs::canonicalize(&self.raiz)
            .map_err(|error| Error::Io(error).with_path(&self.raiz))?;
        let destino_abs = normalizar_lexica(&absoluta(destino)?);

        if destino_abs.starts_with(&raiz_lexica) {
            return Err(Error::Input(
                "el destino no puede estar dentro del proyecto".to_string(),
            ));
        }
        if std::fs::symlink_metadata(&destino_abs).is_ok() && !forzar {
            return Err(Error::Input(format!(
                "el destino ya existe: {} (usa --forzar)",
                destino.display()
            )));
        }

        // Resolver enlaces simbólicos del destino y de sus ancestros existentes.
        // Sin esto, un alias externo puede apuntar a un archivo del proyecto.
        let destino_real = resolver_destino(&destino_abs)?;
        if destino_real.starts_with(&raiz_real) {
            return Err(Error::Input(
                "el destino no puede estar dentro del proyecto".to_string(),
            ));
        }
        Ok(destino_real)
    }

    /// Verifica el proyecto; intenta recuperar si existe `.escritor`.
    pub fn verificar(&self) -> Result<Escaneo> {
        match self.lectura() {
            Ok(sesion) => {
                let mut escaneo = sesion.escaneo;
                if let Some(log) = sesion.log.as_ref() {
                    for abort in log.abortadas() {
                        if abort.motivo == "conflicto" {
                            escaneo.avisos.push(format!(
                                "operación {}: reversión con conflicto; se conservó el cambio externo",
                                abort.op_id
                            ));
                        }
                    }
                    let objetos = self.ruta_escritor().join("objetos");
                    if objetos.is_dir() {
                        let blobs = BlobStore::abrir(&self.ruta_escritor())?;
                        for operacion in log.operaciones() {
                            for efecto in &operacion.prepare.efectos {
                                for hash in [&efecto.hash_anterior, &efecto.hash_nuevo]
                                    .into_iter()
                                    .flatten()
                                {
                                    if !blobs.tiene(hash).unwrap_or(false) {
                                        escaneo.avisos.push(format!(
                                            "revisión {}: falta el blob {hash} para {}",
                                            operacion.prepare.target_rev, efecto.ruta
                                        ));
                                    }
                                }
                            }
                        }
                    } else if log.revision_actual() > 0 {
                        escaneo.avisos.push(
                            "faltan todos los blobs del historial (.escritor/objetos)".to_string(),
                        );
                    }
                    escaneo.avisos.sort();
                    escaneo.avisos.dedup();
                }
                Ok(escaneo)
            }
            Err(error) => {
                let mut escaneo = verify::escanear(&self.raiz)?;
                escaneo.errores.push(error.to_string());
                Ok(escaneo)
            }
        }
    }

    /// Aplica una transacción completa.
    pub fn commit_efectos(
        &self,
        sesion: &mut SesionEscritura,
        cambios: Vec<Cambio>,
        autor: &str,
        scope: &str,
        task_id: Option<&str>,
        modelo: Option<&str>,
    ) -> Result<u64> {
        if cambios.is_empty() {
            return Ok(sesion.log.revision_actual());
        }
        let mut vistos = BTreeSet::new();
        let mut preparados = Vec::new();
        for cambio in cambios {
            if !vistos.insert(cambio.ruta.clone()) {
                return Err(Error::Integrity(format!(
                    "ruta repetida en la transacción: {}",
                    cambio.ruta
                )));
            }
            let ruta = fsutil::ruta_del_proyecto(&self.raiz, &cambio.ruta)?;
            let anterior = fsutil::leer_opcional(&ruta)?;
            let hash_anterior = anterior.as_ref().map(|bytes| hash::hash(bytes));
            if cambio.crear_solo && hash_anterior.is_some() {
                return Err(Error::Conflict(format!(
                    "el archivo ya existe: {}",
                    cambio.ruta
                )));
            }
            let hash_nuevo = cambio.contenido.as_ref().map(|bytes| hash::hash(bytes));
            if hash_anterior == hash_nuevo {
                continue;
            }
            if let Some(bytes) = &anterior {
                sesion.blobs.poner(bytes)?;
            }
            if let Some(bytes) = &cambio.contenido {
                sesion.blobs.poner(bytes)?;
            }
            preparados.push(Preparado {
                efecto: Efecto {
                    ruta: cambio.ruta,
                    hash_nuevo,
                    hash_anterior,
                },
                contenido: cambio.contenido,
            });
        }
        if preparados.is_empty() {
            return Ok(sesion.log.revision_actual());
        }

        let op_id = ulid::nuevo()?;
        let base_rev = sesion.log.revision_actual();
        let efectos: Vec<Efecto> = preparados
            .iter()
            .map(|preparado| preparado.efecto.clone())
            .collect();
        sesion.log.agregar_prepare(crate::oplog::PrepareInput {
            op_id: &op_id,
            base_rev,
            autor,
            scope,
            task_id,
            modelo,
            efectos,
        })?;
        sesion.log.sincronizar()?;
        fault::punto("prepare")?;

        let mut aplicados: Vec<Efecto> = Vec::new();
        let mut error_aplicacion: Option<Error> = None;
        for preparado in &preparados {
            let ruta = fsutil::ruta_del_proyecto(&self.raiz, &preparado.efecto.ruta)?;
            match aplicar_efecto(&ruta, preparado) {
                Ok(()) => {
                    aplicados.push(preparado.efecto.clone());
                    if let Err(error) = fault::despues_de_escribir(aplicados.len()) {
                        error_aplicacion = Some(error);
                        break;
                    }
                }
                Err(error) => {
                    // El reemplazo o el borrado puede haberse completado antes
                    // de que falle el fsync del directorio. Este efecto debe
                    // entrar en el rollback: allí se comprueba el hash actual y
                    // solo se restaura si de verdad cambió el disco.
                    aplicados.push(preparado.efecto.clone());
                    error_aplicacion = Some(error);
                    break;
                }
            }
        }

        if let Some(error) = error_aplicacion {
            let conflictos = revertir_aplicados(&self.raiz, &sesion.blobs, &aplicados)?;
            let es_conflicto = matches!(error, Error::Conflict(_)) || !conflictos.is_empty();
            let motivo = if es_conflicto { "conflicto" } else { "error" };
            if !sesion.log.abortada(&op_id) {
                sesion.log.agregar_abort(&op_id, motivo)?;
                sesion.log.sincronizar()?;
            }
            if !conflictos.is_empty() {
                return Err(Error::Conflict(format!(
                    "{error}; al revertir: {}",
                    conflictos.join("; ")
                )));
            }
            return Err(error);
        }

        sesion.log.agregar_commit(&op_id, base_rev + 1)?;
        sesion.log.sincronizar()?;
        fault::punto("commit")?;

        if let Err(error) = sesion.refrescar() {
            eprintln!("aviso: no se pudo reconstruir el índice: {error}");
        }
        Ok(base_rev + 1)
    }

    fn ruta_escritor(&self) -> PathBuf {
        self.raiz.join(".escritor")
    }

    fn ruta_log(&self) -> PathBuf {
        self.ruta_escritor().join("operaciones.log")
    }
}

impl SesionEscritura {
    /// Vuelve a escanear y reconstruye el índice.
    pub fn refrescar(&mut self) -> Result<()> {
        self.escaneo = verify::escanear(&self.raiz)?;
        let (escenas, notas) = self.escaneo.filas_indice();
        self.indice.reconstruir(
            &escenas,
            &notas,
            &self.log.operaciones(),
            self.log.revision_actual(),
        )?;
        Ok(())
    }
}

fn aplicar_efecto(ruta: &Path, preparado: &Preparado) -> Result<()> {
    fault::antes_de_comprobar(ruta)?;
    let actual = hash::hash_archivo(ruta)?;
    if actual != preparado.efecto.hash_anterior {
        return Err(Error::Conflict(format!(
            "CAS: {} cambió antes del reemplazo",
            preparado.efecto.ruta
        )));
    }
    fault::antes_de_reemplazar(ruta)?;
    match &preparado.contenido {
        Some(bytes) => fsutil::escribir_atomico(ruta, bytes),
        None => fsutil::eliminar_archivo(ruta),
    }
}

fn revertir_aplicados(raiz: &Path, blobs: &BlobStore, aplicados: &[Efecto]) -> Result<Vec<String>> {
    let mut conflictos = Vec::new();
    for efecto in aplicados.iter().rev() {
        let ruta = fsutil::ruta_del_proyecto(raiz, &efecto.ruta)?;
        let actual = hash::hash_archivo(&ruta)?;
        if actual == efecto.hash_nuevo {
            match &efecto.hash_anterior {
                Some(anterior) => {
                    let bytes = blobs.obtener(anterior)?;
                    fsutil::escribir_atomico(&ruta, &bytes)?;
                }
                None => fsutil::eliminar_archivo(&ruta)?,
            }
        } else if actual == efecto.hash_anterior {
            // Ya está en el estado anterior; se sincroniza para no persistir un
            // ABORT sobre un estado que aún no es durable.
            if efecto.hash_anterior.is_some() {
                fsutil::sincronizar_archivo(&ruta)?;
            } else if let Some(padre) = ruta.parent() {
                fsutil::fsync_dir(padre)?;
            }
        } else {
            conflictos.push(format!(
                "{}: cambió por fuera durante la reversión",
                efecto.ruta
            ));
        }
    }
    Ok(conflictos)
}

fn siguiente_orden(escaneo: &Escaneo, libro: &str, capitulo: &str) -> i64 {
    let prefijo = format!("manuscrito/{libro}/{capitulo}/");
    escaneo
        .escenas
        .iter()
        .filter(|escena| escena.relativa.starts_with(&prefijo))
        .map(|escena| escena.escena.orden)
        .max()
        .map(|orden| (orden / 10 + 1) * 10)
        .unwrap_or(10)
}

fn nombre_escena_libre(raiz: &Path, libro: &str, capitulo: &str, titulo: &str) -> Result<String> {
    let directorio = raiz.join("manuscrito").join(libro).join(capitulo);
    let mut numero = siguiente_prefijo(&directorio);
    loop {
        let nombre = format!("{numero:03}-{}.md", model::slug(titulo));
        let relativa = format!("manuscrito/{libro}/{capitulo}/{nombre}");
        if !directorio.join(&nombre).exists() {
            return Ok(relativa);
        }
        numero += 1;
    }
}

fn siguiente_prefijo(directorio: &Path) -> u32 {
    let maximo = std::fs::read_dir(directorio)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entrada| entrada.ok())
        .filter_map(|entrada| {
            let nombre = entrada.file_name().to_string_lossy().to_string();
            if !nombre.ends_with(".md") {
                return None;
            }
            nombre.split_once('-')?.0.parse::<u32>().ok()
        })
        .max()
        .unwrap_or(0);
    maximo.saturating_add(1)
}

fn nombre_nota_libre(raiz: &Path, tipo: &str, titulo: &str) -> Result<String> {
    let directorio = raiz.join("notas").join(tipo);
    let base = model::slug(titulo);
    let mut nombre = format!("{base}.md");
    let mut numero = 2_u32;
    while directorio.join(&nombre).exists() {
        nombre = format!("{base}-{numero}.md");
        numero += 1;
    }
    Ok(format!("notas/{tipo}/{nombre}"))
}

fn primer_hijo(directorio: &Path) -> Option<String> {
    let mut hijos: Vec<String> = std::fs::read_dir(directorio)
        .ok()?
        .filter_map(|entrada| entrada.ok())
        .filter(|entrada| {
            entrada
                .file_type()
                .map(|tipo| tipo.is_dir())
                .unwrap_or(false)
        })
        .map(|entrada| entrada.file_name().to_string_lossy().to_string())
        .collect();
    hijos.sort();
    hijos.into_iter().next()
}

fn validar_componente(componente: &str) -> Result<()> {
    if componente.is_empty()
        || componente.contains('/')
        || componente.contains('\\')
        || componente == "."
        || componente == ".."
    {
        return Err(Error::Input(format!(
            "componente de ruta no válido: {componente}"
        )));
    }
    Ok(())
}

fn resultado_busqueda(documento: Documento, fragmento: String) -> ResultadoBusqueda {
    let tipo = match &documento {
        Documento::Escena(_) => "escena",
        Documento::Nota(_) => "nota",
    };
    ResultadoBusqueda {
        id: documento.id().to_string(),
        ruta: documento.relativa().to_string(),
        titulo: documento.titulo().to_string(),
        tipo: tipo.to_string(),
        fragmento,
    }
}

fn terminos_busqueda(consulta: &str) -> Result<Vec<String>> {
    let terminos: Vec<String> = consulta
        .split_whitespace()
        .filter(|termino| termino.chars().any(char::is_alphanumeric))
        .map(str::to_string)
        .collect();
    if terminos.is_empty() {
        return Err(Error::Input(
            "la consulta no contiene términos de búsqueda".into(),
        ));
    }
    Ok(terminos)
}

/// Convierte una consulta humana en términos FTS5 citados.
///
/// La UI y la CLI reciben texto libre; citar cada término evita que `"`, `*`,
/// `-` u otros operadores cambien el significado de la búsqueda. El fallback
/// sin índice aplica la misma semántica de términos (todos deben aparecer).
fn consulta_fts(terminos: &[String]) -> String {
    terminos
        .iter()
        .map(|termino| format!("\"{}\"", termino.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalizar_cuerpo(cuerpo: &str) -> String {
    let mut normalizado = cuerpo.replace("\r\n", "\n").replace('\r', "\n");
    if !normalizado.is_empty() && !normalizado.ends_with('\n') {
        normalizado.push('\n');
    }
    normalizado
}

fn exportar_markdown(escaneo: &Escaneo, incluir_notas: bool) -> String {
    let mut salida = String::new();
    let mut libro_actual = String::new();
    let mut capitulo_actual = String::new();
    let mut primera_escena = true;
    for documento in escaneo.ordenados() {
        let Documento::Escena(escena) = documento else {
            continue;
        };
        let partes: Vec<&str> = escena.relativa.split('/').collect();
        if partes.len() != 4 {
            continue;
        }
        let libro = model::titulo_humano(partes[1]);
        let capitulo = model::titulo_humano(partes[2]);
        if libro != libro_actual {
            if !salida.is_empty() {
                salida.push_str("\n\n");
            }
            salida.push_str(&format!("# {libro}\n"));
            libro_actual = libro;
            capitulo_actual.clear();
        }
        if capitulo != capitulo_actual {
            salida.push_str(&format!("\n## {capitulo}\n"));
            capitulo_actual = capitulo;
        }
        if !primera_escena {
            salida.push_str("\n***\n");
        }
        primera_escena = false;
        salida.push_str(&format!("\n### {}\n\n", escena.escena.titulo));
        salida.push_str(escena.escena.cuerpo.trim_end());
        salida.push('\n');
    }
    if incluir_notas {
        let mut tipo_actual = String::new();
        if !salida.is_empty() {
            salida.push_str("\n\n");
        }
        salida.push_str("# Notas\n");
        let mut notas = escaneo.notas.clone();
        notas.sort_by_key(|a| (a.nota.tipo.clone(), a.nota.titulo.clone()));
        for nota in notas {
            if nota.nota.tipo != tipo_actual {
                salida.push_str(&format!("\n## {}\n", model::titulo_humano(&nota.nota.tipo)));
                tipo_actual = nota.nota.tipo.clone();
            }
            salida.push_str(&format!("\n### {}\n\n", nota.nota.titulo));
            salida.push_str(nota.nota.cuerpo.trim_end());
            salida.push('\n');
        }
    }
    if !salida.ends_with('\n') {
        salida.push('\n');
    }
    salida
}

fn absoluta(ruta: &Path) -> Result<PathBuf> {
    if ruta.is_absolute() {
        return Ok(ruta.to_path_buf());
    }
    let actual = std::env::current_dir().map_err(Error::Io)?;
    Ok(actual.join(ruta))
}

/// Normaliza `.` y `..` sin tocar el sistema de archivos.
fn normalizar_lexica(ruta: &Path) -> PathBuf {
    use std::path::Component;
    let mut salida = PathBuf::new();
    for componente in ruta.components() {
        match componente {
            Component::Prefix(prefix) => salida.push(prefix.as_os_str()),
            Component::RootDir => salida.push(componente.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                salida.pop();
            }
            Component::Normal(parte) => salida.push(parte),
        }
    }
    salida
}

/// Resuelve los enlaces simbólicos del destino aunque el archivo final no exista.
///
/// Canonicaliza el ancestro existente más profundo y vuelve a añadir los
/// componentes que faltan. Así un alias externo que apunta al proyecto no puede
/// colarse como destino de exportación.
fn resolver_destino(ruta: &Path) -> Result<PathBuf> {
    let mut actual = ruta.to_path_buf();
    let mut faltantes: Vec<std::ffi::OsString> = Vec::new();
    loop {
        match std::fs::canonicalize(&actual) {
            Ok(mut resuelto) => {
                for componente in faltantes.iter().rev() {
                    resuelto.push(componente);
                }
                return Ok(resuelto);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(nombre) = actual.file_name() else {
                    return Err(Error::Io(error).with_path(&actual));
                };
                faltantes.push(nombre.to_os_string());
                let Some(padre) = actual.parent() else {
                    return Err(Error::Io(error).with_path(&actual));
                };
                if padre == actual {
                    return Err(Error::Io(error).with_path(&actual));
                }
                actual = padre.to_path_buf();
            }
            Err(error) => return Err(Error::Io(error).with_path(&actual)),
        }
    }
}

fn parece_corrupto(error: &Error) -> bool {
    let texto = error.to_string().to_lowercase();
    texto.contains("malformed")
        || texto.contains("not a database")
        || texto.contains("corrupt")
        || texto.contains("disk image")
}

fn abrir_indice(
    escritor: &Path,
    escenas: &[EscenaFila],
    notas: &[NotaFila],
    operaciones: &[Operacion],
    revision: u64,
) -> Result<Indice> {
    let mut indice = Indice::abrir(escritor)?;
    if let Ok(resumen) = indice.resumen() {
        if resumen.escenas == escenas.len()
            && resumen.notas == notas.len()
            && resumen.revision == revision
        {
            return Ok(indice);
        }
    }
    match indice.reconstruir(escenas, notas, operaciones, revision) {
        Ok(()) => Ok(indice),
        Err(error) if parece_corrupto(&error) => {
            drop(indice);
            Indice::apartar(&escritor.join("indice.sqlite"))?;
            let mut nuevo = Indice::abrir(escritor)?;
            nuevo.reconstruir(escenas, notas, operaciones, revision)?;
            Ok(nuevo)
        }
        Err(error) => Err(error),
    }
}
