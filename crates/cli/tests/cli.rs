//! Pruebas de contrato de la CLI.
//!
//! Se ejecutan sobre el binario real, no sobre funciones internas: lo que
//! promete la ayuda es lo que el proceso devuelve. Los fallos simulados usan
//! procesos hijos y `WRITER_FAULT`, como un corte real.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn writer() -> Command {
    Command::new(env!("CARGO_BIN_EXE_writer"))
}

fn proyecto_temporal() -> (tempfile::TempDir, PathBuf) {
    let directorio = tempfile::tempdir().expect("temporal");
    let raiz = directorio.path().join("novela");
    let salida = writer()
        .arg("init")
        .arg(&raiz)
        .args(["--titulo", "Novela de prueba"])
        .output()
        .expect("init");
    assert!(
        salida.status.success(),
        "init falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    (directorio, raiz)
}

fn ejecutar_en(raiz: &Path, args: &[&str]) -> Output {
    writer()
        .args(args)
        .arg(raiz)
        .output()
        .expect("ejecutar writer")
}

fn texto(salida: &Output) -> String {
    String::from_utf8_lossy(&salida.stdout).to_string()
}

fn crear_escena(raiz: &Path, titulo: &str) -> (String, String) {
    let salida = writer()
        .args(["create", "escena", "--titulo", titulo])
        .arg(raiz)
        .output()
        .expect("crear escena");
    assert!(
        salida.status.success(),
        "crear escena falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    let salida = texto(&salida);
    let mut partes = salida.split_whitespace();
    (
        partes.next().expect("id").to_string(),
        partes.next().expect("ruta").to_string(),
    )
}

fn leer_cuerpo(raiz: &Path, id: &str) -> String {
    let salida = writer()
        .args(["read", id])
        .arg(raiz)
        .output()
        .expect("read");
    assert!(
        salida.status.success(),
        "read falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    texto(&salida)
}

fn ultimo_tipo_log(raiz: &Path) -> String {
    let log = std::fs::read_to_string(raiz.join(".escritor/operaciones.log")).expect("log");
    let linea = log
        .lines()
        .rfind(|linea| !linea.trim().is_empty())
        .expect("última línea");
    let json = linea.split('\t').next().expect("json");
    let valor: serde_json::Value = serde_json::from_str(json).expect("json válido");
    valor["tipo"].as_str().expect("tipo").to_string()
}

#[test]
fn version_informa_binario_y_esquema() {
    let salida = writer().arg("--version").output().expect("ejecutar writer");
    assert!(salida.status.success(), "código de salida inesperado");
    let texto = String::from_utf8(salida.stdout).expect("salida UTF-8");
    assert!(texto.starts_with("writer "), "salida inesperada: {texto}");
    assert!(
        texto.contains("esquema 1"),
        "falta la versión de esquema: {texto}"
    );
}

#[test]
fn ayuda_lista_los_comandos_previstos() {
    let salida = writer().arg("--help").output().expect("ejecutar writer");
    assert!(salida.status.success());
    let texto = String::from_utf8(salida.stdout).expect("salida UTF-8");
    for comando in [
        "init", "info", "list", "buscar", "read", "create", "write", "move", "delete", "history",
        "show", "revert", "verify", "reindex", "export",
    ] {
        assert!(
            texto.contains(comando),
            "la ayuda no menciona `{comando}`:\n{texto}"
        );
    }
}

#[test]
fn comando_desconocido_sale_con_codigo_2() {
    let salida = writer()
        .arg("comando-que-no-existe")
        .output()
        .expect("ejecutar writer");
    assert_eq!(salida.status.code(), Some(2));
    let error = String::from_utf8(salida.stderr).expect("stderr UTF-8");
    assert!(
        error.contains("unrecognized subcommand") || error.contains("comando desconocido"),
        "stderr: {error}"
    );
}

#[test]
fn init_verify_list_y_read() {
    let (_directorio, raiz) = proyecto_temporal();
    assert!(raiz.join(".escritor/operaciones.log").is_file());

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    let salida = texto(&verify);
    assert!(salida.contains("escenas: 1"));
    assert!(salida.contains("resultado: OK"));

    let listado = ejecutar_en(&raiz, &["list"]);
    assert!(listado.status.success());
    assert!(texto(&listado).contains("Primera escena"));

    let info = ejecutar_en(&raiz, &["info", "--json"]);
    assert!(info.status.success());
    let info: serde_json::Value = serde_json::from_slice(&info.stdout).expect("json");
    assert_eq!(info["revision"], 1);
    assert_eq!(info["escenas"], 1);
}

#[test]
fn buscar_encuentra_escenas_y_notas() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, _) = crear_escena(&raiz, "El faro");
    let escritura = ejecutar_en(
        &raiz,
        &["write", &id, "--texto", "Una luz sobre el puerto.\n"],
    );
    assert!(
        escritura.status.success(),
        "write: {}",
        String::from_utf8_lossy(&escritura.stderr)
    );

    let busqueda = ejecutar_en(&raiz, &["buscar", "puerto", "--json"]);
    assert!(
        busqueda.status.success(),
        "buscar: {}",
        String::from_utf8_lossy(&busqueda.stderr)
    );
    let resultados: serde_json::Value =
        serde_json::from_slice(&busqueda.stdout).expect("json de búsqueda");
    let resultados = resultados.as_array().expect("array");
    assert_eq!(resultados.len(), 1);
    assert_eq!(resultados[0]["id"], id);
    assert_eq!(resultados[0]["tipo"], "escena");
    assert!(resultados[0]["fragmento"]
        .as_str()
        .expect("fragmento")
        .contains("[puerto]"));

    let vacia = ejecutar_en(&raiz, &["buscar", "   "]);
    assert!(!vacia.status.success(), "una consulta vacía debe fallar");
}

#[test]
fn ciclo_completo_de_escritura_historial_y_exportacion() {
    let (directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "El faro");
    assert_eq!(ruta, "manuscrito/01-libro/01-capitulo/002-el-faro.md");

    let revision = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "Una luz sobre el puerto.\n"])
        .output()
        .expect("write");
    assert!(
        revision.status.success(),
        "write: {}",
        String::from_utf8_lossy(&revision.stderr)
    );
    assert!(texto(&revision).contains("revisión 3"));
    assert_eq!(leer_cuerpo(&raiz, &id), "Una luz sobre el puerto.\n");

    let nota = writer()
        .args([
            "create",
            "nota",
            "--tipo",
            "personajes",
            "--titulo",
            "Marta",
        ])
        .arg(&raiz)
        .output()
        .expect("crear nota");
    assert!(nota.status.success());
    let nota_id = texto(&nota)
        .split_whitespace()
        .next()
        .expect("id nota")
        .to_string();

    let historial = ejecutar_en(&raiz, &["history", "--json"]);
    assert!(historial.status.success());
    let historial: serde_json::Value = serde_json::from_slice(&historial.stdout).expect("json");
    assert_eq!(historial.as_array().expect("array").len(), 4);

    let mostrar = writer()
        .args(["show", "3"])
        .arg(&raiz)
        .output()
        .expect("show");
    assert!(mostrar.status.success());
    assert!(texto(&mostrar).contains("Una luz sobre el puerto."));

    let destino = directorio.path().join("salida.md");
    let export = writer()
        .args(["export", "md"])
        .arg(&destino)
        .arg(&raiz)
        .args(["--notas"])
        .output()
        .expect("export");
    assert!(
        export.status.success(),
        "export: {}",
        String::from_utf8_lossy(&export.stderr)
    );
    let exportado = std::fs::read_to_string(&destino).expect("leer export");
    assert!(exportado.contains("# Libro"));
    assert!(exportado.contains("### El faro"));
    assert!(exportado.contains("# Notas"));
    assert!(exportado.contains("### Marta"));

    let borrado = writer()
        .args(["delete", &nota_id])
        .arg(&raiz)
        .output()
        .expect("delete");
    assert!(borrado.status.success());
    assert!(writer()
        .args(["read", &nota_id])
        .arg(&raiz)
        .output()
        .expect("read")
        .status
        .code()
        .is_some_and(|codigo| codigo != 0));

    let revision_borrado = 5_u64;
    let revertir = writer()
        .args(["revert", &revision_borrado.to_string()])
        .arg(&raiz)
        .output()
        .expect("revert");
    assert!(
        revertir.status.success(),
        "revert: {}",
        String::from_utf8_lossy(&revertir.stderr)
    );
    let restaurada = writer()
        .args(["read", &nota_id, "--completo"])
        .arg(&raiz)
        .output()
        .expect("read restaurada");
    assert!(restaurada.status.success());
    assert!(texto(&restaurada).contains("Marta"));
}

