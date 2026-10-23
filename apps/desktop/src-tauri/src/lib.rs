//! Comandos Tauri de la primera rebanada de la Fase 2.
//!
//! La UI no implementa reglas: abre un proyecto, lista documentos, lee cuerpos,
//! verifica y escribe a través de `writer-core`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use writer_core::goals::{Desafio, MetaArchivada, MetaDiaria, MetaLibro, Metas, Pausa, Sesion};
use writer_core::project::Project;
use writer_core::verify::{Documento, Escaneo};
use writer_core::Error as ErrorNucleo;

/// Resumen del proyecto para la cabecera y la barra de estado.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ProyectoDto {
    /// Ruta absoluta.
    pub raiz: String,
    /// Título del manifiesto; texto de sustitución si es inválido.
    pub titulo: String,
    /// ULID del proyecto.
    pub id: String,
    /// Autor, si existe.
    pub autor: Option<String>,
    /// Idioma, si existe.
    pub idioma: Option<String>,
    /// Revisión confirmada actual.
    pub revision: u64,
    /// Escenas válidas.
    pub escenas: usize,
    /// Notas válidas.
    pub notas: usize,
    /// Errores de verificación.
    pub errores: Vec<String>,
    /// Avisos de verificación.
    pub avisos: Vec<String>,
}

/// Documento de la lista o del panel de lectura.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DocumentoDto {
    /// `escena` o `nota`.
    pub tipo: String,
    /// ULID.
    pub id: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Título.
    pub titulo: String,
    /// Cuerpo Markdown, sin frontmatter.
    pub cuerpo: String,
    /// Hash del archivo en el momento de la lectura.
    pub hash_archivo: String,
    /// Orden de escena.
    pub orden: Option<i64>,
    /// Estado de escena.
    pub estado: Option<String>,
    /// Ubicación de archivo o papelera.
    pub archivo: Option<String>,
    /// Etiquetas.
    pub etiquetas: Vec<String>,
    /// Tipo de nota.
    pub clase: Option<String>,
}

/// Familia de error del motor, para que la UI decida cómo reaccionar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(rename_all = "kebab-case")]
pub enum TipoError {
    /// Fallo de entrada/salida.
    Io,
    /// Documento fuera del esquema.
    Esquema,
    /// Proyecto inválido o no escribible.
    Proyecto,
    /// Cambio externo detectado por el CAS.
    Conflicto,
    /// Otro proceso mantiene el lock.
    Ocupado,
    /// El proyecto necesita recuperación.
    Recuperacion,
    /// Error del índice SQLite.
    Indice,
    /// Error de serialización del log.
    Json,
    /// Documento o revisión inexistente.
    NoEncontrado,
    /// Entrada inválida.
    Entrada,
    /// Archivo con formato inesperado.
    Formato,
    /// Fallo de integridad.
    Integridad,
    /// Cualquier otro error.
    Otro,
}

/// Error estructurado que cruza el puente Tauri.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ErrorDto {
    /// Familia del error.
    pub tipo: TipoError,
    /// Mensaje legible, ya con la ruta cuando aplica.
    pub mensaje: String,
}

/// Resultado de guardar un cuerpo.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GuardadoDto {
    /// Revisión confirmada.
    pub revision: u64,
    /// Hash del archivo después del guardado.
    pub hash_archivo: String,
}

/// Resultado de una búsqueda.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct BusquedaDto {
    /// ULID del documento.
    pub id: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Título.
    pub titulo: String,
    /// `escena` o `nota`.
    pub tipo: String,
    /// Fragmento con la coincidencia entre corchetes.
    pub fragmento: String,
}

/// Entrada del historial.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RevisionDto {
    /// Revisión.
    pub revision: u64,
    /// ULID de la operación.
    pub op_id: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Autor.
    pub autor: String,
    /// Alcance.
    pub scope: String,
    /// Número de efectos.
    pub efectos: usize,
    /// Rutas afectadas.
    pub rutas: Vec<String>,
}

/// Contenido de un archivo en una revisión.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ContenidoRevisionDto {
    /// Ruta relativa.
    pub ruta: String,
    /// Texto posterior al efecto; `null` si el archivo se eliminó.
    pub contenido: Option<String>,
}

/// Detalle de una revisión.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct RevisionDetalleDto {
    /// Revisión.
    pub revision: u64,
    /// ULID de la operación.
    pub op_id: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Autor.
    pub autor: String,
    /// Alcance.
    pub scope: String,
    /// Contenido de cada archivo afectado.
    pub contenidos: Vec<ContenidoRevisionDto>,
}

/// Resultado de revertir.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ReversionDto {
    /// Revisión nueva creada.
    pub revision: u64,
    /// Errores que quedan después de la reversión.
    pub errores_pendientes: Vec<String>,
}

/// Copia de cuarentena.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct CuarentenaDto {
    /// Nombre del archivo en .escritor/cuarentena.
    pub archivo: String,
    /// Ruta original, si el informe la conserva.
    pub ruta_original: Option<String>,
    /// Motivo, si el informe lo conserva.
    pub motivo: Option<String>,
    /// Fecha, si el informe la conserva.
    pub ts: Option<String>,
    /// Tamaño en bytes.
    pub tamano: u64,
}

