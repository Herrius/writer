# Fase 2 local/offline — Revisión QA independiente

- **Commit revisado:** `33435d09a87e83884875a87dcba589a6e3d0f829` (HEAD, `main`)
- **Fecha:** 2026-10-23
- **Entorno:** Linux x86_64; Rust 1.99.1; pnpm 12.14.0; Node 26.11.1; Chromium de Playwright; `tauri-driver` 2.0.6; `WebKitWebDriver`; X11 (`DISPLAY=:1`); `xdotool`; `gitleaks`; `zizmor`.
- **Árbol de trabajo:** limpio al empezar y al terminar. No se modificó código, tests, configuración ni documentación de producto. Las sondas se hicieron fuera del repositorio, en `/tmp/qa-writer-probe`, con la librería `writer-core` como dependencia de path.
- **Decisión:** **no apto para firmar el cierre funcional todavía** (ver §9). No encontré ningún P1, pero hay varios P2 que afectan a integridad de datos, exportación y estados de UI.

## 1. Resumen ejecutivo

Los gates del commit están en verde, incluidos `e2e-tauri` y `empaquetado`, y los conteos de pruebas coinciden con el cierre. Sin embargo, los tests nuevos no cubren los caminos donde aparecen los fallos:

- el backup **incluye** `bloqueo`, `indice.sqlite*` y `cuarentena/`, aunque el cierre y la iteración 23 afirman lo contrario;
- `crear_backup` permite **sobrescribir archivos del proyecto con una ruta relativa** (reproducido sobre `proyecto.yml`);
- un sidecar corrupto de metas/planificación deja estado del proyecto anterior en memoria y la UI puede **sobrescribir datos válidos del proyecto actual**;
- un solo sidecar de comentarios corrupto **inutiliza todos los comentarios** y `verify` no lo detecta ni lo aparta;
- la exportación DOCX imprime los ULID internos de comentarios v2 y de `[[nota:...]]`;
- la importación DOCX inyecta etiquetas OOXML de tablas en el Markdown y la importación RTF destruye los acentos;
- el temporizador asocia la sesión al documento que está activo al **finalizar**, no al que estaba activo al iniciar, y la racha no usa la meta diaria.

La deuda de PDF nativo, edición/borrado de comentarios, contenido de plantillas y Windows **sí está declarada** en el cierre y en `HANDOFF`; el problema es el titular de “funcionalidad completa” y algunas afirmaciones concretas de las iteraciones, no que esos cuatro puntos se hayan ocultado.

## 2. Gates ejecutados

| Gate | Resultado | Evidencia |
|---|---|---|
| `pnpm typecheck` | **OK** | `tsc --noEmit`. |
| `pnpm ui:test` | **OK** | 63/63 en 5 archivos. |
| `pnpm e2e` | **OK** | 34/34 Chromium. |
| `cargo test --workspace --all-targets --locked` | **OK** | 61 unitarias del core + 36 de integración + 25 de CLI = 122; 0 fallos. |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | **OK** | 7/7. |
| `python scripts/ci.py interfaz` | **OK** | Instalación congelada, typecheck, UI, build, Tauri, clippy y 7 pruebas. |
| `python scripts/ci.py producto` | **OK** | Autopruebas, fixtures, determinismo, 1.000 escenas y CLI. |
| `python scripts/ci.py seguridad` | **OK** | `cargo-deny`, `gitleaks`; `zizmor` omitido por no haber `.github/`. |
| `python scripts/ci.py calidad` (auxiliar) | **OK** | `fmt`, `clippy -D warnings`, tests. |
| `python scripts/ci.py e2e-tauri` (auxiliar) | **OK** | `Ctrl+S` y cierre real; 2/2 escenarios en 145,8 s. |
| `python scripts/ci.py empaquetado` (auxiliar) | **OK** | `.deb` y `.rpm` verificados. |

No se probó la app instalada desde el `.deb`/`.rpm`, ni Word/LibreOffice, ni Windows; el cierre ya lo declara.

### 2.1 Controles que sí funcionan

- `fsutil::validar_ruta_relativa` rechazó `../fuera.txt` en la restauración; el archivo no se escribió fuera del destino (la restauración dejó un destino parcial, QA-12, pero no hubo traversal).
- `leer_limitado` aplica el límite de 20 MB; el DOCX suma tamaños declarados y el lector `zip` limita la salida al tamaño declarado, por lo que el vector clásico de zip bomb está acotado.
- DOCX/HTML no ejecutan macros ni descargan recursos remotos; HTML descarta `<script>` en minúsculas.
- `crear_backup` rechaza symlinks encontrados durante el escaneo (aunque el error de exclusión de QA-01 hace que también pueda fallar por un symlink en cuarentena).
- Los sidecars de comentarios/metas/planificación usan escritura atómica y validan que el directorio operativo sea real; el fallo está en la lectura y en la falta de cuarentena, no en la escritura atómica.
- La restauración rechaza un destino no vacío; `Project::verificar` detecta documentos con IDs fuera de sitio.

