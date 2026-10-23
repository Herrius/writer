//! Pruebas de integración del motor sobre proyectos temporales.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use writer_core::model::{Escena, Nota};
use writer_core::project::{Cambio, Project};
use writer_core::verify;
use writer_core::Error;

fn temporal() -> tempfile::TempDir {
    tempfile::tempdir().expect("directorio temporal")
}

fn fixture(nombre: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(nombre)
}

#[test]
fn init_crea_proyecto_verificable() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto
        .init("El umbral", Some("Autora"), Some("es"))
        .expect("init");
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(escaneo.errores.is_empty(), "errores: {:?}", escaneo.errores);
    assert_eq!(escaneo.escenas.len(), 1);
    assert_eq!(escaneo.notas.len(), 0);
    assert_eq!(proyecto.historial().expect("historial").len(), 1);
}

#[test]
fn crear_escribir_mover_y_revertir() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");

    let (id, ruta) = proyecto
        .crear_escena("El faro", None, None, Some(20))
        .expect("crear escena");
    assert_eq!(ruta, "manuscrito/01-libro/01-capitulo/002-el-faro.md");
    let revision = proyecto
        .escribir_cuerpo(&id, "Primera versión.\n")
        .expect("escribir");
    assert_eq!(revision, 3);
    let documento = proyecto.leer(&id).expect("leer");
    assert_eq!(documento.cuerpo(), "Primera versión.\n");

    let (nota_id, _) = proyecto
        .crear_nota("personajes", "Marta")
        .expect("crear nota");
    let nota = proyecto.leer(&nota_id).expect("leer nota");
    assert_eq!(nota.titulo(), "Marta");

    let nueva_ruta = proyecto
        .mover(&id, Some("02-libro"), Some("01-capitulo"), None)
        .expect("mover");
    assert_eq!(nueva_ruta, "manuscrito/02-libro/01-capitulo/002-el-faro.md");
    let historial = proyecto.historial().expect("historial");
    assert_eq!(historial.len(), 5);
    let revision_movimiento = historial.last().expect("última").prepare.target_rev;

    let resultado = proyecto
        .revertir(revision_movimiento)
        .expect("revertir movimiento");
    assert_eq!(resultado.revision, revision_movimiento + 1);
    assert!(resultado.errores_pendientes.is_empty());
    assert!(proyecto
        .leer("manuscrito/02-libro/01-capitulo/002-el-faro.md")
        .is_err());
    let restaurada = proyecto.leer(&id).expect("leer restaurada");
    assert_eq!(restaurada.cuerpo(), "Primera versión.\n");
}

#[test]
fn renombrar_y_duplicar_documentos() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");

    let (id, ruta) = proyecto
        .crear_escena("Original", None, None, Some(20))
        .expect("crear escena");
    proyecto
        .escribir_cuerpo(&id, "Texto de la escena.\n")
        .expect("escribir");

    let renombrada = proyecto.renombrar(&id, "Renombrada").expect("renombrar");
    assert_eq!(renombrada, ruta);
    assert_eq!(proyecto.leer(&id).expect("leer").titulo(), "Renombrada");

    proyecto
        .actualizar_escena(
            &id,
            Some("revision"),
            &["clave".to_string(), "clave".to_string()],
        )
        .expect("actualizar escena");
    match proyecto.leer(&id).expect("leer metadatos") {
        verify::Documento::Escena(encontrada) => {
            assert_eq!(encontrada.escena.estado.as_deref(), Some("revision"));
            assert_eq!(encontrada.escena.etiquetas, Some(vec!["clave".to_string()]));
        }
        _ => panic!("se esperaba una escena"),
    }

    let (copia_id, copia_ruta) = proyecto.duplicar(&id).expect("duplicar escena");
    assert_ne!(copia_id, id);
    assert!(copia_ruta.contains("copia-de-renombrada"), "{copia_ruta}");
    let copia = proyecto.leer(&copia_id).expect("leer copia");
    assert_eq!(copia.titulo(), "Copia de Renombrada");
    assert_eq!(copia.cuerpo(), "Texto de la escena.\n");

    let (nota_id, _) = proyecto
        .crear_nota("personajes", "Marta")
        .expect("crear nota");
    let (nota_copia_id, nota_copia_ruta) = proyecto.duplicar(&nota_id).expect("duplicar nota");
    assert_ne!(nota_copia_id, nota_id);
    assert!(
        nota_copia_ruta.contains("copia-de-marta"),
        "{nota_copia_ruta}"
    );
    assert_eq!(
        proyecto.leer(&nota_copia_id).expect("leer nota").titulo(),
        "Copia de Marta"
    );

    let escaneo = proyecto.verificar().expect("verificar");
    assert!(escaneo.errores.is_empty(), "errores: {:?}", escaneo.errores);
}

#[test]
fn intercambiar_orden_de_escenas() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (a, _) = proyecto
        .crear_escena("A", None, None, Some(20))
        .expect("crear A");
    let (b, _) = proyecto
        .crear_escena("B", None, None, Some(30))
        .expect("crear B");
    proyecto.intercambiar_orden(&a, &b).expect("intercambiar");
    let documentos = proyecto.listar(false).expect("listar");
    let orden_a = documentos
        .iter()
        .find(|documento| documento.id() == a)
        .and_then(|documento| match documento {
            verify::Documento::Escena(escena) => Some(escena.escena.orden),
            _ => None,
        });
    let orden_b = documentos
        .iter()
        .find(|documento| documento.id() == b)
        .and_then(|documento| match documento {
            verify::Documento::Escena(escena) => Some(escena.escena.orden),
            _ => None,
        });
    assert_eq!(orden_a, Some(30));
    assert_eq!(orden_b, Some(20));
}