#[test]
fn fault_prepare_no_publica_cambios_y_la_recuperacion_revierte() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con fallo");
    let ruta_abs = raiz.join(&ruta);
    let original = std::fs::read_to_string(&ruta_abs).expect("original");

    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "no debe verse\n"])
        .env("WRITER_FAULT", "prepare")
        .output()
        .expect("write con fallo");
    assert_eq!(salida.status.code(), Some(70));
    assert_eq!(
        std::fs::read_to_string(&ruta_abs).expect("leer"),
        original,
        "el archivo no debe cambiar antes de PREPARE"
    );

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert_eq!(std::fs::read_to_string(&ruta_abs).expect("leer"), original);
    let log = std::fs::read_to_string(raiz.join(".escritor/operaciones.log")).expect("log");
    assert!(log.contains("\"tipo\":\"ABORT\""), "log: {log}");
    assert!(!leer_cuerpo(&raiz, &id).contains("no debe verse"));
}

#[test]
fn fault_after_write_se_revierte_al_abrir() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con fallo");
    let ruta_abs = raiz.join(&ruta);
    let original = std::fs::read_to_string(&ruta_abs).expect("original");

    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "escrito antes del corte\n"])
        .env("WRITER_FAULT", "after_write:1")
        .output()
        .expect("write con fallo");
    assert_eq!(salida.status.code(), Some(70));
    assert_ne!(std::fs::read_to_string(&ruta_abs).expect("leer"), original);

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert_eq!(std::fs::read_to_string(&ruta_abs).expect("leer"), original);
    let historial = ejecutar_en(&raiz, &["history", "--json"]);
    let historial: serde_json::Value = serde_json::from_slice(&historial.stdout).expect("json");
    assert_eq!(historial.as_array().expect("array").len(), 2);
}