## 3. Hallazgos P1

Ninguno.

## 4. Hallazgos P2

### QA-01 — [P2] El backup incluye cuarentena, índice y lock, y la restauración los vuelve a escribir

- **Esperado:** `docs/importacion-v1.md` §7 y la iteración 23 §1/§4: el backup excluye `bloqueo`, `indice.sqlite` y `cuarentena/`; el cierre §3 repite la misma promesa.
- **Real:** `crates/core/src/import.rs:154-159` compara rutas relativas con `"bloqueo"`, `"indice.sqlite"` y `"cuarentena/"`, pero en el proyecto real esas rutas son `.escritor/bloqueo`, `.escritor/indice.sqlite` y `.escritor/cuarentena/...`. La restauración tiene el mismo error en `import.rs:219-220`.
- **Reproducción:** creé un proyecto, añadí `.escritor/cuarentena/copia.md`, `.escritor/indice.sqlite`, `.escritor/indice.sqlite-wal` y `.escritor/bloqueo`, ejecuté `crear_backup` y listé el ZIP. Aparecieron las cuatro entradas:
  ```text
  .escritor/bloqueo
  .escritor/cuarentena/copia.md
  .escritor/indice.sqlite
  .escritor/indice.sqlite-wal
  ```
  Tras `restaurar_backup`, `restaurado/.escritor/cuarentena/copia.md` y `restaurado/.escritor/bloqueo` existen.
- **Impacto:** el backup puede duplicar tamaño y, más importante, **incluye copias de documentos inválidos que el usuario podría enviar a soporte o compartir**. Incumple una garantía explícita del contrato y del cierre. Además, un enlace simbólico dentro de `cuarentena/` hace fallar el backup completo, porque el escaneo sí rechaza symlinks.
- **Corrección:** comparar componentes normalizados, no prefijos de texto:
  `.escritor/bloqueo`, `.escritor/indice.sqlite`, `.escritor/indice.sqlite-wal`, `.escritor/indice.sqlite-shm` y `.escritor/cuarentena/**`.
  Añadir un test que abra el ZIP y afirme qué entradas **no** existen, no solo el round-trip.

### QA-02 — [P2] `crear_backup` puede sobrescribir `proyecto.yml` (y otros archivos) con una ruta relativa

- **Esperado:** el backup no puede escribirse dentro del proyecto; el cierre dice que el destino se valida.
- **Real:** `import.rs:127-131` canonicaliza la raíz, pero compara `destino.starts_with(&raiz)` con el `Path` **sin canonicalizar**. Una ruta relativa nunca empieza por la raíz absoluta. `std::fs::rename` después reemplaza el destino.
- **Reproducción:** con el proceso en la raíz del proyecto, `crear_backup(raiz, Path::new("proyecto.yml"))` devuelve `Ok`. El archivo `proyecto.yml` queda empezando por `PK\x03\x04` (un ZIP) y `Project::verificar()` devuelve un error de esquema del manifiesto (`retorno de carro aislado`). El proyecto queda inservible hasta restaurar el manifiesto.
- **Impacto:** **pérdida de datos/sobrescritura arbitraria** si el comando Tauri se invoca con una ruta relativa. La UI usa el diálogo nativo, que normalmente devuelve una ruta absoluta, por lo que no es el flujo normal; pero el comando es público y no valida el contrato de ruta.
- **Corrección:** exigir ruta absoluta y canonicalizar/resolver el padre del destino (incluidos symlinks), como ya hace `preparar_destino` para exportar; rechazar cualquier destino dentro de la raíz real. Crear el temporal con `create_new` y no seguir symlinks.
- **Nota:** el mismo mecanismo permite que un backup con destino `proyecto.yml` o `../../...` sea aceptado; no es solo un problema de auto-inclusión del temporal.

### QA-03 — [P2] Un sidecar corrupto deja estado del proyecto anterior y permite sobrescribir datos válidos del proyecto actual

- **Esperado:** abrir un proyecto debe cargar sus metas, sesiones y Plot Grid, o dejar la UI en un estado de error de solo lectura; nunca reutilizar datos del proyecto anterior.
- **Real:** `ui/src/App.tsx:526-547` carga los tres sidecars con un único `Promise.all`; si falla uno, descarta los tres resultados y solo hace `setMetas(null)`/`setSesiones([])`. No resetea `plotGrid`, `tramaSeleccionada`, `metaDiaria`, `metaLibro`, `desafioObjetivo` ni `desafioLimite`, y `cargar()` (`App.tsx:550-614`) tampoco los limpia al empezar. `guardarMetasPanel` (`709-743`) y `anadirPuntoGrid` (`1162-1197`) escriben con ese estado.
- **Reproducción (dos proyectos):**
  1. En el proyecto A configuro meta diaria 500 y creo la trama “A”.
  2. En el proyecto B dejo válidos `metas.json` y `plot-grid.json` pero corrompo `.escritor/sesiones/2026-10.jsonl` con una línea inválida.
  3. Abro A y después B. El `Promise.all` de B falla; el panel Plan sigue mostrando la trama “A” (no hay error de Plan) y los campos de Metas conservan los valores de A.
  4. Pulo “Añadir escena activa” o “Guardar metas”: B recibe el `plot-grid`/las metas de A. Si B tenía un grid válido, se pierde al no haberse cargado.
