//! Binario `writer`: CLI del motor.
//!
//! CLI-first: todo lo que hará la aplicación de escritorio se puede hacer antes
//! por aquí. La CLI no implementa reglas: llama a `writer-core`.

use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use serde_json::json;
use writer_core::project::Project;
use writer_core::verify::{self, Documento, Escaneo};
use writer_core::Error;

#[derive(Debug, Parser)]
#[command(
    name = "writer",
    version = concat!(env!("CARGO_PKG_VERSION"), " (esquema 1)"),
    about = "Motor de proyectos de escritura con archivos como fuente de verdad",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    comando: Comando,
}

#[derive(Debug, Subcommand)]
enum Comando {
    /// Crea un proyecto nuevo en un directorio vacío.
    Init {
        /// Directorio del proyecto.
        #[arg(default_value = ".")]
        ruta: PathBuf,
        /// Título del proyecto.
        #[arg(long, default_value = "Mi novela")]
        titulo: String,
        /// Autor.
        #[arg(long, default_value = "")]
        autor: String,
        /// Código de idioma.
        #[arg(long, default_value = "es")]
        idioma: String,
    },
    /// Muestra un resumen del proyecto.
    Info {
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Lista escenas (y notas con --notas).
    List {
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        notas: bool,
        #[arg(long)]
        json: bool,
    },
    /// Busca texto en escenas y notas.
    Buscar {
        /// Texto a buscar.
        consulta: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
        /// Número máximo de resultados.
        #[arg(long, default_value_t = 20)]
        limite: usize,
        #[arg(long)]
        json: bool,
    },
    /// Lee un documento por ID o ruta relativa.
    Read {
        objetivo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
        /// Muestra el archivo completo, con frontmatter.
        #[arg(long)]
        completo: bool,
        #[arg(long)]
        json: bool,
    },
    /// Crea una escena o una nota.
    Create {
        #[command(subcommand)]
        tipo: Crear,
    },
    /// Reemplaza el cuerpo de un documento.
    Write {
        objetivo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
        /// Texto nuevo.
        #[arg(long)]
        texto: Option<String>,
        /// Archivo del que leer el texto nuevo.
        #[arg(long)]
        archivo: Option<PathBuf>,
        /// Lee el texto nuevo de la entrada estándar.
        #[arg(long)]
        stdin: bool,
    },
    /// Mueve una escena o nota sin cambiar su ID.
    Move {
        objetivo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        libro: Option<String>,
        #[arg(long)]
        capitulo: Option<String>,
        #[arg(long)]
        tipo: Option<String>,
    },
    /// Elimina un documento (reversible desde el historial).
    Delete {
        objetivo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
    },
    /// Muestra el historial de revisiones.
    History {
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Muestra una revisión y su contenido.
    Show {
        revision: u64,
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Revierte una revisión creando una revisión nueva.
    Revert {
        revision: u64,
        #[arg(default_value = ".")]
        ruta: PathBuf,
    },
    /// Verifica el esquema y los invariantes del proyecto.
    Verify {
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        json: bool,
        /// Copia los archivos inválidos a .escritor/cuarentena.
        #[arg(long)]
        cuarentena: bool,
    },
    /// Reconstruye el índice SQLite.
    Reindex {
        #[arg(default_value = ".")]
        ruta: PathBuf,
        /// Elimina blobs no referenciados.
        #[arg(long)]
        limpiar: bool,
    },
    /// Lista o descarta copias de cuarentena.
    Quarantine {
        #[command(subcommand)]
        accion: CuarentenaAccion,
    },
    /// Exporta el manuscrito a Markdown.
    Export {
        #[command(subcommand)]
        formato: Exportar,
    },
}

#[derive(Debug, Subcommand)]
enum Crear {
    /// Crea una escena.
    Escena {
        #[arg(long)]
        titulo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        libro: Option<String>,
        #[arg(long)]
        capitulo: Option<String>,
        #[arg(long)]
        orden: Option<i64>,
    },
    /// Crea una nota.
    Nota {
        #[arg(long)]
        tipo: String,
        #[arg(long)]
        titulo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum Exportar {
    /// Exporta a un archivo Markdown.
    Md(MdArgs),
    /// Exporta a un archivo DOCX.
    Docx(DocxArgs),
}

#[derive(Debug, Subcommand)]
enum CuarentenaAccion {
    /// Lista las copias en cuarentena.
    List {
        #[arg(default_value = ".")]
        ruta: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Descarta una copia de cuarentena sin tocar el original.
    Discard {
        /// Nombre del archivo dentro de .escritor/cuarentena.
        archivo: String,
        #[arg(default_value = ".")]
        ruta: PathBuf,
    },
}

#[derive(Debug, Args)]
struct MdArgs {
    /// Archivo de destino.
    destino: PathBuf,
    #[arg(default_value = ".")]
    ruta: PathBuf,
    /// Incluye las notas al final.
    #[arg(long)]
    notas: bool,
    /// Sobrescribe el destino.
    #[arg(long)]
    forzar: bool,
}

#[derive(Debug, Args)]
struct DocxArgs {
    /// Archivo de destino.
    destino: PathBuf,
    #[arg(default_value = ".")]
    ruta: PathBuf,
    /// Incluye las notas al final.
    #[arg(long)]
    notas: bool,
    /// Sobrescribe el destino.
    #[arg(long)]
    forzar: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match ejecutar(cli.comando) {
        Ok(codigo) => ExitCode::from(codigo),
        Err(error) => {
            eprintln!("writer: {error}");
            ExitCode::from(1)
        }
    }
}

fn ejecutar(comando: Comando) -> writer_core::Result<u8> {
    match comando {
        Comando::Init {
            ruta,
            titulo,
            autor,
            idioma,
        } => {
            let proyecto = Project::new(&ruta);
            proyecto.init(&titulo, Some(&autor), Some(&idioma))?;
            println!("Proyecto creado en {}", ruta.display());
            println!("Escena inicial: manuscrito/01-libro/01-capitulo/001-primera-escena.md");
            Ok(0)
        }
        Comando::Info { ruta, json } => info(&ruta, json),
        Comando::List { ruta, notas, json } => listar(&ruta, notas, json),
        Comando::Buscar {
            consulta,
            ruta,
            limite,
            json,
        } => buscar(&ruta, &consulta, limite, json),
        Comando::Read {
            objetivo,
            ruta,
            completo,
            json,
        } => leer(&ruta, &objetivo, completo, json),
        Comando::Create { tipo } => crear(tipo),
        Comando::Write {
            objetivo,
            ruta,
            texto,
            archivo,
            stdin,
        } => escribir(&ruta, &objetivo, texto, archivo, stdin),
        Comando::Move {
            objetivo,
            ruta,
            libro,
            capitulo,
            tipo,
        } => {
            let proyecto = Project::new(&ruta);
            let destino = proyecto.mover(
                &objetivo,
                libro.as_deref(),
                capitulo.as_deref(),
                tipo.as_deref(),
            )?;
            println!("{destino}");
            Ok(0)
        }
        Comando::Delete { objetivo, ruta } => {
            let proyecto = Project::new(&ruta);
            let eliminado = proyecto.eliminar(&objetivo)?;
            let revision = proyecto
                .historial()?
                .last()
                .map(|operacion| operacion.prepare.target_rev)
                .unwrap_or(0);
            println!("eliminado: {eliminado} (revisión {revision})");
            Ok(0)
        }
        Comando::History { ruta, json } => historial(&ruta, json),
        Comando::Show {
            revision,
            ruta,
            json,
        } => mostrar(&ruta, revision, json),
        Comando::Revert { revision, ruta } => {
            let proyecto = Project::new(&ruta);
            let resultado = proyecto.revertir(revision)?;
            println!(
                "revisión {revision} revertida en la revisión {}",
                resultado.revision
            );
            if resultado.errores_pendientes.is_empty() {
                Ok(0)
            } else {
                println!(
                    "aviso: quedan {} error(es) en el proyecto; ejecuta `writer verify`",
                    resultado.errores_pendientes.len()
                );
                for error in &resultado.errores_pendientes {
                    println!("  - {error}");
                }
                Ok(1)
            }
        }
        Comando::Verify {
            ruta,
            json,
            cuarentena,
        } => verificar(&ruta, json, cuarentena),
        Comando::Reindex { ruta, limpiar } => {
            let proyecto = Project::new(&ruta);
            let resultado = proyecto.reindexar(limpiar)?;
            println!(
                "índice reconstruido: {} escenas, {} notas, revisión {}",
                resultado.escenas, resultado.notas, resultado.revision
            );
            if limpiar {
                println!("blobs huérfanos eliminados: {}", resultado.blobs_eliminados);
            }
            Ok(0)
        }
        Comando::Quarantine { accion } => match accion {
            CuarentenaAccion::List { ruta, json } => cuarentena_listar(&ruta, json),
            CuarentenaAccion::Discard { archivo, ruta } => {
                Project::new(&ruta).descartar_cuarentena(&archivo)?;
                println!("descartado: {archivo}");
                Ok(0)
            }
        },
        Comando::Export { formato } => match formato {
            Exportar::Md(args) => {
                let proyecto = Project::new(&args.ruta);
                let destino = proyecto.exportar(&args.destino, args.notas, args.forzar)?;
                println!("exportado: {}", destino.display());
                Ok(0)
            }
            Exportar::Docx(args) => {
                let proyecto = Project::new(&args.ruta);
                let destino = proyecto.exportar_docx(&args.destino, args.notas, args.forzar)?;
                println!("exportado: {}", destino.display());
                Ok(0)
            }
        },
    }
}

fn info(ruta: &PathBuf, json: bool) -> writer_core::Result<u8> {
    let proyecto = Project::new(ruta);
    let sesion = proyecto.lectura()?;
    let escaneo = &sesion.escaneo;
    let revision = sesion
        .log
        .as_ref()
        .map(|log| log.revision_actual())
        .unwrap_or(0);
    if json {
        let valor = json!({
            "raiz": ruta,
            "titulo": escaneo.proyecto.as_ref().map(|proyecto| &proyecto.titulo),
            "id": escaneo.proyecto.as_ref().map(|proyecto| &proyecto.id),
            "esquema": escaneo.proyecto.as_ref().map(|proyecto| proyecto.esquema),
            "autor": escaneo.proyecto.as_ref().and_then(|proyecto| proyecto.autor.clone()),
            "revision": revision,
            "escenas": escaneo.escenas.len(),
            "notas": escaneo.notas.len(),
            "errores": escaneo.errores.len(),
            "avisos": escaneo.avisos.len(),
            "escritor": ruta.join(".escritor").exists(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).map_err(Error::Json)?
        );
    } else {
        println!("Proyecto: {}", ruta.display());
        if let Some(proyecto) = &escaneo.proyecto {
            println!("Título: {}", proyecto.titulo);
            println!("ID: {}", proyecto.id);
            println!("Esquema: {}", proyecto.esquema);
            if let Some(autor) = &proyecto.autor {
                println!("Autor: {autor}");
            }
        }
        println!("Revisión: {revision}");
        println!("Escenas: {}", escaneo.escenas.len());
        println!("Notas: {}", escaneo.notas.len());
        println!("Errores: {}", escaneo.errores.len());
        println!("Avisos: {}", escaneo.avisos.len());
        println!(
            "Índice: {}",
            if ruta.join(".escritor/indice.sqlite").exists() {
                "presente"
            } else {
                "sin crear"
            }
        );
    }
    Ok(0)
}

fn listar(ruta: &PathBuf, notas: bool, json: bool) -> writer_core::Result<u8> {
    let proyecto = Project::new(ruta);
    let documentos = proyecto.listar(notas)?;
    if json {
        let filas: Vec<_> = documentos
            .iter()
            .map(|documento| match documento {
                Documento::Escena(escena) => json!({
                    "tipo": "escena",
                    "id": escena.escena.id,
                    "ruta": escena.relativa,
                    "titulo": escena.escena.titulo,
                    "orden": escena.escena.orden,
                    "estado": escena.escena.estado,
                }),
                Documento::Nota(nota) => json!({
                    "tipo": "nota",
                    "id": nota.nota.id,
                    "ruta": nota.relativa,
                    "titulo": nota.nota.titulo,
                    "clase": nota.nota.tipo,
                }),
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&filas).map_err(Error::Json)?
        );
    } else {
        for documento in documentos {
            match documento {
                Documento::Escena(escena) => println!(
                    "{}  {:>5}  {:<9}  {}  {}",
                    escena.escena.id,
                    escena.escena.orden,
                    escena.escena.estado.as_deref().unwrap_or("borrador"),
                    escena.relativa,
                    escena.escena.titulo
                ),
                Documento::Nota(nota) => println!(
                    "{}  {:<10}  {}  {}",
                    nota.nota.id, nota.nota.tipo, nota.relativa, nota.nota.titulo
                ),
            }
        }
    }
    Ok(0)
}

fn buscar(ruta: &PathBuf, consulta: &str, limite: usize, json: bool) -> writer_core::Result<u8> {
    let resultados = Project::new(ruta).buscar(consulta, limite)?;
    if json {
        let filas: Vec<_> = resultados
            .iter()
            .map(|resultado| {
                json!({
                    "id": resultado.id,
                    "ruta": resultado.ruta,
                    "tipo": resultado.tipo,
                    "titulo": resultado.titulo,
                    "fragmento": resultado.fragmento,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&filas).map_err(Error::Json)?
        );
    } else if resultados.is_empty() {
        println!("sin resultados");
    } else {
        for resultado in resultados {
            println!(
                "{}  {}  {}\n  {}",
                resultado.tipo, resultado.titulo, resultado.ruta, resultado.fragmento
            );
        }
    }
    Ok(0)
}

fn leer(ruta: &PathBuf, objetivo: &str, completo: bool, json: bool) -> writer_core::Result<u8> {
    let proyecto = Project::new(ruta);
    let documento = proyecto.leer(objetivo)?;
    if json {
        let valor = match &documento {
            Documento::Escena(escena) => json!({
                "tipo": "escena",
                "id": escena.escena.id,
                "ruta": escena.relativa,
                "titulo": escena.escena.titulo,
                "orden": escena.escena.orden,
                "estado": escena.escena.estado,
                "cuerpo": escena.escena.cuerpo,
                "hash": escena.hash,
            }),
            Documento::Nota(nota) => json!({
                "tipo": "nota",
                "id": nota.nota.id,
                "ruta": nota.relativa,
                "titulo": nota.nota.titulo,
                "clase": nota.nota.tipo,
                "cuerpo": nota.nota.cuerpo,
                "hash": nota.hash,
            }),
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).map_err(Error::Json)?
        );
    } else if completo {
        let contenido = std::fs::read_to_string(documento.ruta())
            .map_err(|error| Error::Io(error).with_path(documento.ruta()))?;
        print!("{contenido}");
    } else {
        print!("{}", documento.cuerpo());
        if !documento.cuerpo().ends_with('\n') {
            println!();
        }
    }
    Ok(0)
}

fn crear(tipo: Crear) -> writer_core::Result<u8> {
    match tipo {
        Crear::Escena {
            titulo,
            ruta,
            libro,
            capitulo,
            orden,
        } => {
            let proyecto = Project::new(&ruta);
            let (id, ruta_escena) =
                proyecto.crear_escena(&titulo, libro.as_deref(), capitulo.as_deref(), orden)?;
            println!("{id}  {ruta_escena}");
        }
        Crear::Nota { tipo, titulo, ruta } => {
            let proyecto = Project::new(&ruta);
            let (id, ruta_nota) = proyecto.crear_nota(&tipo, &titulo)?;
            println!("{id}  {ruta_nota}");
        }
    }
    Ok(0)
}

fn escribir(
    ruta: &PathBuf,
    objetivo: &str,
    texto: Option<String>,
    archivo: Option<PathBuf>,
    stdin: bool,
) -> writer_core::Result<u8> {
    let entradas =
        usize::from(texto.is_some()) + usize::from(archivo.is_some()) + usize::from(stdin);
    if entradas != 1 {
        return Err(Error::Input(
            "indica exactamente una fuente: --texto, --archivo o --stdin".to_string(),
        ));
    }
    let cuerpo = if let Some(texto) = texto {
        texto
    } else if let Some(archivo) = archivo {
        std::fs::read_to_string(&archivo).map_err(|error| Error::Io(error).with_path(&archivo))?
    } else {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(Error::Io)?;
        buffer
    };
    let proyecto = Project::new(ruta);
    let revision = proyecto.escribir_cuerpo(objetivo, &cuerpo)?;
    println!("{objetivo}  revisión {revision}");
    Ok(0)
}

fn historial(ruta: &PathBuf, json: bool) -> writer_core::Result<u8> {
    let proyecto = Project::new(ruta);
    let operaciones = proyecto.historial()?;
    if json {
        let filas: Vec<_> = operaciones
            .iter()
            .map(|operacion| {
                json!({
                    "rev": operacion.prepare.target_rev,
                    "op_id": operacion.prepare.op_id,
                    "ts": operacion.prepare.ts,
                    "autor": operacion.prepare.autor,
                    "scope": operacion.prepare.scope,
                    "efectos": operacion.prepare.efectos.len(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&filas).map_err(Error::Json)?
        );
    } else if operaciones.is_empty() {
        println!("sin revisiones");
    } else {
        for operacion in operaciones {
            println!(
                "{:>4}  {}  {:<7} {:<12} {} efecto(s)  {}",
                operacion.prepare.target_rev,
                operacion.prepare.ts,
                operacion.prepare.autor,
                operacion.prepare.scope,
                operacion.prepare.efectos.len(),
                operacion.prepare.op_id
            );
        }
    }
    Ok(0)
}

fn mostrar(ruta: &PathBuf, revision: u64, json: bool) -> writer_core::Result<u8> {
    let proyecto = Project::new(ruta);
    let detalle = proyecto.mostrar(revision)?;
    if json {
        let efectos: Vec<_> = detalle
            .contenidos
            .iter()
            .map(|(ruta, contenido)| {
                json!({
                    "ruta": ruta,
                    "contenido": contenido.as_ref().map(|bytes| String::from_utf8_lossy(bytes).to_string()),
                })
            })
            .collect();
        let valor = json!({
            "rev": detalle.operacion.prepare.target_rev,
            "op_id": detalle.operacion.prepare.op_id,
            "ts": detalle.operacion.prepare.ts,
            "autor": detalle.operacion.prepare.autor,
            "scope": detalle.operacion.prepare.scope,
            "efectos": efectos,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).map_err(Error::Json)?
        );
    } else {
        println!(
            "revisión {} · {} · {} · {}",
            detalle.operacion.prepare.target_rev,
            detalle.operacion.prepare.ts,
            detalle.operacion.prepare.autor,
            detalle.operacion.prepare.scope
        );
        for (ruta, contenido) in detalle.contenidos {
            println!("\n--- {ruta} ---");
            match contenido {
                Some(bytes) => print!("{}", String::from_utf8_lossy(&bytes)),
                None => println!("(eliminado)"),
            }
        }
    }
    Ok(0)
}

fn cuarentena_listar(ruta: &PathBuf, json: bool) -> writer_core::Result<u8> {
    let items = Project::new(ruta).cuarentena()?;
    if json {
        let filas: Vec<_> = items
            .iter()
            .map(|item| {
                json!({
                    "archivo": item.archivo,
                    "ruta_original": item.ruta_original,
                    "motivo": item.motivo,
                    "ts": item.ts,
                    "tamano": item.tamano,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&filas).map_err(Error::Json)?
        );
    } else if items.is_empty() {
        println!("sin archivos en cuarentena");
    } else {
        for item in items {
            println!(
                "{}  {} bytes  {}  {}  {}",
                item.archivo,
                item.tamano,
                item.ruta_original.as_deref().unwrap_or("(desconocido)"),
                item.motivo.as_deref().unwrap_or("(sin motivo)"),
                item.ts.as_deref().unwrap_or("(sin fecha)")
            );
        }
    }
    Ok(0)
}

fn verificar(ruta: &PathBuf, json: bool, cuarentena: bool) -> writer_core::Result<u8> {
    let proyecto = Project::new(ruta);
    let escaneo = proyecto.verificar()?;
    if cuarentena {
        let copiados = verify::cuarentenar(proyecto.raiz(), &escaneo)?;
        if !json {
            println!("cuarentena: {copiados} archivo(s) copiado(s)");
        }
    }
    imprimir_escaneo(&escaneo, json)?;
    Ok(u8::from(!escaneo.errores.is_empty()))
}

fn imprimir_escaneo(escaneo: &Escaneo, json: bool) -> writer_core::Result<()> {
    if json {
        let valor = json!({
            "escenas": escaneo.escenas.len(),
            "notas": escaneo.notas.len(),
            "errores": escaneo.errores,
            "avisos": escaneo.avisos,
            "invalidos": escaneo.invalidos.iter().map(|invalido| json!({
                "ruta": invalido.relativa,
                "motivo": invalido.motivo,
            })).collect::<Vec<_>>(),
            "proyecto": escaneo.proyecto.as_ref().map(|proyecto| json!({
                "id": proyecto.id,
                "titulo": proyecto.titulo,
            })),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).map_err(Error::Json)?
        );
    } else {
        println!(
            "escenas: {} · notas: {} · errores: {} · avisos: {}",
            escaneo.escenas.len(),
            escaneo.notas.len(),
            escaneo.errores.len(),
            escaneo.avisos.len()
        );
        for aviso in &escaneo.avisos {
            println!("aviso: {aviso}");
        }
        for error in &escaneo.errores {
            println!("ERROR: {error}");
        }
        println!(
            "resultado: {}",
            if escaneo.errores.is_empty() {
                "OK".to_string()
            } else {
                format!("{} error(es)", escaneo.errores.len())
            }
        );
    }
    Ok(())
}
