//! Comandos Tauri de la primera rebanada de la Fase 2.
//!
//! La UI no implementa reglas: abre un proyecto, lista documentos, lee cuerpos,
//! verifica y escribe a través de `writer-core`.

use serde::{Deserialize, Serialize};
use tauri::Manager;
use ts_rs::TS;
use writer_core::changes::CambioLocal;
use writer_core::goals::{Desafio, MetaArchivada, MetaDiaria, MetaLibro, Metas, Pausa, Sesion};
use writer_core::project::Project;
use writer_core::trash::EntradaPapelera;
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
    /// Rol del personaje.
    pub rol: Option<String>,
    /// Cast del personaje.
    pub cast: Vec<String>,
    /// Alias del personaje.
    pub alias: Vec<String>,
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

/// Una línea de una diferencia.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct LineaDiffDto {
    /// `igual`, `insertada` o `borrada`.
    pub tipo: String,
    /// Número de línea en la base.
    pub numero_base: Option<usize>,
    /// Número de línea en la propuesta o documento actual.
    pub numero_propuesta: Option<usize>,
    /// Texto de la línea.
    pub texto: String,
}

/// Resumen de un cambio local pendiente.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct CambioResumenDto {
    /// ULID del cambio.
    pub id: String,
    /// ID del documento.
    pub documento: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Autor.
    pub autor: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Revisión base.
    pub base_revision: u64,
    /// Líneas insertadas.
    pub insertadas: usize,
    /// Líneas borradas.
    pub borradas: usize,
}

/// Detalle de un cambio local.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct CambioDetalleDto {
    /// ULID del cambio.
    pub id: String,
    /// ID del documento.
    pub documento: String,
    /// Ruta relativa.
    pub ruta: String,
    /// Autor.
    pub autor: String,
    /// Fecha RFC 3339.
    pub ts: String,
    /// Revisión base.
    pub base_revision: u64,
    /// Hash base.
    pub base_hash: String,
    /// Cuerpo base.
    pub base: String,
    /// Cuerpo propuesto.
    pub propuesta: String,
    /// Líneas insertadas.
    pub insertadas: usize,
    /// Líneas borradas.
    pub borradas: usize,
    /// Diferencia por líneas.
    pub lineas: Vec<LineaDiffDto>,
}

/// Comparación de una revisión con el documento actual.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ComparacionDto {
    /// Revisión comparada.
    pub revision: u64,
    /// Ruta relativa.
    pub ruta: String,
    /// Si el documento existía en la revisión.
    pub base_existe: bool,
    /// Líneas insertadas.
    pub insertadas: usize,
    /// Líneas borradas.
    pub borradas: usize,
    /// Diferencia por líneas.
    pub lineas: Vec<LineaDiffDto>,
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

/// Punto de una trama en el Plot Grid.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PuntoDto {
    /// ULID de la escena.
    pub escena: String,
    /// Orden.
    pub orden: i64,
    /// Etiqueta.
    pub etiqueta: Option<String>,
}

/// Trama del Plot Grid.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct TramaGridDto {
    /// ULID de la nota.
    pub id: String,
    /// Nombre.
    pub nombre: String,
    /// Color.
    pub color: String,
    /// Puntos.
    pub puntos: Vec<PuntoDto>,
}

/// Plot Grid completo.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlotGridDto {
    /// Versión.
    pub version: u32,
    /// Tramas.
    pub tramas: Vec<TramaGridDto>,
}

impl From<writer_core::planning::Punto> for PuntoDto {
    fn from(valor: writer_core::planning::Punto) -> Self {
        Self {
            escena: valor.escena,
            orden: valor.orden,
            etiqueta: valor.etiqueta,
        }
    }
}

impl From<PuntoDto> for writer_core::planning::Punto {
    fn from(valor: PuntoDto) -> Self {
        Self {
            escena: valor.escena,
            orden: valor.orden,
            etiqueta: valor.etiqueta,
        }
    }
}