- **Variante de un solo proyecto:** con `plot-grid.json` corrupto, el panel Plan muestra “Sin tramas” sin aviso; “Crear trama” guarda un grid nuevo y **sobrescribe el archivo corrupto** sin cuarentena ni posibilidad de recuperarlo.
- **Impacto:** contaminación cruzada entre proyectos y pérdida de datos recuperables. Es el peor estado roto de la integración de la fase.
- **Corrección:** cargar cada sidecar con `Promise.allSettled`, mantener un error por feature, resetear el estado al inicio de `cargar`, y **prohibir escrituras** de metas/plan mientras la última lectura haya fallado. Programar la cuarentena/recuperación del sidecar antes de permitir guardar.

### QA-04 — [P2] Un comentario corrupto inutiliza todos los comentarios; `verify` no revisa sidecars

- **Esperado:** `docs/comentarios-v2.md` §10 y `docs/esquema-v2.md` §3.2: `verify` detecta un sidecar corrupto, lo aparta y no toca el manuscrito; el resto de hilos sigue disponible.
- **Real:** `comments::listar_comentarios` (`crates/core/src/comments.rs:285-288`) hace `serde_json::from_slice(...)?` por cada `.json`; el primer archivo inválido aborta toda la lista. La UI (`App.tsx:1775-1786`) hace `setComentarios([])` y muestra el error, sin lista parcial ni recuperación. `Project::verificar` no lee `.escritor/comentarios`, `.escritor/metas` ni `.escritor/planificacion` (solo escanea Markdown, `verify.rs:282-410`).
- **Reproducción:** crear un hilo válido, escribir `{no es json` en otro `<ULID>.json` y ejecutar `listar_comentarios`: devuelve error y no se ve ningún hilo. `Project::verificar()` devuelve `errores=[]`, `avisos=[]`.
- **Impacto:** una sola escritura corrupta deja el panel de comentarios inutilizable y el proyecto se declara “sin errores”. No hay forma de borrar el hilo desde la UI (no existe “borrar hilo”), así que la recuperación requiere editar a mano `.escritor/`.
- **Corrección:** leer cada sidecar de forma aislada, ignorar/mover a cuarentena el inválido con aviso y devolver el resto; añadir a `verificar` la comprobación de `comentarios`, `metas`, `sesiones` y `planificacion` que exigen los contratos.

### QA-05 — [P2] La exportación DOCX imprime ULID internos de comentarios v2 y de `@menciones`

- **Esperado:** `docs/exportacion-v2.md` §3: “Los comentarios no se imprimen; el texto marcado sí” y “Las @menciones se imprimen como texto visible, no como ULID”.
- **Real:** `crates/core/src/docx.rs:576-590` reconoce comentarios solo si `interior.split('|').count() == 2`, es decir, la sintaxis v1. Un comentario v2 (`[[comentario:ULID|texto|marcado]]`) tiene tres campos y se deja intacto; `"nota"` no está en la lista de clases y también se deja intacta.
- **Reproducción:** escena con
  `Antes [[comentario:01...|revisar|texto marcado]] y [[nota:01...|ficha de Marta]] después.`
  y `exportar_docx`. En `word/document.xml` aparecen literalmente los ULID y los delimitadores (partidos en varios `w:t`):
  ```text
  Antes [ [ comentario:01...|revisar|texto marcado ] ] y
  [ [ nota:01...|ficha de Marta ] ] después.
  ```
- **Impacto:** el DOCX de revisión/editorial expone identificadores internos y sintaxis que el contrato prohíbe. Afecta a cualquier manuscrito con comentarios v2 o @menciones, no a un caso raro. `README.md:361` también afirma que la exportación DOCX va “sin delimitadores”, lo que ya no es cierto para estas marcas.
- **Corrección:** en `parsear_marca`, tratar `comentario` v2 tomando el tercer campo como contenido visible y añadir `nota` como marca de texto visible; tests de exportación con comentario v2 y @mención, no solo con v1.

### QA-06 — [P2] Importación DOCX/RTF: se inyecta OOXML y se destruyen acentos