#[test]
fn exportar_pdf_genera_documento_local() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto
        .init("Novela", Some("Autora"), Some("es"))
        .expect("init");
    let destino = directorio.path().join("salida.pdf");
    let exportado = proyecto
        .exportar_pdf(&destino, false)
        .expect("exportar PDF");
    let bytes = std::fs::read(exportado).expect("leer PDF");
    assert!(bytes.starts_with(b"%PDF-1.4"));
    assert!(bytes.windows(5).any(|ventana| ventana == b"%%EOF"));
}

#[test]
fn escritura_con_hash_base_detecta_cambios_externos() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let escaneo = proyecto.verificar().expect("verificar");
    let id = escaneo.escenas[0].escena.id.clone();
    let inicial = proyecto.leer(&id).expect("leer inicial");
    let hash_inicial = inicial.hash().to_string();

    let resultado = proyecto
        .escribir_cuerpo_con_base(&id, "Primera versión.\n", Some(&hash_inicial))
        .expect("escribir con hash vigente");
    assert_eq!(resultado.revision, 2);
    let actual = proyecto.leer(&id).expect("leer actual");
    assert_eq!(actual.cuerpo(), "Primera versión.\n");
    assert_eq!(actual.hash(), resultado.hash_archivo);

    let error = proyecto
        .escribir_cuerpo_con_base(&id, "Cambio con hash viejo.\n", Some(&hash_inicial))
        .expect_err("el hash inicial ya no sirve");
    assert!(matches!(error, Error::Conflict(_)));
    let conservado = proyecto.leer(&id).expect("leer conservado");
    assert_eq!(conservado.cuerpo(), "Primera versión.\n");
}

#[test]
fn escribir_cuerpo_preserva_frontmatter() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let escaneo = proyecto.verificar().expect("verificar");
    let encontrada = &escaneo.escenas[0];
    let original = std::fs::read_to_string(&encontrada.ruta).expect("leer original");
    let cierre = original[4..]
        .find("\n---\n")
        .map(|indice| indice + 4 + 5)
        .expect("cierre de frontmatter");
    let cabecera = original[..cierre].to_string();
    let id = encontrada.escena.id.clone();

    proyecto
        .escribir_cuerpo(&id, "Cuerpo nuevo.\n")
        .expect("escribir cuerpo");

    let nuevo = std::fs::read_to_string(&encontrada.ruta).expect("leer nuevo");
    assert!(nuevo.starts_with(&cabecera), "el frontmatter cambió");
    assert!(nuevo.ends_with("Cuerpo nuevo.\n"));
}

#[test]
fn eliminar_se_puede_revertir() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (id, _) = proyecto
        .crear_escena("Borrable", None, None, None)
        .expect("crear");
    let _ruta = proyecto.eliminar(&id).expect("eliminar");
    assert!(proyecto.leer(&id).is_err());
    let revision_borrado = proyecto
        .historial()
        .expect("historial")
        .last()
        .expect("última revisión")
        .prepare
        .target_rev;
    let resultado = proyecto.revertir(revision_borrado).expect("revertir");
    assert!(resultado.revision > revision_borrado);
    assert!(resultado.errores_pendientes.is_empty());
    assert_eq!(proyecto.leer(&id).expect("leer").titulo(), "Borrable");
}

#[test]
fn revertir_nota_borrada_repara_referencias() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (marta, _) = proyecto
        .crear_nota("personajes", "Marta")
        .expect("crear nota");

    let mut escena = Escena::nueva("Con Marta", 20).expect("escena");
    escena.personajes = Some(vec![marta.clone()]);
    let mut sesion = proyecto.escritura().expect("sesion");
    proyecto
        .commit_efectos(
            &mut sesion,
            vec![Cambio::nuevo(
                "manuscrito/02-libro/01-capitulo/001-con-marta.md",
                escena.a_bytes(),
            )],
            "humano",
            "estructura",
            None,
            None,
        )
        .expect("commit escena");
    drop(sesion);
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(
        escaneo.errores.is_empty(),
        "errores tras crear la escena: {:#?}",
        escaneo.errores
    );

    proyecto.eliminar(&marta).expect("eliminar nota");
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(
        escaneo
            .errores
            .iter()
            .any(|error| error.contains("id inexistente")),
        "errores: {:#?}",
        escaneo.errores
    );

    let revision = proyecto
        .historial()
        .expect("historial")
        .last()
        .expect("última")
        .prepare
        .target_rev;
    let resultado = proyecto.revertir(revision).expect("revertir");
    assert!(resultado.errores_pendientes.is_empty());
    assert_eq!(proyecto.leer(&marta).expect("leer nota").titulo(), "Marta");
    assert!(proyecto.verificar().expect("verificar").errores.is_empty());
}

#[test]
fn exportar_markdown_ordena_y_no_toca_el_proyecto() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    proyecto
        .crear_escena("Segunda", None, None, Some(20))
        .expect("crear");
    let destino = directorio.path().join("salida.md");
    proyecto.exportar(&destino, true, false).expect("exportar");
    let texto = std::fs::read_to_string(&destino).expect("leer");
    assert!(texto.contains("# Libro"));
    assert!(texto.contains("## Capitulo"));
    assert!(texto.contains("### Primera escena"));
    assert!(texto.contains("### Segunda"));
    assert!(texto.contains("\n***\n"));
    assert!(proyecto.exportar(&destino, true, false).is_err());
}

