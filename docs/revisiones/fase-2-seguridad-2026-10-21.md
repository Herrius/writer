# Fase 2 — Security review

- **Commit revisado:** `bd1757aed07e12d980b6f598d3acf13340d2371b`
- **HEAD del pipeline:** `b37219deae9bb63941caa510ae99d5b98ce74c4b`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64; Rust 1.99.1; `cargo-deny` 0.20.2; `zizmor` 1.31.1; `gitleaks` 8.x; pnpm 12.14.0; Node 26.11.1; Python 3.14.8. Herramientas en `/home/enrique/.local/bin` y `~/.cargo/bin`.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). `git diff --stat bd1757a..HEAD` solo muestra `docs/revisiones/README.md` y `docs/revisiones/fase-2-00-congelado-r4.md`; no hay cambios de código, tests ni configuración. Al terminar solo queda este informe sin seguimiento.
- **Decisión:** **bloqueado**

## 1. Alcance y modelo de amenaza

Revisé el artefacto congelado, no la justificación del builder. La aplicación es local/offline, pero escribe archivos del usuario, exporta DOCX y abre proyectos que pueden venir de terceros. El modelo de amenaza considerado es:

- proyecto malicioso o manipulado (Markdown, `proyecto.yml`, `.escritor/operaciones.log`, blobs, índice SQLite y cuarentena);
- enlaces simbólicos, uniones/junctions, rutas `..`, rutas absolutas y componentes de Windows (`\`, `C:`, UNC);
- YAML/frontmatter, Markdown, XML/OOXML y ZIP malformados;
- agotamiento de recursos (archivos enormes, muchas entradas, enlaces, listas);
- superficie Tauri: comandos IPC, DTOs, features de prueba, CSP, `devCsp` y capacidades;
- cadena de suministro: `deny.toml`, excepciones RUSTSEC, lockfiles, scripts de instalación y secretos en historial/árbol.

No abrí ventanas ni ejecuté la aplicación de escritorio, según la restricción de la revisión. Para Tauri usé análisis estático de `tauri.conf.json`, `capabilities/default.json`, `lib.rs` y la evidencia de la QA r3 (CSP de producción, `eval`, IPC y diálogos nativos). Las pruebas propias se hicieron con la CLI (`target/debug/writer`), que comparte `writer-core` con los comandos Tauri, y en directorios de `/tmp/opencode/sec-review`; no se tocó el producto.

## 2. Gate de seguridad (comandos y resultados)

Comando principal:

```bash
PATH="/home/enrique/.local/bin:$PATH" python3 scripts/ci.py seguridad
```

El gate se detuvo en `gitleaks git` con código 1; por eso ejecuté las comprobaciones restantes por separado, como permite la instrucción.

| Comprobación | Comando | Resultado |
|---|---|---|
| `cargo-deny` núcleo | `cargo deny --all-features --manifest-path Cargo.toml check` | **OK**: `advisories ok, bans ok, licenses ok, sources ok`. |
| `cargo-deny` desktop | `cargo deny --manifest-path apps/desktop/src-tauri/Cargo.toml check` | **OK**: `advisories ok, bans ok, licenses ok, sources ok`. Solo avisos de versiones duplicadas (`multiple-versions = "warn"`). |
| `gitleaks` historial | `gitleaks git . --redact --no-banner` | **Falla**: 20 commits, 1 hallazgo `generic-api-key` en `e2e-tauri/tauri.test.mjs` (línea 16 del commit histórico `fe8eadd`; línea 19 en el árbol actual). Es un **falso positivo** sobre literales de ruta (`'01-capitulo'`, `'001-primera-escena.md'`). |
| `gitleaks` árbol | `gitleaks dir . --redact --no-banner` | **Falla**: 11 hallazgos; 10 son `libmuda-*.rmeta` de `apps/desktop/src-tauri/target/` (artefactos de build no versionados) y 1 es el mismo falso positivo anterior. |
| `gitleaks` solo archivos versionados | `git archive HEAD \| tar -x -C /tmp/... && gitleaks dir ...` | **Falla**: 1 hallazgo, el mismo falso positivo del test. No hay secretos reales en el árbol versionado. |
| `zizmor` | `zizmor .` | **Falla**: código 3, `fatal: no audit was performed`, `no inputs collected`. No existe `.github/` ni workflows que auditar. |

Detalle del falso positivo: `e2e-tauri/tauri.test.mjs:19` contiene:

```js
  '01-capitulo',
  '001-primera-escena.md',
```

No es una credencial. `gitleaks dir` tampoco respeta `.gitignore`: recorre `target/` y `node_modules/`, lo que hace que el gate dependa de artefactos locales y produzca falsos positivos. El acta r4 y `docs/ci.md` presentan el gate como ejecutable, pero tal como está configurado no puede quedar verde (SEC-03).

## 3. Revisión por superficie

### 3.1 Rutas, escritura atómica y enlaces simbólicos

**Lo que está bien:**

- `recorrer_md` usa `DirEntry::file_type()` y **ignora enlaces simbólicos** antes de descender (`crates/core/src/verify.rs:592-625`); un `.md` enlazado no se indexa ni se muestra (probado).
- La exportación Markdown/DOCX valida el destino con `normalizar_lexica` y `resolver_destino`, rechazando destinos dentro del proyecto, incluidos alias simbólicos y `..` (`crates/core/src/project.rs:925-952`). Probado con destino directo y con un alias de la raíz: ambos rechazados.
- La cuarentena es la superficie mejor protegida: `directorio_cuarentena` exige `.escritor` y `cuarentena` como directorios reales, y `informe.jsonl` como archivo real (`crates/core/src/fsutil.rs:214-242`). Los enlaces en cuarentena se ignoran al listar y se rechazan al descartar (probado; coincide con `docs/cuarentena.md`).
- `escribir_atomico` conserva el modo del archivo en Unix (probado: `chmod 600` se mantiene tras `write`).

**Lo que falla:**

- `ruta_del_proyecto` solo separa por `/` y rechaza componentes vacíos, `.` y `..` (`crates/core/src/fsutil.rs:171-181`). **No comprueba enlaces simbólicos ni uniones en ningún componente**, y en Windows no reconoce `\`, `C:` ni prefijos UNC. El escaneo ignora enlaces, pero la recuperación, `revertir`, `mover`, `crear_escena` y `commit_efectos` vuelven a construir la ruta y siguen el enlace.
- `validar_efecto` del log hace la misma validación incompleta (`crates/core/src/oplog.rs:608-636`): rechaza rutas absolutas POSIX y `..` con `/`, pero acepta `..\`, `C:\...` y `\\servidor\...` en Windows.
- `.escritor` no se valida como directorio real en `lectura`/`escritura`: `escritor.exists()` sigue un enlace y `ProjectLock::adquirir` crea/trunca `.escritor/bloqueo` (`crates/core/src/project.rs:227-296`, `crates/core/src/lock.rs:23-60`). La validación de `directorio_real` existe, pero solo se usa en cuarentena (`fsutil.rs:185-242`). Es una asimetría clara.

La carrera TOCTOU de la exportación está documentada y no la cuento como hallazgo; el problema anterior no es una carrera: es una ruta que se sigue de forma determinista.

### 3.2 Parser YAML/frontmatter

`yaml.rs` implementa un subconjunto estricto: exige `---\n`, rechaza comentarios, indentación, listas en bloque, mapas anidados, comillas simples, anclas/alias (`&`, `*`, `!`, `|`, `>`), claves duplicadas y caracteres de control (`crates/core/src/yaml.rs:88-225`). `leer_texto` normaliza BOM/CRLF y rechaza CR aislado o UTF-8 inválido (`crates/core/src/model.rs:562-596`).

Probé frontmatter sin cierre, clave `id` duplicada, tabulación inicial y `U+0001` en un valor: los cuatro producen error de esquema y el proyecto queda sin escenas, sin pánico ni bloqueo. La lista se divide respetando comillas y no hay recursión; no veo una bomba YAML clásica. El riesgo residual es de tamaño (SEC-04), no de interpretación.

### 3.3 Log de operaciones, checksums y recuperación

`Log::cargar` lee el archivo completo, exige checksum BLAKE3 por línea, secuencia continua, `op_id` ULID, `ts` RFC 3339, `target_rev = base_rev + 1` y efectos con hashes válidos (`crates/core/src/oplog.rs:183-290`, `:292-386`). Una última línea truncada se descarta; una línea completa corrupta detiene la carga. La recuperación hace un preflight de blobs antes de tocar archivos (`crates/core/src/recovery.rs:30-54`).

El checksum protege contra corrupción accidental, **no autentica** el log: cualquiera puede calcular BLAKE3 y fabricar un `PREPARE` válido. Eso es aceptable si las rutas de los efectos se validan; no ocurre (SEC-01).

### 3.4 DOCX/OOXML/XML/ZIP

- `escapar_xml` escapa `&`, `<`, `>`, `"`; los atributos usan comillas dobles. `validar_xml` rechaza caracteres fuera de XML 1.0 antes de escribir el destino (`crates/core/src/docx.rs:600-622`).
- Los nombres de las partes del ZIP son constantes; no hay extracción de ZIP ni de DOCX de terceros. `zip` solo se usa para escribir.
- Probé título `T<&"`, escena `A<&"B`, enlace con `&` y `"` y `<script>` en el cuerpo: `word/document.xml`, `document.xml.rels` y `docProps/core.xml` pasan `xmllint`; el script queda como texto escapado y el enlace queda con `&amp;` y `&quot;`.
- Un `U+0001` en el cuerpo aborta la exportación con `entrada: ... U+0001`; un destino existente queda intacto (probado con `--forzar`). `unzip -t` no detecta errores en un DOCX normal.
- No hay zip-bomb de lectura porque writer no descomprime DOCX; el riesgo es de memoria al generar (SEC-04). Las URLs de enlace se copian como relaciones externas, incluidas `javascript:`/`file:`; Word no las ejecuta al abrir, pero endurecerlo a `http`/`https`/`mailto` sería defensa en profundidad (P4).

### 3.5 IPC de Tauri, DTOs y feature `e2e`

- Comandos expuestos: `abrir_proyecto`, `listar`, `leer`, `verificar`, `buscar`, `historial`, `mostrar`, `revertir`, `exportar_docx`, `cuarentena`, `descartar_cuarentena`, `proyecto_e2e`, `escribir` (`apps/desktop/src-tauri/src/lib.rs:481-495`). No hay `shell`, `fs`, `http` ni `updater` inicializados; `tauri-plugin-dialog` es el único plugin explícito.
- `proyecto_e2e` está registrado siempre, pero el cuerpo está bajo `#[cfg(feature = "e2e")]` y en producción devuelve `None` (`lib.rs:387-400`). La feature no está en las features por defecto de `Cargo.toml`; el build de release no la activa. La QA r3 comprobó que `invoke('proyecto_e2e')` devuelve `null` en producción.
- `buscar` limita a 1..100 en el núcleo; `descartar_cuarentena` rechaza `/`, `\`, `:`, `.`, `..`, vacío e `informe.jsonl`; `exportar_docx` valida el destino en el núcleo; `revision` es `u64` y `ruta`/`cuerpo` son `String`. No encontré una inyección de comandos ni un `unwrap` alcanzable desde IPC.
- **Observación P4:** ningún comando comprueba que `ruta` sea el proyecto abierto. Cualquier script en el webview podría pasar otra ruta absoluta y leer un proyecto con forma válida, o exportar con `forzar=true` sobre cualquier archivo escribible. No es explotable con la superficie actual (no hay XSS conocido y el CSP bloquea `eval`), pero un allowlist de proyectos abiertos en el backend reduciría el impacto de un futuro fallo del renderer.

### 3.6 CSP, `devCsp` y capacidades

`tauri.conf.json` define en producción:

```text
default-src 'self'; connect-src ipc: http://ipc.localhost;
img-src 'self' asset: http://asset.localhost data:;
style-src 'self' 'unsafe-inline'; script-src 'self'
```

La QA r3 verificó en la app real que `eval('1+1')` se rechaza, que no hay `unsafe-eval` y que la UI y el IPC cargan bajo esa CSP. `devCsp` añade `'unsafe-inline' 'unsafe-eval'` y `ws://localhost:1420`, pero solo se usa en desarrollo; no lo encontré activo en el build de producción.

`capabilities/default.json` limita la ventana `main`, incluye `core:default`, `core:window:allow-destroy` (necesario para el cierre) y `dialog:default`. No hay capacidades `fs`, `shell` ni `http`. La app no configura `on_navigation` ni un handler de nuevas ventanas; los enlaces Markdown se renderizan con `target="_blank"` y `rel="noopener noreferrer nofollow"`, y Tiptap valida el esquema de URL al renderizar. No observé una navegación explotable, pero no hay una política explícita que rechace navegaciones externas.

**P4 de endurecimiento:** `style-src 'unsafe-inline'`; faltan `object-src 'none'`, `base-uri 'none'`, `form-action 'none'` y `frame-ancestors 'none'` (aunque `default-src 'self'` cubre varios); `dialog:default` y `core:default` son más amplios de lo necesario; `devCsp` conserva `unsafe-eval`. Nada de esto es un P1/P2 con la superficie actual.

### 3.7 Dependencias y scripts de instalación

Ver §6. No hay `unsafe` en código propio (`#![forbid(unsafe_code)]` en `crates/core/src/lib.rs:11`). No encontré `Command::new` con entrada de usuario, `sh -c` ni interpolación de shell en el producto; el único `std::process` es el hook de fallos (`WRITER_FAULT`, documentado) y el lock.

### 3.8 Cadenas de confianza y documentación

El README y `HANDOFF.md` describen garantías de recuperación y de rutas relativas (`docs/protocolo-commit.md` §2) que SEC-01 incumple. La cuarentena sí cumple su contrato de enlaces; el resto de `.escritor` no. Ver §7.

## 4. Pruebas adversariales

Todas en `/tmp/opencode/sec-review`; la CLI es `target/debug/writer`. No se ejecutó la app de escritorio.

| # | Prueba | Resultado |
|---|---|---|
| 1 | Proyecto válido con `manuscrito/01-libro/01-capitulo/extra` enlazado a un directorio externo; `PREPARE` pendiente con efecto `manuscrito/.../extra/pwned.txt`, `hash_nuevo: null` y blob propio; checksum BLAKE3 recalculado | **Falla de seguridad**: `writer verify` devuelve `escenas: 1 · errores: 0 · resultado: OK` (solo un aviso de enlace ignorado) y crea `outside/pwned.txt` con el contenido del blob. **SEC-01**. |
| 2 | Proyecto con `manuscrito` enlazado a un directorio externo y el mismo `PREPARE` | **Falla de seguridad**: `verify` termina con errores de escaneo, pero la recuperación ya escribió `outside/pwned.txt`. El orden «recuperar antes de verificar» hace que el error visible no impida la escritura. **SEC-01**. |
| 3 | `.escritor` enlazado a un directorio externo con un `bloqueo` preexistente (`CONTENIDO-EXTERNO`) | **Falla**: `writer info` devuelve 0 y reemplaza el `bloqueo` externo por el JSON del lock. **SEC-02**. |
| 4 | `.escritor/objetos` enlazado a un directorio externo con un archivo de 64 hex; `writer reindex --limpiar` | **Falla**: informa `blobs huérfanos eliminados: 1` y borra el archivo externo. **SEC-02**. |
| 5 | DOCX a destino dentro del proyecto, directo y por alias simbólico de la raíz | **OK**: `el destino no puede estar dentro del proyecto`; el archivo no se crea. |
| 6 | DOCX con `U+0001` en el cuerpo y destino existente con `--forzar` | **OK**: aborta con `U+0001`; el destino conserva `EXISTENTE\n` byte a byte. |
| 7 | DOCX con `<`, `&`, `"`, `<script>` y enlace con `&`/`"`; `xmllint` y `unzip -t` | **OK**: XML bien formado, entidades escapadas, ZIP íntegro; el script es texto. |
| 8 | Frontmatter sin cierre, `id` duplicado, tabulación inicial y `U+0001` | **OK**: errores de esquema claros; sin pánico. |
| 9 | Dos escenas con el mismo `id`; `verify`, `list`, `buscar` | **OK**: dos errores, ambas en `invalidos`, `list` devuelve 0; sin error `UNIQUE`. |
| 10 | `read ../../etc/passwd`, `read /etc/passwd` | **OK**: `documento no encontrado`; no hay lectura fuera del conjunto escaneado. |
| 11 | `.md` enlazado a `/etc/hostname`; `verify` | **OK**: aviso `enlace simbólico ignorado`; no se indexa. |
| 12 | Enlace en cuarentena; `quarantine list` y `discard` | **OK**: no se lista; el descarte se rechaza con `no es un archivo de cuarentena`. |
| 13 | Escena de 21 MB; `info` y `export docx` | **OK en tiempo**: 0,506 s y 1,790 s; DOCX de 47 KB. Sin límites de tamaño en código (**SEC-04**). |
| 14 | Proyecto con `.escritor` real en directorio de solo lectura; `info`/`verify` | **Limitación**: falla con `Permission denied` en `.escritor/bloqueo`; no es un fallo de seguridad, pero contradice la idea de abrir proyectos de solo lectura con historial. |
| 15 | `cargo test -p writer-core --test proyecto --locked` y `cargo test -p writer-cli --test cli --locked` | **OK**: 28/28 y 25/25. No hay pruebas existentes para SEC-01/SEC-02. |

La prueba 1 es la reproducción mínima de SEC-01. El `PREPARE` se puede construir con cualquier herramienta BLAKE3; no hace falta una clave. El proyecto resultante sigue teniendo una escena válida y `verify` sale 0, de modo que la interfaz no muestra un error que advierta de la escritura.

## 5. Hallazgos

### SEC-01 — [P1] La recuperación escribe fuera del proyecto con un log manipulado y un enlace simbólico

- **Esperado:** que toda ruta de efecto de `operaciones.log` quede confinada al proyecto y que la recuperación no siga enlaces simbólicos, uniones ni componentes de Windows. `docs/protocolo-commit.md` §2 define los efectos como rutas relativas; `docs/esquema-v1.md` presenta la recuperación como una operación sobre archivos del proyecto.
- **Real:** `ruta_del_proyecto` solo separa por `/` (`crates/core/src/fsutil.rs:171-181`) y `validar_efecto` solo rechaza `/`, `.` y `..` (`crates/core/src/oplog.rs:608-636`). `recovery::recuperar` llama a `revertir_efecto`, que construye la ruta y escribe el blob con `escribir_atomico` sin comprobar los componentes (`crates/core/src/recovery.rs:30-54`, `:97-111`). Un proyecto con `manuscrito/01-libro/01-capitulo/extra -> /tmp/fuera` y un `PREPARE` pendiente con `ruta: "manuscrito/01-libro/01-capitulo/extra/pwned.txt"`, `hash_nuevo: null` y `hash_anterior: blake3:<hash del blob>` hace que `writer verify` cree `/tmp/fuera/pwned.txt` **antes** de verificar. En la prueba, `verify` salió 0 con una escena válida.
- **Reproducción:** la prueba 1 de §4. El log se puede forjar porque el checksum es BLAKE3 público; la secuencia, el ULID y el `ts` son válidos. No hace falta que el proyecto tenga errores: el efecto se aplica en `lectura()` (`project.rs:227-265`), y `abrir_proyecto` de Tauri llama a `verificar()` (`apps/desktop/src-tauri/src/lib.rs:230-263`). Abrir el proyecto en la GUI es suficiente.
- **Impacto:** escritura arbitraria de contenido controlado por el atacante fuera de la raíz del proyecto, con los permisos del usuario, al abrir el proyecto. Si el atacante conoce la ruta, puede crear un archivo de arranque (por ejemplo, un `.desktop` en `~/.config/autostart` o un archivo en Startup en Windows) o sobrescribir un archivo cuyo hash conozca; como mínimo, crea archivos fuera del proyecto. También puede escribir rutas internas no contempladas (`proyecto.yml`, el propio log), lo que rompe la integridad del proyecto.
- **Variante Windows (no ejecutada aquí):** `validar_efecto` y `ruta_del_proyecto` no reconocen `\`, `C:` ni `\\servidor\...`. En Windows, un efecto `..\..\..\Users\...\archivo` o `C:\...` no contiene `/` y pasa la validación; `PathBuf::push` lo interpreta como ruta de Windows y sale de la raíz. Es la misma causa y el mismo impacto; debe corregirse y probarse en Windows antes del release.
- **Corrección sugerida:** rechazar en `ruta_del_proyecto` todo componente que no sea `Component::Normal` (sin prefijo, raíz, `..`, `\`, `:` ni UNC), normalizar separadores con `Path::components`, y comprobar con `symlink_metadata` que **ningún** componente existente sea enlace o unión antes de leer/escribir. Limitar los efectos a `manuscrito/` y `notas/`. Para escrituras, usar `openat`/`O_NOFOLLOW` o una comprobación equivalente entre el padre real y la raíz. Añadir pruebas de regresión con un enlace en un subdirectorio de `manuscrito`, un enlace en `.escritor` y rutas de Windows.

### SEC-02 — [P2] `.escritor` enlazado se acepta: se trunca, escribe o borra fuera del proyecto

- **Esperado:** que `.escritor`, `operaciones.log`, `objetos`, `indice.sqlite` y `cuarentena` sean reales, como ya exige `docs/cuarentena.md` §4 para la cuarentena. El README afirma que «la cuarentena rechaza enlaces simbólicos también en el ancestro `.escritor`»; la garantía debería ser general para las rutas operativas.
- **Real:** `Project::lectura` comprueba `escritor.exists()` (sigue el enlace) y llama a `ProjectLock::adquirir(&escritor)` (`crates/core/src/project.rs:227-238`), que crea `.escritor/bloqueo` y lo trunca al escribir la información del lock (`crates/core/src/lock.rs:23-60`). `Project::escritura` crea `.escritor/objetos` y `.escritor/cuarentena` sin validar el ancestro (`project.rs:280-296`); `BlobStore::abrir` usa `create_dir_all` (`crates/core/src/blob.rs:18-22`). `directorio_real`/`archivo_real` existen, pero solo se usan en `directorio_cuarentena` (`fsutil.rs:185-242`). Probado: `writer info` con `.escritor -> /tmp/fuera` y `/tmp/fuera/bloqueo` preexistente reemplaza el contenido externo por el JSON del lock; `reindex --limpiar` con `.escritor/objetos -> /tmp/fuera` borra un archivo de 64 hex externo.
- **Impacto:** truncado/escritura de archivos fuera del proyecto en operaciones de lectura (`info`, `verify`, `listar`, `historial`) y borrado de archivos con nombre de blob en `reindex --limpiar`. Puede destruir datos del usuario, corromper otra base SQLite si `indice.sqlite` es un enlace, o exponer blobs del manuscrito en un directorio externo. Requiere abrir un proyecto manipulado; no es una carrera.
- **Corrección sugerida:** validar `.escritor` con `directorio_real` en `lectura` y `escritura`; validar `operaciones.log`, `objetos`, `indice.sqlite` y `cuarentena` con `archivo_real`/`directorio_real` antes de abrirlos; rechazar enlaces y uniones; no crear `.escritor` si existe como enlace. Reutilizar la lógica ya probada de cuarentena y añadir regresiones para `.escritor` y `.escritor/objetos`.

### SEC-03 — [P3] El gate `seguridad` no puede quedar verde: falsos positivos de `gitleaks` y `zizmor` sin entradas

- **Esperado:** que `python3 scripts/ci.py seguridad` termine 0 y que sus hallazgos sean accionables.
- **Real:** `gitleaks git` falla con un falso positivo `generic-api-key` en `e2e-tauri/tauri.test.mjs` (línea 16 del commit `fe8eadd`; línea 19 en el árbol actual; literales de ruta); `gitleaks dir` falla además con 10 falsos positivos en `apps/desktop/src-tauri/target/**/libmuda-*.rmeta` porque escanea artefactos no versionados. `zizmor .` sale con código 3 y `no inputs collected` porque no hay `.github/`. El gate se detiene en el primer fallo y nunca ejecuta el resto.
- **Impacto:** no hay evidencia de un gate de seguridad verde; el pipeline §9 no puede aprobar la fase con este estado, aunque no se haya encontrado un secreto real. Un fallo verdadero quedaría sepultado por el ruido de artefactos.
- **Corrección sugerida:** añadir `.gitleaks.toml` con una allowlist específica (fingerprint o `gitleaks:allow` en la línea del test) y excluir `target/`, `node_modules/`, `test-results/` y `ui/dist/` del escaneo de árbol (por ejemplo, escaneando `git ls-files` en un directorio temporal). Ejecutar `zizmor` solo si existe `.github/workflows`, o añadir los workflows antes de exigir el gate. Documentar la excepción en `docs/ci.md`.

### SEC-04 — [P3] Consumo de recursos sin límites sobre proyectos no confiables (DoS local)

- **Esperado:** que un proyecto de terceros no pueda agotar memoria o colgar la app con un archivo, un log o una entrada especial.
- **Real:** `leer_texto` hace `std::fs::read` completo (`crates/core/src/model.rs:562-596`); `verify` vuelve a leer el archivo para el hash (`verify.rs:322-324`, `:355-357`); `Log::cargar` lee el log completo (`oplog.rs:183-192`); `cuarentena()` hace `read_to_string` del informe (`project.rs:455`); `docx::exportar` construye todo el documento y el ZIP en memoria (`docx.rs:19-74`, `:445-500`). No hay límites de tamaño, número de escenas, efectos, enlaces ni entradas de log. `recovery` llama a `hash::hash_archivo` sobre la ruta del efecto (`recovery.rs:41-42`), que sigue enlaces: un enlace a `/dev/zero`, a un FIFO o a un archivo enorme puede bloquear o agotar la memoria al abrir. No ejecuté el caso `/dev/zero` para no agotar la máquina; el de 21 MB funcionó en 0,5 s (`info`) y 1,8 s (DOCX), lo que confirma que no hay un límite explícito, solo el coste.
- **Impacto:** denegación de servicio local al abrir, buscar o exportar un proyecto manipulado; no hay pérdida de datos ni ejecución de código. P3.
- **Corrección sugerida:** comprobar `symlink_metadata` y `is_file()` antes de leer; rechazar FIFOs, dispositivos y enlaces en rutas de efectos; imponer límites configurables (por ejemplo, 16 MiB por documento y 256 MiB por proyecto) con errores claros; leer/hashear por streaming; limitar entradas de log y de cuarentena; para DOCX, escribir el ZIP a un archivo temporal fuera del proyecto en vez de acumularlo en memoria.

### SEC-05 — [P4] Endurecimiento pendiente de CSP, capacidades e IPC (defensa en profundidad)

- **Esperado:** mínima superficie de la cáscara y del renderer.
- **Real:** CSP de producción correcta en lo esencial (`script-src 'self'`, sin `unsafe-eval`; QA r3 verificó `eval` bloqueado), pero `style-src 'unsafe-inline'` y faltan `object-src 'none'`, `base-uri 'none'`, `form-action 'none'` y `frame-ancestors 'none'`. `devCsp` usa `'unsafe-eval'` (solo desarrollo). `dialog:default` y `core:default` son más amplios de lo necesario. Los comandos IPC aceptan cualquier `ruta` absoluta y no la ligan a un proyecto abierto (`lib.rs:230-416`); un compromiso del renderer podría leer proyectos con forma válida o exportar con `forzar=true` sobre cualquier archivo escribible. No hay XSS conocido: React no usa `dangerouslySetInnerHTML`, Tiptap valida el esquema de URL y el CSP bloquea `eval`.
- **Impacto:** no explotable con la superficie actual; reduce el coste de un futuro fallo del renderer. P4.
- **Corrección sugerida:** añadir las directivas que falten, sustituir `'unsafe-inline'` por hashes/nonces si es viable, limitar `dialog:default` a `allow-open`/`allow-save`, y mantener en el backend un conjunto de proyectos abiertos para validar `ruta` en cada comando.

### SEC-06 — [P4] Deriva documental sobre QA, herramientas y alcance de los enlaces simbólicos

- **Esperado:** que `HANDOFF.md`, `README.md` y `docs/` describan el estado real del commit congelado.
- **Real:** `HANDOFF.md:290-291` dice que `gitleaks` y `zizmor` «no se ejecutaron aquí (herramientas ausentes)», pero el acta r4 los da por instalados y esta revisión los ejecutó. `README.md:26` sigue diciendo «pendiente QA r3», aunque la r3 está aprobada con observaciones y existe el acta r4. `README.md:290` y `docs/cuarentena.md` describen el rechazo de enlaces en cuarentena, pero no aclaran que `.escritor` y las rutas de recuperación no tienen esa garantía (SEC-02). `docs/ci.md` describe el job de seguridad con workflows, pero no hay `.github/`.
- **Impacto:** un lector puede asumir garantías que el código no cumple o un estado de gate que no existe. P4.
- **Corrección sugerida:** actualizar el estado de QA y de las herramientas, y documentar explícitamente el alcance de la validación de enlaces (cuarentena sí; rutas operativas y recuperación, después de la corrección).

## 6. Dependencias y cadena de suministro

| Área | Estado |
|---|---|
| `cargo-deny` núcleo | **Verde**: advisories, bans, licenses y sources. Sin yanked; sin comodines; solo crates.io. |
| `cargo-deny` desktop | **Verde** con la misma política. Avisos de versiones duplicadas (`toml`, `windows-sys`, etc.), no bloqueantes. |
| Excepción RUSTSEC | `RUSTSEC-2024-0370` (`proc-macro-error` 1.0.4, unmaintained) está documentada en `deny.toml` con motivo; llega por gtk-rs/Tauri en Linux y no tiene reemplazo compatible con Tauri 2.12. No es una vulnerabilidad explotable en writer; revisar al actualizar Tauri/gtk-rs. |
| Versiones | Tauri 2.12.4, `tauri-plugin-dialog` 2.8.3, `tauri-build` 2.7.3 y `ts-rs` 12.0.1 fijados con `=`. `serde`/`serde_json` usan caret, pero `Cargo.lock` está versionado y los gates usan `--locked`. |
| Lockfiles | `Cargo.lock`, `apps/desktop/src-tauri/Cargo.lock` y `pnpm-lock.yaml` están versionados; no hay `package-lock.json` ni `yarn.lock`. |
| pnpm | `packageManager` fija pnpm 12.14.0. `pnpm-workspace.yaml` autoriza el build solo de `esbuild` y desactiva `edgedriver`/`geckodriver`. No se usa `npm`/`npx`. |
| Plugins Tauri | Solo `dialog` explícito; `tauri-plugin-fs` aparece como dependencia transitiva de `dialog`, pero no se inicializa ni tiene capacidades `fs`. No hay `shell`, `http` ni updater. |
| Red | No encontré `fetch`/WebSocket en la UI ni un cliente HTTP activo en el grafo de runtime; `reqwest`/`hyper` están en el lock del desktop pero `cargo tree -i reqwest` no encuentra un consumidor activo en Linux. |
| Secretos | `gitleaks` sobre 20 commits y sobre el árbol versionado: 1 falso positivo (`e2e-tauri/tauri.test.mjs:19`); ningún secreto real. No hay `.env`, claves ni tokens versionados. |
| Workflows | No existe `.github/`; `zizmor` no tiene entradas. No hay Dependabot ni CI remoto; `docs/ci.md` lo reconoce como pendiente. |
| Instaladores de herramientas | Playwright/Chromium, `tauri-driver` y `WebKitWebDriver` se instalan fuera del repo sin hash de artefacto; son herramientas de desarrollo, no del producto. Recomendación P4: fijar versión y, si sea posible, checksum en la documentación del gate. |

No hay una cadena de suministro comprometida en lo revisado. El punto débil es de proceso: el gate de seguridad no está verde por configuración (SEC-03), no por una dependencia vulnerable.

## 7. Documentación vs. realidad

| Documento / afirmación | Realidad observada |
|---|---|
| `HANDOFF.md:290-291`: `gitleaks` y `zizmor` no se ejecutaron por ausencia | Falso en este entorno: están instalados y se ejecutaron; el acta r4 también los da por instalados. |
| `README.md:26`: Fase 2 «pendiente QA r3» | La r3 está aprobada con observaciones y el acta r4 congela el commit para seguridad. |
| `README.md:290` y `docs/cuarentena.md` §4: rechazo de enlaces en `.escritor`/cuarentena | Cierto **solo para cuarentena**; `.escritor`, `objetos`, `indice.sqlite` y la recuperación no validan enlaces (SEC-02). |
| `docs/protocolo-commit.md` §2/§5: efectos sobre rutas relativas y recuperación sobre archivos del proyecto | SEC-01 demuestra que un efecto puede seguir un enlace y escribir fuera; no hay limitación documentada para enlaces en rutas de efectos. |
| `docs/ci.md` §1: el gate de seguridad incluye `cargo-deny`, `gitleaks` y `zizmor` | `cargo-deny` verde; `gitleaks` falla por falsos positivos; `zizmor` no tiene entradas. El gate completo no es ejecutable en verde. |
| `docs/docx-v1.md`: control de XML antes de escribir, destino intacto | Cierto: `U+0001` aborta y el destino existente no se toca; XML/ZIP bien formados con `<`, `&`, `"` y enlaces. |
| `docs/cuarentena.md`: enlaces ignorados al listar y rechazados al descartar; informe real | Cierto en las pruebas 12; el informe y las copias enlazadas se rechazan como describe el contrato. |
| `docs/esquema-v1.md`: IDs duplicados a cuarentena y resto operativo | Cierto: dos errores, ambos inválidos, `list` vacío, sin `UNIQUE`. |
| `README.md`/`HANDOFF.md`: proyectos de solo lectura sin `.escritor` | Cierto sin `.escritor`; con `.escritor` real en un directorio sin permiso de escritura, `info`/`verify` fallan al crear el lock. Es una limitación funcional, no de seguridad. |
| Conteos de QA: 86 Rust, 55 UI, 7 E2E navegador, 2 E2E escritorio, 2 Tauri | No los repetí completos; la r3 los documenta. Mis pruebas focalizadas: 28/28 de integración del núcleo y 25/25 de CLI. |

## 8. Conclusión y decisión

**Decisión: bloqueado.**

El artefacto no puede pasar a release engineer, UX ni product owner. Hay un **P1** reproducible: un proyecto de terceros con un `PREPARE` pendiente y un enlace simbólico en una ruta de efecto hace que la recuperación escriba contenido arbitrario fuera de la raíz al abrir el proyecto; en la prueba, `verify` salió 0 y el proyecto seguía teniendo una escena válida. La misma validación incompleta deja una variante de Windows con `\`, `C:` y UNC que debe probarse en esa plataforma.

Hay además un **P2** independiente: `.escritor` se acepta como enlace y las operaciones de lectura truncan/escriben el lock fuera del proyecto; `reindex --limpiar` puede borrar archivos externos con nombre de blob. La cuarentena ya tiene la validación correcta; falta aplicarla a las rutas operativas.

Para desbloquear:

1. Corregir SEC-01: confinar los efectos a `manuscrito/`/`notas/`, validar todos los componentes con semántica de plataforma y rechazar enlaces/uniones en cualquier ancestro; añadir regresiones con enlaces en subdirectorios y rutas de Windows.
2. Corregir SEC-02: validar `.escritor`, `operaciones.log`, `objetos`, `indice.sqlite` y `cuarentena` como reales antes de usarlos; reutilizar `directorio_real`/`archivo_real`.
3. Arreglar SEC-03 para que el gate de seguridad sea reproducible en verde o tenga una excepción documentada y acotada; no aceptar un gate que falla por artefactos de build.
4. Registrar SEC-04, SEC-05 y SEC-06 como deuda P3/P4; SEC-04 debe resolverse antes de aceptar proyectos de terceros a gran escala, aunque no bloquee por sí solo.
5. Repetir la revisión de seguridad sobre el commit corregido y solo después continuar el pipeline §9. La Fase 3 no se inicia.

---

**Resumen de severidades:** P1: 1 (SEC-01) · P2: 1 (SEC-02) · P3: 2 (SEC-03, SEC-04) · P4: 2 (SEC-05, SEC-06).