- **Esperado:** `docs/importacion-v1.md` §5/§6: DOCX conserva párrafos/encabezados/listas o convierte tablas a texto con aviso; RTF se convierte a texto/Markdown básico sin corromper el contenido.
- **Real (DOCX):** `texto_docx` (`import.rs:271-295`) busca el prefijo `<w:t` con `find`, que también coincide con `<w:tbl>`, `<w:tr>`, `<w:tc>` y `<w:tab/>`. Al encontrar una tabla, toma como “texto” todo el XML hasta el primer `</w:t>`. Resultado de una tabla simple:
  ```text
  Antes
  <w:tr><w:tc><w:p><w:r><w:t>Celda A
  <w:p><w:r><w:t>Celda B
  Después
  ```
  El Markdown importado contiene etiquetas OOXML; `analizarCompatibilidad` lo tratará como HTML y abrirá el documento en solo lectura. Además, `if resto.contains("<w:p")` añade un salto después de **cada run**; dos runs en el mismo párrafo (`Hola` + ` mundo`) se importan como `"Hola\n mundo\nSegundo\n"`, perdiendo la estructura de párrafos.
- **Real (RTF):** `texto_rtf` (`import.rs:358-403`) no interpreta `\'hh` ni `\uNNNN` ni los escapes `\\`, `\{`, `\}`. `{\rtf1\ansi Caf\'e9 y ni\'f1o. \u233?xito\par}` se importa como `"1 Caf'e9 y ni'f1o. 233?xito\n"`.
- **Impacto:** documentos de Word con tablas quedan corruptos o en solo lectura, y cualquier RTF en español con acentos se guarda con el texto dañado. El original no se toca, pero el usuario puede creer que la importación fue correcta.
- **Corrección:** no usar búsqueda de prefijos XML para parsear OOXML; al menos exigir el nombre exacto `<w:t>`/`<w:t ` y manejar `w:tbl`/`w:tab`/`w:br`/`w:p` como estructura. Para RTF, implementar `\'hh`, `\uN` y escapes, o rechazar RTF con aviso si no se va a soportar. Añadir fixtures de tabla, runs y acentos.

### QA-07 — [P2] El temporizador registra la sesión en el documento equivocado

- **Esperado:** `docs/metas-v1.md` §3/§6: la sesión queda asociada al documento activo **al iniciar**; el temporizador sigue funcionando al cambiar de documento.
- **Real:** `iniciarTemporizador` (`App.tsx:745-760`) guarda solo `inicioSesionRef` y `palabrasInicioRef`; no guarda el ID/ruta del documento. `finalizarTemporizador` (`App.tsx:782-804`) usa `activoRef.current` en ese momento y escribe `documento.ruta` y `palabrasDocumento - palabrasInicioRef` del documento actual.
- **Reproducción:** abrir la escena A con 100 palabras, iniciar 5 min, cambiar a la escena B con 0 palabras, finalizar. La sesión queda con `documento = B` y `palabras = 0`; lo escrito en A no se atribuye a A. Si B tiene 200 palabras, se registran 100 palabras en B. Es exactamente el flujo que el contrato dice que debe soportar.
- **Impacto:** estadísticas y metas diarias incorrectas; no hay pérdida del manuscrito, pero sí de la métrica y de la confianza en el panel.
- **Corrección:** guardar `documentoInicioRef` (ruta/ID) y `palabrasInicio` por sesión; al finalizar, usar ese documento y, si se quiere medir el delta del documento actual, impedir/avisar el cambio o guardar snapshots por documento.

### QA-08 — [P2] Las plantillas ignoran el contenido de `manuscrito/` y `notas/` y añaden una escena que la plantilla no define

- **Esperado:** `docs/plantillas-v1.md` §4.3: “Se copia la estructura y las notas de la plantilla”; la ubicación de §2 incluye `manuscrito/` y `notas/`.
- **Real:** `templates::crear_desde_plantilla` (`crates/core/src/templates.rs:152-184`) solo lee `plantilla.json` y genera escenas/notas vacías con `crear_escena`/`crear_nota`; no recorre `manuscrito/` ni `notas/`. Además, `Project::init` crea siempre `001-primera-escena.md` y luego el bucle añade las escenas de la plantilla: una plantilla 1×2×1 produce 3 escenas, no 2 (la prueba de `templates.rs:258-261` lo consolida como comportamiento esperado).
- **Reproducción:** plantilla 1×2×1 sin contenido → 3 escenas (`001-primera-escena.md`, `002-escena-1.md`, `001-escena-1.md`). Plantilla con `manuscrito/.../001.md` que contiene “Contenido personalizado” → el proyecto creado no contiene ese texto.
- **Impacto:** la feature de plantillas no cumple su contrato y el usuario obtiene una escena en blanco que no pidió. La deuda está reconocida en el cierre §4 y en la iteración 24 §3, así que no es una omisión documental; impide, sin embargo, declarar “funcionalidad completa”.
- **Corrección:** decidir si se enmienda el contrato (plantilla solo estructural) o se implementa la copia validada de contenido; en cualquier caso, no crear la escena inicial cuando se usa una plantilla y añadir tests de contenido/estructura exacta.

## 5. Hallazgos P3

### QA-09 — [P3] Semántica de anclas y operaciones de comentarios incompleta