#[test]
fn indice_busca_y_se_reconstruye() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (id, _) = proyecto
        .crear_escena("El faro", None, None, None)
        .expect("crear");
    proyecto
        .escribir_cuerpo(&id, "Una canción sobre el puerto.\n")
        .expect("escribir");
    let sesion = proyecto.escritura().expect("sesion");
    let resultados = sesion.indice.buscar("puerto", 10).expect("buscar");
    assert_eq!(resultados.len(), 1);
    assert_eq!(resultados[0].0, id);
    drop(sesion);

    let resultados = proyecto.buscar("puerto", 10).expect("buscar en proyecto");
    assert_eq!(resultados.len(), 1);
    assert_eq!(resultados[0].id, id);
    assert_eq!(resultados[0].tipo, "escena");
    assert!(resultados[0].fragmento.contains("\u{1}puerto\u{2}"));
    assert_eq!(
        proyecto.buscar("cancion", 10).expect("sin diacríticos")[0].id,
        id
    );
    assert!(proyecto.buscar("   ", 10).is_err());
    assert!(proyecto.buscar("***", 10).is_err());
    assert!(proyecto.buscar("\"puerto", 10).is_ok());

    let (nota_id, _) = proyecto
        .crear_nota("personajes", "Marta")
        .expect("crear nota");
    proyecto
        .escribir_cuerpo(&nota_id, "Marta vuelve al puerto.\n")
        .expect("escribir nota");
    let notas = proyecto.buscar("Marta", 10).expect("buscar nota");
    assert_eq!(notas.len(), 1);
    assert_eq!(notas[0].id, nota_id);
    assert_eq!(notas[0].tipo, "nota");

    let resultado = proyecto.reindexar(true).expect("reindexar");
    assert_eq!(resultado.escenas, 2);
    assert_eq!(resultado.blobs_eliminados, 0);
}

#[test]
fn init_rechaza_rutas_invalidas_y_destinos_no_vacios() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    Project::new(&raiz)
        .init("Novela", None, None)
        .expect("init");

    for ruta in [
        PathBuf::from(""),
        PathBuf::from("."),
        PathBuf::from(".."),
        raiz.join("otra").join(".."),
        directorio
            .path()
            .join("no-existe")
            .join("..")
            .join("novela"),
        raiz.clone(),
    ] {
        assert!(
            Project::new(&ruta).init("Otra", None, None).is_err(),
            "debió rechazar {ruta:?}"
        );
    }
    assert!(std::fs::read_to_string(raiz.join("proyecto.yml"))
        .expect("manifiesto intacto")
        .contains("Novela"));

    let no_vacio = directorio.path().join("no-vacio");
    std::fs::create_dir(&no_vacio).expect("no-vacio");
    std::fs::write(no_vacio.join("dato.txt"), b"no tocar").expect("dato");
    assert!(Project::new(&no_vacio).init("Otra", None, None).is_err());
    assert_eq!(
        std::fs::read(no_vacio.join("dato.txt")).expect("dato intacto"),
        b"no tocar"
    );

    let anidado = raiz.join("manuscrito").join("hijo");
    let error = Project::new(&anidado)
        .init("Anidado", None, None)
        .expect_err("proyecto dentro de otro");
    assert!(error.to_string().contains("dentro de otro proyecto"));
    assert!(!anidado.exists());

    let invalido = directorio.path().join("titulo-invalido");
    assert!(Project::new(&invalido).init("   ", None, None).is_err());
    assert!(!invalido.exists(), "no debe quedar una carpeta vacía");
}

#[cfg(unix)]
#[test]
fn init_rechaza_un_destino_enlazado() {
    use std::os::unix::fs::symlink;

    let directorio = temporal();
    let fuera = directorio.path().join("fuera");
    std::fs::create_dir(&fuera).expect("fuera");
    let enlace = directorio.path().join("novela");
    symlink(&fuera, &enlace).expect("enlace");

    let error = Project::new(&enlace)
        .init("Novela", None, None)
        .expect_err("destino enlazado");
    assert!(error.to_string().contains("enlace"));
    assert!(std::fs::read_dir(&fuera).expect("fuera").next().is_none());
}

#[cfg(unix)]
#[test]
fn recuperacion_no_escribe_fuera_del_proyecto_por_un_enlace() {
    use std::os::unix::fs::symlink;
    use writer_core::blob::BlobStore;
    use writer_core::oplog::{Efecto, Log, PrepareInput};

    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");

    let fuera = directorio.path().join("fuera");
    std::fs::create_dir_all(&fuera).expect("fuera");
    let ancestro = raiz.join("manuscrito/01-libro/01-capitulo/extra");
    symlink(&fuera, &ancestro).expect("enlace de ancestro");

    let escritor = raiz.join(".escritor");
    let bytes = b"contenido externo";
    let hash_anterior = writer_core::hash::hash(bytes);
    let blobs = BlobStore::abrir(&escritor).expect("blobs");
    blobs.poner(bytes).expect("blob");
    let mut log = Log::cargar(&escritor.join("operaciones.log")).expect("log");
    let op_id = writer_core::ulid::nuevo().expect("ulid");
    log.agregar_prepare(PrepareInput {
        op_id: &op_id,
        base_rev: 1,
        autor: "humano",
        scope: "test",
        task_id: None,
        modelo: None,
        efectos: vec![Efecto {
            ruta: "manuscrito/01-libro/01-capitulo/extra/pwned.txt".to_string(),
            hash_nuevo: None,
            hash_anterior: Some(hash_anterior),
        }],
    })
    .expect("prepare");
    log.sincronizar().expect("sincronizar");

    let escaneo = proyecto.verificar().expect("verificar");
    assert!(
        escaneo.errores.iter().any(|error| error.contains("enlace")),
        "errores: {:#?}",
        escaneo.errores
    );
    assert!(!fuera.join("pwned.txt").exists());
}