#[test]
fn fault_commit_conserva_lo_confirmado() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena confirmada");
    let ruta_abs = raiz.join(&ruta);

    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "confirmado\n"])
        .env("WRITER_FAULT", "commit")
        .output()
        .expect("write con fallo");
    assert_eq!(salida.status.code(), Some(70));

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    let contenido = std::fs::read_to_string(&ruta_abs).expect("leer");
    assert!(contenido.contains("confirmado\n"));
    assert_eq!(leer_cuerpo(&raiz, &id), "confirmado\n");
    let historial = ejecutar_en(&raiz, &["history", "--json"]);
    let historial: serde_json::Value = serde_json::from_slice(&historial.stdout).expect("json");
    assert_eq!(historial.as_array().expect("array").len(), 3);
}

#[test]
fn fault_before_replace_documenta_last_writer_wins() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con carrera");
    let ruta_abs = raiz.join(&ruta);

    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "contenido del motor\n"])
        .env("WRITER_FAULT", "before_replace")
        .env("WRITER_FAULT_EXTERNAL", "contenido externo\n")
        .output()
        .expect("write con carrera");
    assert!(
        salida.status.success(),
        "write: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    // La ventana CAS no es atómica: el motor gana y el cambio externo se pierde.
    let contenido = std::fs::read_to_string(&ruta_abs).expect("leer");
    assert!(contenido.contains("contenido del motor\n"));
    assert!(!contenido.contains("contenido externo"));
}

