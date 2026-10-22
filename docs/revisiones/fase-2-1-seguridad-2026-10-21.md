# Fase 2.1 — Security review

- **Commit revisado:** `376c0b8a07c4381f36c56f9a0620ea150580b352`
- **HEAD del pipeline:** `0b0df6f5c57b7ec0f345b003374b71687697adf6`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64; Rust 1.99.1; `cargo-deny` 0.20.2; `zizmor` 1.31.1; gitleaks local (el binario no reporta número de versión); pnpm 12.14.0; Node 26.11.1; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). `git diff --stat 376c0b8..HEAD` no contiene código, tests ni configuración: incluye `HANDOFF.md` y cuatro documentos de `docs/revisiones/` (`README.md`, acta inicial, acta de seguridad y QA 2.1). La parte de `docs/revisiones/` está dentro del alcance; `HANDOFF.md` queda fuera de esa ruta (observación de proceso P4). Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **bloqueado** — P1: 0 · P2: 1 (SEC21-01) · P3 nuevos: 2 · P4 nuevos: 3 (+ observaciones vigentes).

## 1. Alcance y superficie nueva

Revisé el artefacto congelado, no la justificación del builder. La superficie nueva es:

- comando Tauri `crear_proyecto(ruta, titulo, autor, idioma, notasIniciales)` y su registro en `invoke_handler`;
- `Project::init` y `Project::crear_nota` en el motor;
- asistente modal de `ui/src/Inicio.tsx`, llamada IPC de `ui/src/api.ts` y recientes de `ui/src/recientes.ts`;
- `localStorage` `writer.recientes.v1`, tarjetas, búsqueda, color y apertura de recientes;
- diálogo nativo de carpeta (`tauri-plugin-dialog`) y capacidades Tauri;
- `tauri.conf.json` y `capabilities/default.json` (sin cambios en este commit respecto de la Fase 2).

Método y límites:

- **No abrí la app Tauri ni ventanas.** Usé el binario `target/debug/writer` recompilado desde el commit (`cargo build --locked --bin writer`), el motor y la CLI, análisis estático y un arnés en `/tmp/opencode/sec21*`.
- Para el caso de ruta vacía usé una **sonda Rust fuera del árbol** que enlaza `writer-core` y llama a `Project::new("").init(...)`; el comando Tauri ejecuta exactamente `Project::new(ruta).init(...)`.
- Para el escenario de notas fallidas usé el mismo camino del motor que ejecuta el comando: `Project::init` y después `Project::crear_nota`, con permisos restringidos en `notas/`.
- No ejecuté `e2e`, `e2e-tauri`, `empaquetado` ni `interfaz`; sí ejecuté los tests Rust, Tauri, TypeScript y UI que no abren ventanas.
- El modelo de amenaza es el de las revisiones anteriores (proyecto de terceros manipulado, enlaces simbólicos/duros, rutas de Windows, archivos especiales, recursos y superficie Tauri), más la nueva frontera IPC de creación de archivos.

Estado comprobado:

```text
git status --porcelain                         vacío
git rev-parse HEAD                             0b0df6f5c57b7ec0f345b003374b71687697adf6
git diff --stat 376c0b8..HEAD                  HANDOFF.md + 4 docs de docs/revisiones/
cargo build --locked --bin writer              OK
```

## 2. Gate de seguridad