#[cfg(unix)]
#[test]
fn reindex_no_toca_el_indice_externo_con_un_proyecto_corrupto() {
    use std::os::unix::fs::symlink;

    let directorio = temporal();
    let raiz = directorio.path().join("novela-corrupta");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");

    let escritor_externo = directorio.path().join("escritor-real");
    std::fs::rename(raiz.join(".escritor"), &escritor_externo).expect("mover .escritor");
    let indice = escritor_externo.join("indice.sqlite");
    let wal = escritor_externo.join("indice.sqlite-wal");
    let shm = escritor_externo.join("indice.sqlite-shm");
    std::fs::write(&indice, b"INDICE-EXTERNO").expect("indice externo");
    std::fs::write(&wal, b"WAL-EXTERNO").expect("wal externo");
    std::fs::write(&shm, b"SHM-EXTERNO").expect("shm externo");
    symlink(&escritor_externo, raiz.join(".escritor")).expect("enlace .escritor");

    let error = proyecto
        .reindexar(false)
        .expect_err("reindex debe rechazar .escritor enlazado");
    assert!(error.to_string().contains(".escritor"));
    assert_eq!(
        std::fs::read(&indice).expect("indice intacto"),
        b"INDICE-EXTERNO"
    );
    assert_eq!(std::fs::read(&wal).expect("wal intacto"), b"WAL-EXTERNO");
    assert_eq!(std::fs::read(&shm).expect("shm intacto"), b"SHM-EXTERNO");
    let corruptos: Vec<_> = std::fs::read_dir(&escritor_externo)
        .expect("leer escritor")
        .filter_map(Result::ok)
        .filter(|entrada| entrada.file_name().to_string_lossy().contains("corrupto-"))
        .collect();
    assert!(corruptos.is_empty(), "no debe apartar el índice externo");
}

#[cfg(unix)]
#[test]
fn rechaza_enlaces_duros_en_rutas_operativas() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");

    let victima = directorio.path().join("victima");
    std::fs::write(&victima, b"SENTINELA").expect("victima");
    let bloqueo = raiz.join(".escritor/bloqueo");
    std::fs::remove_file(&bloqueo).expect("quitar lock");
    std::fs::hard_link(&victima, &bloqueo).expect("enlace duro");

    let error = proyecto
        .lectura()
        .expect_err("lectura debe rechazar el enlace duro");
    assert!(error.to_string().contains("enlaces duros"));
    assert_eq!(
        std::fs::read(&victima).expect("victima intacta"),
        b"SENTINELA"
    );
}

#[cfg(unix)]
#[test]
fn rechaza_un_fifo_en_una_ruta_de_efecto() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");
    let fifo = raiz.join("manuscrito/01-libro/01-capitulo/fifo.md");
    let estado = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("mkfifo");
    assert!(estado.success());

    let error =
        writer_core::fsutil::ruta_del_proyecto(&raiz, "manuscrito/01-libro/01-capitulo/fifo.md")
            .expect_err("un FIFO no es un archivo regular");
    assert!(error.to_string().contains("archivo regular"));
}

#[cfg(unix)]
#[test]
fn rechaza_enlaces_en_rutas_de_proyecto_y_escritor() {
    use std::os::unix::fs::symlink;

    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");

    let fuera = directorio.path().join("fuera");
    std::fs::create_dir_all(&fuera).expect("fuera");
    let ancestro = raiz.join("manuscrito/01-libro/01-capitulo/extra");
    symlink(&fuera, &ancestro).expect("enlace de ancestro");

    let error = writer_core::fsutil::ruta_del_proyecto(
        &raiz,
        "manuscrito/01-libro/01-capitulo/extra/pwned.txt",
    )
    .expect_err("la ruta atraviesa un enlace");
    assert!(error.to_string().contains("enlace"));

    let escritor = raiz.join(".escritor");
    let escritor_externo = directorio.path().join("escritor-externo");
    std::fs::rename(&escritor, &escritor_externo).expect("mover .escritor");
    symlink(&escritor_externo, &escritor).expect("enlace de .escritor");

    let error = proyecto
        .lectura()
        .expect_err("lectura debe rechazar .escritor");
    assert!(error.to_string().contains(".escritor"));
    let error = proyecto
        .reindexar(false)
        .expect_err("reindexar debe rechazar .escritor");
    assert!(error.to_string().contains(".escritor"));

    // El archivo externo que quedó dentro del .escritor real no se toca.
    assert!(escritor_externo.join("operaciones.log").is_file());
}

#[test]
fn buscar_reindexa_tras_una_edicion_externa() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let escaneo = proyecto.verificar().expect("verificar");
    let ruta = escaneo.escenas[0].ruta.clone();
    proyecto
        .escribir_cuerpo(&escaneo.escenas[0].escena.id, "contenido alfa original\n")
        .expect("escribir");

    let original = std::fs::read_to_string(&ruta).expect("leer archivo");
    assert!(original.contains("contenido alfa original"));
    std::fs::write(
        &ruta,
        original.replace("contenido alfa original", "contenido omega externo"),
    )
    .expect("editar por fuera");

    let omega = proyecto.buscar("omega", 10).expect("buscar omega");
    assert_eq!(
        omega.len(),
        1,
        "el índice debe reindexar el archivo cambiado"
    );
    assert!(proyecto.buscar("alfa", 10).expect("buscar alfa").is_empty());
}