- `sincronizar_huerfanos` (`comments.rs:344-359`) usa `texto.contains(ancla.texto)`. Un ancla `"texto"` se considera presente dentro de `"contexto"`; lo reproduje: `huerfano=false`. No actualiza offsets ni hash, aunque el contrato (§4) lo exige; `inicio`/`fin` quedan siempre en 0 (`comments.rs:202-207`).
- `Comentario::validar` no comprueba que `ancla.texto`/`hash` no estén vacíos, que el ID coincida con el nombre del archivo, que `tipo` y `ancla` sean coherentes, ni que los mensajes `resolucion`/`edicion` correspondan al estado. Un sidecar con ancla `""` y `hash: ""` se acepta (`leer_comentario` devuelve `Ok(Some(""))`).
- El backend acepta texto con `|` y `]]` (`comentario con '|' aceptado: true`); la restricción solo está en el editor. También acepta crear un hilo para un documento inexistente.
- Faltan operaciones del contrato: reabrir, borrar hilo con archivo en `archivados/`, salto al ancla, autor/fecha por mensaje y lista separada de sticky notes. La edición/borrado de mensajes y la previsualización sí están declaradas como deuda; reabrir/borrar hilo/salto no aparecen en el cierre.
- `App.tsx:1768-1791` solo resincroniza huérfanos al cambiar de documento (`activo.id/ruta/proyecto.raiz`), no cuando cambia `activo.cuerpo`: tras borrar el texto anclado y autoguardar, el hilo sigue mostrándose como abierto hasta cambiar de documento.
- Los hilos no usan el lock del proyecto; dos ventanas o dos respuestas rápidas pueden perder mensajes por read-modify-write (`comments.rs:298-311`).
- **Impacto:** estados de huérfano incorrectos y funciones de contrato ausentes; no hay pérdida del manuscrito.

### QA-10 — [P3] El panel Plan permite añadir notas como “escenas” y no muestra los puntos

- `anadirPuntoGrid` (`App.tsx:1162-1197`) usa cualquier `activoRef.current`, sin comprobar que sea una escena. Abrir una nota de personajes, elegir una trama y pulsar “Añadir escena activa” guarda el ULID de la nota como `punto.escena`.
- El panel solo muestra `puntos.length` (`App.tsx:3002-3014`); no hay lista de puntos, etiquetas, escenas desconocidas ni forma de editar/eliminar/mover puntos o renombrar/cambiar color de una trama. La vista gráfica y la edición/borrado están declaradas como deuda, pero no la ausencia total de detalle de puntos ni la adición de notas.
- Crear una nota de tipo `tramas` con “Nueva nota” no la incorpora al Plot Grid; “Crear trama” sí. El usuario puede terminar con notas de trama fuera del grid.
- Las `@menciones` `[[nota:ULID|texto]]` solo se conservan en el Markdown: `ui/src/editor/referencias-plugin.ts` decora únicamente `personaje` y `trama`, no resuelve la nota ni ofrece salto al hacer clic. El cierre dice que falta “resolución de @menciones a títulos”, pero no el clic/diagnóstico de referencia desconocida que exige `docs/planificacion-v1.md` §5.
- **Impacto:** datos de planificación incoherentes y una UI que no permite verificar qué contiene una trama ni navegar por sus referencias.

### QA-11 — [P3] Metas: racha, gráfico y estados no cumplen el contrato

- `racha` (`App.tsx:1857-1873`) cuenta cualquier día con `palabras > 0`; no usa la meta diaria. Con meta 500 y 1 palabra/día la racha sigue creciendo. El contrato §4 dice que se rompe si no se alcanza la meta diaria.
- Solo se cargan las sesiones del mes actual (`App.tsx:526-530`); la racha no puede cruzar de mes aunque recorra 365 días, y el contrato pide probar el cambio de mes. Tampoco hay “mejor racha”.
- El gráfico es de 7 días (`1841-1855`), no de 30 como pide el contrato §4; el cierre lo llama “gráfico de 7 días” sin registrar la reducción.
- El pie muestra siempre `0 hoy` (`App.tsx:3079` y `2484`) aunque el panel calcule `palabrasHoy`; `palabrasHoy` solo suma sesiones, así que escribir sin temporizador no progresa la meta.
- No hay “Limpiar” temporizador (contrato §3), ni archivar metas, ni editar la fecha `desde` de la meta diaria. `archivadas` se conserva pero nunca se modifica desde la UI.
- **Impacto:** métricas engañosas y funciones de contrato ausentes; no toca el manuscrito.

### QA-12 — [P3] La restauración escribe antes de validar todo el ZIP y deja destinos parciales

