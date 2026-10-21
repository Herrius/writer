# Fase 2 — Security review (r2)

- **Commit revisado:** `7b7734418b11fca4f2f29fd14639679b23e2cf90`
- **HEAD del pipeline:** `0b7457653e208dc47344873dad3d7335917bd15d`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64; Rust 1.99.1; `cargo-deny` 0.20.2; `zizmor` 1.31.1; gitleaks local (el binario no reporta número de versión); pnpm 12.14.0; Node 26.11.1; Python 3.14.8.
- **Árbol:** limpio al empezar y al terminar (`git status --porcelain` vacío). `git diff --stat 7b77344..HEAD` solo muestra `docs/revisiones/README.md` y `docs/revisiones/fase-2-00-congelado-r5.md`; no hay cambios de código, tests ni configuración.
- **Decisión:** **bloqueado**

## 1. Alcance y método

Revisé el artefacto congelado, no la justificación del builder. El modelo de amenaza es el mismo de la r1: proyecto de terceros manipulado (Markdown, `proyecto.yml`, `operaciones.log`, blobs, índice, cuarentena), enlaces simbólicos y duros, rutas de Windows (`\`, `C:`, UNC, `..\`), Unicode/controles, YAML/OOXML malformados, agotamiento de recursos y superficie Tauri.

No ejecuté la GUI ni la app de escritorio. Usé la CLI `target/debug/writer` (comparte `writer-core` con los comandos Tauri), análisis estático de `lib.rs`, `tauri.conf.json` y `capabilities/default.json`, y un arnés propio en `/tmp/opencode/sec-r2`. El arnés incluye un generador independiente de checksums BLAKE3 (`forge`, fuera del repositorio) para fabricar logs válidos sin usar código del producto.

La comprobación de estado fue:

```text
git status --porcelain                         vacío
git rev-parse HEAD                             0b7457653e208dc47344873dad3d7335917bd15d
git diff --stat 7b77344..HEAD                  solo docs/revisiones/
```

## 2. Verificación de SEC-01/02/03

### SEC-01 — Rutas de efecto y enlaces

**Estado: cerrado en Linux para enlaces simbólicos y rutas léxicas; residual TOCTOU y validación Windows no ejecutada.**

Probé un `PREPARE` pendiente fabricado con checksum válido y un blob propio, sobre copias de un proyecto `init` válido. En todos los casos el error fue visible y `verify` salió con código 1 (ya no sale 0 como en la r1):

| Caso | Resultado |
|---|---|
| Enlace en `manuscrito/01-libro/01-capitulo/extra` → directorio externo; efecto `.../extra/pwned.txt` | **OK**: `ERROR: integridad: la ruta del proyecto contiene un enlace simbólico`; no se creó el archivo externo. |
| `manuscrito` enlazado a un directorio externo; efecto dentro de `manuscrito/...` | **OK**: enlace ignorado en el escaneo y rechazado en la recuperación; sin escritura externa. |
| `notas` enlazado a un directorio externo | **OK**: rechazado; sin escritura externa. |
| Enlace en el componente final (`.../pwned.txt` → archivo externo existente) | **OK**: rechazado; el archivo externo conservó `SENTINELA-EXTERNO`. |
| Enlace final roto (`.../pwned.txt` → ruta externa inexistente) | **OK**: rechazado; no se creó nada. |
| `..\..\fuera\pwned.txt`, `C:\tmp\pwned.txt`, `\\servidor\recurso\pwned.txt` | **OK**: `ruta de efecto no válida` al cargar el log; sin escritura. |
| `manuscrito/../../fuera/pwned.txt`, `manuscrito/.../../../../pwned.txt` | **OK**: rechazado por componente `..`. |
| `proyecto.yml`, `.escritor/operaciones.log`, `otro/pwned.md` como efecto | **OK**: rechazado por `validar_ruta_documento`; solo `manuscrito/` y `notas/`. |
| Nombres reservados (`CON.md`, `AUX.md`, `COM1.md`, `LPT9.md`), punto/espacio final, `:`, componente vacío, ruta absoluta | **OK**: rechazados. |
| Control `U+0001` en la ruta | **OK**: rechazado por `char::is_control`. |
| `%2e%2e%2fpwned.md` y `U+202E` en el nombre | **OK**: no escapan; se crean como nombres literales **dentro** de `manuscrito/01-libro/01-capitulo/` y el escaneo los marca inválidos. |
| Enlace duro en el componente final de una escena, seguido de `write` | **OK**: la escritura atómica reemplaza la entrada y **no** modifica el archivo externo. |

La recuperación valida todas las rutas de todos los `PREPARE` pendientes en el preflight antes de escribir; un enlace en cualquier efecto impide la recuperación completa. `Project::verificar` devuelve el error en `escaneo.errores`, y `abrir_proyecto` de Tauri llama a `verificar` (`lib.rs:230-263`), por lo que la UI recibiría el error en el DTO. No ejecuté la GUI.

Quedan dos residuales:

- **TOCTOU**: entre `ruta_del_proyecto`/`validar_escritor` y el `open`/`rename` hay una ventana en la que un proceso local puede sustituir un ancestro por un enlace. Está documentado en el acta r5 y en `README.md`; no lo cuento como hallazgo nuevo, pero sigue siendo real (P3 residual).
- **Windows**: no hay ejecución en Windows. La validación léxica de `\`, `C:`, UNC y nombres reservados está probada en Linux a nivel de cadena. En std de Rust actual, `FileType::is_symlink` marca como enlace los reparse points con el bit name-surrogate, incluido el tag estándar de junction (`IO_REPARSE_TAG_MOUNT_POINT`, `0xA0000003`), pero falta la prueba runtime y no se cubren reparse points de filtros de terceros. Lo registro como P4 de validación, no como bypass reproducido.

### SEC-02 — `.escritor` y rutas operativas

**Estado: cerrado en Linux para enlaces simbólicos; abierto por un camino de error de `reindex` (P2) y por enlaces duros (P3).**

Probé `.escritor` y cada ruta operativa enlazada. Todos los comandos rechazaron antes de tocar el exterior y los digests externos no cambiaron:

| Ruta enlazada | Comandos | Resultado |
|---|---|---|
| `.escritor` → directorio externo | `info`, `verify`, `list`, `reindex --limpiar`, `write` | **OK**: `se rechaza .escritor enlazado`; log y lock externos intactos. |
| `.escritor/operaciones.log` → archivo externo | `info`, `verify` | **OK**: `se rechaza una ruta operativa enlazada`; víctima intacta. |
| `.escritor/bloqueo` → archivo externo | `info` | **OK**: rechazado antes del lock; víctima intacta. |
| `.escritor/objetos` → directorio externo | `info`, `verify`, `reindex --limpiar` | **OK**: rechazado; el blob externo no se borró. |
| `.escritor/cuarentena` → directorio externo | `verify --cuarentena`, `quarantine list` | **OK**: `se esperaba un directorio real`; no se copió nada fuera. |
| `.escritor/indice.sqlite` → archivo externo | `info`, `list`, `buscar`, `write` | **OK**: rechazado; SQLite externo intacto. |
| `.escritor/indice.sqlite-wal` → archivo externo | `info` | **OK**: rechazado; víctima intacta. |
| `.escritor/objetos/<hash>` → archivo externo | `reindex --limpiar`, `verify` | **OK**: se elimina solo el enlace; el archivo externo permanece. Un blob enlazado y referenciado se leería a través del enlace, pero no se escribe fuera. |
| `.escritor` como archivo, `objetos` como archivo, `indice.sqlite` como directorio | `info` | **OK**: rechazados por tipo. |
| `proyecto.yml` enlazado | `verify` | **OK**: `se esperaba un archivo real`; no se escribió. |

**Pero `reindex` tiene un bypass determinista (P2).** `Project::reindexar` captura *cualquier* error de `escritura_con(true)` cuyo texto contenga `malformed`, `not a database`, `corrupt` o `disk image`, y antes de revalidar llama a `Indice::apartar(&self.ruta_escritor().join("indice.sqlite"))` (`crates/core/src/project.rs:350-357`, `:1475-1481`). Si el proyecto se llama, por ejemplo, `novela-corrupta`, el error legítimo `se rechaza .escritor enlazado: .../novela-corrupta/.escritor` contiene `corrupt`, y `apartar` opera a través del `.escritor` enlazado:

```text
/tmp/opencode/sec-r2/corrupt-reindex/novela
  .escritor -> /tmp/opencode/sec-r2/corrupt-reindex/fuera/escritor-real
  fuera/escritor-real/indice.sqlite       (15 bytes)
  fuera/escritor-real/indice.sqlite-wal   (12 bytes)
  fuera/escritor-real/indice.sqlite-shm   (12 bytes)

writer reindex corrupt-reindex/novela
  writer: integridad: se rechaza .escritor enlazado: corrupt-reindex/novela/.escritor
  rc=1

Después:
  fuera/escritor-real/indice.sqlite.corrupto-20261021T232102666Z  (15 bytes)
  fuera/escritor-real/indice.sqlite-wal                            AUSENTE
  fuera/escritor-real/indice.sqlite-shm                            AUSENTE
```

Es decir: renombra un archivo externo y **borra** sus sidecars, aunque el comando termina mostrando el error de `.escritor`. La condición de nombre es controlable por el atacante (`corrupta`, `malformed-...`, `not a database`, `disk image`). `reindex` no se ejecuta al abrir, pero es un comando de reparación documentado y el daño es determinista. Es SEC-02-R, P2.

**Enlaces duros (P3).** `validar_escritor` rechaza `is_symlink`, pero acepta un archivo regular que sea enlace duro. Reproduje:

| Ruta operativa | Antes | Después de la operación |
|---|---|---|
| `.escritor/bloqueo` enlace duro a víctima de 22 bytes | 22 bytes | 85 bytes con el JSON del lock (`info` sale 0). |
| `.escritor/operaciones.log` enlace duro a una copia válida del log | 616 bytes, 2 líneas | 1296 bytes, 4 líneas (`write` sale 0; el inodo es el mismo). |
| `.escritor/indice.sqlite` enlace duro a una copia del índice | hash `afc08a...` | hash `3ed1e5...` (`write` sale 0). |
| `.escritor/indice.sqlite-wal` enlace duro a un archivo externo | hash `afc08a...` | hash `663048...`; SQLite escribió en el WAL externo y luego desapareció la entrada del proyecto. |
| Escena de `manuscrito/` con enlace duro a un archivo externo | contenido externo | **sin cambios**; la escritura atómica reemplaza la entrada. |
| `cuarentena/informe.jsonl` con enlace duro | — | **sin cambios**; ya hay test y lo confirmé. |

El enlace duro requiere crearlo localmente o extraer un archivo que lo preserve (GNU tar con `-P` y un `linkname` absoluto lo permite; un ZIP normal no). Por eso lo clasifico P3 y no P2: no es el vector de un proyecto descargado en ZIP, pero sí un residual de la garantía «no escribe fuera» si el proyecto llega por una extracción que preserve enlaces duros o si hay un atacante local.

### SEC-03 — Gate de seguridad

**Estado: cerrado.** `python3 scripts/ci.py seguridad` termina 0 con `cargo-deny` (núcleo y desktop), `gitleaks git` (22 commits, sin hallazgos), `gitleaks dir` sobre una copia de `git ls-files` (1,46 MB, sin hallazgos) y `zizmor` omitido explícitamente porque no hay `.github/`.

Comprobé además que `gitleaks` sin `.gitleaks.toml` tampoco encuentra nada en los 22 commits ni en el árbol versionado. La allowlist actual (`001-primera-escena\.md`) es innecesaria con esta versión y es más ancha de lo que debería: un futuro secreto que contuviera ese literal quedaría silenciado. Es P4, no reabre SEC-03.

También comprobé que `_exigir_herramientas` exige `zizmor` **antes** de decidir omitirlo. Con un `PATH` que contiene `cargo-deny` y `gitleaks` pero no `zizmor`, el gate falla con `faltan herramientas de seguridad: zizmor` aunque no haya workflows. Es una incoherencia de reproducibilidad P4. Si existiera `.github/` sin workflows, `zizmor .` devolvería código 3; hoy no aplica.

## 3. Gate de seguridad

| Comprobación | Comando | Resultado |
|---|---|---|
| Tests Rust | `nice -n 19 cargo test --workspace --all-targets --locked` | **OK**: 25 CLI + 34 unitarias + 30 de integración = **89** pruebas; 0 fallos. |
| Calidad | `python3 scripts/ci.py calidad` | **OK** (fmt, clippy con `-D warnings`, tests). |
| Producto | `python3 scripts/ci.py producto` | **OK**: fixture, determinismo, 12 hallazgos, 1000 escenas en 0,3 s, CLI Rust y enlaces. |
| Seguridad | `python3 scripts/ci.py seguridad` | **OK**: `cargo-deny` núcleo/desktop, gitleaks historial/árbol, zizmor omitido. |
| `cargo-deny` núcleo | `cargo deny --all-features --manifest-path Cargo.toml check` | `advisories ok, bans ok, licenses ok, sources ok`; aviso de ignore no usado para RUSTSEC-2024-0370 y duplicados. |
| `cargo-deny` desktop | `cargo deny --manifest-path apps/desktop/src-tauri/Cargo.toml check` | `advisories ok, bans ok, licenses ok, sources ok`; solo avisos de versiones duplicadas. |
| gitleaks historial | incluido en el gate | 22 commits, 0 hallazgos. |
| gitleaks árbol versionado | incluido en el gate | 0 hallazgos; sin `target/` ni `node_modules/`. |
| zizmor | incluido en el gate | Omitido: no existe `.github/`. |

El conteo real es **89**, no 88: el acta r5 y `HANDOFF.md` siguen diciendo 88 (33 unitarias + 30 integración + 25 CLI). La prueba unitaria nueva `valida_rutas_sin_formas_de_windows` explica el +1. Es deriva documental P4.

No ejecuté `interfaz`, `e2e`, `e2e-tauri` ni `empaquetado`; la instrucción prohíbe GUI/escritorio y no eran necesarios para esta revisión. El gate `calidad` y los tests cubren el núcleo y la CLI.

## 4. Pruebas adversariales

Todas en `/tmp/opencode/sec-r2`; la CLI es `target/debug/writer`. No se tocó el repositorio.

| # | Prueba | Resultado |
|---|---|---|
| 1 | `PREPARE` pendiente con enlace en ancestro de `manuscrito/` | **OK**: error visible, sin escritura externa (SEC-01). |
| 2 | `manuscrito`/`notas` enlazados, enlace final y enlace final roto | **OK**: rechazados; sin escritura externa. |
| 3 | Matriz léxica Windows/`..`/reservados/controles/absolutas | **OK**: todos rechazados al cargar el log. |
| 4 | `%2e%2e%2f` y `U+202E` | **OK**: quedan como nombres literales dentro del proyecto; no escapan. |
| 5 | Enlace duro en escena final + `write` | **OK**: el archivo externo no cambia. |
| 6 | `.escritor`, log, lock, objetos, cuarentena, índice y sidecars enlazados | **OK**: rechazados; digests externos intactos. |
| 7 | Enlaces duros en lock, log, índice y WAL | **Falla**: SEC-02-H (P3); ver §2. |
| 8 | `reindex` con `.escritor` enlazado y ruta que contiene `corrupt` | **Falla**: SEC-02-R (P2); renombra/borra fuera. |
| 9 | FIFO como ruta de efecto + `PREPARE` válido; `timeout 3 writer verify` | **Falla**: cuelga (exit 124). SEC-04; no hay escritura externa, pero es DoS local determinista. |
| 10 | TOCTOU entre validación y escritura | **Residual documentado**: no hay `openat`/`O_NOFOLLOW` ni comprobación de inodo; un proceso local puede cambiar un ancestro en la ventana. |
| 11 | Exportación DOCX con `<`, `&`, `<script>`; `xmllint` y `unzip -t` | **OK**: XML bien formado, script escapado, ZIP íntegro. |
| 12 | DOCX con `U+0001` y destino existente con `--forzar` | **OK**: aborta; destino byte a byte intacto. |
| 13 | Exportación a destino dentro del proyecto y por alias simbólico | **OK**: rechazada; no se crea el archivo. |
| 14 | Ciclo crear escena/nota, escribir, mover, borrar, revertir, listar, buscar, exportar MD/DOCX, cuarentena y reindex | **OK**: sin regresiones; reversión reparadora y cuarentena correctas. |
| 15 | Inyección de fallos `prepare`, `after_write:1`, `commit`, `before_check` | **OK**: `prepare` y `after_write` recuperan al reabrir; `commit` persiste; `before_check` conserva el cambio externo y deja `ABORT` con conflicto. |
| 16 | Búsqueda tras edición externa | **OK**: reindexa y encuentra el texto nuevo. |
| 17 | `proyecto.yml` enlazado | **OK**: rechazado; sin escritura. |

El caso 9 se reproduce con:

```text
mkfifo .../manuscrito/01-libro/01-capitulo/fifo.md
# PREPARE válido con ruta manuscrito/01-libro/01-capitulo/fifo.md
timeout 3 writer verify ...   # exit 124
```

`ruta_del_proyecto` acepta el FIFO porque solo comprueba enlaces y, para componentes intermedios, que sean directorios; el componente final no se exige regular. `hash::hash_archivo` hace `std::fs::read` y se bloquea. Es la manifestación más directa de SEC-04.

## 5. Hallazgos nuevos

### SEC-02-R — [P2] `reindex` aparta el índice antes de revalidar `.escritor` y puede renombrar o borrar fuera del proyecto

- **Esperado:** que cualquier operación sobre `.escritor` enlazado se rechace antes de tocar rutas operativas, también en el camino de reparación del índice. `docs/protocolo-commit.md` §9 y el acta r5 presentan el rechazo como previo a abrir el lock/log/índice.
- **Real:** `Project::reindexar` usa `parece_corrupto(&error)` sobre el texto de **cualquier** error de `escritura_con(true)`. Si el texto contiene `malformed`, `not a database`, `corrupt` o `disk image`, llama a `Indice::apartar(&self.ruta_escritor().join("indice.sqlite"))` sin volver a validar `.escritor`. Con un proyecto llamado `novela-corrupta` y `.escritor` enlazado, el error `se rechaza .escritor enlazado: .../novela-corrupta/.escritor` contiene `corrupt`; `apartar` renombra el `indice.sqlite` externo a `indice.sqlite.corrupto-<ts>` y borra `indice.sqlite-wal`/`-shm` externos. Reproducido en `/tmp/opencode/sec-r2/corrupt-reindex`; el comando sale 1 y muestra el error, pero el daño ya está hecho.
- **Impacto:** renombrado y borrado de archivos fuera del proyecto con nombres de índice, con permisos del usuario, en un comando de reparación documentado. No es automático al abrir, pero es determinista y el nombre del proyecto es controlable por el atacante.
- **Corrección sugerida:** validar `.escritor` y sus rutas operativas **antes** de `Indice::apartar`; no decidir con `parece_corrupto` sobre errores de integridad que incluyen rutas. Tipar el error (`Error::Index`/`Error::Other` de SQLite) o comprobar que el error proviene de `Indice::abrir` sobre un `.escritor` ya validado. Añadir una regresión con el proyecto en un directorio que contenga `corrupt` y `.escritor` enlazado.

### SEC-02-H — [P3] Los enlaces duros eluden `validar_escritor` en lock, log, índice y sidecars

- **Esperado:** que `.escritor/bloqueo`, `operaciones.log`, `indice.sqlite` y sus sidecars no permitan escribir/truncar fuera del proyecto, como ya se exige para los enlaces simbólicos.
- **Real:** `validar_escritor` comprueba `is_symlink` y tipo, pero no el número de enlaces. Un enlace duro a un archivo externo es un archivo regular válido: `info` trunca el lock externo y escribe el JSON; `write` añade líneas al log externo; `write` hace que SQLite modifique el índice externo; un `-wal` enlazado recibe escrituras. Reproducido con las tablas de §2.
- **Impacto:** corrupción/truncado de archivos externos. Requiere crear el enlace duro localmente o extraer un archivo que lo preserve (GNU tar con `-P` y `linkname` absoluto; ZIP normal no). Por eso P3 y no P2.
- **Corrección sugerida:** rechazar `nlink > 1` en las rutas operativas (Unix `MetadataExt::nlink`, Windows `MetadataExt::number_of_links`), o abrir con `openat`/`O_NOFOLLOW` y verificar el inodo antes de truncar. Documentar que la garantía es para enlaces simbólicos/junctions y que los enlaces duros quedan fuera del modelo hasta que se corrijan.

### SEC-04 — [P3, vigente] Sin límites de tamaño ni de tipo; un FIFO en una ruta de efecto cuelga `verify`

- **Esperado:** que un proyecto de terceros no pueda colgar la app con un archivo especial o agotar memoria.
- **Real:** `hash::hash_archivo` hace `std::fs::read` completo y no exige que el destino sea un archivo regular; `leer_texto`, `Log::cargar`, `cuarentena()` y `docx::exportar` tampoco tienen límites. Un FIFO en `manuscrito/...` con un `PREPARE` válido hace que `writer verify` se bloquee indefinidamente (probado con `timeout 3`: exit 124). Un archivo enorme sigue consumiendo memoria/tiempo; el caso de 21 MB de la r1 ya lo mostraba.
- **Impacto:** denegación de servicio local al abrir/verificar/exportar un proyecto manipulado; no hay pérdida de datos ni ejecución de código. P3.
- **Corrección sugerida:** comprobar `symlink_metadata` + `is_file()` antes de leer/hashear rutas de efecto y blobs; rechazar FIFOs, dispositivos y sockets; imponer límites configurables por documento/proyecto y leer por streaming; limitar entradas de log y cuarentena.

### SEC-03-R — [P4] El gate exige `zizmor` aunque lo omita, y la allowlist de gitleaks es innecesaria/amplia

- **Real:** `_exigir_herramientas` incluye `zizmor` incondicionalmente; con un `PATH` sin `zizmor`, `seguridad` falla antes de llegar a la omisión. La allowlist `001-primera-escena\.md` no hace falta con la versión actual de gitleaks (sin configuración también hay 0 hallazgos en 22 commits y en el árbol versionado) y podría ocultar un futuro secreto que contenga ese literal.
- **Impacto:** reproducibilidad del gate y defensa en profundidad. P4.
- **Corrección sugerida:** exigir `zizmor` solo si existe `.github/`; eliminar o acotar la allowlist (fingerprint o `gitleaks:allow` en la línea concreta); si `.github/` existe sin workflows, omitir también por ausencia de entradas.

### SEC-05 — [P4, vigente] CSP, capacidades e IPC sin endurecer

- **Real:** CSP de producción correcta en lo esencial (`script-src 'self'`, sin `unsafe-eval`), pero `style-src 'unsafe-inline'` y faltan `object-src 'none'`, `base-uri 'none'`, `form-action 'none'` y `frame-ancestors 'none'`. `devCsp` usa `'unsafe-eval'` (solo desarrollo). `dialog:default` y `core:default` son amplios. Los comandos IPC aceptan cualquier `ruta` absoluta y no la ligan a un proyecto abierto; un compromiso del renderer podría leer proyectos con forma válida o exportar con `forzar=true` sobre cualquier archivo escribible. No hay XSS conocido: React no usa `dangerouslySetInnerHTML`, Tiptap valida el esquema de URL y el CSP bloquea `eval`.
- **Impacto:** no explotable con la superficie actual; reduce el coste de un futuro fallo del renderer. P4.
- **Corrección sugerida:** añadir las directivas que faltan, sustituir `'unsafe-inline'` por hashes/nonces si es viable, limitar `dialog:default` y mantener en el backend un allowlist de proyectos abiertos.

### SEC-06 — [P4, vigente] Deriva documental

- `HANDOFF.md:290-291` sigue diciendo que `gitleaks` y `zizmor` no se ejecutaron por ausencia de herramientas; en este entorno están instalados y se ejecutaron.
- `HANDOFF.md:17`, `README.md:32`, `docs/iteraciones/12-fase-2.md` y el acta r5 dicen 88 pruebas; el árbol actual tiene **89** (34 unitarias + 30 de integración + 25 de CLI).
- `docs/protocolo-commit.md` §2/§9 dice que `.escritor`, el log y el lock «no enlaces» / «un enlace se rechaza»; el código rechaza enlaces simbólicos, pero acepta enlaces duros (P3) y no se probó Windows. Conviene precisar el alcance.
- `docs/ci.md` describe la omisión de `zizmor`, pero no que la herramienta se exige igualmente; ver SEC-03-R.

## 6. Dependencias y cadena de suministro

| Área | Estado |
|---|---|
| `cargo-deny` núcleo | **Verde**: advisories, bans, licenses y sources. Aviso de ignore no usado para `RUSTSEC-2024-0370` (el crate no aparece en el grafo del núcleo) y duplicados. |
| `cargo-deny` desktop | **Verde** con la misma política. `RUSTSEC-2024-0370` (`proc-macro-error` 1.0.4, unmaintained) sigue documentado y llega por gtk-rs/Tauri; no hay reemplazo compatible con Tauri 2.12. Solo avisos de versiones duplicadas. |
| Lockfiles | `Cargo.lock`, `apps/desktop/src-tauri/Cargo.lock` y `pnpm-lock.yaml` versionados; los gates usan `--locked`. No hay `package-lock.json` ni `yarn.lock`. |
| pnpm | `packageManager` fija pnpm 12.14.0; `pnpm-workspace.yaml` autoriza el build solo de `esbuild` y desactiva `edgedriver`/`geckodriver`. No se usa npm/npx. |
| Plugins Tauri | Solo `dialog` explícito; `tauri-plugin-fs` es transitivo de `dialog` y no se inicializa ni tiene capacidades `fs`. No hay `shell`, `http` ni updater. |
| Secretos | `gitleaks` sobre 22 commits y sobre el árbol versionado: 0 hallazgos. Sin `.env`, claves ni tokens versionados. La allowlist es innecesaria (P4). |
| Workflows | No existe `.github/`; `zizmor` se omite. No hay CI remoto ni Dependabot. |
| `unsafe` | `#![forbid(unsafe_code)]` en `crates/core/src/lib.rs`; no encontré `Command::new` con entrada de usuario ni interpolación de shell en el producto. |
| Instaladores de herramientas | Playwright/Chromium, `tauri-driver` y `WebKitWebDriver` se instalan fuera del repo sin hash de artefacto; son herramientas de desarrollo. Recomendación P4 ya presente en la r1. |

No hay una dependencia vulnerable conocida en lo revisado. El punto débil de proceso es que el gate de seguridad no cubre Windows ni la app real; el acta ya lo declara.

## 7. Documentación vs. realidad

| Documento / afirmación | Realidad observada |
|---|---|
| `HANDOFF.md:290-291`: gitleaks/zizmor no ejecutados por ausencia | Falso en este entorno: están instalados y se ejecutaron. |
| `HANDOFF.md:17`, `README.md:32`, iteración 12 y acta r5: 88 pruebas | 89 en Linux (34 unitarias + 30 integración + 25 CLI). |
| `docs/protocolo-commit.md` §2/§9: `.escritor`/lock «no enlaces» | Cierto para enlaces simbólicos; los enlaces duros se aceptan (SEC-02-H). Windows no se probó. |
| `README.md:270`: «se rechazan enlaces simbólicos en componentes existentes» | Cierto en Linux; la redacción es correcta y no promete enlaces duros. |
| `docs/ci.md` §1: zizmor se omite si no hay `.github/` | Cierto en ejecución, pero `_exigir_herramientas` exige el binario igualmente (SEC-03-R). |
| `docs/cuarentena.md`: el informe se reemplaza atómicamente y un enlace duro no modifica otro archivo | Cierto: test existente y prueba propia. |
| `docs/esquema-v1.md`: IDs duplicados a cuarentena y resto operativo | Cierto en los tests; sin regresión. |
| `README.md:26`: fase 2 pendiente de seguridad/release/UX | Correcto tras la actualización; la fase sigue sin aprobar. |
| Acta r5: SEC-01/SEC-02 corregidos | Cierto en Linux para enlaces simbólicos y rutas; **no** cubre el camino `reindex`/`parece_corrupto` (P2) ni enlaces duros (P3). |

## 8. Conclusión y decisión

**Decisión: bloqueado.**

Las correcciones de la r5 son sustanciales y funcionan en Linux para el vector principal:

- **SEC-01:** cerrado para enlaces simbólicos, rutas `..`, formas de Windows, nombres reservados, controles y enlaces en el componente final; la recuperación valida antes de escribir y el error es visible.
- **SEC-02:** cerrado para `.escritor`, log, lock, objetos, cuarentena, índice y sidecars enlazados; `info`, `verify`, `list`, `write` y `reindex --limpiar` rechazan sin tocar el exterior.
- **SEC-03:** cerrado; el gate termina 0 y el escaneo de solo versionados elimina los falsos positivos de artefactos.

Pero hay un **P2 reproducible**: `reindex` decide apartar el índice con una heurística de texto (`parece_corrupto`) sobre un error que incluye la ruta del proyecto. Si el proyecto se llama `novela-corrupta` y `.escritor` es un enlace, renombra el `indice.sqlite` externo y borra sus sidecars antes de volver a validar. El comando muestra el error, pero el daño ya está hecho. Es exactamente la clase de garantía que SEC-02 debía cerrar, en un comando de reparación documentado.

Además quedan:

- **P3:** enlaces duros en `.escritor` (lock, log, índice, sidecars) que truncados/escriben fuera; requiere creación local o extracción que preserve enlaces duros.
- **P3:** SEC-04 sin límites; un FIFO en una ruta de efecto cuelga `verify` de forma determinista.
- **P4:** SEC-03-R (zizmor exigido aunque se omita; allowlist innecesaria/amplia), SEC-05 (CSP/capacidades/IPC) y SEC-06 (deriva documental y conteo 88/89).
- **Residual aceptado:** TOCTOU entre validación y escritura; no hay `openat`/`O_NOFOLLOW` ni comprobación de inodo. Está documentado, pero debe seguir registrado.
- **Windows:** no se ejecutó. La semántica de `is_symlink` de std cubre los reparse points con name-surrogate (incluido el junction estándar), pero falta prueba runtime y no se cubren filtros de terceros.

Para desbloquear:

1. Corregir SEC-02-R: validar `.escritor` antes de `Indice::apartar` y eliminar la decisión por texto de error; añadir regresión con un directorio que contenga `corrupt`.
2. Decidir y documentar SEC-02-H: rechazar `nlink > 1` en rutas operativas o aceptar el residual con una justificación explícita; no dejar que la documentación prometa más de lo que el código garantiza.
3. Resolver o aceptar formalmente SEC-04 (FIFO/tamaño) antes de aceptar proyectos de terceros a gran escala.
4. Repetir la revisión de seguridad sobre el commit corregido; después release, UX/accesibilidad/rendimiento y product owner.

**Resumen de severidades de esta r2:** P1: 0 · P2: 1 (SEC-02-R) · P3: 2 (SEC-02-H nuevo; SEC-04 vigente) · P4: 3 (SEC-03-R nuevo; SEC-05 y SEC-06 vigentes). El P2 bloquea; los P3/P4 no bloquean por sí solos, pero deben registrarse.