#[test]
fn ids_duplicados_no_rompen_busqueda_reindexar_ni_revertir() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (id, _ruta) = proyecto
        .crear_escena("Segunda", None, None, Some(20))
        .expect("crear segunda");
    proyecto
        .escribir_cuerpo(&id, "Una escena distinta.\n")
        .expect("escribir");
    let escaneo = proyecto.verificar().expect("verificar");
    let copia = proyecto
        .raiz()
        .join("manuscrito/01-libro/01-capitulo/003-duplicada.md");
    let original = escaneo
        .escenas
        .iter()
        .find(|escena| escena.escena.id == id)
        .expect("escena original")
        .ruta
        .clone();
    std::fs::copy(&original, &copia).expect("copiar con el mismo id");

    let escaneo = proyecto.verificar().expect("verificar duplicado");
    assert!(
        escaneo
            .errores
            .iter()
            .any(|error| error.contains("duplicado")),
        "errores: {:#?}",
        escaneo.errores
    );

    let _ = proyecto
        .buscar("escena", 10)
        .expect("buscar con duplicados no debe fallar");
    let reindexado = proyecto
        .reindexar(false)
        .expect("reindexar con duplicados no debe fallar");
    assert_eq!(
        reindexado.escenas, 1,
        "solo la escena sin duplicar entra en el índice"
    );
    proyecto
        .revertir(1)
        .expect("revertir con duplicados no debe fallar por UNIQUE");

    // La copia duplicada sigue en disco (la cuarentena copia, no mueve), pero
    // queda fuera del índice y del árbol.
    assert!(copia.exists());
}

#[test]
fn buscar_en_fixture_sin_escritor_no_crea_directorio() {
    let raiz = fixture("proyecto-pequeno");
    let proyecto = Project::new(&raiz);
    assert!(!raiz.join(".escritor").exists());
    let resultados = proyecto.buscar("marea", 5).expect("buscar en memoria");
    assert!(!resultados.is_empty());
    assert!(resultados
        .iter()
        .all(|resultado| resultado.tipo == "escena"));
    assert!(resultados
        .iter()
        .any(|resultado| resultado.fragmento.contains("\u{1}marea\u{2}")));
    assert!(!proyecto
        .buscar("lucia", 5)
        .expect("sin diacríticos")
        .is_empty());
    assert!(!proyecto
        .buscar("marea molino", 5)
        .expect("todos los términos")
        .is_empty());
    assert!(proyecto
        .buscar("marea termino-que-no-existe", 5)
        .expect("término ausente")
        .is_empty());
    assert!(!raiz.join(".escritor").exists());
}

#[test]
fn buscar_fallback_coincide_con_fts_en_unicode_y_guiones() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let capitulo = raiz.join("manuscrito/01-libro/01-capitulo");
    std::fs::create_dir_all(&capitulo).expect("crear capítulo");
    std::fs::copy(
        fixture("proyecto-pequeno").join("proyecto.yml"),
        raiz.join("proyecto.yml"),
    )
    .expect("copiar manifiesto");

    let escenas = [
        (
            "001-descompuesto.md",
            "01KDVDNXH0HP47XVCD2KR6TFZF",
            "Descompuesto",
            10,
            "Una cancio\u{301}n del puerto.\n",
        ),
        (
            "002-guion.md",
            "01KDVDNXH0HP47XVCD2KR6TFZG",
            "Con guion",
            20,
            "El Puerto-Viejo al norte.\n",
        ),
        (
            "003-junto.md",
            "01KDVDNXH0HP47XVCD2KR6TFZH",
            "Junto",
            30,
            "El puertoviejo al sur.\n",
        ),
    ];
    for (archivo, id, titulo, orden, cuerpo) in escenas {
        std::fs::write(
            capitulo.join(archivo),
            format!("---\nid: {id}\ntitulo: \"{titulo}\"\norden: {orden}\nestado: borrador\n---\n\n{cuerpo}"),
        )
        .expect("escribir escena");
    }

    let proyecto = Project::new(&raiz);
    assert!(!raiz.join(".escritor").exists());

    let descompuesto = proyecto.buscar("cancion", 10).expect("buscar NFD");
    assert_eq!(descompuesto.len(), 1);
    assert_eq!(descompuesto[0].id, "01KDVDNXH0HP47XVCD2KR6TFZF");

    let con_guion = proyecto
        .buscar("Puerto-Viejo", 10)
        .expect("buscar con guion");
    assert_eq!(con_guion.len(), 1);
    assert_eq!(con_guion[0].id, "01KDVDNXH0HP47XVCD2KR6TFZG");
    assert!(con_guion[0].fragmento.contains("\u{1}Puerto-Viejo\u{2}"));

    let junto = proyecto
        .buscar("puertoviejo", 10)
        .expect("buscar palabra unida");
    assert_eq!(junto.len(), 1);
    assert_eq!(junto[0].id, "01KDVDNXH0HP47XVCD2KR6TFZH");
    assert!(!raiz.join(".escritor").exists());
}

