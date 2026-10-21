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
    assert!(resultados[0].fragmento.contains("[puerto]"));
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
        .any(|resultado| resultado.fragmento.contains("[marea]")));
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
    assert!(con_guion[0].fragmento.contains("[Puerto-Viejo]"));

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
