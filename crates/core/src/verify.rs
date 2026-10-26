//! Escaneo del proyecto y verificación de invariantes.
//!
//! Los archivos son la verdad: este módulo no usa el índice para decidir si algo
//! es válido. El índice solo se reconstruye a partir de lo que aquí se acepta.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::fsutil;
use crate::hash;
use crate::index::{EscenaFila, NotaFila};
use crate::model::{self, Escena, Nota, Proyecto, TIPOS_NOTA};

/// Escena válida encontrada en disco.
#[derive(Clone, Debug)]
pub struct EscenaEncontrada {
    /// Ruta absoluta.
    pub ruta: PathBuf,
    /// Ruta relativa con `/`.
    pub relativa: String,
    /// Hash del archivo.
    pub hash: String,
    /// mtime en nanosegundos.
    pub mtime_ns: i64,
    /// Tamaño.
    pub tamano: i64,
    /// Modelo.
    pub escena: Escena,
}

/// Nota válida encontrada en disco.
#[derive(Clone, Debug)]
pub struct NotaEncontrada {
    /// Ruta absoluta.
    pub ruta: PathBuf,
    /// Ruta relativa con `/`.
    pub relativa: String,
    /// Hash del archivo.
    pub hash: String,
    /// mtime en nanosegundos.
    pub mtime_ns: i64,
    /// Tamaño.
    pub tamano: i64,
    /// Modelo.
    pub nota: Nota,
}

/// Documento válido.
#[derive(Clone, Debug)]
pub enum Documento {
    /// Escena.
    Escena(EscenaEncontrada),
    /// Nota.
    Nota(NotaEncontrada),
}

impl Documento {
    /// ID del documento.
    pub fn id(&self) -> &str {
        match self {
            Documento::Escena(escena) => &escena.escena.id,
            Documento::Nota(nota) => &nota.nota.id,
        }
    }

    /// Título.
    pub fn titulo(&self) -> &str {
        match self {
            Documento::Escena(escena) => &escena.escena.titulo,
            Documento::Nota(nota) => &nota.nota.titulo,
        }
    }

    /// Cuerpo.
    pub fn cuerpo(&self) -> &str {
        match self {
            Documento::Escena(escena) => &escena.escena.cuerpo,
            Documento::Nota(nota) => &nota.nota.cuerpo,
        }
    }

    /// Ruta relativa.
    pub fn relativa(&self) -> &str {
        match self {
            Documento::Escena(escena) => &escena.relativa,
            Documento::Nota(nota) => &nota.relativa,
        }
    }

    /// Ruta absoluta.
    pub fn ruta(&self) -> &Path {
        match self {
            Documento::Escena(escena) => &escena.ruta,
            Documento::Nota(nota) => &nota.ruta,
        }
    }

    /// Hash del archivo completo, tal como lo calculó el escaneo.
    ///
    /// Sirve para el control optimista de concurrencia: la interfaz lo guarda
    /// al abrir y lo devuelve al escribir. Si el archivo cambió por fuera, el
    /// hash no coincide y la escritura se rechaza como conflicto.
    pub fn hash(&self) -> &str {
        match self {
            Documento::Escena(escena) => &escena.hash,
            Documento::Nota(nota) => &nota.hash,
        }
    }

    /// Metadatos.
    pub fn metadatos(&self) -> (i64, i64) {
        match self {
            Documento::Escena(escena) => (escena.mtime_ns, escena.tamano),
            Documento::Nota(nota) => (nota.mtime_ns, nota.tamano),
        }
    }
}

/// Archivo inválido que iría a cuarentena.
#[derive(Clone, Debug)]
pub struct ArchivoInvalido {
    /// Ruta relativa.
    pub relativa: String,
    /// Motivo.
    pub motivo: String,
    /// Hash de los bytes originales.
    pub hash: String,
}

/// Resultado de escanear el proyecto.
#[derive(Clone, Debug, Default)]
pub struct Escaneo {
    /// Raíz del proyecto.
    pub raiz: PathBuf,
    /// Manifiesto, si es válido.
    pub proyecto: Option<Proyecto>,
    /// Escenas válidas.
    pub escenas: Vec<EscenaEncontrada>,
    /// Notas válidas.
    pub notas: Vec<NotaEncontrada>,
    /// Errores bloqueantes.
    pub errores: Vec<String>,
    /// Avisos.
    pub avisos: Vec<String>,
    /// Archivos inválidos para cuarentena.
    pub invalidos: Vec<ArchivoInvalido>,
}