| Comprobación | Comando | Resultado |
|---|---|---|
| Tests Rust | `nice -n 19 cargo test --workspace --all-targets --locked` | **OK**: **92** pruebas (25 CLI + 34 unitarias + 33 de integración), 0 fallos. |
| Tests Tauri | `nice -n 19 cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | **OK**: **3/3** (tipos, comandos básicos y `crear_proyecto` con/sin notas). |
| TypeScript | `nice -n 19 pnpm typecheck` | **OK**. |
| UI | `nice -n 19 pnpm ui:test` | **OK**: **58/58** en 5 archivos. |
| Calidad | `nice -n 19 python3 scripts/ci.py calidad` | **OK** (fmt, clippy `-D warnings`, tests). |
| Producto | `nice -n 19 python3 scripts/ci.py producto` | **OK**: fixture 3 escenas/18 notas, determinismo, 12 errores/4 avisos, 1.000 escenas en 0,3 s, CLI y enlaces. |
| Seguridad | `nice -n 19 python3 scripts/ci.py seguridad` | **OK**: `cargo-deny` núcleo/desktop, gitleaks historial/árbol, zizmor omitido. |
| gitleaks historial | incluido | **46 commits**, ~1,63 MB, 0 hallazgos. |
| gitleaks árbol | incluido | ~1,61 MB, 0 hallazgos; solo `git ls-files`. |
| `.gitleaks.toml` | — | Ausente; el gate usa la configuración por defecto. |
| `.gitleaksignore` | — | Solo dos fingerprints exactos de falsos positivos históricos; sin allowlist amplia. |
| zizmor | incluido | Omitido: no existe `.github/`. |
| `cargo-deny` núcleo/desktop | incluido | `advisories ok, bans ok, licenses ok, sources ok`. |

No ejecuté `interfaz`, `e2e`, `e2e-tauri` ni `empaquetado`: la instrucción prohíbe abrir la app/ventanas. `calidad` y los tests cubren el núcleo, la CLI, la cáscara Tauri y la UI sin GUI.

## 3. Revisión de creación de proyectos

### 3.1 Flujo real

```text
crear_proyecto(ruta, titulo, autor, idioma, notas_iniciales)   [Tauri]
  Project::new(ruta).init(titulo, autor, idioma)               [core]
    if raiz.exists() {
      if read_dir(raiz).next().is_some() -> "no está vacío"
    } else create_dir_all(raiz)
    Proyecto::nuevo(...) -> escribir_atomico(raiz/proyecto.yml)
    create_dir_all(manuscrito, notas, .escritor/objetos, .escritor/cuarentena)
    Log::crear_vacio(raiz/.escritor/operaciones.log)
    self.escritura() -> validar_escritor() -> commit de la primera escena
  for tipo in [personajes, tramas, lugares]:
    crear_nota(tipo, etiqueta)                                 [core]
  abrir_proyecto(ruta)                                          [Tauri]
```

La UI valida en `Inicio.tsx`:

- título no vacío (trim);
- carpeta padre no vacía;
- nombre no vacío, distinto de `.`/`..`, sin `/ \ : * ? " < > |`, sin espacio inicial;
- la ruta se construye como `${carpetaPadre.replace(/[\\/]+$/, '')}/${nombre}`.

Esa validación **no es una frontera de seguridad**: `crear_proyecto` recibe `ruta` como `String` y no la valida. El backend acepta rutas absolutas, relativas, vacías, con `..`, symlinks y formas de Windows. La comprobación de «directorio no vacío» se hace sobre la ruta literal, no sobre la ruta canónica.

### 3.2 Matriz de rutas

| Caso | Resultado observado |
|---|---|
| `writer init p --titulo X` | **OK**: manifiesto, `.escritor`, escena inicial; `verify` OK. |
| `writer init` en directorio existente **no vacío** | **OK**: rc 1, `el directorio no está vacío`; no escribe. |
| `writer init` en directorio existente **vacío** | **OK**: crea el proyecto. |
| `writer init` sobre archivo, FIFO o socket | **OK**: rc 1, `Not a directory`; sin cuelgue y sin escritura. |
| `writer init` sobre symlink colgante | **OK**: rc 1, `File exists`; no crea el destino. |
| `writer init link` con `link` → directorio externo **vacío** | **Falla**: rc 0; crea el proyecto en el destino del symlink. |
| `writer init link` con `link` → directorio externo **no vacío** | **OK**: rc 1, no escribe. |
| `writer init 'a/..'` en un cwd con `proyecto.yml` existente | **Falla**: rc 0; **sobrescribe** `proyecto.yml` y crea `.escritor/`/`manuscrito/`/`notas/` en el cwd. |
| `writer init 'a/../b'` con `b/proyecto.yml` existente | **Falla**: rc 0; **sobrescribe** `b/proyecto.yml` y crea el proyecto en `b`. |
| `Project::new("").init(...)` en un cwd con `proyecto.yml` existente (sonda Rust) | **Falla**: devuelve `Err("proyecto: no existe el directorio del proyecto: ")` pero **sobrescribe** `proyecto.yml` y crea `.escritor`, `manuscrito` y `notas` en el cwd. |
| `writer init` con título vacío | **Parcial**: rc 1, deja el directorio vacío creado. |
| `writer init` con título o autor con carácter de control | **Parcial**: rc 1, deja el directorio vacío creado; el manifiesto no se escribe. |
| `writer init` con `--idioma zz` | **Aceptado**: el manifiesto guarda `idioma: zz`; el motor no limita a `es`/`en`. |
| `writer init` con `C:\tmp\win`, `punto.` o `CON` en Linux | **Aceptado**: crea directorios literales; no hay validación de formas Windows. En Windows no se ejecutó. |
| `writer init` dentro de un proyecto (`p/manuscrito/.../anidado`) | **Aceptado**: crea un proyecto anidado; el `verify` del padre pasa a fallar por la escena anidada. |
| `writer init` dentro de `p/.escritor/colado` | **Aceptado**: crea un proyecto dentro de `.escritor`; el `verify` del padre no lo ve. |
| `writer init` con padre de solo lectura | **OK**: rc 1, `Permission denied`; no crea el hijo. |
| `writer init` con `umask 000` | **Falla de permisos**: directorios `0777` y archivos `0666`; `.escritor`, manifiesto y log quedan escribibles por cualquier usuario local. |
| Dos `writer init` concurrentes sobre la misma ruta | **Uno gana, uno falla**; 10/10 el proyecto resultante verifica. No se reprodujo corrupción, pero no hay reserva atómica. |