#[test]
fn fault_before_check_conserva_el_cambio_externo_y_aborta() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con conflicto");
    let ruta_abs = raiz.join(&ruta);
    let externo = std::fs::read_to_string(&ruta_abs)
        .expect("original")
        .replace("Escena con conflicto", "Escena externa");

    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "contenido del motor\n"])
        .env("WRITER_FAULT", "before_check")
        .env("WRITER_FAULT_EXTERNAL", &externo)
        .output()
        .expect("write con conflicto");
    assert!(!salida.status.success());
    let error = String::from_utf8_lossy(&salida.stderr);
    assert!(error.contains("conflicto"), "stderr: {error}");
    assert_eq!(std::fs::read_to_string(&ruta_abs).expect("leer"), externo);

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert!(texto(&verify).contains("resultado: OK"));
    assert!(
        texto(&verify).contains("conflicto") || texto(&verify).contains("recuperación"),
        "el conflicto debe ser visible: {}",
        texto(&verify)
    );
}

#[test]
fn fault_move_after_write_recupera_el_archivo_original() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena que se mueve");
    let original = raiz.join(&ruta);

    let salida = writer()
        .args(["move", &id])
        .arg(&raiz)
        .args(["--libro", "02-libro"])
        .env("WRITER_FAULT", "after_write:1")
        .output()
        .expect("move con fallo");
    assert_eq!(salida.status.code(), Some(70));
    assert!(!original.exists(), "el primer efecto ya borró el original");

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert!(
        original.exists(),
        "la recuperación debe restaurar el original"
    );
    assert!(leer_cuerpo(&raiz, &id).trim().is_empty());
    assert!(!raiz
        .join("manuscrito/02-libro/01-capitulo/002-escena-que-se-mueve.md")
        .exists());
}

#[test]
fn falta_blob_deja_el_proyecto_en_modo_recuperacion() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, _ruta) = crear_escena(&raiz, "Escena sin blob");
    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "contenido no confirmado\n"])
        .env("WRITER_FAULT", "after_write:1")
        .output()
        .expect("write con fallo");
    assert_eq!(salida.status.code(), Some(70));

    let objetos = raiz.join(".escritor/objetos");
    for entrada in std::fs::read_dir(&objetos).expect("objetos") {
        std::fs::remove_file(entrada.expect("entrada").path()).expect("borrar blob");
    }

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(!verify.status.success());
    let salida = texto(&verify);
    assert!(
        salida.contains("modo recuperación") || salida.contains("falta el blob"),
        "salida: {salida}"
    );
}

#[test]
fn un_abort_antiguo_no_deshace_un_guardado_posterior() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, _ruta) = crear_escena(&raiz, "Escena con historial");

    let a = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "A\n"])
        .output()
        .expect("write A");
    assert!(a.status.success());

    let fallo = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "B\n"])
        .env("WRITER_FAULT", "after_write:1")
        .output()
        .expect("write B abortado");
    assert_eq!(fallo.status.code(), Some(70));

    let recuperacion = ejecutar_en(&raiz, &["verify"]);
    assert!(
        recuperacion.status.success(),
        "verify: {}",
        texto(&recuperacion)
    );
    assert_eq!(leer_cuerpo(&raiz, &id), "A\n");

    let b = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "B\n"])
        .output()
        .expect("write B confirmado");
    assert!(b.status.success());
    assert_eq!(leer_cuerpo(&raiz, &id), "B\n");
    // Reabrir otra vez no debe reaplicar la recuperación del ABORT antiguo.
    assert_eq!(leer_cuerpo(&raiz, &id), "B\n");
    let historial = ejecutar_en(&raiz, &["history", "--json"]);
    let historial: serde_json::Value = serde_json::from_slice(&historial.stdout).expect("json");
    assert_eq!(historial.as_array().expect("array").len(), 4);
}

