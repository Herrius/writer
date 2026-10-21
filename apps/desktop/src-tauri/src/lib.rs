//! Comandos Tauri de la primera rebanada de la Fase 2.
//!
//! La UI no implementa reglas: abre un proyecto, lista documentos, lee cuerpos,
//! verifica y escribe a través de `writer-core`.

use serde::Serialize;
use ts_rs::TS;
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
    std::env::var("WRITER_E2E_PROJECT")
        .ok()
        .filter(|ruta| !ruta.trim().is_empty())
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
            escribir
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
}