- `restaurar_backup` valida cada nombre dentro del bucle (`import.rs:210-235`); si un ZIP contiene primero archivos válidos y después `../fuera.txt`, ya ha escrito los primeros y luego devuelve error. Reproducido: `proyecto.yml` y `manuscrito/ok.md` quedan en el destino, `../fuera.txt` no se escribe (la validación funciona), pero el destino queda ocupado y no se puede reintentar sin limpiar a mano.
- No valida manifiesto, hashes ni versión de extensiones antes de escribir, aunque `docs/importacion-v1.md:73` lo exige. `Project::verificar()` al final solo escanea el manuscrito.
- Restaurar en una subcarpeta vacía del proyecto actual está permitido; el proyecto exterior pasa a tener un `.md` con `id` fuera de `manuscrito/`/`notas/` y `verificar` lo marca como error. Reproducido con `restaurar_backup(backup, proyecto/restaurado)`.
- **Impacto:** destinos parciales y proyectos exteriores contaminados; requiere limpieza manual.

### QA-13 — [P3] Límites y UX de importación incompletos

- Los archivos vacíos se aceptan: `importar_markdown`, `importar_html` y `importar_rtf` crean una escena vacía; el contrato §3 dice rechazarlos. Un Markdown con solo frontmatter también se acepta.
- No se aplican los límites de 5.000 párrafos/1.000 imágenes del DOCX ni los 2M de caracteres de HTML/RTF; el DOCX tampoco limita el número de entradas del ZIP.
- HTML: `<SCRIPT>` en mayúsculas no se elimina (el `find("<script")` es sensible a mayúsculas); su contenido aparece como texto (`alert('x')`). `&eacute;`/`&nbsp;` no se decodifican y `<iframe>` deja el fallback. No se descargan recursos, lo cual es correcto.
- La UI etiqueta el botón “Importar Markdown o DOCX” aunque el diálogo acepta HTML/RTF; siempre importa como escena (`tipoNota: null`), nunca como nota; no hay previsualización, progreso ni cancelación; el título para `.html`/`.rtf` conserva la extensión.
- La detección es solo por extensión en la UI; un archivo renombrado cae en Markdown. El contrato §4.2 pide detectar también por contenido.
- Si `escribir_cuerpo` falla después de crear la escena/nota, queda un documento vacío; no hay rollback.

### QA-14 — [P3] Plantillas: symlinks, límites y limpieza