impl Escaneo {
    /// Indica si se puede escribir sin reparar el proyecto.
    pub fn escribible(&self) -> bool {
        self.proyecto.is_some() && self.errores.is_empty()
    }

    /// Busca un documento por ID.
    pub fn por_id(&self, id: &str) -> Option<Documento> {
        if let Some(escena) = self.escenas.iter().find(|escena| escena.escena.id == id) {
            return Some(Documento::Escena(escena.clone()));
        }
        self.notas
            .iter()
            .find(|nota| nota.nota.id == id)
            .map(|nota| Documento::Nota(nota.clone()))
    }

    /// Busca un documento por ruta relativa.
    pub fn por_ruta(&self, relativa: &str) -> Option<Documento> {
        if let Some(escena) = self
            .escenas
            .iter()
            .find(|escena| escena.relativa == relativa)
        {
            return Some(Documento::Escena(escena.clone()));
        }
        self.notas
            .iter()
            .find(|nota| nota.relativa == relativa)
            .map(|nota| Documento::Nota(nota.clone()))
    }

    /// Indica si una ruta quedó marcada como inválida (por ejemplo, por un ID
    /// duplicado). Esos documentos no entran en el índice ni en el árbol.
    pub fn es_invalida(&self, relativa: &str) -> bool {
        self.invalidos
            .iter()
            .any(|invalido| invalido.relativa == relativa)
    }

    /// Documentos en orden de lectura: escenas por libro/capítulo/orden y notas
    /// agrupadas por tipo.
    pub fn ordenados(&self) -> Vec<Documento> {
        let mut escenas: Vec<_> = self
            .escenas
            .iter()
            .filter(|escena| !self.es_invalida(&escena.relativa))
            .cloned()
            .collect();
        escenas.sort_by_key(|a| clave_lectura(&a.relativa, a.escena.orden));
        let mut notas: Vec<_> = self
            .notas
            .iter()
            .filter(|nota| !self.es_invalida(&nota.relativa))
            .cloned()
            .collect();
        notas.sort_by_key(|a| {
            (
                a.nota.tipo.clone(),
                a.nota.titulo.clone(),
                a.relativa.clone(),
            )
        });
        let mut salida: Vec<Documento> = escenas.into_iter().map(Documento::Escena).collect();
        salida.extend(notas.into_iter().map(Documento::Nota));
        salida
    }

    /// Filas para reconstruir el índice.
    pub fn filas_indice(&self) -> (Vec<EscenaFila>, Vec<NotaFila>) {
        let escenas = self
            .escenas
            .iter()
            .filter(|encontrada| !self.es_invalida(&encontrada.relativa))
            .map(|encontrada| {
                let escena = &encontrada.escena;
                EscenaFila {
                    id: escena.id.clone(),
                    ruta: encontrada.relativa.clone(),
                    hash: encontrada.hash.clone(),
                    mtime_ns: encontrada.mtime_ns,
                    tamano: encontrada.tamano,
                    titulo: escena.titulo.clone(),
                    estado: escena.estado.clone(),
                    pov: escena.pov.clone(),
                    orden: escena.orden,
                    fecha_narrativa: escena.fecha_narrativa.clone(),
                    palabras: model::contar_palabras(&escena.cuerpo),
                    tramas: escena.tramas.clone().unwrap_or_default(),
                    personajes: escena.personajes.clone().unwrap_or_default(),
                    etiquetas: escena.etiquetas.clone().unwrap_or_default(),
                    cuerpo: escena.cuerpo.clone(),
                }
            })
            .collect();
        let notas = self
            .notas
            .iter()
            .filter(|encontrada| !self.es_invalida(&encontrada.relativa))
            .map(|encontrada| {
                let nota = &encontrada.nota;
                NotaFila {
                    id: nota.id.clone(),
                    ruta: encontrada.relativa.clone(),
                    tipo: nota.tipo.clone(),
                    titulo: nota.titulo.clone(),
                    fija: nota.fija.unwrap_or(false),
                    hash: encontrada.hash.clone(),
                    mtime_ns: encontrada.mtime_ns,
                    tamano: encontrada.tamano,
                    etiquetas: nota.etiquetas.clone().unwrap_or_default(),
                    refs: nota.refs.clone().unwrap_or_default(),
                    cuerpo: nota.cuerpo.clone(),
                }
            })
            .collect();
        (escenas, notas)
    }

