# Fase 2 — Security review (r3)

- **Commit revisado:** `b336467726373ea863eb960796f817ae7b8d0b48`
- **HEAD del pipeline:** `f35a11da22808ba8b0ce8f86f5107d1bce245076`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64; Rust 1.99.1; `cargo-deny` 0.20.2; `zizmor` 1.31.1; gitleaks local (el binario no reporta número de versión); pnpm 12.14.0; Node 26.11.1; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). `git diff --stat b336467..HEAD` solo muestra `docs/revisiones/README.md` y `docs/revisiones/fase-2-00-congelado-r6.md`; no hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD. Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **aprobado con observaciones** (P1: 0 · P2: 0; P3 nuevos: 2; P3 residuales: 2; P4 nuevos: 6).

## 1. Alcance y método

Revisé el artefacto congelado, no la justificación del builder. El modelo de amenaza es el de la r1/r2: proyecto de terceros manipulado (Markdown, `proyecto.yml`, `operaciones.log`, blobs, índice, cuarentena, sidecars SQLite), enlaces simbólicos y duros, rutas de Windows (`\`, `C:`, UNC, `..\`), Unicode/controles, YAML/OOXML malformados, archivos especiales (FIFO, sockets, dispositivos), agotamiento de recursos y superficie Tauri.

No ejecuté la GUI ni la app de escritorio. Usé el binario `target/debug/writer` recompilado desde el commit congelado (`cargo build --locked --bin writer`), análisis estático de `crates/core/src/{fsutil,hash,project,index,blob,oplog,lock,recovery}.rs`, `apps/desktop/src-tauri/src/lib.rs`, `tauri.conf.json` y `capabilities/default.json`, y un arnés propio en `/tmp/opencode/sec-r3`. Para fabricar un `PREPARE` pendiente válido usé la inyección de fallos del propio producto (`WRITER_FAULT=after_write:1` y `WRITER_FAULT=prepare`), no un generador de checksums externo; así el log lo valida el mismo código que se revisa. No se modificó ningún archivo de código, tests ni configuración del repositorio; este informe es la única escritura.

La comprobación de estado fue:

```text
git status --porcelain                         vacío
git rev-parse HEAD                             f35a11da22808ba8b0ce8f86f5107d1bce245076
git diff --stat b336467..HEAD                  solo docs/revisiones/README.md y el acta r6
cargo build --locked --bin writer              OK
```

## 2. Verificación de SEC-02-R, SEC-02-H, SEC-04 FIFO y SEC-03-R

### SEC-02-R — cerrado

Reproduje el escenario exacto de la r2 y sus variantes. El escenario base fue un proyecto en un directorio cuyo nombre contenía una de las palabras clave (`novela-corrupta`, `novela-malformed`, `not a database`, `disk image`, `corrupt`), `.escritor` como enlace simbólico a un directorio externo con `indice.sqlite`, `indice.sqlite-wal` e `indice.sqlite-shm`, y ejecuté `reindex` con y sin `--limpiar`, además de `info` y `write`. Después probé las variantes de error de integridad de la tabla.

| Caso | Resultado |
|---|---|
| `reindex` / `reindex --limpiar` con `.escritor` enlazado y proyecto con palabra clave | **OK**: rc 1, `se rechaza .escritor enlazado: ...`; los tres archivos externos conservan tamaño y hash; no aparece ningún `*.corrupto-*`. |
| `info` y `write` con `.escritor` enlazado | **OK**: rc 1, mismo rechazo; sin efecto externo. |
| `.escritor` como archivo regular, proyecto `a-corrupta` | **OK**: rc 1, `se esperaba un directorio .escritor real`; no se renombra nada. |
| `indice.sqlite` como directorio, proyecto `b-corrupta` | **OK**: rc 1, rechazo de ruta operativa; no se renombra el directorio. |
| `operaciones.log` con enlace duro a víctima externa, proyecto `c-corrupta` | **OK**: rc 1, `se rechaza una ruta operativa con enlaces duros`; la víctima conserva `SENTINELA-LOG`. |
| `indice.sqlite` como enlace simbólico a archivo externo, proyecto `d-corrupta` | **OK**: rc 1, `se rechaza una ruta operativa enlazada`; el índice externo queda intacto. |
| `.escritor` enlazado y **`indice.sqlite` externo genuinamente corrupto** + `--limpiar` | **OK**: rc 1 antes de abrir el índice; no se crea `.corrupto-*` fuera. |
| Índice corrupto **dentro** de un proyecto llamado `e-corrupta` | **OK**: rc 0, reconstruye y deja `indice.sqlite.corrupto-<ts>` **dentro** de `.escritor`, como reparación legítima. |

La corrección es la esperada: `Project::reindexar` ya no captura cualquier error de `escritura_con(true)`; `parece_corrupto` exige `Error::Index`, y `abrir_indice` aparta el índice solo desde el `escritor` canónico que ya pasó `directorio_real` y `archivo_real`. El P2 de la r2 **no se reproduce**. Queda una observación P4: `parece_corrupto` sigue decidiendo por subcadenas inglesas (`malformed`, `not a database`, `corrupt`, `disk image`); sería más robusto usar `rusqlite::ErrorCode` (`DatabaseCorrupt`, `NotADatabase`) o un error tipado.

### SEC-02-H — cerrado en Linux; residual Windows

Probé enlaces duros en cada ruta operativa y en cada comando pedido. En todos los casos la validación rechazó antes de abrir/truncar y el archivo víctima conservó bytes y hash.

| Ruta operativa con enlace duro | Comandos | Resultado |
|---|---|---|
| `.escritor/bloqueo` | `info`, `write`, `reindex --limpiar` | **OK**: rc 1, `se rechaza una ruta operativa con enlaces duros`; víctima de 22 bytes intacta. |
| `.escritor/operaciones.log` | `write`, `info`, `reindex --limpiar` | **OK**: rc 1; víctima de 616 bytes intacta. |
| `.escritor/indice.sqlite` | `info`, `write`, `reindex --limpiar`, `buscar` | **OK**: rc 1; víctima de 122.880 bytes intacta. |
| `.escritor/indice.sqlite-wal` | `info`, `reindex --limpiar` | **OK**: rc 1; víctima de 13 bytes intacta. |
| `.escritor/indice.sqlite-shm` | `info` | **OK**: rc 1; víctima de 13 bytes intacta. |
| Ambos sidecars a la vez | `write` | **OK**: rc 1; ninguna víctima cambia. |
| Documento de `manuscrito/` con enlace duro | `write`, `move`, `delete` | **OK**: la escritura atómica reemplaza la entrada; el archivo externo conserva contenido y sigue existiendo tras el borrado. |
| `proyecto.yml` con enlace duro | `write` | **OK**: rc 0; `proyecto.yml` no se escribe y la víctima no cambia. |
| `.escritor/cuarentena/informe.jsonl` con enlace duro | `verify --cuarentena` | **OK**: la entrada del proyecto se reemplaza atómicamente; la víctima conserva su hash. |
| Copia de cuarentena con enlace duro | `quarantine discard` | **OK**: se elimina solo la entrada del proyecto; la víctima sigue existiendo. |
| Blob **referenciado** con enlace duro | `reindex --limpiar` | **OK**: 0 huérfanos; el blob sigue referenciado y la víctima no cambia. |
| Blob **huérfano** con enlace duro | `reindex --limpiar` | **OK**: se elimina la entrada del proyecto (1 huérfano), la víctima externa sobrevive. |
| `.escritor/indice.sqlite-journal` con enlace duro o simbólico | `write`, `reindex`, `info` | **OK en cuanto a escritura externa**: en `write`/`reindex` SQLite elimina la entrada del proyecto y no el archivo víctima; `info` no abre el índice y la deja como está. No se reproduce escritura fuera. |

La comprobación `nlink > 1` de `validar_escritor` cubre Unix y las rutas listadas. Quedan dos residuales:

- **Windows:** `es_enlace_duro` devuelve `false` fuera de Unix; los enlaces duros de Windows no se rechazan. No es una regresión de Linux ni se reprodujo (no hay ejecución Windows), pero la paridad de la garantía no existe todavía. Es el mismo SEC-02-H residual, P3.
- **Diagnóstico y falsos positivos (P4):** el chequeo de `nlink` se ejecuta antes que el de tipo. Un `indice.sqlite` que es un directorio (nlink ≥ 2) se reporta como «enlaces duros» en vez de «tipo incorrecto», y un enlace duro legítimo a una copia de seguridad dentro del proyecto se rechaza. Es conservador y seguro, pero conviene reordenar tipo → nlink y documentar el falso positivo.

### SEC-04 FIFO — ruta de efecto cerrada; SEC-04 sigue abierto en blobs y `-journal`

Reproduje el caso de la r2 con un `PREPARE` pendiente válido y un FIFO como componente final de la ruta de efecto:

```text
WRITER_FAULT=prepare writer write <ID> p --texto pendiente   # deja PREPARE pendiente
rm p/manuscrito/01-libro/01-capitulo/001-primera-escena.md
mkfifo p/manuscrito/01-libro/01-capitulo/001-primera-escena.md
timeout 3 writer verify p
```

Resultado: **rc 1 en ~11 ms**, con `ERROR: integridad: la ruta no es un archivo regular: ...` y `ERROR: el proyecto no tiene escenas`; no hay cuelgue. `info`, `list` y `write` también salen rc 1 de inmediato.

| Caso SEC-04 | Resultado |
|---|---|
| FIFO como ruta de efecto final con `PREPARE` pendiente | **OK**: `verify`, `info`, `list` y `write` fallan rápido (rc 1). |
| FIFO en `manuscrito/` sin `PREPARE` | **OK**: se ignora en el escaneo; `verify` rc 0, sin cuelgue. |
| Socket Unix como ruta de efecto final con `PREPARE` pendiente | **OK**: `verify` rc 1, `la ruta no es un archivo regular`; sin cuelgue. |
| FIFO como `proyecto.yml` | **OK**: `verify` rc 1, `falta proyecto.yml`; sin cuelgue. |
| FIFO como `bloqueo`, `operaciones.log`, `indice.sqlite`, `-wal` o `-shm` | **OK**: `info` rc 1, `tipo incorrecto en la ruta operativa`; sin cuelgue. |
| Nodo de dispositivo | **No ejecutado**: `mknod` devuelve `EPERM` como usuario sin privilegios. `/dev/null` como enlace simbólico se ignora en el escaneo. |
| **FIFO como blob referenciado** | **Falla**: cuelga `verify`, `info`, `list`, `write` y `show`; ver SEC-04-B. |
| **FIFO como `indice.sqlite-journal`** | **Falla**: cuelga `write`, `reindex` y `buscar`; ver SEC-04-C. |

La ruta de efecto está cerrada: `ruta_del_proyecto` exige archivo regular en el componente final y `hash_archivo` lo revalida. Pero el arreglo no cubre los blobs ni el journal de SQLite, y los límites de tamaño siguen pendientes (P3).

### SEC-03-R — cerrado

| Comprobación | Resultado |
|---|---|
| `.gitleaks.toml` ausente | **OK**: `test ! -e .gitleaks.toml`; el gate usa la configuración por defecto. |
| `.gitleaksignore` | **OK**: solo dos fingerprints exactos, uno por cada falso positivo histórico; no hay allowlist por ruta, regla ni patrón. |
| `gitleaks git . --redact --no-banner` | **OK**: 24 commits, ~1,28 MB, sin hallazgos. |
| `gitleaks dir` sobre copia de `git ls-files` | **OK**: ~1,26 MB, sin hallazgos; sin `target/` ni `node_modules/`. |
| Gate sin `zizmor` en `PATH` | **OK**: con `HOME` temporal y un `PATH` que contiene `cargo`, `cargo-deny`, `gitleaks` y `git` pero no `zizmor`, `seguridad` termina 0 y muestra `zizmor: sin .github/ ni workflows; se omite`. |
| Clon sin `.gitleaksignore` | **Confirma el alcance**: `gitleaks git` encuentra exactamente los dos falsos positivos históricos (dos literales de ruta consecutivos en `e2e-tauri/tauri.test.mjs` y en el informe r2); no hay un tercer hallazgo. |

El gate de seguridad está verde sin `.gitleaks.toml` y `zizmor` ya no se exige si no hay `.github/`. Quedan dos observaciones P4:

- Si existiera `.github/` **sin workflows**, `_exigir_herramientas` volvería a exigir `zizmor` y `zizmor .` saldría 3 (`no inputs collected`). Hoy no aplica porque no hay `.github/`, pero el caso «directorio existe, workflows no» no está cubierto.
- Los fingerprints de `.gitleaksignore` incluyen commit, archivo, regla y línea. Son exactos —lo correcto para no ocultar secretos— pero un rebase, squash o edición del informe los invalida y el gate vuelve a fallar por el mismo falso positivo. Conviene un `gitleaks:allow` en la línea concreta o documentar la fragilidad.

## 3. Gate de seguridad

| Comprobación | Comando | Resultado |
|---|---|---|
| Tests Rust | `nice -n 19 cargo test --workspace --all-targets --locked` | **OK**: **92** pruebas (25 CLI + 34 unitarias + 33 de integración), 0 fallos. |
| Calidad | `nice -n 19 python3 scripts/ci.py calidad` | **OK** (fmt, clippy con `-D warnings`, tests). |
| Producto | `nice -n 19 python3 scripts/ci.py producto` | **OK**: fixture 3 escenas/18 notas/21 ids, determinismo, 12 errores y 4 avisos esperados, 1.000 escenas en 0,2 s, CLI Rust y enlaces. |
| Seguridad | `nice -n 19 python3 scripts/ci.py seguridad` | **OK**: `cargo-deny` núcleo y desktop, gitleaks historial/árbol, zizmor omitido. |
| `cargo-deny` núcleo | incluido en el gate | `advisories ok, bans ok, licenses ok, sources ok`; aviso de ignore no usado para `RUSTSEC-2024-0370`. |
| `cargo-deny` desktop | incluido en el gate | `advisories ok, bans ok, licenses ok, sources ok`; solo avisos de versiones duplicadas. |
| gitleaks historial | incluido en el gate | 24 commits, 0 hallazgos. |
| gitleaks árbol versionado | incluido en el gate | 0 hallazgos. |
| zizmor | incluido en el gate | Omitido: no existe `.github/`. |
| Gate sin zizmor | `PATH` sin `zizmor` | **OK**: termina 0 y omite zizmor explícitamente. |

No ejecuté `interfaz`, `e2e`, `e2e-tauri` ni `empaquetado`: la instrucción prohíbe GUI/escritorio y no eran necesarios para esta revisión. `calidad` y los tests cubren el núcleo y la CLI.

## 4. Pruebas adversariales

Todas en `/tmp/opencode/sec-r3`; la CLI es `target/debug/writer` y se ejecutó con `nice -n 19`. No se tocó el repositorio.

| # | Prueba | Resultado |
|---|---|---|
| 1 | Regresión SEC-01: `PREPARE` pendiente y documento convertido en enlace simbólico a víctima externa | **OK**: `verify` rc 1, `la ruta del proyecto contiene un enlace simbólico`; la víctima conserva su hash y sigue existiendo. |
| 2 | `.escritor`, log, lock, objetos, cuarentena, índice y sidecars enlazados | **OK**: rechazados antes de tocar el exterior; digests externos intactos. |
| 3 | Enlaces duros en lock, log, índice, WAL, SHM y sidecars | **OK en Linux**: rechazados; víctimas intactas (tabla §2). |
| 4 | Enlaces duros en documentos: `write`, `move`, `delete` | **OK**: la escritura atómica y el renombrado/borrado solo afectan a la entrada del proyecto; la víctima no cambia. |
| 5 | `proyecto.yml` con enlace duro y con enlace simbólico | **OK**: el duro se lee sin escribirse; el simbólico se rechaza como archivo no real. |
| 6 | `informe.jsonl` y copia de cuarentena con enlace duro | **OK**: reemplazo atómico y borrado de la entrada del proyecto; la víctima sobrevive. |
| 7 | Blobs referenciados y huérfanos con enlace duro + `reindex --limpiar` | **OK**: el referenciado se conserva; el huérfano se elimina como entrada, no como archivo externo. |
| 8 | `indice.sqlite-journal` con enlace duro o simbólico | **OK en cuanto a escritura externa**: `write`/`reindex` eliminan la entrada del proyecto; `info` no abre el índice; la víctima no cambia. |
| 9 | `indice.sqlite-journal` como FIFO | **Falla**: cuelga `write`, `reindex` y `buscar`; SEC-04-C. |
| 10 | Blob referenciado como FIFO | **Falla**: cuelga `verify`, `info`, `list`, `write` y `show`; SEC-04-B. |
| 11 | FIFO/socket como ruta de efecto final con `PREPARE` pendiente | **OK**: fallan rápido con error de integridad; sin cuelgue. |
| 12 | Dispositivo como ruta de efecto | **No ejecutado**: `mknod` EPERM; `/dev/null` por enlace simbólico se ignora. |
| 13 | Exportación DOCX a un destino con enlace duro y `--forzar` | **OK**: reemplazo atómico; la víctima conserva bytes y el inodo de la entrada cambia. |
| 14 | Exportación DOCX a un enlace simbólico externo y `--forzar` | **Observación P4**: `preparar_destino` resuelve el enlace y escribe en el destino, que queda sobrescrito. Es la semántica de escribir a través de un enlace elegido por el usuario, pero un atacante local con permiso en el directorio destino puede usar el symlink para hacer que writer sobrescriba otro archivo escribible por el usuario. |
| 15 | `reindex` en proyectos `*-corrupt*`, `*-malformed*`, `not a database`, `disk image` con `.escritor` enlazado | **OK**: rechazo; ningún `.corrupto-*` fuera. |
| 16 | Índice corrupto real dentro del proyecto | **OK**: se aparta dentro de `.escritor` y se reconstruye. |
| 17 | TOCTOU entre validación y `open`/`rename` | **Residual documentado**: no hay `openat`/`O_NOFOLLOW` ni comprobación de inodo; un proceso local puede sustituir un ancestro en la ventana. No se reprodujo de forma determinista. |

## 5. Hallazgos nuevos

### SEC-04-B — [P3] Un FIFO en `.escritor/objetos/<hash>` referenciado por un blob cuelga `verify`, `info`, `list`, `write` y `show`

- **Esperado:** que un proyecto de terceros no pueda colgar la apertura, la verificación ni la lectura del historial con un archivo especial; es la misma garantía que SEC-04 exige para las rutas de efecto.
- **Real:** `validar_escritor` comprueba que `objetos` sea un directorio real, pero no inspecciona su contenido. `BlobStore::obtener` y `obtener_opcional` llaman a `fsutil::leer_opcional`, que hace `std::fs::read` sin exigir archivo regular. Si un blob referenciado es un FIFO, la lectura se bloquea.
- **Reproducción:** proyecto con un `PREPARE` pendiente creado con `WRITER_FAULT=after_write:1`; se sustituye el blob de `hash_anterior` por un FIFO y se ejecuta:

```text
timeout 3 writer verify p     # rc 124
timeout 3 writer info p       # rc 124
timeout 3 writer list p       # rc 124
timeout 3 writer write <ID> p --texto x   # rc 124
timeout 3 writer show 1 p     # rc 124 (también sin PREPARE pendiente)
```

- **Impacto:** denegación de servicio local al abrir/verificar/escribir/mostrar un proyecto manipulado; no hay pérdida de datos ni escritura fuera. Un FIFO puede llegar en un tar; un ZIP normal no lo conserva.
- **Corrección sugerida:** centralizar una lectura de archivo regular (`symlink_metadata` + `is_file`, rechazo de FIFO/dispositivo/socket y límite de tamaño) y usarla en `BlobStore::obtener`/`obtener_opcional`, `fsutil::leer_opcional`, `Log::cargar`, `verify::escanear`, `cuarentena` y `docx`. Añadir una regresión con un FIFO como blob y un `PREPARE` pendiente.

### SEC-04-C — [P3] Un FIFO en `.escritor/indice.sqlite-journal` cuelga `write`, `reindex` y `buscar`

- **Esperado:** que cualquier ruta operativa de SQLite se valide antes de abrirla; `validar_escritor` ya valida `indice.sqlite`, `-wal` y `-shm`.
- **Real:** `indice.sqlite-journal` no está en la lista de `validar_escritor` ni en la limpieza de `Indice::apartar`. SQLite intenta abrir/crear el journal y, si es un FIFO, el `open` se bloquea.
- **Reproducción:**

```text
mkfifo p/.escritor/indice.sqlite-journal
timeout 3 writer write <ID> p --texto x   # rc 124
timeout 3 writer reindex p                # rc 124
timeout 3 writer buscar texto p           # rc 124
writer info p                             # rc 0: no abre el índice
writer verify p                           # rc 0: no abre el índice si no hay recuperación pendiente
```

- **Impacto:** denegación de servicio local en las operaciones que abren el índice; no hay escritura fuera. Un `-journal` con enlace duro o simbólico sí se elimina como entrada del proyecto sin tocar la víctima en las pruebas; el problema es el tipo FIFO y la falta de validación.
- **Corrección sugerida:** añadir `("indice.sqlite-journal", false)` a `validar_escritor`, incluirlo en la limpieza de `Indice::apartar` y añadir una regresión con FIFO. Conviene que el chequeo de tipo preceda al de `nlink` para que el diagnóstico sea correcto.

### SEC-02-K — [P4] El chequeo de `nlink` precede al de tipo y rechaza enlaces duros legítimos

- **Real:** `validar_escritor` comprueba `nlink > 1` antes de comprobar si la ruta operativa es archivo o directorio. Un `indice.sqlite` que es un directorio se reporta como «enlaces duros»; un enlace duro legítimo a una copia de seguridad dentro del proyecto se rechaza aunque no haya riesgo externo.
- **Impacto:** diagnóstico engañoso y falsos positivos; no abre un bypass (el rechazo es conservador).
- **Corrección:** comprobar primero el tipo esperado y después `nlink`; documentar que los enlaces duros se rechazan incluso si el otro extremo está dentro del proyecto.

### SEC-03-R2 — [P4] `.github/` sin workflows y fingerprints de gitleaks atados a commit/línea

- **Real:** `_exigir_herramientas` exige `zizmor` si existe el directorio `.github`, aunque no haya workflows; en ese caso `zizmor .` sale 3. Los fingerprints de `.gitleaksignore` incluyen commit y línea, por lo que un rebase o una edición del informe invalidan la excepción y el gate vuelve a fallar por el mismo falso positivo.
- **Impacto:** reproducibilidad del gate, no seguridad del producto.
- **Corrección:** exigir `zizmor` solo si hay workflows (`*.yml`/`*.yaml` bajo `.github/workflows`); usar `gitleaks:allow` en la línea concreta o documentar la fragilidad.

### SEC-02-L — [P4] `parece_corrupto` sigue basado en subcadenas inglesas

- **Real:** aunque ahora exige `Error::Index`, decide con `texto.contains("malformed")`, `"not a database"`, `"corrupt"` y `"disk image"`. Una versión de SQLite con otro mensaje, o un error tipado que se convierta a `Other` con `with_path`, dejaría de reconocer la corrupción en la ruta de `reconstruir`.
- **Impacto:** robustez de la reparación; no es un bypass de escritura fuera. `Indice::abrir` sigue apartando en el `open`, así que el caso principal está cubierto.
- **Corrección:** usar `rusqlite::ErrorCode::{DatabaseCorrupt, NotADatabase}` o un error propio tipado.

### SEC-05-B — [P4] La salida CLI no sanea caracteres de control de nombres de archivo o del informe de cuarentena

- **Real:** `verify` imprime rutas relativas tomadas del sistema de archivos y `cuarentena` imprime `ruta_original`/`motivo` leídos de `informe.jsonl`; ninguno filtra secuencias ANSI/controles. Un proyecto con un archivo cuyo nombre contenga `ESC [` puede inyectar secuencias en la terminal. La UI React escapa el texto, así que el riesgo es de terminal/CLI.
- **Impacto:** manipulación de la salida de la terminal; P4. No hay ejecución ni escritura.
- **Corrección:** escapar o eliminar caracteres de control en la salida legible (no en el JSON), o usar un escritor que los represente.

### SEC-02-M — [P4] `proyecto.yml` no se valida en `validar_escritor`

- **Real:** `proyecto.yml` no está en la lista de rutas operativas. Un enlace simbólico se rechaza indirectamente en `verify::leer_proyecto` (`archivo_real`), y un enlace duro solo se lee (no se escribe), pero la validación no es explícita ni cubre el caso antes de `escritura_con`.
- **Impacto:** consistencia y defensa en profundidad; no se reprodujo escritura fuera.
- **Corrección:** validar `proyecto.yml` como archivo real (sin enlaces) al abrir una sesión de escritura, o documentar por qué queda fuera.

### SEC-02-N — [P4] La exportación a un enlace simbólico externo con `--forzar` sigue el enlace

- **Real:** `preparar_destino` resuelve el enlace y devuelve el destino; `escribir_atomico` reemplaza el archivo destino. Un atacante local con permiso en el directorio donde el usuario exporta puede plantar un symlink y hacer que writer sobrescriba otro archivo escribible por el usuario.
- **Impacto:** requiere una acción explícita del usuario (`--forzar`) y un directorio controlado por el atacante; no afecta a la garantía «no escribe fuera del proyecto» de las operaciones internas. P4.
- **Corrección:** rechazar un destino que sea symlink (o reemplazar el symlink en vez de seguirlo) y documentar la semántica.

### Vigentes (no nuevos)

- **SEC-04, límites de tamaño — [P3]:** `hash_archivo`, `Log::cargar`, `verify::escanear`, `cuarentena` y `docx` siguen leyendo archivos completos sin límite. Un archivo enorme o un proyecto con miles de documentos puede agotar memoria/tiempo. No se reprodujo un fallo nuevo de escritura; es deuda de recursos.
- **SEC-02-H Windows — [P3 residual]:** los enlaces duros de Windows no se rechazan (`es_enlace_duro` devuelve `false` fuera de Unix). No hay ejecución Windows; la defensa es symlinks/junctions.
- **TOCTOU — [P3 residual]:** entre la validación y el `open`/`rename` no hay `openat`/`O_NOFOLLOW` ni comprobación de inodo. Un proceso local puede sustituir un ancestro. Está documentado en `HANDOFF.md` y `README.md`.
- **Blob con enlace simbólico o duro — [P4 residual]:** `BlobStore::obtener` lee el destino; no se escribe fuera, pero un blob enlazado a un archivo externo legible se puede leer si el hash coincide. Requiere crear el enlace localmente o extraer un tar que lo preserve.
- **SEC-05 — [P4]:** CSP con `style-src 'unsafe-inline'`, sin `object-src`/`base-uri`/`form-action`/`frame-ancestors`; `dialog:default` y `core:default` amplios; los comandos IPC aceptan cualquier ruta absoluta y no la ligan a un proyecto abierto. No hay XSS conocido.
- **SEC-06 — [P4]:** deriva documental; ver §7.
- **`RUSTSEC-2024-0370` — [P4]:** `proc-macro-error` unmaintained por gtk-rs/Tauri; excepción documentada en `deny.toml`, sin reemplazo compatible con Tauri 2.12.

**Resumen de severidades de esta r3:** P1: 0 · P2: 0 · P3 nuevos: 2 (SEC-04-B, SEC-04-C) · P3 residuales: 2 (SEC-02-H Windows, SEC-04 tamaño) · P4 nuevos: 6 (SEC-02-K, SEC-03-R2, SEC-02-L, SEC-05-B, SEC-02-M, SEC-02-N) · P4 vigentes: SEC-05, SEC-06 y `RUSTSEC-2024-0370`.

## 6. Dependencias y cadena de suministro

| Área | Estado |
|---|---|
| Lockfiles | `Cargo.lock`, `apps/desktop/src-tauri/Cargo.lock` y `pnpm-lock.yaml` **sin cambios** entre la r2 y este commit; los gates usan `--locked`. No hay `package-lock.json` ni `yarn.lock`. |
| `cargo-deny` núcleo | **Verde**: advisories, bans, licenses y sources. Aviso de ignore no usado para `RUSTSEC-2024-0370` (no aparece en el grafo del núcleo). |
| `cargo-deny` desktop | **Verde** con la misma política. `RUSTSEC-2024-0370` llega por gtk-rs/Tauri; solo avisos de versiones duplicadas. |
| pnpm | `packageManager` fija pnpm 12.14.0; `pnpm-workspace.yaml` autoriza el build solo de `esbuild` y desactiva `edgedriver`/`geckodriver`. No se usa npm/npx. |
| Plugins Tauri | Solo `dialog` explícito; `tauri-plugin-fs` es transitivo de `dialog` y no se inicializa ni tiene capacidades `fs`. No hay `shell`, `http` ni updater. |
| Secretos | gitleaks sobre 24 commits y sobre el árbol versionado: 0 hallazgos. `.gitleaksignore` solo contiene dos fingerprints exactos de falsos positivos históricos. Sin `.env`, claves ni tokens versionados. |
| Workflows | No existe `.github/`; `zizmor` se omite. No hay CI remoto ni Dependabot. |
| `unsafe` | `#![forbid(unsafe_code)]` en `crates/core/src/lib.rs`; no encontré `Command::new` con entrada de usuario ni interpolación de shell en el producto. |
| Herramientas de desarrollo | Playwright/Chromium, `tauri-driver` y `WebKitWebDriver` se instalan fuera del repo sin hash de artefacto; es una recomendación P4 ya presente en r1/r2. El binario de gitleaks no reporta número de versión, lo que dificulta reproducir el gate exacto. |

No hay una dependencia vulnerable conocida en lo revisado. El punto débil de proceso es que el gate no cubre Windows ni la app real; el acta ya lo declara.

## 7. Documentación vs. realidad

| Documento / afirmación | Realidad observada |
|---|---|
| `README.md:32-33`: «92 pruebas (33 unitarias, 28 de integración y 25 de CLI)» | El desglose suma 86, no 92. El árbol tiene **34 unitarias + 33 de integración + 25 CLI = 92**. `HANDOFF.md:17` sí tiene el desglose correcto. |
| Acta r6 y `docs/iteraciones/12-fase-2.md`: SEC-04 FIFO corregido; «un FIFO ya no cuelga `verify`» | Cierto para **rutas de efecto**. Un FIFO como **blob** referenciado cuelga `verify`/`info`/`list`/`write`/`show` (SEC-04-B) y un FIFO como `indice.sqlite-journal` cuelga `write`/`reindex`/`buscar` (SEC-04-C). La afirmación necesita el matiz. |
| `docs/protocolo-commit.md:38-40` y `:153-154`: `.escritor`, log y lock «no enlaces» | Cierto para enlaces simbólicos y, en Unix, para enlaces duros; en Windows los enlaces duros no se rechazan. La documentación no menciona la excepción Windows. |
| `docs/ci.md` §1: si no existe `.github/`, el gate omite `zizmor` | Cierto en ejecución. No cubre el caso `.github/` existente sin workflows, que volvería a exigir `zizmor` y fallaría. |
| `HANDOFF.md:290-291`: gitleaks y zizmor instalados; el gate ejecuta ambos (zizmor omitido sin `.github/`) | Cierto en este entorno; el gate terminó 0. |
| `README.md:270`: se rechazan enlaces simbólicos en componentes existentes | Cierto en Linux; además se rechazan enlaces duros operativos en Unix. La redacción no promete enlaces duros. |
| `docs/cuarentena.md`: el informe se reemplaza atómicamente y un enlace duro no modifica otro archivo | Cierto: `informe.jsonl` con enlace duro conserva la víctima; la copia con enlace duro se elimina solo como entrada del proyecto. |
| `docs/esquema-v1.md` y fixture roto | Sin regresión: 12 errores y 4 avisos esperados; `producto` verde. |
| `README.md:26`: fase 2 pendiente de seguridad/release/UX | Correcto antes de esta r3; la fase sigue sin aprobación del product owner. |
| `docs/revisiones/README.md` | No incluye todavía la r3; el builder deberá actualizar el estado y el acta. |

## 8. Conclusión y decisión

**Decisión: aprobado con observaciones.**

Las correcciones de la r6 cierran el bloqueo de la r2:

- **SEC-02-R:** cerrado. `reindex` rechaza `.escritor` enlazado antes de apartar el índice, con proyectos llamados `novela-corrupta`, `novela-malformed`, `not a database`, `disk image` y `corrupt`, con y sin `--limpiar`, en `reindex`, `info` y `write`. No se renombra ni se borra nada fuera; un índice corrupto real se repara dentro de `.escritor`.
- **SEC-02-H:** cerrado en Linux para `bloqueo`, `operaciones.log`, `indice.sqlite`, `-wal`, `-shm` y sidecars, con `info`, `write` y `reindex --limpiar`; las víctimas conservan bytes y hashes. Queda el residual Windows de enlaces duros (P3) y el falso positivo/diagnóstico de `nlink` (P4).
- **SEC-04 FIFO en rutas de efecto:** cerrado; `verify` falla en milisegundos, no cuelga, y no escribe fuera. Un socket como ruta de efecto también se rechaza.
- **SEC-03-R:** cerrado. El gate termina 0 sin `.gitleaks.toml`, `.gitleaksignore` solo contiene dos fingerprints exactos y `zizmor` no se exige sin `.github/`.

Quedan dos **P3 nuevos** que no bloquean según la tabla de severidades, pero que deben registrarse y corregirse antes de aceptar proyectos de terceros a gran escala:

1. **SEC-04-B:** un FIFO en `.escritor/objetos/<hash>` referenciado por un blob cuelga `verify`, `info`, `list`, `write` y `show`.
2. **SEC-04-C:** un FIFO en `.escritor/indice.sqlite-journal` cuelga `write`, `reindex` y `buscar`.

Y residuales P3 ya conocidos: límites de tamaño (SEC-04), enlaces duros en Windows y TOCTOU. Los P4 nuevos son de robustez, diagnóstico, documentación o semántica de exportación; ninguno afecta a la garantía de las operaciones internas del proyecto (SEC-02-N es una escritura al destino de exportación elegido por el usuario).

Para la siguiente iteración (no bloqueante para el pipeline, salvo que el product owner exija paridad Windows):

1. Añadir `indice.sqlite-journal` a `validar_escritor` y a la limpieza de `Indice::apartar`; reordenar tipo → `nlink` (SEC-04-C, SEC-02-K).
2. Hacer que `BlobStore::obtener`/`obtener_opcional` exijan archivo regular y límite de tamaño; añadir regresión con FIFO y `PREPARE` pendiente (SEC-04-B).
3. Decidir Windows: `number_of_links` o aceptar y documentar el residual de enlaces duros (SEC-02-H).
4. Corregir el desglose de pruebas de `README.md:32-33` y matizar la afirmación de SEC-04 en el acta r6 y en la iteración 12 (SEC-06).
5. Sustituir `parece_corrupto` por un código de error tipado de SQLite (SEC-02-L) y, si se toca el gate, cubrir `.github/` sin workflows y la fragilidad de los fingerprints (SEC-03-R2).
6. Mantener SEC-05 (CSP/capacidades/IPC) como P4 hasta la revisión de release.

No hay P1 ni P2 abiertos en este commit. La fase puede avanzar a release/UX con las observaciones anteriores registradas.