- `listar_plantillas` y `crear_desde_plantilla` siguen enlaces simbólicos de la carpeta de plantilla (`templates.rs:98-119` y `144-150`); una plantilla enlazada se lista y se crea, aunque `docs/plantillas-v1.md` §5 prohíbe symlinks. Reproducido: una plantilla symlink aparece en `listar_plantillas` y `crear_desde_plantilla` devuelve `Ok`.
- El slug acepta `CON` y nombres con espacios/controles (solo rechaza vacío, `/`, `\` y `.`); en Linux funciona, en Windows puede ser inválido. El contrato pide nombre de archivo seguro y nombres reservados de Windows.
- La estructura permite 999×999×999 escenas (≈997 millones) sin límite agregado, progreso ni cancelación; una plantilla local mal formada puede llenar el disco.
- Si falla la creación después de `Project::init`, `remove_dir_all(destino)` borra también una carpeta preexistente vacía que el usuario había elegido.
- **Impacto:** seguridad/robustez y experiencia; no hay ejecución de scripts.

### QA-15 — [P3] Los sidecars no se validan como archivos reales (symlinks)

- `leer_plot_grid`, `leer_metas` y `listar_comentarios` leen `plot-grid.json`, `metas.json` y `<ULID>.json` con `std::fs::read`/`leer_opcional`, sin `fsutil::archivo_real`. Un proyecto con un symlink en esos nombres lee el archivo enlazado; lo reproduje con `plot-grid.json` y `metas.json` apuntando a un JSON externo: ambos devuelven `Ok` y `Project::verificar()` no avisa.
- El resto de la política del proyecto rechaza symlinks en rutas operativas; esta es una inconsistencia de seguridad. El impacto es local (proyecto no confiable abierto por el usuario), pero puede mostrar datos de otro archivo si su contenido encaja con el esquema.

### QA-16 — [P3] El backup no es una foto consistente y la UI no fuerza el guardado

- `crear_backup` no adquiere el lock del proyecto ni sincroniza el log; `hacerBackup` (`App.tsx:1226-1241`) no llama a `guardarAhora()`. Si se pulsa “Crear backup ZIP” dentro de los 900 ms del autosave, el ZIP puede no incluir la última edición; si coincide con un `commit`, puede copiar un `prepare` sin su `commit` o blobs a medio escribir.
- El contrato de backup no exige explícitamente un snapshot transaccional, pero la expectativa de “backup del proyecto” sí. La restauración puede quedar en recuperación por una operación a medias.

### QA-17 — [P3] `verify` no comprueba los sidecars y los contratos lo exigen

- `verify::escanear` solo recorre Markdown y `proyecto.yml`; no llama a `comments`, `goals` ni `planning`. Los contratos `comentarios-v2` §10, `metas-v1` §5 y `planificacion-v1` §9 piden apartar sidecars corruptos y avisar.
- Consecuencia directa: QA-03 y QA-04 no se detectan; un proyecto con `metas.json`/`plot-grid.json` corrupto aparece “sin errores” en el estado global.

## 6. Hallazgos P4

### QA-18 — [P4] Controles visibles sin acción o con etiqueta engañosa

- `Inicio.tsx:325-345` mantiene la pestaña “Plantillas” deshabilitada con “Plantillas próximamente” y `469-475` el botón “Crear desde plantilla” deshabilitado, aunque el selector de plantilla ya existe en el modal (`683-701`). Es una excepción aprobada en `HANDOFF.md:437-439`, pero ahora resulta contradictoria.
- En el panel Plan sin tramas, el `<select>` “Trama” está vacío y “Añadir escena activa” sigue habilitado; al pulsarlo no ocurre nada (`App.tsx:1165-1171`). Es un control sin acción.
- “Agregar nota” y “Responder” están habilitados con texto vacío y no dan feedback (`crearStickyActivo`/`responderHilo` hacen `return` silencioso).
- El pie del editor muestra “0 hoy”/“0 Hoy día” de forma fija aunque haya sesiones (`App.tsx:2484`, `3079`).
- El botón de importar dice solo “Markdown o DOCX” pese a aceptar HTML/RTF (`3059`).

### QA-19 — [P4] Demo de comentarios con IDs duplicados

- `ui/src/api.ts:295` devuelve siempre `01ARZ3NDEKTSV4RRFFQ69G5FAV` para cada comentario y `342` siempre `...FAW` para cada sticky. Dos comentarios en la demo comparten `key`, `respuestas[id]` y resolución; los E2E solo crean uno y no lo detectan. No afecta a Tauri, pero degrada la demo y la fidelidad de los tests de navegador.

### QA-20 — [P4] Documentación desactualizada o contradictoria

- `HANDOFF.md:5` dice “Fase 2 funcional completa y verificada”; `HANDOFF.md:45-47` dice que la Fase 2 “está aprobada con observaciones” y que el product owner “firmó la aceptación”, mientras el cierre `docs/iteraciones/25-fase-2-7-cierre.md:78` dice que no se declara cerrada con firma hasta esta QA. Hay que aclarar que la aprobación firmada es la de la Fase 2 original, no la de las rebanadas locales 2.1–2.6.
- `HANDOFF.md:84-89` conserva 59 pruebas UI, 12 E2E y 3 pruebas Tauri; la realidad es 63, 34 y 7. La sección §6 sí tiene los conteos nuevos.
- `HANDOFF.md:180` dice “gráfico de 7 días” y el contrato `metas-v1` §4 pide 30; el cierre repite 7 sin registrar la reducción.
- `HANDOFF.md:187-189` presenta las @menciones como implementadas; solo hay ida y vuelta, sin resolución ni clic (QA-10).
- `docs/iteraciones/23-fase-2-5-importacion.md:15` y `:41` afirman que el backup excluye lock, índice y cuarentena; es falso (QA-01). `:39` dice que las tablas DOCX se ignoran; se inyectan como XML (QA-06).
- `README.md:361` afirma exportación DOCX “sin delimitadores”; es falso para comentarios v2 y @menciones (QA-05).
- `docs/comentarios-v2.md:24-25` dice que los comentarios v1 se muestran como “comentario heredado”; la UI no tiene esa etiqueta ni entrada de panel para v1, solo la marca en el texto.
- `docs/iteraciones/21-fase-2-2-comentarios.md:58,61-62` marca los filtros como pendientes y luego dice que están implementados; contradicción menor.

## 7. Contraste con el cierre y `HANDOFF` (punto 6 del encargo)

| Afirmación | Estado real |
|---|---|
| PDF nativo con formato de manuscrito | **No implementado**, declarado en el cierre §4 y `HANDOFF` §4. Hoy se usa `window.print()` (`App.tsx:2809-2811`). Correctamente declarado. |
| Edición/borrado de comentarios con historial | **No implementado**, declarado en el cierre §4 y `HANDOFF`. Correctamente declarado. |
| Plantillas con contenido personalizado | **No implementado**, declarado en el cierre §4/iteración 24; pero el contrato sigue diciendo que se copia el contenido. La deuda no exime de la discrepancia contractual. |
| Windows | **No construido ni probado**, declarado en cierre §4 y `HANDOFF`. Correctamente declarado. |
| “Funcionalidad de la Fase 2 implementada y verificada” / “funcional completa” | **Exagerado**: faltan operaciones de contrato (reabrir/borrar hilo, exportación correcta de marcas v2, contenido de plantillas, racha/30 días, etc.) y los P2 anteriores. Los gates verdes no equivalen a contrato cumplido. |
| Backup excluye lock, índice y cuarentena | **Falso** (QA-01); repetido en cierre §3, iteración 23 y no corregido en `HANDOFF`. |
| DOCX solo extrae texto y las tablas se ignoran | **Falso/parcial** (QA-06): inyecta OOXML y pierde estructura de runs. |
| Restauración validada | **Parcial**: valida rutas y destino vacío, pero no manifiesto/hashes/versión y escribe antes de validar todo (QA-12). |

## 8. Cobertura y pruebas que faltan

- Los E2E de navegador usan la demo; `crearBackup`/`restaurarBackup` lanzan error en demo y no hay ningún E2E de importación/backup real. Los bugs de QA-01/02/06 no podían aparecer en `pnpm e2e`.
- `cargo test` del core tiene 6 tests de import/backup, pero el de backup solo comprueba el round-trip y `verificar`, no las exclusiones ni el destino relativo; no hay tests de DOCX con tablas, RTF con acentos, HTML con `<SCRIPT>` mayúsculas ni exportación de comentarios v2/@menciones.
- Los 7 tests de escritorio cubren comentarios/plot/metas básicos, no import/backup/plantillas.
- No hay test de UI para el estado de error de `cargarMetas` ni para la contaminación entre proyectos; el bug QA-03 pasó desapercibido.
- No hay test de la asociación temporizador-documento ni de la racha contra la meta diaria.
- `cargo test` y los E2E no prueban los contratos de `verify` sobre sidecars.

## 9. Decisión

**No aprobar el cierre funcional en `33435d0`.** Los gates están verdes y el trabajo de base es sólido, pero los P2 QA-01 a QA-08 afectan a garantías explícitas de backup, exportación, importación, comentarios y métricas. Firmar ahora convertiría en “deuda” fallos que el usuario puede encontrar en flujos normales (importar un DOCX con tabla, exportar un manuscrito con comentarios v2, cambiar de documento con el temporizador, abrir un proyecto con un sidecar corrupto o compartir un backup con cuarentena).

Condiciones mínimas para reabrir la firma:

1. Corregir QA-01/QA-02 (exclusiones y destino del backup) y añadir tests de contenido del ZIP.
2. Corregir QA-03/QA-04/QA-17 (sidecars corruptos: carga aislada, cuarentena y `verify`).
3. Corregir QA-05/QA-06 (exportación v2/@menciones; DOCX/RTF de importación).
4. Corregir QA-07 y QA-11 (sesión ligada al documento inicial y racha contra la meta).
5. Decidir explícitamente sobre QA-08 (plantillas con contenido) y QA-10/QA-13 (planificación/importación) antes de mantener el titular “completa”.
6. Registrar QA-09, QA-12, QA-14, QA-15, QA-16, QA-18–QA-20 como deuda con dueño y criterio de aceptación, o corregirlos.

Una vez corregidos los P2, repetir los gates y esta revisión sobre el nuevo commit. P3/P4 pueden aceptarse como deuda si el product owner enmienda el contrato o acepta explícitamente cada punto; hoy no están todos registrados en el cierre.

## 10. Correcciones posteriores (2026-10-23)

El builder corrigió los ocho P2 y añadió pruebas de regresión:

| ID | Corrección |
|---|---|
| QA-01 | El backup y la restauración excluyen `.escritor/bloqueo`, `.escritor/indice.sqlite*` y `.escritor/cuarentena/**`; prueba que abre el ZIP y comprueba las ausencias. |
| QA-02 | El destino del backup debe ser absoluto y se resuelve el padre real; se rechaza cualquier destino dentro del proyecto. Prueba con `proyecto.yml`. |
| QA-03 | Carga con `Promise.allSettled`, reset de metas/planificación/sesiones al abrir y bloqueo de escrituras de metas/Plot Grid si la lectura falló. |
| QA-04 | `listar_comentarios` omite sidecars corruptos; `verify` valida comentarios, metas, sesiones y Plot Grid y los reporta como error. |
| QA-05 | `parsear_marca` exporta el texto visible de comentarios v2 y de `[[nota:...]]`; prueba de ambas marcas. |
| QA-06 | DOCX importa solo `<w:t>`/`<w:t ...>` y cierra párrafos en `</w:p>`; RTF interpreta `\'hh`, `\uN` y escapes; pruebas de tabla, runs y acentos. |
| QA-07 | La sesión guarda el documento y las palabras al iniciar; al finalizar usa ese documento y su conteo actual. |
| QA-08 | La plantilla elimina la escena inicial de `init`, copia contenido de `manuscrito/` y `notas/`, y prueba contenido personalizado. |

También se corrigieron varios P3: HTML `<SCRIPT>` en mayúsculas, archivos vacíos,
symlinks en sidecars y plantillas, límite agregado de plantillas, anclas con
límites de palabra y offsets/hash, controles sin acción, edición/borrado de
comentarios, reapertura/archivo de hilos, copia con/sin comentarios, salto al
ancla, PDF nativo con formato configurable, racha contra la meta, 30 días,
selección múltiple de importación y edición de tramas/puntos.

Commit de corrección: `9a7469b`. Pendiente de re-revisión: repetir esta QA
sobre ese commit. La deuda P3/P4 restante sigue en el cierre de la Fase 2.