impl From<writer_core::planning::TramaGrid> for TramaGridDto {
    fn from(valor: writer_core::planning::TramaGrid) -> Self {
        Self {
            id: valor.id,
            nombre: valor.nombre,
            color: valor.color,
            puntos: valor.puntos.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<TramaGridDto> for writer_core::planning::TramaGrid {
    fn from(valor: TramaGridDto) -> Self {
        Self {
            id: valor.id,
            nombre: valor.nombre,
            color: valor.color,
            puntos: valor.puntos.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<writer_core::planning::PlotGrid> for PlotGridDto {
    fn from(valor: writer_core::planning::PlotGrid) -> Self {
        Self {
            version: valor.version,
            tramas: valor.tramas.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<PlotGridDto> for writer_core::planning::PlotGrid {
    fn from(valor: PlotGridDto) -> Self {
        Self {
            version: valor.version,
            tramas: valor.tramas.into_iter().map(Into::into).collect(),
        }
    }
}

/// Estructura de una plantilla.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct EstructuraDto {
    /// Libros.
    pub libros: u32,
    /// Capítulos por libro.
    pub capitulos_por_libro: u32,
    /// Escenas por capítulo.
    pub escenas_por_capitulo: u32,
}

/// Plantilla local.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PlantillaDto {
    /// Slug de carpeta.
    pub slug: String,
    /// Versión.
    pub version: u32,
    /// Nombre.
    pub nombre: String,
    /// Descripción.
    pub descripcion: String,
    /// Idioma.
    pub idioma: Option<String>,
    /// Estructura.
    pub estructura: EstructuraDto,
    /// Notas iniciales.
    pub notas_iniciales: Vec<String>,
}

impl From<(String, writer_core::templates::Plantilla)> for PlantillaDto {
    fn from((slug, valor): (String, writer_core::templates::Plantilla)) -> Self {
        Self {
            slug,
            version: valor.version,
            nombre: valor.nombre,
            descripcion: valor.descripcion,
            idioma: valor.idioma,
            estructura: EstructuraDto {
                libros: valor.estructura.libros,
                capitulos_por_libro: valor.estructura.capitulos_por_libro,
                escenas_por_capitulo: valor.estructura.escenas_por_capitulo,
            },
            notas_iniciales: valor.notas_iniciales,
        }
    }
}

impl From<PlantillaDto> for writer_core::templates::Plantilla {
    fn from(valor: PlantillaDto) -> Self {
        Self {
            version: valor.version,
            nombre: valor.nombre,
            descripcion: valor.descripcion,
            idioma: valor.idioma,
            estructura: writer_core::templates::Estructura {
                libros: valor.estructura.libros,
                capitulos_por_libro: valor.estructura.capitulos_por_libro,
                escenas_por_capitulo: valor.estructura.escenas_por_capitulo,
            },
            notas_iniciales: valor.notas_iniciales,
        }
    }
}

/// Entrada de la papelera local de proyectos.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct PapeleraDto {
    /// ULID de la entrada.
    pub id: String,
    /// Título del proyecto.
    pub titulo: String,
    /// Autor, si existía.
    pub autor: Option<String>,
    /// Color de portada, si existía.
    pub color: Option<String>,
    /// Palabras, si se conocían.
    pub palabras: u64,
    /// Ruta original absoluta.
    pub ruta_original: String,
    /// Ruta del contenedor de papelera.
    pub ruta_papelera: String,
    /// Fecha de eliminación RFC 3339.
    pub eliminado: String,
}

impl From<EntradaPapelera> for PapeleraDto {
    fn from(valor: EntradaPapelera) -> Self {
        Self {
            id: valor.id,
            titulo: valor.titulo,
            autor: valor.autor,
            color: valor.color,
            palabras: valor.palabras,
            ruta_original: valor.ruta_original,
            ruta_papelera: valor.ruta_papelera,
            eliminado: valor.eliminado,
        }
    }
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
fn listar_sesiones(ruta: String) -> Result<Vec<SesionDto>, ErrorDto> {
    writer_core::goals::listar_sesiones(std::path::Path::new(&ruta))
        .map(|sesiones| sesiones.into_iter().map(Into::into).collect())
        .map_err(error)
}

#[tauri::command]
fn leer_sesiones(ruta: String, mes: String) -> Result<Vec<SesionDto>, ErrorDto> {
    writer_core::goals::leer_sesiones(std::path::Path::new(&ruta), &mes)
        .map(|sesiones| sesiones.into_iter().map(Into::into).collect())
        .map_err(error)
}

fn directorio_plantillas(app: &tauri::AppHandle) -> Result<std::path::PathBuf, ErrorDto> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join("plantillas"))
        .map_err(|error| ErrorDto {
            tipo: TipoError::Io,
            mensaje: format!("no se pudo resolver el directorio de datos: {error}"),
        })
}

#[tauri::command]
fn previsualizar_importacion(archivo: String) -> Result<(String, String), ErrorDto> {
    writer_core::import::previsualizar(std::path::Path::new(&archivo)).map_err(error)
}

#[tauri::command]
fn importar_markdown(
    ruta: String,
    archivo: String,
    titulo: String,
    tipo_nota: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = writer_core::import::importar_markdown(
        &Project::new(&ruta),
        std::path::Path::new(&archivo),
        &titulo,
        tipo_nota.as_deref(),
    )
    .map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn importar_docx(
    ruta: String,
    archivo: String,
    titulo: String,
    tipo_nota: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = writer_core::import::importar_docx(
        &Project::new(&ruta),
        std::path::Path::new(&archivo),
        &titulo,
        tipo_nota.as_deref(),
    )
    .map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn importar_html(
    ruta: String,
    archivo: String,
    titulo: String,
    tipo_nota: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = writer_core::import::importar_html(
        &Project::new(&ruta),
        std::path::Path::new(&archivo),
        &titulo,
        tipo_nota.as_deref(),
    )
    .map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn importar_rtf(
    ruta: String,
    archivo: String,
    titulo: String,
    tipo_nota: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = writer_core::import::importar_rtf(
        &Project::new(&ruta),
        std::path::Path::new(&archivo),
        &titulo,
        tipo_nota.as_deref(),
    )
    .map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn crear_backup(ruta: String, destino: String) -> Result<String, ErrorDto> {
    writer_core::import::crear_backup(std::path::Path::new(&ruta), std::path::Path::new(&destino))
        .map(|ruta| ruta.display().to_string())
        .map_err(error)
}

#[tauri::command]
fn restaurar_backup(archivo: String, destino: String) -> Result<(), ErrorDto> {
    writer_core::import::restaurar_backup(
        std::path::Path::new(&archivo),
        std::path::Path::new(&destino),
    )
    .map(|_| ())
    .map_err(error)
}

#[tauri::command]
fn listar_plantillas(app: tauri::AppHandle) -> Result<Vec<PlantillaDto>, ErrorDto> {
    let directorio = directorio_plantillas(&app)?;
    writer_core::templates::listar_plantillas(&directorio)
        .map(|plantillas| plantillas.into_iter().map(Into::into).collect())
        .map_err(error)
}

#[tauri::command]
fn guardar_plantilla(
    app: tauri::AppHandle,
    plantilla: PlantillaDto,
) -> Result<PlantillaDto, ErrorDto> {
    let directorio = directorio_plantillas(&app)?;
    let slug = plantilla.slug.clone();
    writer_core::templates::guardar_plantilla(&directorio, &slug, &plantilla.clone().into())
        .map_err(error)?;
    Ok(plantilla)
}

#[tauri::command]
fn eliminar_plantilla(app: tauri::AppHandle, slug: String) -> Result<(), ErrorDto> {
    let directorio = directorio_plantillas(&app)?;
    writer_core::templates::eliminar_plantilla(&directorio, &slug).map_err(error)
}

#[tauri::command]
async fn crear_desde_plantilla(
    app: tauri::AppHandle,
    slug: String,
    ruta: String,
    titulo: String,
    autor: Option<String>,
    idioma: Option<String>,
) -> Result<ProyectoDto, ErrorDto> {
    let directorio = directorio_plantillas(&app)?;
    let ruta_tarea = ruta.clone();
    let resultado = tauri::async_runtime::spawn_blocking(move || {
        writer_core::templates::crear_desde_plantilla(
            &directorio,
            &slug,
            std::path::Path::new(&ruta_tarea),
            &titulo,
            autor.as_deref(),
            idioma.as_deref(),
        )
    })
    .await
    .map_err(|error| ErrorDto {
        tipo: TipoError::Otro,
        mensaje: format!("la creación desde plantilla se interrumpió: {error}"),
    })?;
    resultado.map_err(error)?;
    abrir_proyecto(ruta)
}

#[tauri::command]
fn mover_proyecto_a_papelera(
    ruta: String,
    titulo: String,
    autor: Option<String>,
    color: Option<String>,
    palabras: u64,
) -> Result<PapeleraDto, ErrorDto> {
    writer_core::trash::mover_a_papelera(
        std::path::Path::new(&ruta),
        &titulo,
        autor.as_deref(),
        color.as_deref(),
        palabras,
    )
    .map(Into::into)
    .map_err(error)
}

#[tauri::command]
fn restaurar_proyecto_de_papelera(ruta_papelera: String) -> Result<String, ErrorDto> {
    writer_core::trash::restaurar_de_papelera(std::path::Path::new(&ruta_papelera))
        .map(|ruta| ruta.to_string_lossy().to_string())
        .map_err(error)
}

#[tauri::command]
fn purgar_proyecto_de_papelera(ruta_papelera: String) -> Result<(), ErrorDto> {
    writer_core::trash::purgar_papelera(std::path::Path::new(&ruta_papelera)).map_err(error)
}

#[tauri::command]
fn leer_plot_grid(ruta: String) -> Result<PlotGridDto, ErrorDto> {
    writer_core::planning::leer_plot_grid(std::path::Path::new(&ruta))
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn guardar_plot_grid(ruta: String, grid: PlotGridDto) -> Result<(), ErrorDto> {
    writer_core::planning::guardar_plot_grid(std::path::Path::new(&ruta), &grid.into())
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
fn reabrir_comentario(ruta: String, id: String, autor: String) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::reabrir_comentario(std::path::Path::new(&ruta), &id, &autor)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn editar_mensaje_comentario(
    ruta: String,
    id: String,
    mensaje_id: String,
    autor: String,
    texto: String,
) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::editar_mensaje(
        std::path::Path::new(&ruta),
        &id,
        &mensaje_id,
        &autor,
        &texto,
    )
    .map(Into::into)
    .map_err(error)
}

#[tauri::command]
fn borrar_mensaje_comentario(
    ruta: String,
    id: String,
    mensaje_id: String,
    autor: String,
) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::borrar_mensaje(std::path::Path::new(&ruta), &id, &mensaje_id, &autor)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn archivar_comentario(ruta: String, id: String) -> Result<(), ErrorDto> {
    writer_core::comments::archivar_comentario(std::path::Path::new(&ruta), &id).map_err(error)
}

#[tauri::command]
fn resolver_comentario(ruta: String, id: String, autor: String) -> Result<ComentarioDto, ErrorDto> {
    writer_core::comments::resolver_comentario(std::path::Path::new(&ruta), &id, &autor)
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn sincronizar_comentarios_huerfanos(
    ruta: String,
    documento: String,
    texto: String,
) -> Result<Vec<ComentarioDto>, ErrorDto> {
    writer_core::comments::sincronizar_huerfanos(std::path::Path::new(&ruta), &documento, &texto)
        .map(|comentarios| comentarios.into_iter().map(Into::into).collect())
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
fn listar_carpetas(ruta: String) -> Result<Vec<String>, ErrorDto> {
    Project::new(&ruta).listar_carpetas().map_err(error)
}

#[tauri::command]
fn crear_carpeta(ruta: String, relativa: String) -> Result<String, ErrorDto> {
    Project::new(&ruta).crear_carpeta(&relativa).map_err(error)
}

#[tauri::command]
fn renombrar_carpeta(ruta: String, relativa: String, nombre: String) -> Result<String, ErrorDto> {
    Project::new(&ruta)
        .renombrar_carpeta(&relativa, &nombre)
        .map_err(error)
}

#[tauri::command]
fn eliminar_carpeta_vacia(ruta: String, relativa: String) -> Result<(), ErrorDto> {
    Project::new(&ruta)
        .eliminar_carpeta_vacia(&relativa)
        .map_err(error)
}

#[tauri::command]
fn crear_escena_en(
    ruta: String,
    carpeta: String,
    titulo: String,
) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = Project::new(&ruta)
        .crear_escena_en(&carpeta, &titulo)
        .map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn guardar_texto_exportado(
    destino: String,
    contenido: String,
    forzar: bool,
) -> Result<String, ErrorDto> {
    let destino = std::path::PathBuf::from(destino);
    if destino.exists() && !forzar {
        return Err(error(ErrorNucleo::Conflict(format!(
            "el destino ya existe: {}",
            destino.display()
        ))));
    }
    writer_core::fsutil::escribir_atomico(&destino, contenido.as_bytes()).map_err(error)?;
    Ok(destino.to_string_lossy().to_string())
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

fn lineas_diff(cambio: &CambioLocal) -> Vec<LineaDiffDto> {
    cambio
        .diferencias()
        .lineas
        .into_iter()
        .map(|linea| LineaDiffDto {
            tipo: linea.tipo.como_str().to_string(),
            numero_base: linea.numero_base,
            numero_propuesta: linea.numero_propuesta,
            texto: linea.texto,
        })
        .collect()
}

fn cambio_resumen_dto(cambio: &CambioLocal) -> CambioResumenDto {
    let diferencias = cambio.diferencias();
    CambioResumenDto {
        id: cambio.id.clone(),
        documento: cambio.documento.clone(),
        ruta: cambio.ruta.clone(),
        autor: cambio.autor.clone(),
        ts: cambio.ts.clone(),
        base_revision: cambio.base_revision,
        insertadas: diferencias.insertadas,
        borradas: diferencias.borradas,
    }
}

fn cambio_detalle_dto(cambio: &CambioLocal) -> CambioDetalleDto {
    let diferencias = cambio.diferencias();
    CambioDetalleDto {
        id: cambio.id.clone(),
        documento: cambio.documento.clone(),
        ruta: cambio.ruta.clone(),
        autor: cambio.autor.clone(),
        ts: cambio.ts.clone(),
        base_revision: cambio.base_revision,
        base_hash: cambio.base_hash.clone(),
        base: cambio.base.clone(),
        propuesta: cambio.propuesta.clone(),
        insertadas: diferencias.insertadas,
        borradas: diferencias.borradas,
        lineas: lineas_diff(cambio),
    }
}

#[tauri::command]
fn listar_cambios(ruta: String) -> Result<Vec<CambioResumenDto>, ErrorDto> {
    let cambios = Project::new(&ruta).listar_cambios().map_err(error)?;
    Ok(cambios.iter().map(cambio_resumen_dto).collect())
}

#[tauri::command]
fn leer_cambio(ruta: String, id: String) -> Result<CambioDetalleDto, ErrorDto> {
    let cambio = Project::new(&ruta).leer_cambio(&id).map_err(error)?;
    Ok(cambio_detalle_dto(&cambio))
}

#[tauri::command]
fn registrar_cambio(
    ruta: String,
    objetivo: String,
    cuerpo: String,
    autor: String,
) -> Result<Option<CambioDetalleDto>, ErrorDto> {
    let cambio = Project::new(&ruta)
        .registrar_cambio(&objetivo, &cuerpo, &autor)
        .map_err(error)?;
    Ok(cambio.as_ref().map(cambio_detalle_dto))
}

#[tauri::command]
fn aplicar_cambio(ruta: String, id: String) -> Result<GuardadoDto, ErrorDto> {
    let guardado = Project::new(&ruta).aplicar_cambio(&id).map_err(error)?;
    Ok(GuardadoDto {
        revision: guardado.revision,
        hash_archivo: guardado.hash_archivo,
    })
}

#[tauri::command]
fn descartar_cambio(ruta: String, id: String) -> Result<(), ErrorDto> {
    Project::new(&ruta).descartar_cambio(&id).map_err(error)
}

#[tauri::command]
fn comparar_documento(
    ruta: String,
    objetivo: String,
    revision: u64,
) -> Result<ComparacionDto, ErrorDto> {
    let comparacion = Project::new(&ruta)
        .comparar_documento(&objetivo, revision)
        .map_err(error)?;
    Ok(ComparacionDto {
        revision: comparacion.revision,
        ruta: comparacion.ruta,
        base_existe: comparacion.base_existe,
        insertadas: comparacion.diferencias.insertadas,
        borradas: comparacion.diferencias.borradas,
        lineas: comparacion
            .diferencias
            .lineas
            .into_iter()
            .map(|linea| LineaDiffDto {
                tipo: linea.tipo.como_str().to_string(),
                numero_base: linea.numero_base,
                numero_propuesta: linea.numero_propuesta,
                texto: linea.texto,
            })
            .collect(),
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

/// Formato de manuscrito para PDF.
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FormatoPdfDto {
    /// Fuente en puntos.
    pub fuente: f32,
    /// Interlineado en puntos.
    pub interlineado: f32,
    /// Margen en puntos.
    pub margen: f32,
}

impl From<writer_core::pdf::Formato> for FormatoPdfDto {
    fn from(valor: writer_core::pdf::Formato) -> Self {
        Self {
            fuente: valor.fuente,
            interlineado: valor.interlineado,
            margen: valor.margen,
        }
    }
}

impl From<FormatoPdfDto> for writer_core::pdf::Formato {
    fn from(valor: FormatoPdfDto) -> Self {
        Self {
            fuente: valor.fuente,
            interlineado: valor.interlineado,
            margen: valor.margen,
        }
    }
}

#[tauri::command]
fn leer_formato_pdf(ruta: String) -> Result<FormatoPdfDto, ErrorDto> {
    writer_core::pdf::leer_formato(std::path::Path::new(&ruta))
        .map(Into::into)
        .map_err(error)
}

#[tauri::command]
fn guardar_formato_pdf(ruta: String, formato: FormatoPdfDto) -> Result<(), ErrorDto> {
    writer_core::pdf::guardar_formato(std::path::Path::new(&ruta), &formato.into()).map_err(error)
}

#[tauri::command]
fn exportar_pdf(ruta: String, destino: String, forzar: bool) -> Result<String, ErrorDto> {
    Project::new(&ruta)
        .exportar_pdf(std::path::Path::new(&destino), forzar)
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
fn crear_nota(ruta: String, tipo: String, titulo: String) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = Project::new(&ruta)
        .crear_nota(&tipo, &titulo)
        .map_err(error)?;
    leer(ruta, id)
}

#[tauri::command]
fn crear_escena(
    ruta: String,
    titulo: String,
    libro: Option<String>,
    capitulo: Option<String>,
) -> Result<DocumentoDto, ErrorDto> {
    let (id, _) = Project::new(&ruta)
        .crear_escena(&titulo, libro.as_deref(), capitulo.as_deref(), None)
        .map_err(error)?;
    leer(ruta, id)
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
fn actualizar_personaje(
    ruta: String,
    objetivo: String,
    rol: Option<String>,
    cast: Vec<String>,
    alias: Vec<String>,
) -> Result<DocumentoDto, ErrorDto> {
    Project::new(&ruta)
        .actualizar_personaje(&objetivo, rol.as_deref(), &cast, &alias)
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
fn reordenar_escena(ruta: String, objetivo: String, orden: i64) -> Result<DocumentoDto, ErrorDto> {
    Project::new(&ruta)
        .reordenar_escena(&objetivo, orden)
        .map_err(error)?;
    leer(ruta, objetivo)
}

#[tauri::command]
fn intercambiar_orden(ruta: String, a: String, b: String) -> Result<(), ErrorDto> {
    Project::new(&ruta)
        .intercambiar_orden(&a, &b)
        .map_err(error)
}

#[tauri::command]
fn convertir_idea_en_proyecto(
    ruta: String,
    objetivo: String,
    destino: String,
    titulo: String,
) -> Result<ProyectoDto, ErrorDto> {
    Project::new(&ruta)
        .convertir_idea_en_proyecto(&objetivo, std::path::Path::new(&destino), &titulo)
        .map_err(error)?;
    abrir_proyecto(destino)
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
            rol: None,
            cast: Vec::new(),
            alias: Vec::new(),
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
            rol: encontrada.nota.rol,
            cast: encontrada.nota.cast.unwrap_or_default(),
            alias: encontrada.nota.alias.unwrap_or_default(),
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
            listar_carpetas,
            crear_carpeta,
            renombrar_carpeta,
            eliminar_carpeta_vacia,
            crear_escena_en,
            guardar_texto_exportado,
            leer,
            verificar,
            buscar,
            historial,
            mostrar,
            revertir,
            exportar_docx,
            exportar_pdf,
            leer_formato_pdf,
            guardar_formato_pdf,
            listar_cambios,
            leer_cambio,
            registrar_cambio,
            aplicar_cambio,
            descartar_cambio,
            comparar_documento,
            cuarentena,
            descartar_cuarentena,
            proyecto_e2e,
            escribir,
            crear_nota,
            crear_escena,
            mover,
            actualizar_escena,
            actualizar_nota,
            actualizar_personaje,
            cambiar_archivo,
            reordenar_escena,
            intercambiar_orden,
            convertir_idea_en_proyecto,
            renombrar,
            duplicar,
            leer_metas,
            guardar_metas,
            agregar_sesion,
            leer_sesiones,
            listar_sesiones,
            leer_plot_grid,
            guardar_plot_grid,
            listar_plantillas,
            guardar_plantilla,
            eliminar_plantilla,
            crear_desde_plantilla,
            mover_proyecto_a_papelera,
            restaurar_proyecto_de_papelera,
            purgar_proyecto_de_papelera,
            previsualizar_importacion,
            importar_markdown,
            importar_docx,
            importar_html,
            importar_rtf,
            crear_backup,
            restaurar_backup,
            listar_comentarios,
            crear_comentario,
            crear_sticky,
            agregar_mensaje_comentario,
            editar_mensaje_comentario,
            borrar_mensaje_comentario,
            reabrir_comentario,
            archivar_comentario,
            resolver_comentario,
            sincronizar_comentarios_huerfanos,
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
        LineaDiffDto::export_all(&config).expect("exportar LineaDiffDto");
        CambioResumenDto::export_all(&config).expect("exportar CambioResumenDto");
        CambioDetalleDto::export_all(&config).expect("exportar CambioDetalleDto");
        ComparacionDto::export_all(&config).expect("exportar ComparacionDto");
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
        PuntoDto::export_all(&config).expect("exportar PuntoDto");
        TramaGridDto::export_all(&config).expect("exportar TramaGridDto");
        PlotGridDto::export_all(&config).expect("exportar PlotGridDto");
        EstructuraDto::export_all(&config).expect("exportar EstructuraDto");
        PlantillaDto::export_all(&config).expect("exportar PlantillaDto");
        PapeleraDto::export_all(&config).expect("exportar PapeleraDto");
        FormatoPdfDto::export_all(&config).expect("exportar FormatoPdfDto");
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
    fn comandos_de_cambios_y_comparacion() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("cambios");
        let ruta = raiz.display().to_string();
        crear_proyecto(ruta.clone(), "Cambios".to_string(), None, None, false)
            .expect("crear proyecto");
        let escena = listar(ruta.clone(), false).expect("listar").remove(0);
        let cambio = registrar_cambio(
            ruta.clone(),
            escena.id.clone(),
            "Propuesta de prueba.\n".to_string(),
            "Autora".to_string(),
        )
        .expect("registrar")
        .expect("cambio");
        assert!(cambio.insertadas > 0);
        assert_eq!(
            listar_cambios(ruta.clone()).expect("listar cambios").len(),
            1
        );
        assert_eq!(
            leer(ruta.clone(), escena.id.clone()).expect("leer").cuerpo,
            escena.cuerpo
        );
        let guardado = aplicar_cambio(ruta.clone(), cambio.id.clone()).expect("aplicar");
        assert_eq!(guardado.revision, 2);
        assert!(listar_cambios(ruta.clone()).expect("cambios").is_empty());
        let comparacion = comparar_documento(ruta.clone(), escena.id.clone(), 1).expect("comparar");
        assert!(comparacion.insertadas > 0);
        assert!(comparacion
            .lineas
            .iter()
            .any(|linea| linea.tipo == "insertada" && linea.texto.contains("Propuesta")));
    }

    #[test]
    fn papelera_de_proyectos_sobre_un_proyecto_real() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("papelera");
        Project::new(&raiz)
            .init("Papelera", Some("Autora"), Some("es"))
            .expect("init");
        let entrada = mover_proyecto_a_papelera(
            raiz.display().to_string(),
            "Papelera".to_string(),
            Some("Autora".to_string()),
            Some("#6b5747".to_string()),
            10,
        )
        .expect("mover");
        assert_eq!(entrada.titulo, "Papelera");
        assert!(!raiz.exists());
        let ruta =
            restaurar_proyecto_de_papelera(entrada.ruta_papelera.clone()).expect("restaurar");
        assert_eq!(ruta, raiz.display().to_string());
        assert!(raiz.join("proyecto.yml").is_file());

        let segunda = mover_proyecto_a_papelera(
            raiz.display().to_string(),
            "Papelera".to_string(),
            None,
            None,
            0,
        )
        .expect("mover de nuevo");
        purgar_proyecto_de_papelera(segunda.ruta_papelera).expect("purgar");
        assert!(!raiz.exists());
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
    fn plot_grid_sobre_un_proyecto_real() {
        let directorio = tempfile::tempdir().expect("temporal");
        let raiz = directorio.path().join("planificacion");
        let ruta = raiz.display().to_string();
        crear_proyecto(ruta.clone(), "Plan".to_string(), None, None, false)
            .expect("crear proyecto");
        let documentos = listar(ruta.clone(), true).expect("listar");
        let escena = documentos
            .iter()
            .find(|documento| documento.tipo == "escena")
            .expect("escena");
        let grid = PlotGridDto {
            version: 1,
            tramas: vec![TramaGridDto {
                id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
                nombre: "El faro".to_string(),
                color: "#4d6b8a".to_string(),
                puntos: vec![PuntoDto {
                    escena: escena.id.clone(),
                    orden: 10,
                    etiqueta: Some("giro".to_string()),
                }],
            }],
        };
        guardar_plot_grid(ruta.clone(), grid).expect("guardar grid");
        let leido = leer_plot_grid(ruta).expect("leer grid");
        assert_eq!(leido.tramas.len(), 1);
        assert_eq!(leido.tramas[0].puntos[0].escena, escena.id);
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
    #[test]
    fn guardar_texto_exportado_no_sobrescribe_sin_forzar() {
        let directorio = tempfile::tempdir().expect("temporal");
        let destino = directorio.path().join("salida.md");
        let ruta = destino.display().to_string();
        guardar_texto_exportado(ruta.clone(), "# Hola".to_string(), false).expect("exportar texto");
        assert_eq!(std::fs::read_to_string(&destino).expect("leer"), "# Hola");
        assert!(guardar_texto_exportado(ruta, "otro".to_string(), false).is_err());
    }
}