#[test]
fn exportar_docx_incluye_manuscrito_y_notas() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto
        .init("Novela DOCX", Some("Autora"), None)
        .expect("init");
    let (id, _) = proyecto
        .crear_escena("El faro", None, None, None)
        .expect("crear escena");
    proyecto
        .escribir_cuerpo(
            &id,
            "# Un faro\n\nUna **luz** sobre el puerto, Tom & Jerry < 3. [[personaje:01KDVDNBYG20KW71NVRK6P2FHG|Marta]] y [[comentario:revisar|una frase]].\n\n- Uno\n- Dos\n\n> Una cita.\n\n```text\ncódigo\n```\n\n[Enlace](https://example.com)\n",
        )
        .expect("escribir");
    let (nota_id, _) = proyecto
        .crear_nota("personajes", "Marta")
        .expect("crear nota");
    proyecto
        .escribir_cuerpo(&nota_id, "Marta vuelve al puerto.\n")
        .expect("escribir nota");

    let destino = directorio.path().join("salida.docx");
    let exportado = proyecto
        .exportar_docx(&destino, true, false)
        .expect("exportar DOCX");
    let bytes = std::fs::read(&exportado).expect("leer DOCX");
    assert_eq!(&bytes[..4], b"PK\x03\x04", "no parece un ZIP/DOCX");

    let mut archivo =
        zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("abrir DOCX como ZIP");
    let mut documento = String::new();
    archivo
        .by_name("word/document.xml")
        .expect("document.xml")
        .read_to_string(&mut documento)
        .expect("leer document.xml");
    let mut relaciones = String::new();
    archivo
        .by_name("word/_rels/document.xml.rels")
        .expect("rels")
        .read_to_string(&mut relaciones)
        .expect("leer rels");
    for texto in [
        "Novela DOCX",
        "Autora",
        "El faro",
        "Una ",
        "luz",
        "sobre el puerto",
        "Uno",
        "Dos",
        "Una cita",
        "código",
        "Enlace",
        "Notas",
        "Marta",
        "una frase",
    ] {
        assert!(
            documento.contains(texto),
            "falta {texto:?} en el DOCX:\n{documento}"
        );
    }
    assert!(!documento.contains("[[personaje"));
    assert!(!documento.contains("[[comentario"));
    assert!(relaciones.contains("https://example.com"));
    assert!(documento.contains("Tom &amp; Jerry"));
    assert!(documento.contains("&lt;"));
    assert!(proyecto.exportar_docx(&destino, false, false).is_err());
}

#[test]
fn exportar_docx_rechaza_caracteres_de_control_sin_tocar_destino() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let escaneo = proyecto.verificar().expect("verificar");
    let id = escaneo.escenas[0].escena.id.clone();
    proyecto
        .escribir_cuerpo(&id, "Antes\u{1}después.\n")
        .expect("escribir con control");

    let destino = directorio.path().join("salida.docx");
    std::fs::write(&destino, b"conservar").expect("destino previo");
    let error = proyecto
        .exportar_docx(&destino, false, true)
        .expect_err("XML inválido");
    assert!(matches!(error, Error::Input(_)), "error: {error}");
    assert!(error.to_string().contains("U+0001"), "error: {error}");
    assert_eq!(
        std::fs::read(&destino).expect("leer destino"),
        b"conservar",
        "el destino no debe reemplazarse si la exportación falla"
    );
}

#[test]
fn exportar_docx_conserva_marcas_literales_y_oculta_solo_anotaciones_validas() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Marcas", None, None).expect("init");
    let id = proyecto.listar(false).expect("listar")[0].id().to_string();
    let literal = "[[personaje:01KDVDNBYG20KW71NVRK6P2FHG|Marta]]";
    let invalida = format!("[[personaje:{}|literal inválido]]", "Z".repeat(26));
    let cuerpo = format!(
        "`{literal}`\n\n> ```text\n> {literal}\n> ```\n\n\\{literal}\n\n{invalida}\n\n[[comentario:nota privada|Prosa final]].\n"
    );
    proyecto.escribir_cuerpo(&id, &cuerpo).expect("escribir");
    let destino = directorio.path().join("marcas.docx");
    proyecto
        .exportar_docx(&destino, false, false)
        .expect("exportar");
    let mut archivo =
        zip::ZipArchive::new(std::fs::File::open(destino).expect("archivo")).expect("ZIP");
    let mut xml = String::new();
    archivo
        .by_name("word/document.xml")
        .expect("documento")
        .read_to_string(&mut xml)
        .expect("XML");
    // El parser puede repartir un literal entre varios runs; comprobar el
    // texto conservado evita depender de esas fronteras de serialización.
    let texto: String = xml
        .split("<w:t")
        .skip(1)
        .filter_map(|run| run.split_once('>')?.1.split_once("</w:t>").map(|v| v.0))
        .collect();
    assert_eq!(texto.matches(literal).count(), 3, "{texto}");
    assert!(texto.contains(&invalida), "{texto}");
    assert!(texto.contains("Prosa final"), "{texto}");
    assert!(!texto.contains("nota privada"), "{texto}");
    assert_eq!(proyecto.leer(&id).expect("leer").cuerpo(), cuerpo);
}

#[cfg(unix)]
#[test]
fn cuarentena_rechaza_el_ancestro_escritor_enlazado() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");
    let libro = raiz.join("manuscrito/01-libro");
    let capitulo = libro.join("cuarentena");
    std::fs::rename(libro.join("01-capitulo"), &capitulo).expect("renombrar capítulo");
    let escena = capitulo.join("001-primera-escena.md");
    let original = std::fs::read(&escena).expect("original");
    std::fs::write(capitulo.join("999-roto.md"), "sin frontmatter\n").expect("inválido");
    let escaneo = verify::escanear(&raiz).expect("escanear");
    std::fs::rename(raiz.join(".escritor"), directorio.path().join("estado"))
        .expect("apartar estado");
    std::os::unix::fs::symlink(&libro, raiz.join(".escritor")).expect("enlace de ancestro");
    assert!(matches!(proyecto.cuarentena(), Err(Error::Integrity(_))));
    assert!(matches!(
        proyecto.descartar_cuarentena("001-primera-escena.md"),
        Err(Error::Integrity(_))
    ));
    assert!(matches!(
        verify::cuarentenar(&raiz, &escaneo),
        Err(Error::Integrity(_))
    ));
    assert_eq!(std::fs::read(escena).expect("original intacto"), original);
    assert!(!capitulo.join("informe.jsonl").exists());
}

