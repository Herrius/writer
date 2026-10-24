//! Operaciones de proyecto: apertura, transacciones, creación, edición,
//! movimiento, borrado, historial y exportación.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::blob::BlobStore;
use crate::changes::{self, CambioLocal};
use crate::diff::{self, Diferencias};
use crate::error::{Error, Result};
use crate::fault;
use crate::fsutil;
use crate::hash;
use crate::index::{EscenaFila, Indice, NotaFila};
use crate::lock::ProjectLock;
use crate::model::{self, Escena, Nota, Proyecto, ESTADOS_ARCHIVO, TIPOS_NOTA};
use crate::oplog::{Efecto, Entrada, Log, Operacion};
use crate::recovery;
use crate::time;
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
    hash_esperado: Option<String>,
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
    /// Hash que debe tener el archivo antes de aplicar el cambio.
    pub hash_esperado: Option<String>,
}

impl Cambio {
    /// Escribe o reemplaza un archivo existente.
    pub fn existente(ruta: impl Into<String>, contenido: Vec<u8>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: Some(contenido),
            crear_solo: false,
            hash_esperado: None,
        }
    }

    /// Escribe un archivo existente comprobando el hash base hasta el reemplazo.
    pub fn existente_con_base(
        ruta: impl Into<String>,
        contenido: Vec<u8>,
        hash_base: Option<&str>,
    ) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: Some(contenido),
            crear_solo: false,
            hash_esperado: hash_base.map(str::to_string),
        }
    }

    /// Crea un archivo nuevo; falla si ya existe.
    pub fn nuevo(ruta: impl Into<String>, contenido: Vec<u8>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: Some(contenido),
            crear_solo: true,
            hash_esperado: None,
        }
    }

    /// Elimina un archivo.
    pub fn eliminar(ruta: impl Into<String>) -> Self {
        Self {
            ruta: ruta.into(),
            contenido: None,
            crear_solo: false,
            hash_esperado: None,
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

/// Comparación de un documento actual con una revisión.
#[derive(Clone, Debug)]
pub struct Comparacion {
    /// Revisión comparada.
    pub revision: u64,
    /// Ruta relativa del documento.
    pub ruta: String,
    /// Si el documento existía al terminar la revisión.
    pub base_existe: bool,
    /// Diferencias entre la revisión y el estado actual.
    pub diferencias: Diferencias,
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
    ///
    /// La ruta se normaliza y se valida en el backend, no en la interfaz: se
    /// rechazan rutas vacías, `.`/`..`, enlaces en el destino y nombres no
    /// válidos. El proyecto se crea bajo el padre canónico.
    pub fn init(&self, titulo: &str, autor: Option<&str>, idioma: Option<&str>) -> Result<()> {
        // Validar el manifiesto antes de tocar el disco evita dejar una carpeta
        // vacía cuando el título, el autor o el idioma no son válidos.
        let manifiesto = Proyecto::nuevo(titulo, autor, idioma)?;
        let (raiz, creada) = preparar_raiz_inicial(&self.raiz)?;
        let resultado = (|| -> Result<()> {
            let proyecto = Project::new(&raiz);
            fsutil::escribir_atomico(&raiz.join("proyecto.yml"), &manifiesto.a_bytes())?;
            for directorio in [
                "manuscrito",
                "notas",
                ".escritor/objetos",
                ".escritor/cuarentena",
            ] {
                let ruta = raiz.join(directorio);
                std::fs::create_dir_all(&ruta)
                    .map_err(|error| Error::Io(error).with_path(&ruta))?;
            }
            fsutil::validar_escritor(&raiz)?;
            Log::crear_vacio(&proyecto.ruta_log())?;

            let mut sesion = proyecto.escritura()?;
            let mut escena = Escena::nueva("Primera escena", 10)?;
            escena.cuerpo = String::new();
            let ruta = "manuscrito/01-libro/01-capitulo/001-primera-escena.md".to_string();
            proyecto.commit_efectos(
                &mut sesion,
                vec![Cambio::nuevo(ruta, escena.a_bytes())],
                "humano",
                "estructura",
                None,
                None,
            )?;
            Ok(())
        })();
        if resultado.is_err() && creada {
            // Solo se elimina la carpeta que creó esta operación; una carpeta
            // vacía preexistente pertenece al usuario y no se toca.
            let _ = std::fs::remove_dir_all(&raiz);
        }
        resultado
    }

    /// Abre una sesión de lectura; si existe `.escritor`, recupera pendientes.
    pub fn lectura(&self) -> Result<SesionLectura> {
        let Some(escritor) = fsutil::validar_escritor(&self.raiz)? else {
            let escaneo = verify::escanear(&self.raiz)?;
            return Ok(SesionLectura {
                _lock: None,
                log: None,
                escaneo,
            });
        };
        let lock = ProjectLock::adquirir(&escritor, None)?;
        let ruta_log = escritor.join("operaciones.log");
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
        let (escritor, nuevo) = match fsutil::validar_escritor(&self.raiz)? {
            Some(real) => (real, false),
            None => {
                let nuevo = self.raiz.join(".escritor");
                std::fs::create_dir_all(&nuevo)
                    .map_err(|error| Error::Io(error).with_path(&nuevo))?;
                let real = fsutil::validar_escritor(&self.raiz)?.ok_or_else(|| {
                    Error::Integrity(format!(
                        "no se pudo crear el directorio operativo {}",
                        nuevo.display()
                    ))
                })?;
                (real, true)
            }
        };
        let ruta_log = escritor.join("operaciones.log");
        if nuevo {
            std::fs::create_dir_all(escritor.join("objetos"))
                .map_err(|error| Error::Io(error).with_path(&escritor))?;
            std::fs::create_dir_all(escritor.join("cuarentena"))
                .map_err(|error| Error::Io(error).with_path(&escritor))?;
            Log::crear_vacio(&ruta_log)?;
            fsutil::validar_escritor(&self.raiz)?;
        } else if !ruta_log.is_file() {
            return Err(Error::Recovery(format!(
                "falta {}; el proyecto necesita recuperación",
                ruta_log.display()
            )));
        }
        let lock = ProjectLock::adquirir(&escritor, None)?;
        let mut log = Log::cargar(&ruta_log)?;
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
        // `abrir_indice` ya aparta un índice corrupto usando la ruta canónica y
        // validada de `.escritor`. No se decide por el texto de un error que
        // puede contener la ruta del proyecto (por ejemplo, `novela-corrupta`).
        let mut sesion = self.escritura_con(true)?;
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
        let Some(directorio) = fsutil::directorio_cuarentena(&self.raiz, false)? else {
            return Ok(Vec::new());
        };
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
        let nombre = archivo;
        if nombre.trim().is_empty()
            || nombre == "."
            || nombre == ".."
            || nombre == "informe.jsonl"
            || nombre.contains('/')
            || nombre.contains('\\')
            || nombre.contains(':')
        {
            return Err(Error::Input(format!(
                "nombre de cuarentena no válido: {archivo}"
            )));
        }
        let directorio = fsutil::directorio_cuarentena(&self.raiz, false)?.ok_or_else(|| {
            Error::NotFound(format!("archivo de cuarentena no encontrado: {archivo}"))
        })?;
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

    /// Reordena una escena dentro de su capítulo.
    pub fn reordenar_escena(&self, id_o_ruta: &str, nuevo_orden: i64) -> Result<String> {
        if !(1..=99_999).contains(&nuevo_orden) {
            return Err(Error::Input(
                "'orden' debe estar entre 1 y 99999".to_string(),
            ));
        }
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
        let Documento::Escena(mut encontrada) = documento else {
            return Err(Error::Input("solo se pueden reordenar escenas".to_string()));
        };
        let prefijo = encontrada
            .relativa
            .rsplit_once('/')
            .map(|(padre, _)| format!("{padre}/"))
            .unwrap_or_default();
        if sesion.escaneo.escenas.iter().any(|escena| {
            escena.relativa.starts_with(&prefijo)
                && escena.escena.orden == nuevo_orden
                && escena.escena.id != encontrada.escena.id
        }) {
            return Err(Error::Conflict(format!(
                "'orden' {nuevo_orden} ya está usado en el capítulo"
            )));
        }
        encontrada.escena.orden = nuevo_orden;
        let relativa = encontrada.relativa.clone();
        let bytes = encontrada.escena.a_bytes();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(relativa.clone(), bytes)],
            "humano",
            "estructura",
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Intercambia el orden de dos escenas del mismo capítulo en una sola
    /// transacción.
    pub fn intercambiar_orden(&self, id_a: &str, id_b: &str) -> Result<()> {
        let mut sesion = self.escritura()?;
        let buscar = |id: &str| -> Result<(String, i64, String)> {
            let documento = sesion
                .escaneo
                .por_id(id)
                .or_else(|| sesion.escaneo.por_ruta(id))
                .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id}")))?;
            self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
            let Documento::Escena(encontrada) = documento else {
                return Err(Error::Input("solo se pueden reordenar escenas".to_string()));
            };
            Ok((
                encontrada.relativa.clone(),
                encontrada.escena.orden,
                encontrada
                    .relativa
                    .rsplit_once('/')
                    .map(|(p, _)| p.to_string())
                    .unwrap_or_default(),
            ))
        };
        let (ruta_a, orden_a, padre_a) = buscar(id_a)?;
        let (ruta_b, orden_b, padre_b) = buscar(id_b)?;
        if padre_a != padre_b {
            return Err(Error::Input(
                "las escenas deben estar en el mismo capítulo".to_string(),
            ));
        }
        let mut cambios = Vec::new();
        for (ruta, nuevo) in [(ruta_a.clone(), orden_b), (ruta_b.clone(), orden_a)] {
            let documento = sesion
                .escaneo
                .por_ruta(&ruta)
                .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {ruta}")))?;
            let Documento::Escena(mut encontrada) = documento else {
                continue;
            };
            encontrada.escena.orden = nuevo;
            cambios.push(Cambio::existente(ruta, encontrada.escena.a_bytes()));
        }
        self.commit_efectos(&mut sesion, cambios, "humano", "estructura", None, None)?;
        Ok(())
    }

    /// Convierte una nota de Ideas en un proyecto nuevo.
    pub fn convertir_idea_en_proyecto(
        &self,
        id_o_ruta: &str,
        destino: &std::path::Path,
        titulo: &str,
    ) -> Result<std::path::PathBuf> {
        let documento = self.leer(id_o_ruta)?;
        let Documento::Nota(encontrada) = documento else {
            return Err(Error::Input(
                "solo una nota de Ideas puede convertirse en proyecto".to_string(),
            ));
        };
        if encontrada.nota.tipo != "ideas" {
            return Err(Error::Input("la nota no es de tipo ideas".to_string()));
        }
        let nuevo = Project::new(destino);
        nuevo.init(
            titulo,
            encontrada.nota.extra.get("autor").and_then(|valor| {
                if let crate::yaml::Valor::Texto(texto) = valor {
                    Some(texto.as_str())
                } else {
                    None
                }
            }),
            None,
        )?;
        if let Some(primera) = nuevo.listar(false)?.into_iter().next() {
            let id = primera.id().to_string();
            nuevo.eliminar(&id)?;
        }
        let (escena, _) = nuevo.crear_escena(titulo, None, None, None)?;
        nuevo.escribir_cuerpo(&escena, &encontrada.nota.cuerpo)?;
        Ok(destino.to_path_buf())
    }

    /// Cambia el título visible de una escena o nota.
    ///
    /// No cambia el nombre del archivo: las rutas y los IDs se mantienen para
    /// no romper referencias. El título es metadato del documento.
    pub fn renombrar(&self, id_o_ruta: &str, titulo: &str) -> Result<String> {
        let titulo = titulo.trim();
        if titulo.is_empty() {
            return Err(Error::Input("el título no puede estar vacío".into()));
        }
        model::validar_texto(titulo, "titulo", "documento")?;
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
        let (relativa, bytes, scope) = match documento {
            Documento::Escena(mut encontrada) => {
                encontrada.escena.titulo = titulo.to_string();
                (
                    encontrada.relativa.clone(),
                    encontrada.escena.a_bytes(),
                    "estructura",
                )
            }
            Documento::Nota(mut encontrada) => {
                encontrada.nota.titulo = titulo.to_string();
                (
                    encontrada.relativa.clone(),
                    encontrada.nota.a_bytes(),
                    "notas",
                )
            }
        };
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(relativa.clone(), bytes)],
            "humano",
            scope,
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Actualiza estado y etiquetas de una escena.
    pub fn actualizar_escena(
        &self,
        id_o_ruta: &str,
        estado: Option<&str>,
        etiquetas: &[String],
    ) -> Result<String> {
        if let Some(valor) = estado {
            if !model::ESTADOS_ESCENA.contains(&valor) {
                return Err(Error::Input(format!("estado de escena no válido: {valor}")));
            }
        }
        let etiquetas = normalizar_etiquetas(etiquetas)?;
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
        let Documento::Escena(mut encontrada) = documento else {
            return Err(Error::Input(
                "solo las escenas tienen estado de escena".to_string(),
            ));
        };
        encontrada.escena.estado = estado.map(str::to_string);
        encontrada.escena.etiquetas = etiquetas;
        let relativa = encontrada.relativa.clone();
        let bytes = encontrada.escena.a_bytes();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(relativa.clone(), bytes)],
            "humano",
            "estructura",
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Actualiza las etiquetas de una nota.
    pub fn actualizar_nota(&self, id_o_ruta: &str, etiquetas: &[String]) -> Result<String> {
        let etiquetas = normalizar_etiquetas(etiquetas)?;
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
        let Documento::Nota(mut encontrada) = documento else {
            return Err(Error::Input(
                "actualizar_nota solo acepta notas".to_string(),
            ));
        };
        encontrada.nota.etiquetas = etiquetas;
        let relativa = encontrada.relativa.clone();
        let bytes = encontrada.nota.a_bytes();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(relativa.clone(), bytes)],
            "humano",
            "notas",
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Actualiza los campos de ficha de un personaje.
    pub fn actualizar_personaje(
        &self,
        id_o_ruta: &str,
        rol: Option<&str>,
        cast: &[String],
        alias: &[String],
    ) -> Result<String> {
        if let Some(valor) = rol {
            if !model::ROLES_PERSONAJE.contains(&valor) {
                return Err(Error::Input(format!("rol de personaje no válido: {valor}")));
            }
        }
        let cast = normalizar_etiquetas(cast)?;
        let alias = normalizar_etiquetas(alias)?;
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
        let Documento::Nota(mut encontrada) = documento else {
            return Err(Error::Input(
                "actualizar_personaje solo acepta notas".to_string(),
            ));
        };
        if encontrada.nota.tipo != "personajes" {
            return Err(Error::Input("la nota no es de tipo personajes".to_string()));
        }
        encontrada.nota.rol = rol.map(str::to_string);
        encontrada.nota.cast = cast;
        encontrada.nota.alias = alias;
        let relativa = encontrada.relativa.clone();
        let bytes = encontrada.nota.a_bytes();
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(relativa.clone(), bytes)],
            "humano",
            "notas",
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Cambia la ubicación lógica de una escena o nota.
    ///
    /// `None` la devuelve a la vista activa; `archivado` la oculta del árbol
    /// principal; `papelera` la marca para borrado definitivo.
    pub fn cambiar_archivo(&self, id_o_ruta: &str, archivo: Option<&str>) -> Result<String> {
        if let Some(valor) = archivo {
            if !ESTADOS_ARCHIVO.contains(&valor) {
                return Err(Error::Input(format!(
                    "ubicación de archivo no válida: {valor}"
                )));
            }
        }
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
        let (relativa, bytes, scope) = match documento {
            Documento::Escena(mut encontrada) => {
                encontrada.escena.archivo = archivo.map(str::to_string);
                (
                    encontrada.relativa.clone(),
                    encontrada.escena.a_bytes(),
                    "estructura",
                )
            }
            Documento::Nota(mut encontrada) => {
                encontrada.nota.archivo = archivo.map(str::to_string);
                (
                    encontrada.relativa.clone(),
                    encontrada.nota.a_bytes(),
                    "notas",
                )
            }
        };
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::existente(relativa.clone(), bytes)],
            "humano",
            scope,
            None,
            None,
        )?;
        Ok(relativa)
    }

    /// Duplica una escena o nota con un ID nuevo y sin historial.
    ///
    /// La copia conserva metadatos y cuerpo, pero no comparte ID ni revisión
    /// con el original. En escenas se asigna el siguiente `orden` libre del
    /// capítulo; en notas se reutiliza el tipo.
    pub fn duplicar(&self, id_o_ruta: &str) -> Result<(String, String)> {
        let mut sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        let (id, relativa, bytes, scope) = match documento {
            Documento::Escena(encontrada) => {
                let partes: Vec<&str> = encontrada.relativa.split('/').collect();
                let libro = partes[1];
                let capitulo = partes[2];
                let titulo = format!("Copia de {}", encontrada.escena.titulo);
                let orden = siguiente_orden(&sesion.escaneo, libro, capitulo);
                let relativa = nombre_escena_libre(&self.raiz, libro, capitulo, &titulo)?;
                let mut nueva = Escena::nueva(&titulo, orden)?;
                nueva.estado = encontrada.escena.estado.clone();
                nueva.pov = encontrada.escena.pov.clone();
                nueva.tramas = encontrada.escena.tramas.clone();
                nueva.personajes = encontrada.escena.personajes.clone();
                nueva.fecha_narrativa = encontrada.escena.fecha_narrativa.clone();
                nueva.etiquetas = encontrada.escena.etiquetas.clone();
                nueva.extra = encontrada.escena.extra.clone();
                nueva.cuerpo = encontrada.escena.cuerpo.clone();
                let id = nueva.id.clone();
                (id, relativa, nueva.a_bytes(), "estructura")
            }
            Documento::Nota(encontrada) => {
                let titulo = format!("Copia de {}", encontrada.nota.titulo);
                let relativa = nombre_nota_libre(&self.raiz, &encontrada.nota.tipo, &titulo)?;
                let mut nueva = Nota::nueva(&encontrada.nota.tipo, &titulo)?;
                nueva.fija = encontrada.nota.fija;
                nueva.etiquetas = encontrada.nota.etiquetas.clone();
                nueva.refs = encontrada.nota.refs.clone();
                nueva.extra = encontrada.nota.extra.clone();
                nueva.cuerpo = encontrada.nota.cuerpo.clone();
                let id = nueva.id.clone();
                (id, relativa, nueva.a_bytes(), "notas")
            }
        };
        self.commit_efectos(
            &mut sesion,
            vec![Cambio::nuevo(relativa.clone(), bytes)],
            "humano",
            scope,
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
        self.exigir_sin_cambio_pendiente_lenient(documento.id(), documento.relativa())?;
        self.guardar_cuerpo_en_sesion(&mut sesion, id_o_ruta, cuerpo, hash_base)
    }

    fn guardar_cuerpo_en_sesion(
        &self,
        sesion: &mut SesionEscritura,
        id_o_ruta: &str,
        cuerpo: &str,
        hash_base: Option<&str>,
    ) -> Result<ResultadoGuardado> {
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
            sesion,
            vec![Cambio::existente_con_base(ruta, bytes, hash_base)],
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

    /// Rechaza operaciones estructurales mientras haya una propuesta pendiente.
    ///
    /// Las operaciones destructivas fallan cerradas: si no se puede verificar
    /// el almacén de propuestas, no se mueve, borra ni revierte el documento.
    fn exigir_sin_cambio_pendiente(&self, documento: &str, ruta: &str) -> Result<()> {
        if changes::para_documento(&self.raiz, documento, ruta)?.is_some() {
            return Err(Error::Conflict(format!(
                "hay un cambio propuesto pendiente para {ruta}; acéptalo o recházalo antes de modificarlo"
            )));
        }
        Ok(())
    }

    /// Variante para el guardado normal del manuscrito.
    ///
    /// Un almacén de propuestas ilegible no debe impedir escribir el documento;
    /// el panel de cambios mostrará su propio error. La comprobación estricta se
    /// usa para mover, borrar, cambiar metadatos y revertir.
    fn exigir_sin_cambio_pendiente_lenient(&self, documento: &str, ruta: &str) -> Result<()> {
        match changes::para_documento(&self.raiz, documento, ruta) {
            Ok(Some(_)) => Err(Error::Conflict(format!(
                "hay un cambio propuesto pendiente para {ruta}; acéptalo o recházalo antes de modificarlo"
            ))),
            Ok(None) | Err(_) => Ok(()),
        }
    }

    /// Lista los cambios locales pendientes.
    pub fn listar_cambios(&self) -> Result<Vec<CambioLocal>> {
        let _sesion = self.lectura()?;
        changes::listar(&self.raiz)
    }

    /// Lee un cambio local por ULID.
    pub fn leer_cambio(&self, id: &str) -> Result<CambioLocal> {
        let _sesion = self.lectura()?;
        changes::leer(&self.raiz, id)
    }

    /// Registra o actualiza un cambio propuesto sin tocar el documento.
    ///
    /// Devuelve `None` cuando la propuesta coincide con el cuerpo actual. En ese
    /// caso elimina cualquier propuesta anterior del mismo documento.
    pub fn registrar_cambio(
        &self,
        id_o_ruta: &str,
        cuerpo: &str,
        autor: &str,
    ) -> Result<Option<CambioLocal>> {
        let sesion = self.escritura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        let documento_id = documento.id().to_string();
        let ruta = documento.relativa().to_string();
        let base_hash = documento.hash().to_string();
        let base = normalizar_cuerpo(documento.cuerpo());
        let propuesta = normalizar_cuerpo(cuerpo);
        let base_revision = sesion
            .log
            .operaciones()
            .last()
            .map(|operacion| operacion.prepare.target_rev)
            .unwrap_or(0);

        if propuesta == base {
            changes::eliminar_para_documento(&self.raiz, &documento_id, &ruta)?;
            return Ok(None);
        }

        if let Some(mut existente) = changes::para_documento(&self.raiz, &documento_id, &ruta)? {
            if existente.base_hash == base_hash && existente.base == base {
                existente.autor = autor.trim().to_string();
                existente.ts = time::ahora();
                existente.base_revision = base_revision;
                existente.propuesta = propuesta;
                changes::guardar(&self.raiz, &existente)?;
                return Ok(Some(existente));
            }
            changes::eliminar(&self.raiz, &existente.id)?;
        }

        let cambio = CambioLocal::nuevo(
            &documento_id,
            &ruta,
            autor,
            base_revision,
            &base_hash,
            &base,
            &propuesta,
        )?;
        changes::guardar(&self.raiz, &cambio)?;
        Ok(Some(cambio))
    }

    /// Acepta un cambio: escribe su propuesta como una transacción normal.
    pub fn aplicar_cambio(&self, id: &str) -> Result<ResultadoGuardado> {
        let mut sesion = self.escritura()?;
        let cambio = changes::leer(&self.raiz, id)?;
        {
            let documento = sesion
                .escaneo
                .por_id(&cambio.documento)
                .or_else(|| sesion.escaneo.por_ruta(&cambio.ruta))
                .ok_or_else(|| {
                    Error::NotFound(format!(
                        "documento del cambio no encontrado: {}",
                        cambio.ruta
                    ))
                })?;
            if documento.relativa() != cambio.ruta {
                return Err(Error::Conflict(
                    "el cambio pertenece a otro documento".to_string(),
                ));
            }
            if documento.hash() != cambio.base_hash {
                return Err(Error::Conflict(format!(
                    "el documento cambió después de proponer el cambio: {}",
                    cambio.ruta
                )));
            }
        }
        let resultado = self.guardar_cuerpo_en_sesion(
            &mut sesion,
            &cambio.documento,
            &cambio.propuesta,
            Some(&cambio.base_hash),
        )?;
        if let Err(error) = changes::eliminar(&self.raiz, id) {
            // El documento ya está confirmado; el sidecar antiguo no debe
            // impedir que la UI muestre el guardado. Se intentará limpiar en
            // una operación posterior.
            let _ = error;
        }
        Ok(resultado)
    }

    /// Descarta un cambio sin tocar el documento.
    pub fn descartar_cambio(&self, id: &str) -> Result<()> {
        let _sesion = self.lectura()?;
        changes::eliminar(&self.raiz, id)
    }

    /// Compara un documento con el estado que tenía al terminar una revisión.
    pub fn comparar_documento(&self, id_o_ruta: &str, revision: u64) -> Result<Comparacion> {
        let sesion = self.lectura()?;
        let documento = sesion
            .escaneo
            .por_id(id_o_ruta)
            .or_else(|| sesion.escaneo.por_ruta(id_o_ruta))
            .ok_or_else(|| Error::NotFound(format!("documento no encontrado: {id_o_ruta}")))?;
        let log = sesion
            .log
            .as_ref()
            .ok_or_else(|| Error::NotFound("el proyecto no tiene historial".to_string()))?;
        let operacion = log
            .operaciones()
            .into_iter()
            .find(|operacion| operacion.prepare.target_rev == revision)
            .ok_or_else(|| Error::NotFound(format!("revisión no encontrada: {revision}")))?;
        let efecto = operacion
            .prepare
            .efectos
            .iter()
            .find(|efecto| efecto.ruta == documento.relativa())
            .ok_or_else(|| {
                Error::Input(format!(
                    "la revisión {revision} no modificó {}",
                    documento.relativa()
                ))
            })?;
        let base_existe = efecto.hash_nuevo.is_some();
        let base = match &efecto.hash_nuevo {
            Some(hash) => {
                let blobs = BlobStore::abrir(&self.ruta_escritor())?;
                let bytes = blobs.obtener(hash)?;
                cuerpo_de_archivo(&bytes, documento.relativa())?
            }
            None => String::new(),
        };
        let diferencias = diff::comparar(&base, documento.cuerpo());
        Ok(Comparacion {
            revision,
            ruta: documento.relativa().to_string(),
            base_existe,
            diferencias,
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
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
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
        self.exigir_sin_cambio_pendiente(documento.id(), documento.relativa())?;
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
        if !changes::listar(&self.raiz)?.is_empty() {
            return Err(Error::Conflict(
                "hay cambios propuestos pendientes; acéptalos o recházalos antes de revertir"
                    .to_string(),
            ));
        }
        let operacion = sesion
            .log
            .operaciones()
            .into_iter()
            .find(|operacion| operacion.prepare.target_rev == revision)
            .ok_or_else(|| Error::NotFound(format!("revisión no encontrada: {revision}")))?;
        let mut cambios = Vec::new();
        let mut conflictos = Vec::new();
        for efecto in &operacion.prepare.efectos {
            fsutil::validar_ruta_documento(&efecto.ruta)?;
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

    /// Exporta el manuscrito a PDF con el formato de manuscrito por defecto.
    pub fn exportar_pdf(&self, destino: &Path, forzar: bool) -> Result<PathBuf> {
        let sesion = self.lectura()?;
        if !sesion.escaneo.errores.is_empty() {
            return Err(Error::Project(format!(
                "no se exporta un proyecto con errores: {}",
                sesion.escaneo.errores.join("; ")
            )));
        }
        crate::pdf::exportar(&sesion.escaneo, destino, forzar)
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
                    for entrada in log.entradas() {
                        if let Entrada::Desconocida { seq, tipo } = entrada {
                            escaneo.avisos.push(format!(
                                "log: entrada {seq} de tipo desconocido {tipo:?}; se ignora"
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
            fsutil::validar_ruta_documento(&cambio.ruta)?;
            let ruta = fsutil::ruta_del_proyecto(&self.raiz, &cambio.ruta)?;
            let anterior = fsutil::leer_opcional(&ruta)?;
            let hash_anterior = anterior.as_ref().map(|bytes| hash::hash(bytes));
            if cambio.crear_solo && hash_anterior.is_some() {
                return Err(Error::Conflict(format!(
                    "el archivo ya existe: {}",
                    cambio.ruta
                )));
            }
            if let Some(esperado) = &cambio.hash_esperado {
                if hash_anterior.as_deref() != Some(esperado.as_str()) {
                    return Err(Error::Conflict(format!(
                        "el archivo cambió antes de la transacción: {}",
                        cambio.ruta
                    )));
                }
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
                hash_esperado: cambio.hash_esperado,
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
    let coincide = match &preparado.hash_esperado {
        Some(esperado) => actual.as_deref() == Some(esperado.as_str()),
        None => actual == preparado.efecto.hash_anterior,
    };
    if !coincide {
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

fn normalizar_etiquetas(etiquetas: &[String]) -> Result<Option<Vec<String>>> {
    let mut salida: Vec<String> = Vec::new();
    for etiqueta in etiquetas {
        let valor = etiqueta.trim();
        if valor.is_empty() {
            continue;
        }
        if valor.chars().count() > 100 {
            return Err(Error::Input(
                "una etiqueta no puede superar 100 caracteres".to_string(),
            ));
        }
        model::validar_texto(valor, "etiqueta", "documento")?;
        if !salida.iter().any(|existente| existente == valor) {
            salida.push(valor.to_string());
        }
    }
    if salida.is_empty() {
        Ok(None)
    } else {
        Ok(Some(salida))
    }
}

fn cuerpo_de_archivo(bytes: &[u8], ruta: &str) -> Result<String> {
    let texto = std::str::from_utf8(bytes)
        .map_err(|error| Error::Format(format!("{ruta}: el documento no es UTF-8: {error}")))?;
    let (_, cuerpo) = crate::yaml::parsear_documento(texto, ruta)?;
    Ok(cuerpo)
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

/// Normaliza y valida la raíz de un proyecto que se va a crear.
fn preparar_raiz_inicial(raiz: &Path) -> Result<(PathBuf, bool)> {
    if raiz.as_os_str().is_empty() {
        return Err(Error::Project(
            "la ruta del proyecto no puede estar vacía".into(),
        ));
    }
    if raiz.components().any(|componente| {
        matches!(
            componente,
            std::path::Component::CurDir | std::path::Component::ParentDir
        )
    }) {
        return Err(Error::Project(format!(
            "la ruta no puede contener '.' ni '..': {}",
            raiz.display()
        )));
    }
    let absoluta = if raiz.is_absolute() {
        raiz.to_path_buf()
    } else {
        std::env::current_dir().map_err(Error::Io)?.join(raiz)
    };
    let Some(nombre) = absoluta.file_name() else {
        return Err(Error::Project(format!(
            "la ruta no nombra una carpeta de proyecto: {}",
            raiz.display()
        )));
    };
    let nombre = nombre.to_str().ok_or_else(|| {
        Error::Project(format!("nombre de proyecto no válido: {}", raiz.display()))
    })?;
    fsutil::validar_ruta_relativa(nombre)
        .map_err(|_| Error::Project(format!("nombre de proyecto no válido: {nombre}")))?;
    let padre = absoluta.parent().ok_or_else(|| {
        Error::Project(format!(
            "la ruta no tiene carpeta padre: {}",
            raiz.display()
        ))
    })?;
    let padre = if padre.as_os_str().is_empty() {
        std::env::current_dir().map_err(Error::Io)?
    } else {
        padre.to_path_buf()
    };
    if !padre.is_dir() {
        return Err(Error::Project(format!(
            "la carpeta padre no existe o no es un directorio: {}",
            padre.display()
        )));
    }
    let padre_real =
        std::fs::canonicalize(&padre).map_err(|error| Error::Io(error).with_path(&padre))?;
    let mut ancestro = Some(padre_real.as_path());
    while let Some(directorio) = ancestro {
        if directorio.join("proyecto.yml").is_file() || directorio.join(".escritor").is_dir() {
            return Err(Error::Project(format!(
                "no se puede crear un proyecto dentro de otro proyecto: {}",
                destino_display(&padre_real, nombre)
            )));
        }
        ancestro = directorio.parent();
    }
    let destino = padre_real.join(nombre);
    let creada = match std::fs::symlink_metadata(&destino) {
        Ok(metadatos) => {
            if metadatos.file_type().is_symlink() {
                return Err(Error::Project(format!(
                    "el destino del proyecto no puede ser un enlace: {}",
                    destino.display()
                )));
            }
            if !metadatos.is_dir() {
                return Err(Error::Project(format!(
                    "el destino del proyecto no es un directorio: {}",
                    destino.display()
                )));
            }
            let mut entradas = std::fs::read_dir(&destino)
                .map_err(|error| Error::Io(error).with_path(&destino))?;
            if entradas.next().is_some() {
                return Err(Error::Project(format!(
                    "el directorio no está vacío: {}",
                    destino.display()
                )));
            }
            false
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir(&destino).map_err(|error| Error::Io(error).with_path(&destino))?;
            true
        }
        Err(error) => return Err(Error::Io(error).with_path(&destino)),
    };
    let real =
        std::fs::canonicalize(&destino).map_err(|error| Error::Io(error).with_path(&destino))?;
    if real.parent() != Some(padre_real.as_path()) {
        return Err(Error::Project(format!(
            "la ruta del proyecto escapó de su carpeta padre: {}",
            real.display()
        )));
    }
    Ok((real, creada))
}

fn destino_display(padre: &Path, nombre: &str) -> String {
    padre.join(nombre).display().to_string()
}

fn parece_corrupto(error: &Error) -> bool {
    if !matches!(error, Error::Index(_)) {
        return false;
    }
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
    let escritor = fsutil::directorio_real(escritor)?;
    let ruta_indice = escritor.join("indice.sqlite");
    if std::fs::symlink_metadata(&ruta_indice).is_ok() {
        fsutil::archivo_real(&ruta_indice)?;
    }
    let mut indice = Indice::abrir(&escritor)?;
    if matches!(indice.esta_vigente(escenas, notas, revision), Ok(true)) {
        return Ok(indice);
    }
    match indice.reconstruir(escenas, notas, operaciones, revision) {
        Ok(()) => Ok(indice),
        Err(error) if parece_corrupto(&error) => {
            drop(indice);
            Indice::apartar(&escritor.join("indice.sqlite"))?;
            let mut nuevo = Indice::abrir(&escritor)?;
            nuevo.reconstruir(escenas, notas, operaciones, revision)?;
            Ok(nuevo)
        }
        Err(error) => Err(error),
    }
}