    /// Resumen legible.
    pub fn resumen(&self) -> String {
        format!(
            "escenas: {} · notas: {} · errores: {} · avisos: {}",
            self.escenas.len(),
            self.notas.len(),
            self.errores.len(),
            self.avisos.len()
        )
    }
}

/// Escanea una raíz de proyecto sin modificarla.
pub fn escanear(raiz: &Path) -> Result<Escaneo> {
    if !raiz.is_dir() {
        return Err(Error::Project(format!(
            "no existe el directorio del proyecto: {}",
            raiz.display()
        )));
    }
    let mut escaneo = Escaneo {
        raiz: raiz.to_path_buf(),
        ..Escaneo::default()
    };

    match leer_proyecto(raiz) {
        Ok((proyecto, avisos)) => {
            escaneo.proyecto = Some(proyecto);
            escaneo.avisos.extend(avisos);
        }
        Err(error) => escaneo.errores.push(error.to_string()),
    }

    let mut archivos = Vec::new();
    recorrer_md(raiz, raiz, &mut archivos, &mut escaneo.avisos)?;
    archivos.sort();

    for ruta in archivos {
        let relativa = fsutil::ruta_relativa(&ruta, raiz)?;
        let partes: Vec<&str> = relativa.split('/').collect();
        if partes.first() == Some(&"manuscrito") {
            if partes.len() != 4 {
                escaneo.errores.push(format!(
                    "{relativa}: una escena debe estar en manuscrito/libro/capitulo/"
                ));
                escaneo
                    .invalidos
                    .push(archivo_invalido(&ruta, &relativa, "estructura de escena")?);
                continue;
            }
            match leer_escena(&ruta, &relativa) {
                Ok((escena, avisos)) => {
                    escaneo.avisos.extend(avisos);
                    let metadatos = fsutil::metadatos(&ruta)?;
                    let bytes =
                        std::fs::read(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
                    escaneo.escenas.push(EscenaEncontrada {
                        ruta,
                        relativa,
                        hash: hash::hash(&bytes),
                        mtime_ns: metadatos.mtime_ns,
                        tamano: metadatos.tamano,
                        escena,
                    });
                }
                Err(error) => {
                    escaneo.errores.push(error.to_string());
                    escaneo
                        .invalidos
                        .push(archivo_invalido(&ruta, &relativa, &error.to_string())?);
                }
            }
        } else if partes.first() == Some(&"notas") {
            let tipo_valido = partes.len() == 3 && TIPOS_NOTA.contains(&partes[1]);
            if !tipo_valido {
                escaneo
                    .errores
                    .push(format!("{relativa}: nota fuera de notas/<tipo>/"));
                escaneo
                    .invalidos
                    .push(archivo_invalido(&ruta, &relativa, "estructura de nota")?);
                continue;
            }
            match leer_nota(&ruta, &relativa, partes[1]) {
                Ok((nota, avisos)) => {
                    escaneo.avisos.extend(avisos);
                    let metadatos = fsutil::metadatos(&ruta)?;
                    let bytes =
                        std::fs::read(&ruta).map_err(|error| Error::Io(error).with_path(&ruta))?;
                    escaneo.notas.push(NotaEncontrada {
                        ruta,
                        relativa,
                        hash: hash::hash(&bytes),
                        mtime_ns: metadatos.mtime_ns,
                        tamano: metadatos.tamano,
                        nota,
                    });
                }
                Err(error) => {
                    escaneo.errores.push(error.to_string());
                    escaneo
                        .invalidos
                        .push(archivo_invalido(&ruta, &relativa, &error.to_string())?);
                }
            }
        } else {
            let pretende_ser_proyecto = model::leer_documento(&ruta)
                .map(|(campos, _, _)| campos.contains_key("id"))
                .unwrap_or(false);
            if pretende_ser_proyecto {
                escaneo.errores.push(format!(
                    "{relativa}: frontmatter con 'id' fuera de manuscrito/ y notas/"
                ));
                escaneo.invalidos.push(archivo_invalido(
                    &ruta,
                    &relativa,
                    "Markdown con id fuera de manuscrito/ y notas/",
                )?);
            } else {
                escaneo.avisos.push(format!(
                    "{relativa}: Markdown fuera de manuscrito/ y notas/"
                ));
            }
        }
    }

    verificar_ids_y_referencias(&mut escaneo);
    verificar_orden(&mut escaneo);
    verificar_sidecars(raiz, &mut escaneo);
    if escaneo.escenas.is_empty() {
        escaneo
            .errores
            .push("el proyecto no tiene escenas".to_string());
    }
    escaneo.errores.sort();
    escaneo.errores.dedup();
    escaneo.avisos.sort();
    escaneo.avisos.dedup();
    Ok(escaneo)
}

fn verificar_sidecars(raiz: &Path, escaneo: &mut Escaneo) {
    let escritor = raiz.join(".escritor");
    let revisar = |ruta: &Path| -> Option<String> {
        match std::fs::symlink_metadata(ruta) {
            Ok(meta) if meta.file_type().is_symlink() => {
                Some(format!("{}: sidecar enlazado", ruta.display()))
            }
            Ok(_) => None,
            Err(_) => None,
        }
    };
    let metas = escritor.join("metas/metas.json");
    if metas.exists() {
        if let Some(error) = revisar(&metas) {
            escaneo.errores.push(error);
        } else {
            match std::fs::read(&metas)
                .map_err(|error| Error::Io(error).with_path(&metas))
                .and_then(|bytes| {
                    serde_json::from_slice::<crate::goals::Metas>(&bytes).map_err(Error::Json)
                })
                .and_then(|metas| metas.validar())
            {
                Ok(()) => {}
                Err(error) => escaneo.avisos.push(format!("{}: {error}", metas.display())),
            }
        }
    }
    let grid = escritor.join("planificacion/plot-grid.json");
    if grid.exists() {
        if let Some(error) = revisar(&grid) {
            escaneo.avisos.push(error);
        } else {
            match std::fs::read(&grid)
                .map_err(|error| Error::Io(error).with_path(&grid))
                .and_then(|bytes| {
                    serde_json::from_slice::<crate::planning::PlotGrid>(&bytes).map_err(Error::Json)
                })
                .and_then(|grid| grid.validar())
            {
                Ok(()) => {}
                Err(error) => escaneo.avisos.push(format!("{}: {error}", grid.display())),
            }
        }
    }
    let sesiones = escritor.join("sesiones");
    if let Ok(entradas) = std::fs::read_dir(&sesiones) {
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if !fsutil::es_extension(&ruta, "jsonl") {
                continue;
            }
            if let Some(error) = revisar(&ruta) {
                escaneo.avisos.push(error);
                continue;
            }
            let Ok(texto) = std::fs::read_to_string(&ruta) else {
                escaneo
                    .avisos
                    .push(format!("{}: no se puede leer", ruta.display()));
                continue;
            };
            for (numero, linea) in texto.lines().enumerate() {
                if linea.trim().is_empty() {
                    continue;
                }
                let resultado = serde_json::from_str::<crate::goals::Sesion>(linea)
                    .map_err(Error::Json)
                    .and_then(|sesion| sesion.validar());
                if let Err(error) = resultado {
                    escaneo.avisos.push(format!(
                        "{}: línea {}: {error}",
                        ruta.display(),
                        numero + 1
                    ));
                }
            }
        }
    }
    let comentarios = escritor.join("comentarios");
    if let Ok(entradas) = std::fs::read_dir(&comentarios) {
        for entrada in entradas.flatten() {
            let ruta = entrada.path();
            if !fsutil::es_extension(&ruta, "json") {
                continue;
            }
            if let Some(error) = revisar(&ruta) {
                escaneo.avisos.push(error);
                continue;
            }
            let resultado = std::fs::read(&ruta)
                .map_err(|error| Error::Io(error).with_path(&ruta))
                .and_then(|bytes| {
                    serde_json::from_slice::<crate::comments::Comentario>(&bytes)
                        .map_err(Error::Json)
                })
                .and_then(|comentario| comentario.validar());
            if let Err(error) = resultado {
                escaneo.avisos.push(format!("{}: {error}", ruta.display()));
            }
        }
    }
}