### 3.3 Por qué `'a/..'` y la ruta vacía funcionan

`Project::init` comprueba `self.raiz.exists()` y `read_dir(self.raiz)` sobre la ruta **literal**:

- `a/..` no existe si `a` no existe, así que `exists()` es falso y se salta la comprobación de directorio vacío;
- `create_dir_all("a/..")` crea `a` y resuelve `..` al cwd;
- `self.raiz.join("proyecto.yml")` es `a/../proyecto.yml`, que resuelve al `proyecto.yml` del cwd;
- `Log::crear_vacio` escribe `a/../.escritor/operaciones.log`, que resuelve al log del cwd.

Con `raiz = ""`, `exists()` es falso, `create_dir_all("")` no crea nada y `raiz.join("proyecto.yml")` es el `proyecto.yml` relativo del cwd. La sonda reprodujo la sobrescritura del manifiesto y la creación de `.escritor`, `manuscrito` y `notas` antes de devolver un error.

La UI normal no puede generar `a/..` ni `""`: rechaza `/`, `\` y `..` en el nombre, exige padre no vacío y el diálogo nativo devuelve una ruta de directorio. Pero el comando Tauri es invocable desde cualquier script del webview; la validación de React no protege al backend.

### 3.4 Notas iniciales, títulos y nombres derivados

- `crear_proyecto` crea las notas con tipos y títulos **fijos** (`personajes`/«Personajes», `tramas`/«Tramas», `lugares`/«Lugares»). No hay título de nota controlado por el usuario ni path injection en ese punto.
- `nombre_nota_libre` usa `model::slug`, que solo deja `[a-z0-9-]`; seguro para el nombre del archivo.
- `Nota::nueva` valida el título; los títulos fijos son válidos.
- El nombre de la carpeta del proyecto sí es editable y se usa en la ruta; la validación de la UI cubre los separadores y `..`, pero el backend no la recibe ni la valida.
- `Proyecto::nuevo` valida título/author con `validar_texto`, que rechaza caracteres de control `< ' '`. No limita longitud, DEL, bidi ni el valor de `idioma`.
- `crear_nota` usa `escritura()`, que valida `.escritor`; si `.escritor` o `notas` se sustituyen por enlaces después de `init`, la escritura de la nota falla o se rechaza, pero el proyecto creado por `init` permanece.

### 3.5 `localStorage` de recientes

`ui/src/recientes.ts`:

- `leerRecientes` hace `JSON.parse`, exige array y filtra solo `ruta` y `titulo` como strings;
- `autor`, `palabras` y `color` **no se validan**;
- no recorta a 8 al leer (el recorte a 8 está al escribir en `App.tsx`);
- `guardarRecientes` captura errores de cuota; correcto;
- la clave es fija (`writer.recientes.v1`).

Consecuencias:

- `autor` o `palabras` con un objeto en el JSON pasan el filtro; `Inicio` los renderiza como hijos de React y React lanza `Objects are not valid as a React child`, dejando el panel roto. Lo confirmé con un render de React 19 y con el filtro de `leerRecientes`.
- `palabras` como string produce totales concatenados (`0...`), no numéricos.
- `color` se interpola en `style={{ background: \`linear-gradient(155deg, ${color}, #2f2732)\` }}`. Un valor manipulado puede intentar una inyección CSS; el CSP de producción limita `img-src` a `'self' asset: data:` y no hay `dangerouslySetInnerHTML`, así que **no reproduje XSS ni exfiltración de red**.
- `ruta` se guarda en claro y se usa para abrir un proyecto; un `localStorage` manipulado puede mostrar una tarjeta con título señuelo y una ruta arbitraria. Requiere acceso de escritura al almacenamiento local o ejecución de script; no es una superficie remota.
- Las rutas y títulos de los proyectos quedan en `localStorage` sin cifrar; es una fuga de metadatos/rutas para quien acceda al perfil del WebView.

No encontré `dangerouslySetInnerHTML`, `innerHTML`, `eval`, `new Function` ni `javascript:` en `ui/src`; React escapa los textos de títulos, autores y errores.

### 3.6 Capacidades y diálogo

`capabilities/default.json` contiene:

```json
["core:default", "core:window:allow-destroy",
 "core:webview:allow-set-webview-zoom", "dialog:default"]
```

No hay `fs`, `shell`, `http` ni plugin de red. `tauri-plugin-dialog` es el único plugin explícito; `tauri-plugin-fs` sigue siendo transitivo de `dialog` y no se inicializa. `tauri.conf.json` no cambia en este commit: CSP de producción con `script-src 'self'`, sin `unsafe-eval`; `devCsp` solo en desarrollo; sin `object-src`/`base-uri`/`form-action`/`frame-ancestors` (SEC-05 vigente).

El diálogo nativo devuelve una carpeta elegida por el usuario; el comando no recibe ni conserva esa concesión. `crear_proyecto` usa Rust `std::fs` y no está limitado por el scope del plugin `fs`; por eso la validación del backend es imprescindible.

## 4. Pruebas adversariales

Todas en `/tmp/opencode/sec21*`; no se tocó el repositorio. Las pruebas de `a/..`, `a/../b`, la sonda de ruta vacía, el symlink y la carrera usan centinelas y directorios desechables.

| # | Prueba | Resultado |
|---|---|---|
| 1 | `writer init 'a/..'` en cwd con `proyecto.yml` centinela | **Falla**: rc 0; manifiesto reemplazado por uno válido; `.escritor`, `manuscrito`, `notas` creados en cwd. |
| 2 | `writer init 'a/..'` en copia de un proyecto real | **Falla**: `proyecto.yml` reemplazado (título «Ataque») y `.escritor/operaciones.log` truncado a 0 bytes; el comando devuelve error por el proyecto anidado, pero el daño ya está hecho. |
| 3 | `writer init 'a/../b'` con `b/proyecto.yml` centinela | **Falla**: rc 0; `b/proyecto.yml` reemplazado y proyecto creado en `b`. |
| 4 | `Project::new("").init(...)` en cwd con `proyecto.yml` centinela (sonda Rust) | **Falla**: manifiesto reemplazado; `.escritor`, `manuscrito` y `notas` creados; después `Err("no existe el directorio del proyecto: ")`. |
| 5 | `writer init link` con `link` → directorio externo vacío | **Falla**: proyecto creado en el destino; el symlink se sigue sin validación. |
| 6 | `writer init link` con `link` → directorio externo no vacío | **OK**: rc 1, no escribe. |
| 7 | `writer init` en directorio no vacío | **OK**: rc 1. |
| 8 | `writer init` en archivo/FIFO/socket | **OK**: rc 1, rápido, sin cuelgue. |
| 9 | `writer init` en symlink colgante | **OK**: rc 1, `File exists`. |
| 10 | Carrera: watcher espera `p/manuscrito`, sustituye `.escritor` por symlink a `ext` | **Falla**: `ext/operaciones.log` reemplazado (616 bytes) y `ext/objetos/<hash>` escrito; reproducido en 2/300 y 8/200 intentos. |
| 11 | `init` + `chmod 0555 notas` + `create nota` | **Falla parcial**: rc 1; el proyecto permanece con 0 notas; con una nota previa, permanece con 1 nota y `verify` OK. El comando Tauri no limpia. |
| 12 | `writer init` con título vacío/control | **Parcial**: rc 1; deja el directorio vacío creado. |
| 13 | `writer init --idioma zz` | **Aceptado**: `idioma: zz` en el manifiesto. |
| 14 | `writer init` con `C:\tmp\win`, `punto.`, `CON` en Linux | **Aceptado** como nombres literales; no hay validación Windows. |
| 15 | Proyecto anidado y proyecto dentro de `.escritor` | **Aceptado**; el `verify` del padre falla con el anidado; el de `.escritor` no lo ve. |
| 16 | `umask 000` + `writer init` | **Falla de permisos**: `0777` en directorios y `0666` en manifiesto/log. |
| 17 | Dos `writer init` concurrentes | **Uno gana, uno falla**; 10/10 `verify` OK; sin reserva atómica. |
| 18 | `localStorage` con `autor:{}` y `palabras:{}` | **Falla de robustez**: el filtro los acepta; React lanza al renderizar objetos. |
| 19 | `localStorage` con `color` arbitrario | **Observación**: se interpola en `style.background`; CSP limita cargas externas; no XSS reproducido. |
| 20 | Búsqueda de sinks XSS en `ui/src` | **OK**: sin `dangerouslySetInnerHTML`/`innerHTML`/`eval`. |
| 21 | Capabilities/plugins | **OK**: sin `fs`, `shell`, `http` ni red; `dialog:default` y zoom. |
| 22 | Regresión de creación normal con 0 y 3 notas | **OK**: 1 escena y 0/3 notas; `verify` OK. |
| 23 | Tests y gates | **OK**: 92 Rust, 3 Tauri, 58 UI, typecheck, calidad, producto, seguridad. |

## 5. Hallazgos

### SEC21-01 — [P2] `crear_proyecto`/`Project::init` no validan la ruta: `..`, ruta vacía y symlinks permiten crear o sobrescribir `proyecto.yml` y `.escritor/operaciones.log` fuera de la carpeta elegida

- **Esperado:** que el comando de creación solo cree un proyecto en la carpeta elegida, que rechace rutas vacías, relativas, con `..`, symlinks/junctions y que no toque datos existentes. La UI valida el nombre, pero el backend es la frontera de seguridad de la IPC.
- **Real:** `crear_proyecto` pasa `ruta` sin validar a `Project::new(ruta).init(...)`. `init` comprueba `raiz.exists()`/`read_dir(raiz)` sobre la ruta literal y después usa `create_dir_all`/`raiz.join(...)`, que resuelven `..` y symlinks. Reproducciones:
  - `writer init 'a/..'` en un cwd con un `proyecto.yml` existente: rc 0, manifiesto reemplazado por uno nuevo y `.escritor/`, `manuscrito/`, `notas/` creados en el cwd.
  - `writer init 'a/../b'` con `b/proyecto.yml` existente: rc 0, `b/proyecto.yml` reemplazado.
  - `Project::new("").init(...)` en un cwd con `proyecto.yml`: el manifiesto se reemplaza y se crean `.escritor`, `manuscrito` y `notas`; después la función devuelve un error, pero el daño ya está hecho.
  - `writer init link` con `link` → directorio externo vacío: el proyecto se crea en el destino del symlink.
- **Impacto:** escritura/sobrescritura fuera de la carpeta elegida, con pérdida del `proyecto.yml` y del `operaciones.log` de un proyecto existente en los casos `a/..`, `a/../b` y ruta vacía. El comando puede devolver éxito o error después de escribir, según el contenido del destino; en ambos casos el efecto externo ya ocurrió. La UI normal no genera esas rutas, pero cualquier script del webview puede invocar el comando; una XSS futura o un renderer comprometido convierte esto en un borrado/sobrescritura de metadatos de proyecto. No hay XSS conocido en la superficie actual, pero la validación de React no es una garantía del backend.
- **Clasificación:** **P2** porque es una escritura fuera de la ruta elegida y una garantía explícita del comando («crear un proyecto en la carpeta padre seleccionada»), no un simple defecto de UX. Si el modelo de amenaza declarara explícitamente que el webview es de confianza y que la IPC no se revisa, podría degradarse a P3; no es el criterio de esta revisión.
- **Corrección sugerida:** validar en el backend, no en la UI:
  1. exigir ruta absoluta, sin componentes vacíos, `.` o `..`;
  2. canonicalizar el padre existente y componer el nombre validado en el servidor (no aceptar la ruta concatenada del cliente);
  3. rechazar symlinks/junctions en la ruta y en `.escritor` (`symlink_metadata`, `O_NOFOLLOW`);
  4. crear el proyecto en un directorio temporal hermano y renombrarlo al final, o usar `create_dir` exclusivo + `openat`;
  5. no seguir enlaces al escribir `proyecto.yml`, `operaciones.log` ni blobs.
- **Regresión sugerida:** `crear_proyecto("")`, `crear_proyecto("a/..")`, `crear_proyecto("a/../b")` con un `proyecto.yml` existente, symlink a directorio vacío/no vacío, y comprobación de que los centinelas no cambian.

### SEC21-02 — [P3] TOCTOU en `Project::init`: `.escritor` sustituido por un symlink hace que `operaciones.log` y blobs se escriban fuera

- **Esperado:** que `.escritor` sea real antes de crear `objetos`, `cuarentena` y `operaciones.log`; que una sustitución por enlace no redirija la escritura.
- **Real:** `init` crea `.escritor/objetos` y `.escritor/cuarentena`, escribe `operaciones.log` y solo después llama a `escritura()`/`validar_escritor`. Un proceso local con escritura en el directorio del proyecto puede esperar a que aparezca `manuscrito`, sustituir `.escritor` por un symlink a un directorio externo y conseguir que `Log::crear_vacio` y `BlobStore` escriban fuera. Reproducido con un watcher: `ext/operaciones.log` (centinela) fue reemplazado por un log de 616 bytes y `ext/objetos/<hash>` fue creado; ocurrió en 2/300 y 8/200 intentos.
- **Impacto:** escritura/sobrescritura de `operaciones.log` y blobs en un directorio externo elegido por el atacante; requiere acceso de escritura al directorio del proyecto nuevo y una carrera local. No se reproduce desde la UI normal sin un atacante local o un directorio precreado escribible.
- **Corrección:** crear en un directorio temporal y renombrar; validar `.escritor` inmediatamente después de crearlo y antes de `Log::crear_vacio`; usar `openat`/`O_NOFOLLOW` o `dirfd` para todas las rutas operativas; no usar `create_dir_all` sobre componentes que pueden ser enlaces.
- **Regresión:** watcher que sustituye `.escritor` durante `init`; debe fallar sin escribir en `ext`.

### SEC21-03 — [P3] La creación no es atómica ni tiene rollback: `init` y `crear_proyecto` dejan proyectos parciales

- **Esperado:** que un fallo de validación, permisos, notas o `abrir_proyecto` no deje un proyecto a medias ni una carpeta huérfana.
- **Real:** `Project::init` crea el directorio antes de validar el título y escribe manifiesto/dirs/log/primera escena en pasos separados. `crear_proyecto` añade las notas y `abrir_proyecto` después, sin `?` con limpieza. Reproducciones:
  - `init` con título vacío/control: rc 1 y deja el directorio vacío.
  - `init` + `chmod 0555 notas` + `crear_nota`: rc 1, el proyecto permanece con 0 notas; con una nota previa, permanece con 1 nota y `verify` OK.
  - Si falla un directorio, el log o la primera escena, queda manifiesto/dirs/log parciales; si falla `abrir_proyecto`, el proyecto queda creado y el comando devuelve error.
- **Impacto:** carpetas huérfanas y proyectos parciales; el usuario ve un error, pero el siguiente intento falla con «directorio no está vacío». No hay pérdida de datos existentes; es deuda de robustez.
- **Corrección:** validar título/ruta/nombre antes de crear nada; crear en un directorio temporal y renombrar al final; si algo falla, eliminar solo lo creado por esta operación; crear las notas dentro de la misma transacción lógica o registrar un estado de creación; devolver un error claro con la ruta del proyecto parcial.

### SEC21-04 — [P4] `localStorage` de recientes no valida tipos ni limita entradas; datos manipulables rompen el panel o inyectan estilo

- **Esperado:** que datos corruptos o manipulados en `writer.recientes.v1` no rompan el panel, no inyecten CSS/HTML y no permitan rutas señuelo.
- **Real:** `leerRecientes` solo exige `ruta` y `titulo` strings. `autor`/`palabras` con un objeto pasan el filtro y React lanza `Objects are not valid as a React child` al renderizar la tarjeta; `palabras` string produce totales concatenados. `color` se interpola en `style.background`; no hay validación contra la paleta. No se recorta a 8 al leer; un array grande puede degradar el render. Las rutas se guardan en claro y se usan para abrir proyectos; una entrada manipulada puede mostrar una tarjeta señuelo con una ruta arbitraria. No hay `dangerouslySetInnerHTML` y el texto se escapa; no reproduje XSS.
- **Impacto:** DoS/estado roto del panel y CSS limitado por CSP; fuga de rutas/metadatos en el perfil del WebView. Requiere acceso de escritura a `localStorage` o ejecución de script local, no una superficie remota.
- **Corrección:** validar/coercionar en `leerRecientes` (`autor` string o null, `palabras` número finito no negativo, `color` de la paleta, longitudes máximas), recortar a 8, sanitizar controles y no usar `color` fuera de la lista. Mantener React sin `dangerouslySetInnerHTML`.
- **Regresión:** JSON con `autor:{}`, `palabras:{}`, `palabras:"1"`, `color` arbitrario y 100 entradas; el panel debe seguir usable.

### SEC21-05 — [P4] Validación Windows/nombres incompleta y permisos dependientes de `umask`

- **Esperado:** que el nombre de carpeta sea válido también en Windows y que los datos del proyecto no queden escribibles por otros usuarios.
- **Real:** la UI rechaza `\ / : * ? " < > |`, `.`/`..` y espacio inicial, pero permite `CON`, `PRN`, `AUX`, `NUL`, `COM1`…`LPT9`, puntos/espacios finales, caracteres de control (si el input los admite), nombres >255 y bidi. El backend no valida ninguna forma Windows; acepta `C:\...`, UNC, `\\?\`, ADS con `:` y rutas de dispositivo si se invoca por IPC. No hay ejecución Windows para confirmar el efecto. Con `umask 000`, `init` crea directorios `0777` y archivos `0666`; `.escritor`, manifiesto y log quedan accesibles a otros usuarios locales.
- **Impacto:** fallos de creación o destinos inesperados en Windows y exposición de metadatos/historial en sistemas multiusuario. No se reprodujo un escape fuera de la carpeta en Windows.
- **Corrección:** validar nombres con la misma lista que el motor (`CON`, reservados, punto/espacio final, controles, longitud, bidi) antes de crear; rechazar rutas de dispositivo/UNC/ADS en el backend si no están soportadas; crear `.escritor`/manifiesto con modos restrictivos (`0700`/`0600`) independientes del `umask`.
- **Regresión:** nombres Windows reservados, `.. ` y `nombre.`; modos con `umask 000`.

### SEC21-06 — [P4] Faltan pruebas negativas y la documentación sobreestima la validación

- **Real:** los tests Tauri de `crear_proyecto` solo cubren el caso feliz con/sin notas; no hay tests de ruta vacía, `..`, symlink, directorio no vacío, permisos, notas fallidas ni rollback. `Project::init` no tiene prueba de ruta inválida. `docs/inicio-v1.md:35` e iteración 15 dicen que se validan «título, carpeta y nombre», pero la validación de carpeta/nombre es solo de UI; el backend no la aplica. `docs/inicio-v1.md:10-11` dice máximo 8 recientes, pero `leerRecientes` no recorta al leer (QA21-04). `HANDOFF.md` está fuera de `docs/revisiones/` en el diff del pipeline.
- **Impacto:** la regresión de seguridad no queda cubierta y la documentación puede dar una garantía falsa. P4.
- **Corrección:** añadir tests negativos de backend y de UI; documentar que la validación del nombre es de UI y que la del backend es la garantía; corregir el contrato de recientes y el alcance del diff (solo docs/revisiones o justificar HANDOFF).

### Vigentes (no nuevos)

- **SEC-04-B — [P3]:** FIFO en `.escritor/objetos/<hash>` referenciado cuelga `verify`/`info`/`list`/`write`/`show`.
- **SEC-04-C — [P3]:** FIFO en `.escritor/indice.sqlite-journal` cuelga `write`/`reindex`/`buscar`.
- **SEC-02-H Windows — [P3 residual]:** enlaces duros de Windows no rechazados (`es_enlace_duro` devuelve `false` fuera de Unix).
- **SEC-04 límites — [P3]:** `hash_archivo`, `Log::cargar`, `verify::escanear`, `cuarentena` y `docx` leen archivos completos sin límite; títulos/autor/ruta nuevos tampoco tienen límite.
- **TOCTOU general — [P3 residual]:** sin `openat`/`O_NOFOLLOW`/comprobación de inodo en las rutas de proyecto; SEC21-02 es una manifestación nueva en `init`.
- **SEC-05 — [P4]:** CSP con `style-src 'unsafe-inline'`, sin `object-src`/`base-uri`/`form-action`/`frame-ancestors`; `dialog:default`/`core:default` amplios; comandos IPC sin allowlist de proyectos. `crear_proyecto` añade una escritura a esa superficie.
- **SEC-06 — [P4]:** deriva documental; ver SEC21-06.
- **`RUSTSEC-2024-0370` — [P4]:** `proc-macro-error` unmaintained por gtk-rs/Tauri; excepción documentada en `deny.toml`.

**Resumen de severidades de esta revisión:** P1: 0 · **P2: 1 (SEC21-01)** · P3 nuevos: 2 (SEC21-02, SEC21-03) · P4 nuevos: 3 (SEC21-04, SEC21-05, SEC21-06) · vigentes: SEC-04-B/C, SEC-02-H Windows, límites, TOCTOU, SEC-05, SEC-06 y `RUSTSEC-2024-0370`.

## 6. Dependencias y capacidades

| Área | Estado |
|---|---|
| Lockfiles | `Cargo.lock`, `apps/desktop/src-tauri/Cargo.lock`, `pnpm-lock.yaml` **sin cambios** en el commit de la Fase 2.1; los gates usan `--locked`. |
| Dependencias Rust | Sin altas/bajas en `Cargo.toml`; `cargo-deny` núcleo/desktop verde. `RUSTSEC-2024-0370` sigue documentado. |
| pnpm | `packageManager` fija pnpm 12.14.0; `pnpm-workspace.yaml` autoriza solo el build de `esbuild`; no se usa npm/npx. |
| Plugins Tauri | Solo `tauri-plugin-dialog` explícito; `tauri-plugin-fs` transitivo de `dialog`, sin capacidades `fs`; no hay `shell`, `http`, updater ni red. |
| Capacidades | `core:default`, `core:window:allow-destroy`, `core:webview:allow-set-webview-zoom`, `dialog:default`. No se amplió `fs`/`shell`/red. |
| CSP | Producción `script-src 'self'`, sin `unsafe-eval`; `style-src 'unsafe-inline'`; faltan `object-src`/`base-uri`/`form-action`/`frame-ancestors` (SEC-05). `devCsp` con `unsafe-eval` solo en desarrollo. |
| Zoom | `zoomHotkeysEnabled: true` y permiso `allow-set-webview-zoom`; ya revisado en Fase 2, sin cambio de superficie. |
| Secretos | gitleaks 46 commits/~1,63 MB y árbol/~1,61 MB: 0 hallazgos; `.gitleaksignore` con dos fingerprints exactos; sin `.gitleaks.toml`. |
| Workflows | No existe `.github/`; `zizmor` se omite. No hay CI remoto ni Dependabot. |
| `unsafe` | `#![forbid(unsafe_code)]` en `crates/core/src/lib.rs`; no hay `Command::new` con entrada de usuario ni shell. |
| Herramientas | Playwright/Chromium, `tauri-driver` y `WebKitWebDriver` se instalan fuera del repo sin hash de artefacto; gitleaks no reporta versión. P4 ya conocido. |

La creación de archivos de `crear_proyecto` usa `std::fs` directamente y **no está limitada por el scope del plugin `fs`**. La única barrera es la validación del comando; hoy no existe.

## 7. Conclusión y decisión

**Decisión: bloqueado.**

La funcionalidad normal de creación funciona y está cubierta: con 0 y 3 notas, la CLI y la prueba Tauri producen un proyecto válido; la UI muestra errores en el modal; los gates de calidad, producto, seguridad, Rust, Tauri y UI están verdes. No hay P1.

Pero hay un **P2 reproducible** en la frontera IPC:

- **SEC21-01:** `crear_proyecto` y `Project::init` no validan la ruta. `a/..`, `a/../b` y la ruta vacía saltan la comprobación de directorio vacío y **sobrescriben `proyecto.yml` y `.escritor/operaciones.log`** de un proyecto existente; un symlink a un directorio vacío redirige la creación. La UI normal no genera esas rutas, pero el comando Tauri es invocable desde el webview y la validación de React no es una frontera de seguridad.

Además quedan:

- **SEC21-02 [P3]:** carrera que sustituye `.escritor` por un symlink durante `init` y escribe `operaciones.log`/blobs fuera.
- **SEC21-03 [P3]:** creación parcial sin rollback (notas, permisos, `abrir_proyecto`, título inválido).
- **SEC21-04 [P4]:** `localStorage` sin validación de tipos/límite; panel rompible y CSS limitado por CSP.
- **SEC21-05 [P4]:** nombres Windows incompletos y permisos dependientes de `umask`.
- **SEC21-06 [P4]:** faltan pruebas negativas y la documentación sobreestima la validación.
- Vigentes: SEC-04-B/C, enlaces duros Windows, límites, TOCTOU, SEC-05 y SEC-06.

Para desbloquear:

1. Validar la ruta en el backend: absoluta, sin `..`/componentes vacíos, canonicalizada, sin symlinks/junctions; recibir padre y nombre por separado y validar el nombre en Rust.
2. Crear el proyecto en un directorio temporal y renombrarlo, o usar `create_dir` exclusivo + `openat`/`O_NOFOLLOW`; validar `.escritor` antes de escribir log/blobs.
3. Añadir rollback/limpieza y validar título antes de crear el directorio.
4. Validar `localStorage`, nombres Windows y modos restrictivos; añadir las pruebas negativas de SEC21-01/02/03.
5. Repetir la revisión de seguridad sobre el commit corregido; después release, UX y product owner.

No hay P1. El P2 de SEC21-01 bloquea el avance hasta que el backend deje de confiar en la ruta construida por el webview. No se abrió la app ni se ejecutaron ventanas; Windows no se probó en runtime.
