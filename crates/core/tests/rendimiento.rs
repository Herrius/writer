//! Prueba de rendimiento del núcleo con un proyecto de 1.000 escenas.
//!
//! No usa un fixture versionado: crea el proyecto en un directorio temporal
//! para que la prueba funcione en un clon limpio. Los presupuestos son amplios
//! a propósito; sirven para detectar regresiones graves, no para medir la
//! máquina del desarrollador.

use std::time::{Duration, Instant};

use writer_core::model::Escena;
use writer_core::project::Project;

#[test]
fn mil_escenas_se_verifican_listan_y_buscan_dentro_del_presupuesto() {
    let directorio = tempfile::tempdir().expect("temporal");
    let proyecto = Project::new(directorio.path().join("novela"));
    proyecto
        .init("Rendimiento", Some("Autora"), Some("es"))
        .expect("init");

    let carpeta = proyecto.raiz().join("manuscrito/01-libro/01-capitulo");
    std::fs::create_dir_all(&carpeta).expect("carpeta");
    for indice in 2..=1000_i64 {
        let mut escena = Escena::nueva(&format!("Escena {indice}"), indice * 10).expect("escena");
        escena.cuerpo =
            format!("La escena {indice} mira el faro y recuerda una palabra del pasado.\n");
        let ruta = carpeta.join(format!("{indice:03}-escena-{indice}.md"));
        std::fs::write(ruta, escena.a_bytes()).expect("escribir escena");
    }

    let inicio = Instant::now();
    let escaneo = proyecto.verificar().expect("verificar");
    let tiempo_verificar = inicio.elapsed();
    assert_eq!(escaneo.escenas.len(), 1000);
    assert!(escaneo.errores.is_empty(), "errores: {:?}", escaneo.errores);

    let inicio = Instant::now();
    let documentos = proyecto.listar(false).expect("listar");
    let tiempo_listar = inicio.elapsed();
    assert_eq!(documentos.len(), 1000);

    let inicio = Instant::now();
    proyecto.reindexar(false).expect("reindexar");
    let tiempo_reindexar = inicio.elapsed();

    let inicio = Instant::now();
    let resultados = proyecto.buscar("faro", 30).expect("buscar");
    let tiempo_buscar = inicio.elapsed();
    assert_eq!(resultados.len(), 30);

    eprintln!(
        "1000 escenas: verificar={tiempo_verificar:?} listar={tiempo_listar:?} \
         reindexar={tiempo_reindexar:?} buscar={tiempo_buscar:?}"
    );
    for (nombre, tiempo, limite) in [
        ("verificar", tiempo_verificar, Duration::from_secs(20)),
        ("listar", tiempo_listar, Duration::from_secs(10)),
        ("reindexar", tiempo_reindexar, Duration::from_secs(20)),
        ("buscar", tiempo_buscar, Duration::from_secs(10)),
    ] {
        assert!(
            tiempo <= limite,
            "{nombre} tardó {tiempo:?}, más de {limite:?}"
        );
    }
}