fn leer_proyecto(raiz: &Path) -> Result<(Proyecto, Vec<String>)> {
    let ruta = raiz.join("proyecto.yml");
    if !ruta.is_file() {
        return Err(Error::Project("falta proyecto.yml".to_string()));
    }
    let ruta = fsutil::archivo_real(&ruta)?;
    let (campos, mut avisos) = model::leer_plano(&ruta)?;
    let contexto = "proyecto.yml";
    let (proyecto, avisos_modelo) = Proyecto::desde_campos(&campos, contexto)?;
    let icono = "proyecto.yml".to_string();
    avisos = avisos
        .into_iter()
        .map(|aviso| format!("{icono}: {aviso}"))
        .collect();
    avisos.extend(avisos_modelo);
    Ok((proyecto, avisos))
}

fn leer_escena(ruta: &Path, relativa: &str) -> Result<(Escena, Vec<String>)> {
    let (campos, cuerpo, avisos_bytes) = model::leer_documento(ruta)?;
    let (escena, avisos_modelo) = Escena::desde_campos(&campos, &cuerpo, relativa)?;
    let mut avisos = avisos_bytes
        .into_iter()
        .map(|aviso| format!("{relativa}: {aviso}"))
        .collect::<Vec<_>>();
    avisos.extend(avisos_modelo);
    Ok((escena, avisos))
}