#[cfg(unix)]
#[test]
fn cuarentena_no_modifica_un_original_enlazado_duro_al_informe() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");
    let escena = raiz.join("manuscrito/01-libro/01-capitulo/001-primera-escena.md");
    let original = std::fs::read(&escena).expect("original");
    let informe = raiz.join(".escritor/cuarentena/informe.jsonl");
    std::fs::hard_link(&escena, &informe).expect("enlace duro");
    std::fs::write(
        escena.parent().expect("capítulo").join("999-roto.md"),
        "sin frontmatter\n",
    )
    .expect("inválido");
    let escaneo = verify::escanear(&raiz).expect("escanear");
    assert_eq!(verify::cuarentenar(&raiz, &escaneo).expect("cuarentena"), 1);
    assert_eq!(std::fs::read(escena).expect("original intacto"), original);
    let informe = std::fs::read_to_string(informe).expect("informe");
    let linea: serde_json::Value =
        serde_json::from_str(informe.lines().last().expect("línea")).expect("JSONL");
    assert_eq!(
        linea["ruta_original"],
        "manuscrito/01-libro/01-capitulo/999-roto.md"
    );
}

#[cfg(unix)]
#[test]
fn cuarentena_rechaza_enlaces_en_directorio_informe_y_copias() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");
    let capitulo = raiz.join("manuscrito/01-libro/01-capitulo");
    let escena = capitulo.join("001-primera-escena.md");
    let original = std::fs::read(&escena).expect("leer escena");
    std::fs::write(capitulo.join("999-roto.md"), "sin frontmatter\n").expect("inválido");
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(!escaneo.invalidos.is_empty());

    let cuarentena = raiz.join(".escritor/cuarentena");
    std::fs::remove_dir_all(&cuarentena).expect("quitar cuarentena real");
    std::os::unix::fs::symlink(&capitulo, &cuarentena).expect("symlink de directorio");
    let error = proyecto.cuarentena().expect_err("directorio enlazado");
    assert!(matches!(error, Error::Integrity(_)), "error: {error}");
    let error = proyecto
        .descartar_cuarentena("001-primera-escena.md")
        .expect_err("descarte por enlace");
    assert!(matches!(error, Error::Integrity(_)), "error: {error}");
    assert_eq!(std::fs::read(&escena).expect("leer escena"), original);

    std::fs::remove_file(&cuarentena).expect("quitar symlink");
    std::fs::create_dir_all(&cuarentena).expect("cuarentena real");
    let informe = cuarentena.join("informe.jsonl");
    std::os::unix::fs::symlink(&escena, &informe).expect("symlink de informe");
    let error = verify::cuarentenar(proyecto.raiz(), &escaneo).expect_err("informe enlazado");
    assert!(matches!(error, Error::Integrity(_)), "error: {error}");
    assert_eq!(std::fs::read(&escena).expect("leer escena"), original);

    std::fs::remove_file(&informe).expect("quitar informe enlazado");
    let copia = cuarentena.join("copia-enlazada.md");
    std::os::unix::fs::symlink(&escena, &copia).expect("symlink de copia");
    let items = proyecto.cuarentena().expect("listar");
    assert!(items.is_empty(), "no debe listar enlaces: {items:?}");
    let error = proyecto
        .descartar_cuarentena("copia-enlazada.md")
        .expect_err("descarte de copia enlazada");
    assert!(matches!(error, Error::Input(_)), "error: {error}");
    assert_eq!(std::fs::read(&escena).expect("leer escena"), original);
}

#[cfg(unix)]
#[test]
fn exportar_rechaza_destino_dentro_del_proyecto_por_symlink() {
    let directorio = temporal();
    let raiz = directorio.path().join("novela");
    let proyecto = Project::new(&raiz);
    proyecto.init("Novela", None, None).expect("init");
    let escaneo = proyecto.verificar().expect("verificar");
    let escena = &escaneo.escenas[0];
    let original = std::fs::read(&escena.ruta).expect("leer escena");

    let alias = directorio.path().join("alias");
    std::os::unix::fs::symlink(&raiz, &alias).expect("crear symlink");
    let destino = alias
        .join("manuscrito")
        .join("01-libro")
        .join("01-capitulo")
        .join("001-primera-escena.md");

    let error = proyecto
        .exportar_docx(&destino, false, true)
        .expect_err("DOCX a través de alias");
    assert!(matches!(error, Error::Input(_)), "error: {error}");
    assert_eq!(std::fs::read(&escena.ruta).expect("leer escena"), original);

    let error = proyecto
        .exportar(&destino, false, true)
        .expect_err("Markdown a través de alias");
    assert!(matches!(error, Error::Input(_)), "error: {error}");
    assert_eq!(std::fs::read(&escena.ruta).expect("leer escena"), original);

    assert!(proyecto
        .exportar_docx(&raiz.join("salida.docx"), false, false)
        .is_err());
}

#[test]
fn cuarentena_lista_y_descarta_copias() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let roto = proyecto
        .raiz()
        .join("manuscrito/01-libro/01-capitulo/999-roto.md");
    std::fs::write(&roto, "sin frontmatter\n").expect("escribir inválido");

    let escaneo = proyecto.verificar().expect("verificar");
    assert!(!escaneo.invalidos.is_empty());
    let copiados = verify::cuarentenar(proyecto.raiz(), &escaneo).expect("cuarentenar");
    assert!(copiados >= 1);

    let items = proyecto.cuarentena().expect("listar cuarentena");
    assert_eq!(items.len(), 1);
    assert!(items[0].motivo.is_some(), "items: {items:?}");
    assert!(items[0].tamano > 0);
    assert!(roto.exists(), "el original no debe tocarse");

    for nombre in ["../escape", "..\\escape", "C:escape", "informe.jsonl"] {
        assert!(matches!(
            proyecto.descartar_cuarentena(nombre),
            Err(Error::Input(_))
        ));
    }
    proyecto
        .descartar_cuarentena(&items[0].archivo)
        .expect("descartar copia");
    assert!(proyecto.cuarentena().expect("relistar").is_empty());
    assert!(roto.exists(), "el original sigue existiendo");
}