#[test]
fn checksum_corrupto_en_la_ultima_linea_completa_no_borra_la_revision() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con checksum");
    let ruta_abs = raiz.join(&ruta);
    let a = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "A\n"])
        .output()
        .expect("write A");
    assert!(a.status.success());
    let b = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "B\n"])
        .output()
        .expect("write B");
    assert!(b.status.success());

    let log = raiz.join(".escritor/operaciones.log");
    let mut contenido_log = std::fs::read_to_string(&log).expect("log");
    let inicio_ultima = contenido_log
        .trim_end_matches('\n')
        .rfind('\n')
        .map(|posicion| posicion + 1)
        .unwrap_or(0);
    let tabulador = contenido_log[inicio_ultima..]
        .rfind('\t')
        .expect("tab final")
        + inicio_ultima;
    let original = contenido_log.as_bytes()[tabulador + 1] as char;
    let reemplazo = if original == '0' { "1" } else { "0" };
    contenido_log.replace_range(tabulador + 1..tabulador + 2, reemplazo);
    std::fs::write(&log, contenido_log).expect("corromper checksum");

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(
        !verify.status.success(),
        "verify no debe dar OK con el log corrupto"
    );
    assert!(
        texto(&verify).contains("checksum") || texto(&verify).contains("recuperación"),
        "salida: {}",
        texto(&verify)
    );
    assert!(
        std::fs::read_to_string(&ruta_abs)
            .expect("leer")
            .contains("B\n"),
        "la última revisión confirmada no debe desaparecer"
    );
}

#[test]
fn blob_corrupto_no_elimina_el_destino_del_movimiento() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, _ruta) = crear_escena(&raiz, "Escena que se mueve");
    let destino = raiz.join("manuscrito/02-libro/01-capitulo/002-escena-que-se-mueve.md");

    let salida = writer()
        .args(["move", &id])
        .arg(&raiz)
        .args(["--libro", "02-libro"])
        .env("WRITER_FAULT", "after_write:2")
        .output()
        .expect("move con fallo");
    assert_eq!(salida.status.code(), Some(70));
    assert!(destino.exists());

    let objetos = raiz.join(".escritor/objetos");
    for entrada in std::fs::read_dir(&objetos).expect("objetos") {
        let ruta = entrada.expect("entrada").path();
        let mut bytes = std::fs::read(&ruta).expect("leer blob");
        if let Some(primero) = bytes.first_mut() {
            *primero ^= 0xFF;
        }
        std::fs::write(&ruta, bytes).expect("corromper blob");
    }

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(!verify.status.success());
    assert!(
        destino.exists(),
        "el preflight no debe haber eliminado el destino"
    );
    assert!(!raiz
        .join("manuscrito/01-libro/01-capitulo/002-escena-que-se-mueve.md")
        .exists());
}

#[test]
fn anexar_a_un_log_sin_salto_final_no_introduce_nul() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, _ruta) = crear_escena(&raiz, "Escena sin salto");
    let log = raiz.join(".escritor/operaciones.log");
    let contenido_log = std::fs::read_to_string(&log).expect("log");
    assert!(contenido_log.ends_with('\n'));
    std::fs::write(&log, contenido_log.trim_end_matches('\n')).expect("quitar salto final");

    let write = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "después\n"])
        .output()
        .expect("write");
    assert!(
        write.status.success(),
        "write: {}",
        String::from_utf8_lossy(&write.stderr)
    );
    let bytes = std::fs::read(&log).expect("log");
    assert!(!bytes.contains(&0), "el log no debe contener NUL");
    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    let historial = ejecutar_en(&raiz, &["history", "--json"]);
    let historial: serde_json::Value = serde_json::from_slice(&historial.stdout).expect("json");
    assert_eq!(historial.as_array().expect("array").len(), 3);
}

#[test]
fn fallo_fsync_tras_reemplazo_se_revierte() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con fsync");
    let a = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "A\n"])
        .output()
        .expect("write A");
    assert!(a.status.success());
    let padre = raiz.join(&ruta).parent().expect("padre").to_path_buf();

    let fallo = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "B\n"])
        .env("WRITER_FAULT_FSYNC_DIR", &padre)
        .env("WRITER_FAULT_FSYNC_ONCE", "1")
        .output()
        .expect("write con EIO");
    assert!(!fallo.status.success());
    assert!(
        String::from_utf8_lossy(&fallo.stderr).contains("EIO")
            || String::from_utf8_lossy(&fallo.stderr).contains("fsync"),
        "stderr: {}",
        String::from_utf8_lossy(&fallo.stderr)
    );

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert_eq!(leer_cuerpo(&raiz, &id), "A\n");
    assert_eq!(ultimo_tipo_log(&raiz), "ABORT");
}