fn leer_nota(ruta: &Path, relativa: &str, tipo_carpeta: &str) -> Result<(Nota, Vec<String>)> {
    let (campos, cuerpo, avisos_bytes) = model::leer_documento(ruta)?;
    let (nota, avisos_modelo) = Nota::desde_campos(&campos, &cuerpo, relativa)?;
    if nota.tipo != tipo_carpeta {
        return Err(Error::Schema(format!(
            "{relativa}: 'tipo' {:?} no coincide con la carpeta {tipo_carpeta:?}",
            nota.tipo
        )));
    }
    let mut avisos = avisos_bytes
        .into_iter()
        .map(|aviso| format!("{relativa}: {aviso}"))
        .collect::<Vec<_>>();
    avisos.extend(avisos_modelo);
    Ok((nota, avisos))
}

fn verificar_ids_y_referencias(escaneo: &mut Escaneo) {
    let mut ids: BTreeMap<String, String> = BTreeMap::new();
    let mut ids_escenas: BTreeSet<String> = BTreeSet::new();
    let mut tipos_nota: BTreeMap<String, String> = BTreeMap::new();
    let mut duplicados: Vec<String> = Vec::new();
    let id_proyecto = escaneo
        .proyecto
        .as_ref()
        .map(|proyecto| proyecto.id.clone());

    for escena in &escaneo.escenas {
        let id = escena.escena.id.clone();
        if let Some(anterior) = ids.insert(id.clone(), escena.relativa.clone()) {
            escaneo.errores.push(format!(
                "{}: 'id' duplicado con {anterior} ({id})",
                escena.relativa
            ));
            duplicados.push(anterior);
            duplicados.push(escena.relativa.clone());
        } else {
            ids_escenas.insert(id.clone());
        }
    }
    for nota in &escaneo.notas {
        let id = nota.nota.id.clone();
        if let Some(anterior) = ids.insert(id.clone(), nota.relativa.clone()) {
            escaneo.errores.push(format!(
                "{}: 'id' duplicado con {anterior} ({id})",
                nota.relativa
            ));
            duplicados.push(anterior);
            duplicados.push(nota.relativa.clone());
        } else {
            tipos_nota.insert(id.clone(), nota.nota.tipo.clone());
        }
    }
    for relativa in duplicados {
        agregar_invalido(escaneo, &relativa, "id duplicado");
    }

    if let Some(id) = &id_proyecto {
        for documento in escaneo
            .escenas
            .iter()
            .map(|escena| (&escena.relativa, &escena.escena.id))
            .chain(
                escaneo
                    .notas
                    .iter()
                    .map(|nota| (&nota.relativa, &nota.nota.id)),
            )
        {
            if documento.1 == id {
                escaneo.avisos.push(format!(
                    "{}: el id del documento coincide con el del proyecto",
                    documento.0
                ));
            }
        }
    }

    let mut destinos: BTreeMap<&str, &str> = BTreeMap::new();
    for id in &ids_escenas {
        destinos.insert(id.as_str(), "escena");
    }
    for (id, tipo) in &tipos_nota {
        destinos.insert(id.as_str(), tipo.as_str());
    }
    if let Some(id) = &id_proyecto {
        destinos.insert(id.as_str(), "proyecto");
    }

    let mut referencias: Vec<(&str, &str, &str)> = Vec::new();
    for escena in &escaneo.escenas {
        let relativa = escena.relativa.as_str();
        if let Some(pov) = &escena.escena.pov {
            referencias.push((relativa, "pov", pov));
        }
        for valor in escena.escena.tramas.iter().flatten() {
            referencias.push((relativa, "tramas", valor));
        }
        for valor in escena.escena.personajes.iter().flatten() {
            referencias.push((relativa, "personajes", valor));
        }
    }
    for nota in &escaneo.notas {
        for valor in nota.nota.refs.iter().flatten() {
            referencias.push((&nota.relativa, "refs", valor));
        }
    }

    for (relativa, campo, valor) in referencias {
        if !crate::ulid::es_valido(valor) {
            escaneo.errores.push(format!(
                "{relativa}: referencia no ULID en '{campo}': {valor}"
            ));
            continue;
        }
        let Some(tipo) = destinos.get(valor) else {
            escaneo.errores.push(format!(
                "{relativa}: '{campo}' apunta a un id inexistente: {valor}"
            ));
            continue;
        };
        if (campo == "pov" || campo == "personajes") && *tipo != "personajes" {
            escaneo.errores.push(format!(
                "{relativa}: '{campo}' apunta a un id que no es personaje: {valor} ({tipo})"
            ));
        }
        if campo == "tramas" && *tipo != "tramas" {
            escaneo.errores.push(format!(
                "{relativa}: 'tramas' apunta a un id que no es trama: {valor} ({tipo})"
            ));
        }
    }
}