/// Resultado de `verify` para la barra de estado.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct EscaneoDto {
    /// Escenas válidas.
    pub escenas: usize,
    /// Notas válidas.
    pub notas: usize,
    /// Errores.
    pub errores: Vec<String>,
    /// Avisos.
    pub avisos: Vec<String>,
}

/// Meta diaria.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MetaDiariaDto {
    /// Palabras objetivo por día.
    pub objetivo: i64,
    /// Fecha de inicio.
    pub desde: String,
}

/// Meta de libro.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MetaLibroDto {
    /// Palabras objetivo.
    pub objetivo: i64,
}

/// Desafío con fecha límite.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DesafioDto {
    /// Palabras objetivo.
    pub objetivo: i64,
    /// Fecha límite.
    pub limite: String,
}

/// Meta archivada.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MetaArchivadaDto {
    /// Tipo de meta.
    pub tipo: String,
    /// Palabras objetivo.
    pub objetivo: i64,
    /// Inicio opcional.
    pub desde: Option<String>,
    /// Fin opcional.
    pub hasta: Option<String>,
    /// Motivo.
    pub motivo: Option<String>,
}

/// Metas locales del proyecto.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MetasDto {
    /// Versión de la extensión.
    pub version: u32,
    /// Meta diaria.
    pub diaria: Option<MetaDiariaDto>,
    /// Meta de libro.
    pub libro: Option<MetaLibroDto>,
    /// Desafío.
    pub desafio: Option<DesafioDto>,
    /// Metas archivadas.
    pub archivadas: Vec<MetaArchivadaDto>,
}

/// Pausa de una sesión.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PausaDto {
    /// Inicio RFC 3339.
    pub inicio: String,
    /// Fin RFC 3339.
    pub fin: String,
}

/// Sesión de escritura.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct SesionDto {
    /// ULID.
    pub id: String,
    /// Inicio RFC 3339.
    pub inicio: String,
    /// Fin RFC 3339.
    pub fin: String,
    /// Preset en minutos.
    pub preset: u32,
    /// Pausas.
    pub pausas: Vec<PausaDto>,
    /// Palabras.
    pub palabras: i64,
    /// Documento.
    pub documento: String,
    /// Autor.
    pub autor: String,
}

/// Ancla de un comentario.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct AnclaDto {
    /// Texto marcado.
    pub texto: String,
    /// Hash del texto.
    pub hash: String,
    /// Inicio orientativo.
    pub inicio: usize,
    /// Fin orientativo.
    pub fin: usize,
}

/// Mensaje de un hilo.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct MensajeDto {
    /// ULID.
    pub id: String,
    /// Tipo.
    pub tipo: String,
    /// Autor.
    pub autor: String,
    /// Fecha.
    pub ts: String,
    /// Texto.
    pub texto: String,
    /// Mensaje reemplazado.
    pub reemplaza: Option<String>,
}

/// Hilo de comentarios.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ComentarioDto {
    /// Versión.
    pub version: u32,
    /// ULID.
    pub id: String,
    /// `comentario` o `sticky`.
    pub tipo: String,
    /// Documento.
    pub documento: String,
    /// Ancla.
    pub ancla: Option<AnclaDto>,
    /// Estado.
    pub estado: String,
    /// Creación.
    pub creado: String,
    /// Actualización.
    pub actualizado: String,
    /// Autor.
    pub autor: String,
    /// Huérfano.
    pub huerfano: bool,
    /// Mensajes.
    pub mensajes: Vec<MensajeDto>,
}

impl From<writer_core::comments::Ancla> for AnclaDto {
    fn from(valor: writer_core::comments::Ancla) -> Self {
        Self {
            texto: valor.texto,
            hash: valor.hash,
            inicio: valor.inicio,
            fin: valor.fin,
        }
    }
}

impl From<writer_core::comments::Mensaje> for MensajeDto {
    fn from(valor: writer_core::comments::Mensaje) -> Self {
        Self {
            id: valor.id,
            tipo: valor.tipo,
            autor: valor.autor,
            ts: valor.ts,
            texto: valor.texto,
            reemplaza: valor.reemplaza,
        }
    }
}