#[test]
fn fallo_fsync_tras_borrado_se_revierte() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena que se mueve");
    let origen = raiz.join(&ruta);
    let padre = origen.parent().expect("padre").to_path_buf();

    let fallo = writer()
        .args(["move", &id])
        .arg(&raiz)
        .args(["--libro", "02-libro"])
        .env("WRITER_FAULT_FSYNC_DIR", &padre)
        .env("WRITER_FAULT_FSYNC_ONCE", "1")
        .output()
        .expect("move con EIO");
    assert!(!fallo.status.success());

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert!(origen.exists(), "la recuperación debe restaurar el origen");
    assert!(!raiz
        .join("manuscrito/02-libro/01-capitulo/002-escena-que-se-mueve.md")
        .exists());
    assert_eq!(ultimo_tipo_log(&raiz), "ABORT");
}

#[test]
fn si_falla_el_fsync_del_rollback_no_se_persiste_abort() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, ruta) = crear_escena(&raiz, "Escena con EIO persistente");
    let a = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "A\n"])
        .output()
        .expect("write A");
    assert!(a.status.success());
    let padre = raiz.join(&ruta).parent().expect("padre").to_path_buf();

    let fallo = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "B\n"])
        .env("WRITER_FAULT_FSYNC_DIR", &padre)
        .output()
        .expect("write con EIO persistente");
    assert!(!fallo.status.success());
    assert_eq!(
        ultimo_tipo_log(&raiz),
        "PREPARE",
        "sin una reversión sincronizada no debe quedar ABORT terminal"
    );

    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    assert_eq!(leer_cuerpo(&raiz, &id), "A\n");
    assert_eq!(ultimo_tipo_log(&raiz), "ABORT");
}

#[test]
fn lock_ocupado_falla_sin_esperar_indefinidamente() {
    let (_directorio, raiz) = proyecto_temporal();
    let (id, _) = crear_escena(&raiz, "Escena bloqueada");
    let ruta_bloqueo = raiz.join(".escritor/bloqueo");
    let archivo = File::options()
        .read(true)
        .write(true)
        .open(&ruta_bloqueo)
        .expect("abrir bloqueo");
    archivo.try_lock().expect("tomar lock de prueba");

    let salida = writer()
        .args(["write", &id])
        .arg(&raiz)
        .args(["--texto", "bloqueado\n"])
        .env("WRITER_LOCK_TIMEOUT_MS", "100")
        .output()
        .expect("write bloqueado");
    assert!(!salida.status.success());
    let error = String::from_utf8_lossy(&salida.stderr);
    assert!(error.contains("proyecto ocupado"), "stderr: {error}");
    let _ = archivo.unlock();
}

#[test]
fn indice_corrupto_se_aparta_y_se_reconstruye() {
    let (_directorio, raiz) = proyecto_temporal();
    let indice = raiz.join(".escritor/indice.sqlite");
    std::fs::write(&indice, b"esto no es sqlite").expect("corromper índice");

    let salida = ejecutar_en(&raiz, &["reindex"]);
    assert!(
        salida.status.success(),
        "reindex: {}",
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(texto(&salida).contains("índice reconstruido"));
    let apartados: Vec<_> = std::fs::read_dir(raiz.join(".escritor"))
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
fn fixtures_no_se_modifican_al_verificar_o_listar() {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/proyecto-pequeno");
    let antes = raiz.join(".escritor").exists();
    let verify = ejecutar_en(&raiz, &["verify"]);
    assert!(verify.status.success(), "verify: {}", texto(&verify));
    let list = ejecutar_en(&raiz, &["list"]);
    assert!(list.status.success());
    assert_eq!(raiz.join(".escritor").exists(), antes);
    assert!(!raiz.join(".escritor").exists());
}