fn verificar_orden(escaneo: &mut Escaneo) {
    let mut ordenes: BTreeMap<String, BTreeMap<i64, String>> = BTreeMap::new();
    for escena in &escaneo.escenas {
        let partes: Vec<&str> = escena.relativa.split('/').collect();
        if partes.len() != 4 {
            continue;
        }
        let carpeta = format!("{}/{}", partes[0], partes[1..3].join("/"));
        let orden = escena.escena.orden;
        let capitulo = ordenes.entry(carpeta.clone()).or_default();
        if let Some(anterior) = capitulo.get(&orden) {
            escaneo.errores.push(format!(
                "{}: 'orden' {orden} repetido en la carpeta {carpeta} (ya usado por {anterior})",
                escena.relativa
            ));
        } else {
            capitulo.insert(orden, escena.relativa.clone());
        }
    }
}

fn recorrer_md(
    raiz: &Path,
    actual: &Path,
    salida: &mut Vec<PathBuf>,
    avisos: &mut Vec<String>,
) -> Result<()> {
    let mut entradas: Vec<_> = std::fs::read_dir(actual)
        .map_err(|error| Error::Io(error).with_path(actual))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| Error::Io(error).with_path(actual))?;
    entradas.sort_by_key(|entrada| entrada.file_name());
    for entrada in entradas {
        let ruta = entrada.path();
        let nombre = entrada.file_name().to_string_lossy().to_string();
        if nombre == ".escritor" {
            continue;
        }
        let tipo = entrada
            .file_type()
            .map_err(|error| Error::Io(error).with_path(&ruta))?;
        if tipo.is_symlink() {
            if let Ok(relativa) = fsutil::ruta_relativa(&ruta, raiz) {
                avisos.push(format!("{relativa}: enlace simbólico ignorado"));
            }
            continue;
        }
        if tipo.is_dir() {
            recorrer_md(raiz, &ruta, salida, avisos)?;
        } else if tipo.is_file() && fsutil::es_extension(&ruta, "md") {
            salida.push(ruta);
        }
    }
    Ok(())
}