impl From<writer_core::comments::Comentario> for ComentarioDto {
    fn from(valor: writer_core::comments::Comentario) -> Self {
        Self {
            version: valor.version,
            id: valor.id,
            tipo: valor.tipo,
            documento: valor.documento,
            ancla: valor.ancla.map(Into::into),
            estado: valor.estado,
            creado: valor.creado,
            actualizado: valor.actualizado,
            autor: valor.autor,
            huerfano: valor.huerfano,
            mensajes: valor.mensajes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<MetaDiariaDto> for MetaDiaria {
    fn from(valor: MetaDiariaDto) -> Self {
        Self {
            objetivo: valor.objetivo,
            desde: valor.desde,
        }
    }
}

impl From<MetaDiaria> for MetaDiariaDto {
    fn from(valor: MetaDiaria) -> Self {
        Self {
            objetivo: valor.objetivo,
            desde: valor.desde,
        }
    }
}

impl From<MetaLibroDto> for MetaLibro {
    fn from(valor: MetaLibroDto) -> Self {
        Self {
            objetivo: valor.objetivo,
        }
    }
}

impl From<MetaLibro> for MetaLibroDto {
    fn from(valor: MetaLibro) -> Self {
        Self {
            objetivo: valor.objetivo,
        }
    }
}

impl From<DesafioDto> for Desafio {
    fn from(valor: DesafioDto) -> Self {
        Self {
            objetivo: valor.objetivo,
            limite: valor.limite,
        }
    }
}

impl From<Desafio> for DesafioDto {
    fn from(valor: Desafio) -> Self {
        Self {
            objetivo: valor.objetivo,
            limite: valor.limite,
        }
    }
}

impl From<MetaArchivadaDto> for MetaArchivada {
    fn from(valor: MetaArchivadaDto) -> Self {
        Self {
            tipo: valor.tipo,
            objetivo: valor.objetivo,
            desde: valor.desde,
            hasta: valor.hasta,
            motivo: valor.motivo,
        }
    }
}

impl From<MetaArchivada> for MetaArchivadaDto {
    fn from(valor: MetaArchivada) -> Self {
        Self {
            tipo: valor.tipo,
            objetivo: valor.objetivo,
            desde: valor.desde,
            hasta: valor.hasta,
            motivo: valor.motivo,
        }
    }
}

impl From<MetasDto> for Metas {
    fn from(valor: MetasDto) -> Self {
        Self {
            version: valor.version,
            diaria: valor.diaria.map(Into::into),
            libro: valor.libro.map(Into::into),
            desafio: valor.desafio.map(Into::into),
            archivadas: valor.archivadas.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Metas> for MetasDto {
    fn from(valor: Metas) -> Self {
        Self {
            version: valor.version,
            diaria: valor.diaria.map(Into::into),
            libro: valor.libro.map(Into::into),
            desafio: valor.desafio.map(Into::into),
            archivadas: valor.archivadas.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PausaDto> for Pausa {
    fn from(valor: PausaDto) -> Self {
        Self {
            inicio: valor.inicio,
            fin: valor.fin,
        }
    }
}

impl From<Pausa> for PausaDto {
    fn from(valor: Pausa) -> Self {
        Self {
            inicio: valor.inicio,
            fin: valor.fin,
        }
    }
}

impl From<SesionDto> for Sesion {
    fn from(valor: SesionDto) -> Self {
        Self {
            id: valor.id,
            inicio: valor.inicio,
            fin: valor.fin,
            preset: valor.preset,
            pausas: valor.pausas.into_iter().map(Into::into).collect(),
            palabras: valor.palabras,
            documento: valor.documento,
            autor: valor.autor,
        }
    }
}

impl From<Sesion> for SesionDto {
    fn from(valor: Sesion) -> Self {
        Self {
            id: valor.id,
            inicio: valor.inicio,
            fin: valor.fin,
            preset: valor.preset,
            pausas: valor.pausas.into_iter().map(Into::into).collect(),
            palabras: valor.palabras,
            documento: valor.documento,
            autor: valor.autor,
        }
    }
}

#[tauri::command]
fn leer_metas(ruta: String) -> Result<MetasDto, ErrorDto> {
    writer_core::goals::leer_metas(std::path::Path::new(&ruta))
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn guardar_metas(ruta: String, metas: MetasDto) -> Result<(), ErrorDto> {
    writer_core::goals::guardar_metas(std::path::Path::new(&ruta), &metas.into()).map_err(error)
}

#[tauri::command]
fn agregar_sesion(ruta: String, sesion: SesionDto) -> Result<String, ErrorDto> {
    let mut sesion: Sesion = sesion.into();
    if sesion.id.trim().is_empty() {
        sesion.id = writer_core::ulid::nuevo().map_err(error)?;
    }
    writer_core::goals::agregar_sesion(std::path::Path::new(&ruta), &sesion)
        .map(|ruta| ruta.display().to_string())
        .map_err(error)
}

#[tauri::command]
fn leer_sesiones(ruta: String, mes: String) -> Result<Vec<SesionDto>, ErrorDto> {
    writer_core::goals::leer_sesiones(std::path::Path::new(&ruta), &mes)
        .map(|sesiones| sesiones.into_iter().map(Into::into).collect())
        .map_err(error)
}

#[tauri::command]
fn listar_comentarios(
    ruta: String,
    documento: Option<String>,
) -> Result<Vec<ComentarioDto>, ErrorDto> {
    writer_core::comments::listar_comentarios(std::path::Path::new(&ruta), documento.as_deref())
        .map(|comentarios| comentarios.into_iter().map(Into::into).collect())
        .map_err(error)
}

#[tauri::command]
fn crear_comentario(
    ruta: String,
    documento: String,
    texto: String,
    texto_marcado: String,
    autor: String,
) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::crear_comentario(
        std::path::Path::new(&ruta),
        &documento,
        &texto,
        &texto_marcado,
        &autor,
    )
    .map(Into::into)
    .map_err(error)
}

#[tauri::command]
fn crear_sticky(
    ruta: String,
    documento: String,
    texto: String,
    autor: String,
) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::crear_sticky(std::path::Path::new(&ruta), &documento, &texto, &autor)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn agregar_mensaje_comentario(
    ruta: String,
    id: String,
    tipo: String,
    autor: String,
    texto: String,
) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::agregar_mensaje(std::path::Path::new(&ruta), &id, &tipo, &autor, &texto)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn resolver_comentario(ruta: String, id: String, autor: String) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::resolver_comentario(std::path::Path::new(&ruta), &id, &autor)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn marcar_comentario_huerfano(
    ruta: String,
    id: String,
    huerfano: bool,
) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::marcar_huerfano(std::path::Path::new(&ruta), &id, huerfano)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn crear_proyecto(
    ruta: String,
    titulo: String,
    autor: Option<String>,
    idioma: Option<String>,
    notas_iniciales: bool,
) -> Result<ProyectoDto, ErrorDto> {
    let proyecto = Project::new(&ruta);
    let existia = std::path::Path::new(&ruta).exists();
    let era_vacia = existia
        && std::fs::read_dir(&ruta)
            .map(|mut entradas| entradas.next().is_none())
            .unwrap_or(false);
    let resultado = (|| -> Result<ProyectoDto, ErrorDto> {
        proyecto
            .init(&titulo, autor.as_deref(), idioma.as_deref())
            .map_err(error)?;
        if notas_iniciales {
            for (tipo, etiqueta) in [
                ("personajes", "Personajes"),
                ("tramas", "Tramas"),
                ("lugares", "Lugares"),
            ] {
                proyecto.crear_nota(tipo, etiqueta).map_err(error)?;
            }
        }
        abrir_proyecto(ruta.clone())
    })();
    if resultado.is_err() && (!existia || era_vacia) {
        if let Ok(raiz) = std::fs::canonicalize(&ruta) {
            if raiz.join("proyecto.yml").is_file() {
                let _ = std::fs::remove_dir_all(&raiz);
                if era_vacia {
                    let _ = std::fs::create_dir(&raiz);
                }
            }
        }
    }
    resultado
}

#[tauri::command]
fn abrir_proyecto(ruta: String) -> Result<ProyectoDto, ErrorDto> {
    let proyecto = Project::new(&ruta);
    let escaneo = proyecto.verificar().map_err(error)?;
    let revision = proyecto
        .historial()
        .map(|operaciones| {
            operaciones
                .last()
                .map(|operacion| operacion.prepare.target_rev)
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let (titulo, id, autor, idioma) = match &escaneo.proyecto {
        Some(manifest) => (
            manifest.titulo.clone(),
            manifest.id.clone(),
            manifest.autor.clone(),
            manifest.idioma.clone(),
        ),
        None => ("Proyecto inválido".to_string(), String::new(), None, None),
    };
    Ok(ProyectoDto {
        raiz: ruta,
        titulo,
        id,
        autor,
        idioma,
        revision,
        escenas: escaneo.escenas.len(),
        notas: escaneo.notas.len(),
        errores: escaneo.errores,
        avisos: escaneo.avisos,
    })
}

#[tauri::command]
fn listar(ruta: String, notas: bool) -> Result<Vec<DocumentoDto>, ErrorDto> {
    let documentos = Project::new(&ruta).listar(notas).map_err(error)?;
    Ok(documentos.into_iter().map(documento_dto).collect())
}

#[tauri::command]
fn leer(ruta: String, objetivo: String) -> Result<DocumentoDto, ErrorDto> {
    let documento = Project::new(&ruta).leer(&objetivo).map_err(error)?;
    Ok(documento_dto(documento))
}

#[tauri::command]
fn verificar(ruta: String) -> Result<EscaneoDto, ErrorDto> {
    let escaneo = Project::new(&ruta).verificar().map_err(error)?;
    Ok(escaneo_dto(escaneo))
}

#[tauri::command]
fn buscar(ruta: String, consulta: String, limite: usize) -> Result<Vec<BusquedaDto>, ErrorDto> {
    let resultados = Project::new(&ruta)
        .buscar(&consulta, limite)
        .map_err(error)?;
    Ok(resultados
        .into_iter()
        .map(|resultado| BusquedaDto {
            id: resultado.id,
            ruta: resultado.ruta,
            titulo: resultado.titulo,
            tipo: resultado.tipo,
            fragmento: resultado.fragmento,
        })
        .collect())
}

#[tauri::command]
fn historial(ruta: String) -> Result<Vec<RevisionDto>, ErrorDto> {
    let operaciones = Project::new(&ruta).historial().map_err(error)?;
    Ok(operaciones
        .into_iter()
        .map(|operacion| RevisionDto {
            revision: operacion.prepare.target_rev,
            op_id: operacion.prepare.op_id,
            ts: operacion.prepare.ts,
            autor: operacion.prepare.autor,
            scope: operacion.prepare.scope,
            efectos: operacion.prepare.efectos.len(),
            rutas: operacion
                .prepare
                .efectos
                .into_iter()
                .map(|efecto| efecto.ruta)
                .collect(),
        })
        .collect())
}

#[tauri::command]
fn mostrar(ruta: String, revision: u64) -> Result<RevisionDetalleDto, ErrorDto> {
    let detalle = Project::new(&ruta).mostrar(revision).map_err(error)?;
    Ok(RevisionDetalleDto {
        revision: detalle.operacion.prepare.target_rev,
        op_id: detalle.operacion.prepare.op_id,
        ts: detalle.operacion.prepare.ts,
        autor: detalle.operacion.prepare.autor,
        scope: detalle.operacion.prepare.scope,
        contenidos: detalle
            .contenidos
            .into_iter()
            .map(|(ruta, contenido)| ContenidoRevisionDto {
                ruta,
                contenido: contenido.map(|bytes| String::from_utf8_lossy(&bytes).to_string()),
            })
            .collect(),
    })
}

#[tauri::command]
fn revertir(ruta: String, revision: u64) -> Result<ReversionDto, ErrorDto> {
    let resultado = Project::new(&ruta).revertir(revision).map_err(error)?;
    Ok(ReversionDto {
        revision: resultado.revision,
        errores_pendientes: resultado.errores_pendientes,
    })
}

#[tauri::command]
fn exportar_docx(
    ruta: String,
    destino: String,
    incluir_notas: bool,
    forzar: bool,
) -> Result<String, ErrorDto> {
    Project::new(&ruta)
        .exportar_docx(std::path::Path::new(&destino), incluir_notas, forzar)
        .map(|ruta| ruta.display().to_string())
        .map_err(error)
}

#[tauri::command]
fn cuarentena(ruta: String) -> Result<Vec<CuarentenaDto>, ErrorDto> {
    let items = Project::new(&ruta).cuarentena().map_err(error)?;
    Ok(items
        .into_iter()
        .map(|item| CuarentenaDto {
            archivo: item.archivo,
            ruta_original: item.ruta_original,
            motivo: item.motivo,
            ts: item.ts,
            tamano: item.tamano,
        })
        .collect())
}

#[tauri::command]
fn descartar_cuarentena(ruta: String, archivo: String) -> Result<(), ErrorDto> {
    Project::new(&ruta)
        .descartar_cuarentena(&archivo)
        .map_err(error)
}

/// Solo se usa en E2E: permite abrir un proyecto por variable de entorno.
#[tauri::command]
fn proyecto_e2e() -> Option<String> {
    #[cfg(feature = "e2e")]
    {
        std::env::var("WRITER_E2E_PROJECT")
            .ok()
            .filter(|ruta| !ruta.trim().is_empty())
    }
    #[cfg(not(feature = "e2e"))]
    {
        None
    }
}

#[tauri::command]
fn mover(
    ruta: String,
    objetivo: String,
    libro: Option<String>,
    capitulo: Option<String>,
    tipo: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    let destino = Project::new(&ruta)
        .mover(
            &objetivo,
            libro.as_deref(),
            capitulo.as_deref(),
            tipo.as_deref(),
        )
        .map_err(error)?;
    leer(ruta, destino)
}

#[tauri::command]
fn actualizar_escena(
    ruta: String,
    objetivo: String,
    estado: Option<String>,
    etiquetas: Vec<String>,
) -> Result<DocumentoDto, ErrorDto> {
    Project::new(&ruta)
        .actualizar_escena(&objetivo, estado.as_deref(), &etiquetas)
        .map_err(error)?;
    leer(ruta, objetivo)
}

#[tauri::command]
fn actualizar_nota(
    ruta: String,
    objetivo: String,
    etiquetas: Vec<String>,
) -> Result<DocumentoDto, ErrorDto> {
    Project::new(&ruta)
        .actualizar_nota(&objetivo, &etiquetas)
        .map_err(error)?;
    leer(ruta, objetivo)
}

#[tauri::command]
fn cambiar_archivo(
    ruta: String,
    objetivo: String,
    archivo: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    Project::new(&ruta)
        .cambiar_archivo(&objetivo, archivo.as_deref())
        .map_err(error)?;
    leer(ruta, objetivo)
}

#[tauri::command]
fn renombrar(ruta: String, objetivo: String, titulo: String) -> Result<DocumentoDto, ErrorDto> {
    Project::new(&ruta)
        .renombrar(&objetivo, &titulo)
        .map_err(error)?;
    leer(ruta, objetivo)
}

#[tauri::command]
fn duplicar(ruta: String, objetivo: String) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = Project::new(&ruta).duplicar(&objetivo).map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn escribir(
    ruta: String,
    objetivo: String,
    cuerpo: String,
    hash_base: Option<String>,
) -> Result<GuardadoDto, ErrorDto> {
    let resultado = Project::new(&ruta)
        .escribir_cuerpo_con_base(&objetivo, &cuerpo, hash_base.as_deref())
        .map_err(error)?;
    Ok(GuardadoDto {
        revision: resultado.revision,
        hash_archivo: resultado.hash_archivo,
    })
}

fn documento_dto(documento: Documento) -> DocumentoDto {
    match documento {
        Documento::Escena(encontrada) => DocumentoDto {
            tipo: "escena".to_string(),
            id: encontrada.escena.id,
            ruta: encontrada.relativa,
            titulo: encontrada.escena.titulo,
            cuerpo: encontrada.escena.cuerpo,
            hash_archivo: encontrada.hash,
            orden: Some(encontrada.escena.orden),
            estado: encontrada.escena.estado,
            archivo: encontrada.escena.archivo,
            etiquetas: encontrada.escena.etiquetas.unwrap_or_default(),
            clase: None,
        },
        Documento::Nota(encontrada) => DocumentoDto {
            tipo: "nota".to_string(),
            id: encontrada.nota.id,
            ruta: encontrada.relativa,
            titulo: encontrada.nota.titulo,
            cuerpo: encontrada.nota.cuerpo,
            hash_archivo: encontrada.hash,
            orden: None,
            estado: None,
            archivo: encontrada.nota.archivo,
            etiquetas: encontrada.nota.etiquetas.unwrap_or_default(),
            clase: Some(encontrada.nota.tipo),
        },
    }
}

fn escaneo_dto(escaneo: Escaneo) -> EscaneoDto {
    EscaneoDto {
        escenas: escaneo.escenas.len(),
        notas: escaneo.notas.len(),
        errores: escaneo.errores,
        avisos: escaneo.avisos,
    }
}

fn error(error: ErrorNucleo) -> ErrorDto {
    let tipo = match &error {
        ErrorNucleo::Io(_) => TipoError::Io,
        ErrorNucleo::Schema(_) => TipoError::Esquema,
        ErrorNucleo::Project(_) => TipoError::Proyecto,
        ErrorNucleo::Conflict(_) => TipoError::Conflicto,
        ErrorNucleo::Busy(_) => TipoError::Ocupado,
        ErrorNucleo::Recovery(_) => TipoError::Recuperacion,
        ErrorNucleo::Index(_) => TipoError::Indice,
        ErrorNucleo::Json(_) => TipoError::Json,
        ErrorNucleo::NotFound(_) => TipoError::NoEncontrado,
        ErrorNucleo::Input(_) => TipoError::Entrada,
        ErrorNucleo::Format(_) => TipoError::Formato,
        ErrorNucleo::Integrity(_) => TipoError::Integridad,
        ErrorNucleo::Other(_) => TipoError::Otro,
    };
    ErrorDto {
        tipo,
        mensaje: error.to_string(),
    }
}

/// Arranca la aplicación Tauri.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            crear_proyecto,
            abrir_proyecto,
            listar,
            leer,
            verificar,
            buscar,
            historial,
            mostrar,
            revertir,
            exportar_docx,
            cuarentena,
            descartar_cuarentena,
            proyecto_e2e,
            escribir,
            mover,
            actualizar_escena,
            actualizar_nota,
            cambiar_archivo,
            renombrar,
            duplicar,
            leer_metas,
            guardar_metas,
            agregar_sesion,
            leer_sesiones,
            listar_comentarios,
            crear_comentario,
            crear_sticky,
            agregar_mensaje_comentario,
            resolver_comentario,
            marcar_comentario_huerfano
        ])
        .run(tauri::generate_context!())
        .expect("error al ejecutar writer");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use ts_rs::TS;

    #[test]
    fn exportar_tipos_typescript() {
        let destino = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../ui/src/types");
        std::fs::create_dir_all(&destino).expect("crear directorio de tipos");
        let config = ts_rs::Config::new()
            .with_out_dir(&destino)
            .with_large_int("number");
        ProyectoDto::export_all(&config).expect("exportar ProyectoDto");
        DocumentoDto::export_all(&config).expect("exportar DocumentoDto");
        EscaneoDto::export_all(&config).expect("exportar EscaneoDto");
        ErrorDto::export_all(&config).expect("exportar ErrorDto");
        GuardadoDto::export_all(&config).expect("exportar GuardadoDto");
        BusquedaDto::export_all(&config).expect("exportar BusquedaDto");
        RevisionDto::export_all(&config).expect("exportar RevisionDto");
        ContenidoRevisionDto::export_all(&config).expect("exportar ContenidoRevisionDto");
        RevisionDetalleDto::export_all(&config).expect("exportar RevisionDetalleDto");
        ReversionDto::export_all(&config).expect("exportar ReversionDto");
        CuarentenaDto::export_all(&config).expect("exportar CuarentenaDto");
        MetaDiariaDto::export_all(&config).expect("exportar MetaDiariaDto");
        MetaLibroDto::export_all(&config).expect("exportar MetaLibroDto");
        DesafioDto::export_all(&config).expect("exportar DesafioDto");
        MetaArchivadaDto::export_all(&config).expect("exportar MetaArchivadaDto");
        MetasDto::export_all(&config).expect("exportar MetasDto");
        PausaDto::export_all(&config).expect("exportar PausaDto");
        SesionDto::export_all(&config).expect("exportar SesionDto");
        AnclaDto::export_all(&config).expect("exportar AnclaDto");
        MensajeDto::export_all(&config).expect("exportar MensajeDto");
        ComentarioDto::export_all(&config).expect("exportar ComentarioDto");
    }

    #[test]
    fn crear_proyecto_con_notas_iniciales() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("nuevo");
        let resumen = crear_proyecto(
            raiz.display().to_string(),
            "Nueva novela".to_string(),
            Some("Autora".to_string()),
            Some("es".to_string()),
            true,
        )
        .expect("crear proyecto");
        assert_eq!(resumen.titulo, "Nueva novela");
        assert_eq!(resumen.escenas, 1);
        assert_eq!(resumen.notas, 3);
        assert!(resumen.errores.is_empty(), "errores: {:?}", resumen.errores);

        let documentos = listar(raiz.display().to_string(), true).expect("listar");
        assert_eq!(documentos.len(), 4);
        assert!(documentos
            .iter()
            .any(|documento| documento.tipo == "escena"));
        assert!(documentos.iter().any(|documento| documento.tipo == "nota"));

        let sin_notas = crear_proyecto(
            directorio.path().join("minimo").display().to_string(),
            "Mínimo".to_string(),
            None,
            Some("es".to_string()),
            false,
        )
        .expect("crear sin notas");
        assert_eq!(sin_notas.escenas, 1);
        assert_eq!(sin_notas.notas, 0);

        let invalido = directorio.path().join("invalido");
        assert!(crear_proyecto(
            invalido.display().to_string(),
            "   ".to_string(),
            None,
            Some("es".to_string()),
            true,
        )
        .is_err());
        assert!(!invalido.exists(), "la creación fallida no deja carpeta");
    }

    #[test]
    fn comandos_basicos_sobre_un_proyecto_real() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("novela");
        Project::new(&raiz)
            .init("Prueba", Some("Autora"), Some("es"))
            .expect("init");
        let ruta = raiz.display().to_string();

        let resumen = abrir_proyecto(ruta.clone()).expect("abrir");
        assert_eq!(resumen.titulo, "Prueba");
        assert_eq!(resumen.escenas, 1);
        assert_eq!(resumen.revision, 1);

        let documentos = listar(ruta.clone(), true).expect("listar");
        assert_eq!(documentos.len(), 1);
        let escena = &documentos[0];
        assert_eq!(escena.tipo, "escena");

        let guardado = escribir(
            ruta.clone(),
            escena.id.clone(),
            "Hola\n".to_string(),
            Some(escena.hash_archivo.clone()),
        )
        .expect("escribir");
        assert_eq!(guardado.revision, 2);
        let leida = leer(ruta.clone(), escena.id.clone()).expect("leer");
        assert_eq!(leida.cuerpo, "Hola\n");
        assert_eq!(leida.hash_archivo, guardado.hash_archivo);

        let fallo = escribir(
            ruta.clone(),
            escena.id.clone(),
            "Cambio con hash viejo\n".to_string(),
            Some(escena.hash_archivo.clone()),
        )
        .expect_err("hash viejo");
        assert_eq!(fallo.tipo, TipoError::Conflicto);
        let conservada = leer(ruta.clone(), escena.id.clone()).expect("leer conservada");
        assert_eq!(conservada.cuerpo, "Hola\n");

        let resultados = buscar(ruta.clone(), "Hola".to_string(), 10).expect("buscar");
        assert_eq!(resultados.len(), 1);
        assert_eq!(resultados[0].id, escena.id);
        assert_eq!(resultados[0].tipo, "escena");

        let historial = historial(ruta.clone()).expect("historial");
        assert_eq!(historial.len(), 2);
        assert_eq!(historial.last().expect("última").revision, 2);
        let detalle = mostrar(ruta.clone(), 2).expect("mostrar");
        assert!(detalle.contenidos.iter().any(|contenido| contenido
            .contenido
            .as_deref()
            .is_some_and(|texto| texto.contains("Hola\n"))));

        let reversion = revertir(ruta.clone(), 2).expect("revertir");
        assert_eq!(reversion.revision, 3);
        let restaurada = leer(ruta.clone(), escena.id.clone()).expect("leer restaurada");
        assert_eq!(restaurada.cuerpo, "");

        let destino = directorio.path().join("salida.docx");
        let exportado = exportar_docx(ruta.clone(), destino.display().to_string(), true, false)
            .expect("exportar DOCX");
        let bytes = std::fs::read(exportado).expect("leer DOCX");
        assert_eq!(&bytes[..4], b"PK\x03\x04");

        let cuarentena_dir = raiz.join(".escritor/cuarentena");
        let archivo = "20260101T000000Z-prueba-motivo.md";
        std::fs::write(cuarentena_dir.join(archivo), b"copia").expect("cuarentena");
        std::fs::write(
            cuarentena_dir.join("informe.jsonl"),
            format!(
                "{{\"ts\":\"2026-01-01T00:00:00Z\",\"archivo\":\"{archivo}\",\"ruta_original\":\"manuscrito/x.md\",\"motivo\":\"prueba\",\"hash\":\"blake3:x\"}}\n"
            ),
        )
        .expect("informe");
        let items = cuarentena(ruta.clone()).expect("listar cuarentena");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].archivo, archivo);
        assert_eq!(items[0].motivo.as_deref(), Some("prueba"));
        descartar_cuarentena(ruta.clone(), archivo.to_string()).expect("descartar");
        assert!(!cuarentena_dir.join(archivo).exists());

        let fallo = escribir(
            ruta,
            "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            "x".to_string(),
            None,
        )
        .expect_err("id inexistente");
        assert_eq!(fallo.tipo, TipoError::NoEncontrado);
        assert!(fallo.mensaje.contains("no encontrado"));
    }

    #[test]
    fn renombrar_y_duplicar_sobre_un_proyecto_real() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("documentos");
        let ruta = raiz.display().to_string();
        crear_proyecto(ruta.clone(), "Documentos".to_string(), None, None, false)
            .expect("crear proyecto");
        let documentos = listar(ruta.clone(), true).expect("listar");
        let escena = documentos
            .iter()
            .find(|documento| documento.tipo == "escena")
            .expect("escena");
        let renombrada = renombrar(
            ruta.clone(),
            escena.id.clone(),
            "Escena renombrada".to_string(),
        )
        .expect("renombrar");
        assert_eq!(renombrada.titulo, "Escena renombrada");
        let copia = duplicar(ruta.clone(), escena.id.clone()).expect("duplicar");
        assert_ne!(copia.id, escena.id);
        assert_eq!(copia.titulo, "Copia de Escena renombrada");

        let movida = mover(
            ruta.clone(),
            escena.id.clone(),
            Some("02-libro".to_string()),
            Some("01-capitulo".to_string()),
            None,
        )
        .expect("mover");
        assert!(movida.ruta.starts_with("manuscrito/02-libro/01-capitulo/"));

        let actualizada = actualizar_escena(
            ruta.clone(),
            escena.id.clone(),
            Some("final".to_string()),
            vec!["clave".to_string()],
        )
        .expect("actualizar escena");
        assert_eq!(actualizada.estado.as_deref(), Some("final"));
        assert_eq!(actualizada.etiquetas, vec!["clave".to_string()]);

        let archivada = cambiar_archivo(
            ruta.clone(),
            escena.id.clone(),
            Some("archivado".to_string()),
        )
        .expect("archivar");
        assert_eq!(archivada.archivo.as_deref(), Some("archivado"));
        let restaurada = cambiar_archivo(ruta, escena.id.clone(), None).expect("restaurar");
        assert!(restaurada.archivo.is_none());
    }

    #[test]
    fn comentarios_sobre_un_proyecto_real() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("comentarios");
        let ruta = raiz.display().to_string();
        crear_proyecto(ruta.clone(), "Comentarios".to_string(), None, None, false)
            .expect("crear proyecto");
        let documentos = listar(ruta.clone(), true).expect("listar");
        let escena = documentos
            .iter()
            .find(|documento| documento.tipo == "escena")
            .expect("escena");
        let comentario = crear_comentario(
            ruta.clone(),
            escena.ruta.clone(),
            "revisar".to_string(),
            "texto".to_string(),
            "local".to_string(),
        )
        .expect("crear comentario");
        assert_eq!(comentario.mensajes.len(), 1);
        let respuesta = agregar_mensaje_comentario(
            ruta.clone(),
            comentario.id.clone(),
            "respuesta".to_string(),
            "local".to_string(),
            "de acuerdo".to_string(),
        )
        .expect("responder");
        assert_eq!(respuesta.mensajes.len(), 2);
        let resuelto =
            resolver_comentario(ruta, comentario.id, "local".to_string()).expect("resolver");
        assert_eq!(resuelto.estado, "resuelto");
    }

    #[test]
    fn metas_y_sesiones_sobre_un_proyecto_real() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("metas");
        let ruta = raiz.display().to_string();
        crear_proyecto(
            ruta.clone(),
            "Metas".to_string(),
            Some("Autora".to_string()),
            Some("es".to_string()),
            false,
        )
        .expect("crear proyecto");

        let metas = MetasDto {
            version: 1,
            diaria: Some(MetaDiariaDto {
                objetivo: 500,
                desde: "2026-10-23".to_string(),
            }),
            libro: Some(MetaLibroDto { objetivo: 80_000 }),
            desafio: None,
            archivadas: vec![],
        };
        guardar_metas(ruta.clone(), metas).expect("guardar metas");
        let leidas = leer_metas(ruta.clone()).expect("leer metas");
        assert_eq!(leidas.diaria.as_ref().map(|meta| meta.objetivo), Some(500));

        let sesion = SesionDto {
            id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            inicio: "2026-10-23T10:00:00.000Z".to_string(),
            fin: "2026-10-23T10:25:00.000Z".to_string(),
            preset: 25,
            pausas: vec![],
            palabras: 412,
            documento: "manuscrito/01-libro/01-capitulo/001-escena.md".to_string(),
            autor: "local".to_string(),
        };
        agregar_sesion(ruta.clone(), sesion).expect("agregar sesión");
        let sesiones = leer_sesiones(ruta, "2026-10".to_string()).expect("leer sesiones");
        assert_eq!(sesiones.len(), 1);
        assert_eq!(sesiones[0].palabras, 412);
    }
}