#[test]
fn documento_fuera_de_sitio_con_id_es_error() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let extra = proyecto.raiz().join("extras").join("suelto.md");
    std::fs::create_dir_all(extra.parent().expect("padre")).expect("mkdir");
    std::fs::write(
        &extra,
        "---\nid: 01KDVDNAZ84TFF59TDWH9EDD1R\ntitulo: \"Suelto\"\n---\n\nTexto.\n",
    )
    .expect("escribir");
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(
        escaneo
            .errores
            .iter()
            .any(|error| error.contains("fuera de manuscrito/ y notas/")),
        "errores: {:#?}",
        escaneo.errores
    );
}

#[test]
fn blob_faltante_del_historial_es_aviso() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (id, _) = proyecto
        .crear_escena("Con historial", None, None, None)
        .expect("crear");
    proyecto
        .escribir_cuerpo(&id, "contenido\n")
        .expect("escribir");
    let objetos = proyecto.raiz().join(".escritor/objetos");
    let primero = std::fs::read_dir(&objetos)
        .expect("objetos")
        .next()
        .expect("blob")
        .expect("entrada")
        .path();
    std::fs::remove_file(primero).expect("borrar blob");
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(
        escaneo
            .avisos
            .iter()
            .any(|aviso| aviso.contains("falta el blob")),
        "avisos: {:#?}",
        escaneo.avisos
    );
}

#[test]
fn reindex_funciona_aunque_haya_un_documento_invalido() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let roto = proyecto
        .raiz()
        .join("manuscrito/01-libro/01-capitulo/999-roto.md");
    std::fs::write(&roto, "sin frontmatter\n").expect("escribir roto");
    let resultado = proyecto.reindexar(false).expect("reindexar");
    assert_eq!(resultado.escenas, 1);
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(!escaneo.errores.is_empty());
}

#[test]
fn log_faltante_permite_leer_pero_no_escribir() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (id, _) = proyecto
        .crear_escena("Sin historial", None, None, None)
        .expect("crear");
    std::fs::remove_file(proyecto.raiz().join(".escritor/operaciones.log")).expect("borrar log");
    assert_eq!(proyecto.leer(&id).expect("leer").titulo(), "Sin historial");
    assert!(proyecto.escribir_cuerpo(&id, "no\n").is_err());
    let escaneo = proyecto.verificar().expect("verificar");
    assert!(escaneo
        .errores
        .iter()
        .any(|error| error.contains("modo recuperación")));
}

#[test]
fn indice_con_pagina_corrupta_se_aparta_y_se_reconstruye() {
    let directorio = temporal();
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto.init("Novela", None, None).expect("init");
    let (id, _) = proyecto
        .crear_escena("Con índice", None, None, None)
        .expect("crear");
    proyecto
        .escribir_cuerpo(&id, "contenido\n")
        .expect("escribir");

    let indice = proyecto.raiz().join(".escritor/indice.sqlite");
    {
        let conexion = rusqlite::Connection::open(&indice).expect("abrir índice");
        conexion
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .expect("checkpoint");
    }
    let tamano = std::fs::metadata(&indice).expect("tamaño").len();
    assert!(
        tamano > 4608,
        "el índice debe tener varias páginas: {tamano}"
    );
    let mut archivo = std::fs::OpenOptions::new()
        .write(true)
        .open(&indice)
        .expect("abrir para corromper");
    archivo.seek(SeekFrom::Start(4096)).expect("seek");
    archivo.write_all(&[0xFF; 512]).expect("corromper página");
    archivo.sync_all().expect("sync");

    let resultado = proyecto.reindexar(false).expect("reindexar");
    assert_eq!(resultado.escenas, 2);
    let apartados: Vec<_> = std::fs::read_dir(proyecto.raiz().join(".escritor"))
        .expect("leer .escritor")
        .filter_map(|entrada| entrada.ok())
        .filter(|entrada| {
            entrada
                .file_name()
                .to_string_lossy()
                .starts_with("indice.sqlite.corrupto-")
        })
        .collect();
    assert_eq!(apartados.len(), 1);
}

#[test]
fn verificador_rust_detecta_los_doce_hallazgos_del_fixture_roto() {
    let raiz = fixture("proyecto-roto");
    let escaneo = verify::escanear(&raiz).expect("escanear");
    let esperados = std::fs::read_to_string(raiz.join("esperados.txt")).expect("esperados");
    let mut comprobados = 0;
    for linea in esperados.lines() {
        let linea = linea.trim();
        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }
        let encontrado = escaneo.errores.iter().any(|error| error.contains(linea));
        assert!(
            encontrado,
            "no se detectó el hallazgo esperado: {linea}\nErrores: {:#?}",
            escaneo.errores
        );
        comprobados += 1;
    }
    assert_eq!(comprobados, 12, "esperados.txt debe fijar 12 hallazgos");
}

#[test]
fn fixture_pequeno_verifica_sin_errores_y_sin_crear_escritor() {
    let raiz = fixture("proyecto-pequeno");
    let escaneo = verify::escanear(&raiz).expect("escanear");
    assert!(
        escaneo.errores.is_empty(),
        "errores: {:#?}",
        escaneo.errores
    );
    assert_eq!(escaneo.escenas.len(), 3);
    assert_eq!(escaneo.notas.len(), 18);
    assert!(!raiz.join(".escritor").exists());
}

#[test]
fn modelos_nuevos_son_validos() {
    let escena = Escena::nueva("Título", 10).expect("escena");
    assert_eq!(escena.estado.as_deref(), Some("borrador"));
    let nota = Nota::nueva("tramas", "Trama").expect("nota");
    assert_eq!(nota.tipo, "tramas");
}