fn archivo_invalido(ruta: &Path, relativa: &str, motivo: &str) -> Result<ArchivoInvalido> {
    let bytes = std::fs::read(ruta).map_err(|error| Error::Io(error).with_path(ruta))?;
    Ok(ArchivoInvalido {
        relativa: relativa.to_string(),
        motivo: motivo.to_string(),
        hash: hash::hash(&bytes),
    })
}

fn agregar_invalido(escaneo: &mut Escaneo, relativa: &str, motivo: &str) {
    if escaneo
        .invalidos
        .iter()
        .any(|invalido| invalido.relativa == relativa && invalido.motivo == motivo)
    {
        return;
    }
    let Ok(ruta) = fsutil::ruta_del_proyecto(&escaneo.raiz, relativa) else {
        return;
    };
    let Ok(bytes) = std::fs::read(&ruta) else {
        return;
    };
    escaneo.invalidos.push(ArchivoInvalido {
        relativa: relativa.to_string(),
        motivo: motivo.to_string(),
        hash: hash::hash(&bytes),
    });
}

fn prefijo_orden(relativa: &str, nivel: usize) -> (u64, String) {
    let parte = relativa.split('/').nth(nivel).unwrap_or("");
    match parte.split_once('-') {
        Some((prefijo, _)) if prefijo.bytes().all(|byte| byte.is_ascii_digit()) => {
            (prefijo.parse().unwrap_or(u64::MAX), parte.to_string())
        }
        _ => (u64::MAX, parte.to_string()),
    }
}

fn clave_lectura(relativa: &str, orden: i64) -> ((u64, String), (u64, String), i64, String) {
    (
        prefijo_orden(relativa, 1),
        prefijo_orden(relativa, 2),
        orden,
        relativa.to_string(),
    )
}

/// Copia archivos inválidos a `.escritor/cuarentena/` sin tocar el original.
pub fn cuarentenar(raiz: &Path, escaneo: &Escaneo) -> Result<usize> {
    if escaneo.invalidos.is_empty() {
        return Ok(0);
    }
    let directorio = fsutil::directorio_cuarentena(raiz, true)?
        .ok_or_else(|| Error::Integrity("no se pudo crear la cuarentena".into()))?;
    let informe_ruta = directorio.join("informe.jsonl");
    let mut informe = fsutil::leer_opcional(&informe_ruta)?.unwrap_or_default();
    let mut copiados = 0;
    for invalido in &escaneo.invalidos {
        let origen = fsutil::ruta_del_proyecto(raiz, &invalido.relativa)?;
        let bytes = std::fs::read(&origen).map_err(|error| Error::Io(error).with_path(&origen))?;
        let base = invalido
            .relativa
            .rsplit('/')
            .next()
            .unwrap_or("archivo.md")
            .trim_end_matches(".md");
        let motivo: String = model::slug(&invalido.motivo).chars().take(40).collect();
        let mut nombre = format!(
            "{}-{}-{}.md",
            crate::time::timestamp_slug(crate::time::now_ms()),
            model::slug(base),
            motivo
        );
        let mut destino = directorio.join(&nombre);
        let mut intento = 2_u32;
        while destino.exists() {
            nombre = format!(
                "{}-{}-{}-{intento}.md",
                crate::time::timestamp_slug(crate::time::now_ms()),
                model::slug(base),
                motivo
            );
            destino = directorio.join(&nombre);
            intento += 1;
        }
        fsutil::escribir_atomico(&destino, &bytes)?;
        let linea = serde_json::json!({
            "ts": crate::time::ahora(),
            "archivo": nombre,
            "ruta_original": invalido.relativa,
            "motivo": invalido.motivo,
            "hash": invalido.hash,
        });
        if !informe.is_empty() && !informe.ends_with(b"\n") {
            informe.push(b'\n');
        }
        informe.extend_from_slice(serde_json::to_string(&linea)?.as_bytes());
        informe.push(b'\n');
        fsutil::escribir_atomico(&informe_ruta, &informe)?;
        copiados += 1;
    }
    Ok(copiados)
}
