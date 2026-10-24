# Fase 2 local/offline — Revisión QA independiente (r3)

- **Commit revisado:** `a671a138bea963cef9b63fe592013cb30f040bf7` (`HEAD`, `main`)
- **Fecha:** 2026-10-23
- **Entorno:** Linux x86_64; Rust 1.99.1; pnpm 12.14.0; Node 26.11.1; Chromium de Playwright; Python 3.14.8.
- **Árbol de trabajo:** limpio antes de las sondas y sin cambios de producto al terminar; el único archivo nuevo es este informe de revisión. `HEAD` sigue en `a671a13`. No se modificó código, tests, contratos ni documentación de producto. Las sondas se hicieron en `/tmp/qa-writer-r3`, con `writer-core` como dependencia de path. No se usaron `npm` ni `npx`.
- **Decisión:** **bloqueado**. Las correcciones de reordenar y de codificación del PDF funcionan, pero el P1 de sidecars no está cerrado: un `plot-grid.json` corrupto o un sidecar de comentario corrupto siguen convirtiendo el proyecto en solo lectura al abrirlo. Además aparece un P2 nuevo: el PDF exportado ignora el `orden` de las escenas, por lo que después de Subir/Bajar el PDF no coincide con el manuscrito.

## 1. Resumen ejecutivo

Los gates están en verde y las dos correcciones P2 de la ronda anterior son reales en su núcleo:

- `Project::intercambiar_orden` intercambia ambas escenas en una sola transacción; la sonda confirma que los dos `orden` cambian y que `verify` queda limpio;
- la UI llama a `intercambiar_orden` y el E2E de reordenar pasa;
- el PDF conserva guion largo, comillas tipográficas, puntos suspensivos, acentos y espacio duro, y ya no imprime `# Encabezado` literal;
- `pdfinfo`, `qpdf --check` y el renderizado con `pdftoppm` confirman que el PDF es válido.

Pero el P1 de la r2 sigue abierto en dos sidecars:

- `verify` sigue añadiendo a `escaneo.errores` el `plot-grid.json` inválido (`verify.rs:451`) y el JSON de comentario inválido (`verify.rs:508`);
- `abrir_proyecto` devuelve esos errores y la UI los convierte en solo lectura (`App.tsx:508-519`, `App.tsx:687-693`);
- aunque `cargarMetas` después lea `leer_plot_grid`/`listar_comentarios` y aparte el archivo, el resumen `proyecto.errores` no se refresca: el proyecto sigue bloqueado hasta cerrarlo y volver a abrirlo.

La sonda reproduce el fallo con `Project::verificar()` y con `Project::escribir_cuerpo`; no es solo un problema de presentación.

Además, el PDF ordena las escenas por ruta (`pdf.rs:59`), no por `escena.orden`; tras un intercambio, la UI y DOCX usan el orden nuevo, pero el PDF mantiene el orden de los nombres de archivo. Es un P2 de exportación en un flujo normal (reordenar y exportar).

## 2. Gates ejecutados

| Gate | Resultado | Evidencia |
|---|---|---|
| `pnpm typecheck` | **OK** | `tsc --noEmit`. |
| `pnpm ui:test` | **OK** | 63/63 en 5 archivos. |
| `pnpm e2e` | **OK** | 37/37 Chromium. Incluye `fase-2-1-reordenar.spec.ts`. |
| `cargo test --workspace --all-targets --locked` | **OK** | 69 unitarias del core + 38 de integración + 25 de CLI = 132; 0 fallos. |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | **OK** | 7/7. |
| `python3 scripts/ci.py calidad` | **OK** | `fmt`, `clippy -D warnings` y 132 pruebas. |
| `python3 scripts/ci.py interfaz` | **OK** | Instalación congelada, typecheck, 63 UI, build, Tauri clippy y 7 pruebas. |
| `python3 scripts/ci.py producto` | **OK** | Autopruebas, fixtures, determinismo, 1.000 escenas y CLI. |
| `python3 scripts/ci.py seguridad` | **OK** | `cargo-deny`, `gitleaks`; `zizmor` omitido por no haber `.github/`. |

No se repitieron `e2e-tauri` ni `empaquetado` en esta ronda porque no formaban parte del alcance pedido. La app instalada, los diálogos nativos, Windows y Word/LibreOffice siguen sin validar; el cierre y `HANDOFF` lo declaran así.

## 3. Sidecars: `apartar_sidecar` y `verify`

### 3.1 Lo que sí funciona

`fsutil::apartar_sidecar` (`fsutil.rs:416-428`) crea `.escritor/cuarentena` si falta, valida el directorio con `directorio_cuarentena`, mueve el archivo con `rename` y hace `fsync` del directorio. Las rutas de llamada son:

- `goals::leer_metas` (`goals.rs:219-229`): JSON inválido o `Metas::validar` fallido → aparta y devuelve `Metas::default()`;
- `goals::listar_sesiones` (`goals.rs:292-300`): un error al leer un mes → aparta el `.jsonl` completo;
- `planning::leer_plot_grid` (`planning.rs:131-142`): JSON inválido o `PlotGrid::validar` fallido → aparta y devuelve el grid por defecto;
- `comments::listar_comentarios` (`comments.rs:299-307`): JSON inválido o `Comentario::validar` fallido → aparta y sigue con el resto.

La sonda confirma que los archivos se mueven y que, una vez leídos, el proyecto vuelve a ser escribible:

```text
metas/verify-antes: errores=[] avisos=[".../metas/metas.json: json: key must be a string ..."] escribible=true
metas/leer: objetivo=None cuarentena=["20261023T214449921Z-metas.json"]
metas/verify-despues: errores=[] avisos=[] escribible=true
metas/escritura-despues: escritura OK rev=4

sesiones/verify-antes: errores=[] avisos=[".../sesiones/2026-10.jsonl: línea 1: ...", "... línea 2: ..."] escribible=true
sesiones/listar: n=0 cuarentena=["20261023T213944920Z-2026-10.jsonl"]
sesiones/verify-despues: errores=[] avisos=[] escribible=true
sesiones/escritura-despues: escritura OK rev=4
```

### 3.2 El P1 que sigue abierto: planificación y comentarios

`verify::verificar_sidecars` no trata igual todos los sidecars:

- metas → aviso (`verify.rs:434`);
- sesiones → aviso (`verify.rs:480`);
- **plot-grid → error** (`verify.rs:451`);
- **comentario → error** (`verify.rs:508`).

La sonda, con un proyecto válido y un solo `plot-grid.json` inválido:

```text
grid/verify-antes: errores=[".../planificacion/plot-grid.json: json: key must be a string ..."] avisos=[] escribible=false
grid/escritura-antes: escritura ERR proyecto: el proyecto tiene errores y no se puede escribir: .../plot-grid.json: ...
grid/leer: cuarentena=["20261023T213940753Z-plot-grid.json"]
grid/verify-despues: errores=[] avisos=[] escribible=true
grid/escritura-despues: escritura OK rev=4
```

Y con un comentario inválido:

```text
comentarios/verify-antes: errores=[".../comentarios/01ARZ3NDEKTSV4RRFFQ69G5FAV.json: json: key must be a string ..."] avisos=[] escribible=false
comentarios/escritura-antes: escritura ERR proyecto: el proyecto tiene errores y no se puede escribir: .../comentarios/01ARZ....json: ...
comentarios/listar: n=0 cuarentena=["20261023T213944949Z-01ARZ3NDEKTSV4RRFFQ69G5FAV.json"]
comentarios/verify-despues: errores=[] avisos=[] escribible=true
```

En la UI el orden es el que produce el bloqueo: `cargar` llama primero a `abrirProyecto` (`App.tsx:644`), que ejecuta `verificar`; recién después llama a `cargarMetas` (`App.tsx:670`), que es donde `leerPlotGrid`/`listarComentarios` apartan el sidecar. Como `proyecto.errores` se guarda en `fijarProyecto(resumen)` (`App.tsx:656`) y no se vuelve a consultar, `soloLecturaProyecto` sigue en `true` (`App.tsx:508`) y `puedeEditar` queda en `false` (`App.tsx:514-519`). El usuario ve el proyecto en solo lectura y no tiene un botón para reintentar la verificación.

Esto incumple:

- `docs/comentarios-v2.md` §10: «`verify` detecta sidecar corrupto y lo aparta sin tocar el manuscrito»;
- `docs/planificacion-v1.md` §9: «Sidecar corrupto se aparta sin tocar notas/escenas»;
- la propia condición de la r2, que pedía demostrar que el manuscrito sigue editable.

La afirmación de `docs/revisiones/fase-2-local-qa-2026-10-23-r2.md:235-251` («un sidecar inválido de metas, planificación o sesiones se aparta… ya no bloquea el proyecto; `verify` los reporta como aviso») es incorrecta para planificación y para comentarios: el JSON inválido sigue siendo `errores`, no `avisos`, y bloquea antes de que la lectura pueda apartarlo.

**Corrección mínima:** en `verificar_sidecars`, para contenido inválido, apartar el sidecar y añadir un aviso (manteniendo como error los enlaces simbólicos y los problemas de integridad); o bien hacer que `abrir_proyecto` lea y aparte los sidecars antes de construir el DTO y vuelva a escanear. Después, refrescar `proyecto.errores` en la UI tras `cargarMetas`. Añadir pruebas de `verify` + escritura para metas, planificación, sesiones y comentarios, y una prueba de UI que abra un proyecto con grid/comentario corrupto y compruebe que se puede escribir.

### 3.3 Observaciones menores de cuarentena

- `apartar_sidecar` mueve el archivo pero **no escribe `informe.jsonl`**. `Project::cuarentena()` lista la copia con `ruta_original: None`, `motivo: None` y `ts: None`. La sonda lo confirma:

  ```text
  metas/informe: archivo=20261023T214449921Z-metas.json ruta_original=None motivo=None ts=None
  ```

  La r2 pedía «mover/copiar … con informe y aviso». El aviso visual existe, la trazabilidad no.
- El destino se forma con `timestamp_slug(now_ms())` y el nombre base; `rename` reemplaza en POSIX si ya existe un archivo con el mismo nombre. No hay bucle anti-colisión como en `verify::cuarentenar` (`verify.rs:800-830`). Es poco probable, pero puede sobrescribir una copia anterior.
- `listar_sesiones` aparta el `.jsonl` completo ante una sola línea inválida. Se pierden de la vista las sesiones válidas de ese mes; el archivo queda en cuarentena, pero no se intenta salvar línea a línea. Es coherente con «archivo corrupto se aparta», aunque conviene registrarlo como decisión.
- `listar_sesiones` ignora el error de `apartar_sidecar` (`let _ =`); si la cuarentena no se puede crear, la corrupción desaparece de la lista sin aviso.
- No hay pruebas de planificación, sesiones ni comentarios corruptos. Solo hay una prueba unitaria de metas (`goals.rs:481-495`) y la de intercambio; el cierre dice «sidecars … con pruebas» con un alcance mayor del real.

## 4. Reordenar escenas

### 4.1 Núcleo

`Project::intercambiar_orden` (`project.rs:724-770`) localiza ambas escenas, exige que compartan carpeta padre, calcula los bytes con los `orden` intercambiados y llama a `commit_efectos` con dos `Cambio::existente`. La transacción es única; si falla la segunda escritura, el mecanismo de rollback de `commit_efectos` revierte la primera.

Sonda con tres escenas en el mismo capítulo (10, 20, 30):

```text
orden/antes: Primera escena=10, A=20, B=30
orden/swap: OK
orden/despues: Primera escena=10, B=20, A=30
orden/listar: ["Primera escena", "B", "A"]
orden/verify: errores=[] avisos=[] escribible=true
```

No hay `Conflict`: el intercambio no pasa por el orden que ya ocupa la vecina, que era el fallo de la r2. La prueba `intercambiar_orden_de_escenas` (`crates/core/tests/proyecto.rs:136-164`) comprueba los dos valores.

### 4.2 UI

`reordenarActivo` (`App.tsx:1022-1046`) selecciona la vecina por `orden` dentro del mismo libro/capítulo, llama una sola vez a `intercambiarOrden(raiz, documento.id, vecina.id)` (`api.ts:744-754`) y recarga con `listar`, que usa `Escaneo::ordenados()` y ordena por `clave_lectura` (libro, capítulo, `orden`, ruta). El comando Tauri `intercambiar_orden` (`lib.rs:1417-1422`) delega en el núcleo. La integración es correcta.

Limitaciones de prueba:

- `e2e/fase-2-1-reordenar.spec.ts` solo pulsa Subir y espera el mensaje «Orden de escenas actualizado.»; no comprueba que las dos escenas hayan cambiado de posición.
- La demo (`api.ts:744-751`) intercambia los valores `orden` dentro del array, pero `demoFiltrado` devuelve el array en su orden original; el E2E no puede detectar un fallo de orden visual en la demo. En Tauri sí se reordena porque `listar` ordena por `orden`.
- No hay prueba de UI de Subir/Bajar en dos escenas reales de un capítulo con el orden visible.

### 4.3 P2 nuevo: el PDF no respeta el orden

`pdf::exportar` ordena por ruta:

```rust
let mut escenas: Vec<_> = escaneo.escenas.iter().collect();
escenas.sort_by(|a, b| comparar(a.relativa.as_str(), b.relativa.as_str()));
```

(`pdf.rs:58-59`). En cambio, DOCX usa `escaneo.ordenados()` (`docx.rs:25`) y Markdown también parte de `ordenados` (`project.rs:1785`). El contrato es explícito: `docs/esquema-v1.md:51` dice que los nombres «no definen el orden de lectura (eso lo hace `orden`)», y §3.1 ordena las escenas por `orden` ascendente.

Sonda: dos escenas A (20) y B (30), con contenido distinto; se intercambian y se exporta el PDF:

```text
orden 10: Primera escena
orden 20: B
orden 30: A
--- PDF después del intercambio ---
Primera escena
A
Contenido A.
B
Contenido B.
```

El PDF mantiene A antes que B; la UI y DOCX muestran B antes que A. Esto es un P2 de exportación: el usuario reordena, exporta y obtiene un PDF con el orden anterior. La prueba `exportar_pdf_genera_documento_local` no cubre el orden.

**Corrección:** usar `escaneo.ordenados()` (filtrando escenas) o `clave_lectura` en `pdf::exportar`, y añadir una prueba que intercambie dos escenas y compruebe el orden en `pdftotext`.

## 5. PDF: caracteres, encabezado y validez

### 5.1 Lo que se pidió

La sonda exportó una escena con:

```text
# Encabezado

—¿Qué tal?— dijo. Añadí café, niño, ¡olé! «comillas» “tipográficas”… guion — y más.
```

`pdftotext -enc UTF-8` devuelve:

```text
Novela
Autora
Primera escena

Capítulo
Encabezado
—¿Qué tal?— dijo. Añadí café, niño, ¡olé! «comillas» “tipográficas”…
guion — y más.

1/1
```

Se comprueba:

- el guion largo `—` conserva su carácter (mapeo WinAnsi `\227` en `pdf.rs:248`);
- las comillas `“ ”`, `‘ ’`, los puntos suspensivos `…`, el euro `€` y el espacio duro se mapean a WinAnsi (`pdf.rs:249-255`);
- los acentos y `ñ`/`¡`/`«»` se conservan (Latin-1/WinAnsi);
- `# Encabezado` ya no sale literal: `limpiar_markdown` (`pdf.rs:89-106`) elimina las almohadillas y el prefijo de cita;
- el PDF es válido: `pdfinfo` informa PDF 1.4, A4, 1 página; `qpdf --check` no encuentra errores de sintaxis ni de stream; `pdftoppm` genera la página sin fallo.

El ajuste de caracteres y el «no literal» del encabezado se pueden dar por cerrados para este alcance.

### 5.2 Lo que sigue pendiente (P3)

El «formato de manuscrito» completo de `docs/exportacion-v2.md` §5 sigue sin implementarse:

- `Formato` solo tiene `fuente`, `interlineado` y `margen` (`pdf.rs:12-31`); no hay sangría de primera línea;
- `maquetar` (`pdf.rs:112-155`) imprime el título del proyecto, autor y título de escena, pero no aplica formato de capítulo/escena ni encabezado repetido por página;
- `limpiar_markdown` elimina `#`, `**`, `__`, `` ` ``, `~~` y `> `, pero no convierte encabezados en estilos ni conserva énfasis; el PDF resultante es texto plano con títulos sueltos;
- no hay numeración de páginas con el formato editorial del contrato (solo el `1/1` inferior).

La r2 ya lo había clasificado como P3 y pedía implementarlo o enmendar el contrato. El commit solo añadió el limpiado de marcas y el mapeo de caracteres; no hay enmienda del contrato ni deuda nueva registrada en el cierre. Se mantiene como P3, no como bloqueo por sí solo, pero no debe presentarse como «formato de manuscrito completo».

## 6. P1/P2 nuevos en los flujos tocados

| ID | Severidad | Hallazgo |
|---|---|---|
| QA-R3-01 | **P1** | `verify` sigue bloqueando el proyecto con `plot-grid.json` corrupto y con un sidecar de comentario corrupto. `escribible=false`, `escribir_cuerpo` falla y la UI queda en solo lectura hasta reabrir. |
| QA-R3-02 | **P2** | El PDF ordena por ruta, no por `orden`; después de Subir/Bajar el PDF no coincide con el manuscrito ni con DOCX. |
| QA-R3-03 | P3 | El formato de manuscrito del PDF sigue incompleto (sangría, encabezado por página, capítulos/escenas). |
| QA-R3-04 | P3 | La cuarentena de sidecars no escribe `informe.jsonl`; las copias quedan sin ruta/motivo/fecha. Posible colisión de nombre por `rename`. |
| QA-R3-05 | P3 | `listar_sesiones` aparta el mes completo ante una línea inválida y oculta errores de cuarentena; no hay pruebas de planificación/sesiones/comentarios corruptos. |
| QA-R3-06 | P3 | El E2E de reordenar solo comprueba el mensaje; la demo no refleja el nuevo orden en el árbol. |
| QA-R3-07 | P4 | `HANDOFF` y el cierre mantienen afirmaciones obsoletas o demasiado amplias (ver §7). |

No se encontró un P1 de pérdida de manuscrito, de seguridad ni de corrupción del proyecto. El P1 es de disponibilidad de escritura, como en la r2. El reordenar y el PDF de caracteres no introducen conflictos ni corrupción.

## 7. `HANDOFF` y cierre: Windows y Word/LibreOffice

**No prometen Windows ni Word/LibreOffice.** Las afirmaciones son correctas en ese punto:

- Windows: `HANDOFF.md:9-10` dice «implementado y validado en Linux x86_64; Windows pendiente de build y pruebas»; `HANDOFF.md:419` repite que no se probó; `docs/iteraciones/25-fase-2-7-cierre.md:68-69` dice «Windows sigue sin build, instalador ni pruebas; release público no autorizado».
- Word/LibreOffice: `HANDOFF.md:50` los pone entre las tareas pendientes antes del release; `HANDOFF.md:194` dice «validación en Word/LibreOffice» pendiente; `docs/exportacion-v2.md:37` y §7 los exigen antes de declarar cerrada la fase, pero el cierre no afirma que se hayan validado. No hay una promesa falsa.

Sí hay documentación desactualizada o demasiado amplia, que conviene corregir antes de la firma:

- `HANDOFF.md:5-6` dice «los P2 de QA corregidos»; es cierto para reordenar y PDF, pero el P1 de sidecars sigue abierto y el propio `HANDOFF` se declara pendiente de re-revisión.
- `docs/iteraciones/25-fase-2-7-cierre.md:76-77` afirma que los ocho P2 se corrigieron «con pruebas (backup, sidecars, DOCX/RTF, exportación v2, temporizador y plantillas)». No hay pruebas de planificación/sesiones/comentarios corruptos ni de la cuarentena de sidecars, y el P1 sigue sin cerrar.
- `HANDOFF.md:45-47` mantiene «La Fase 2 está aprobada… QA r3… sin P1/P2, y el product owner firmó», mientras la línea 5 dice que la re-revisión está pendiente. Hay que separar la aprobación histórica de la Fase 2 original de esta Fase 2 local.
- `HANDOFF.md:180` habla de gráfico de 7 días; la UI usa 30. `HANDOFF.md:183` dice «pendiente… reordenar dentro del capítulo», `HANDOFF.md:185-186` dice que faltan edición/borrado de comentarios y previsualización, y `HANDOFF.md:193` dice que falta PDF nativo, cuando ya existen. Es deuda documental, no funcional.
- `docs/revisiones/fase-2-local-qa-2026-10-23-r2.md:235-251` debe corregirse: la afirmación de que planificación ya no bloquea y de que `verify` lo reporta como aviso no coincide con el código ni con la sonda.
- `docs/iteraciones/25-fase-2-7-cierre.md:18` sigue diciendo «gráfico de 7 días»; la UI usa 30.

## 8. Decisión

**Bloqueado.** No se puede firmar la Fase 2 local en `a671a13`:

1. **QA-R3-01 (P1):** un `plot-grid.json` corrupto o un sidecar de comentario corrupto dejan el proyecto en solo lectura al abrirlo; `verify` los trata como errores y la UI no refresca el estado después de que `cargarMetas` los aparte. Incumple `comentarios-v2` §10 y `planificacion-v1` §9.
2. **QA-R3-02 (P2):** el PDF no respeta el `orden` de las escenas; después de reordenar, el PDF conserva el orden de los nombres de archivo.

Condiciones mínimas para reabrir la firma:

1. Corregir `verificar_sidecars` para que el contenido inválido de planificación y comentarios se aparte y se reporte como aviso (o apartarlo en `abrir_proyecto` antes de devolver el resumen), manteniendo como error los enlaces y problemas de integridad; refrescar `proyecto.errores` en la UI y añadir pruebas de `verify` + escritura y una prueba de UI para grid/comentario corrupto.
2. Hacer que `pdf::exportar` use el orden de lectura (`Escaneo::ordenados` o `clave_lectura`) y añadir una prueba de PDF tras intercambiar escenas.
3. Decidir explícitamente si se implementa el formato completo de `exportacion-v2` §5 o se enmienda el contrato y se registra como deuda; como mínimo, no presentarlo como completo.
4. Añadir informe a `apartar_sidecar` y protección contra colisiones; decidir si las sesiones se salvan línea a línea o se documenta la pérdida del mes.
5. Corregir `HANDOFF` y el cierre para no decir «P2 corregidos con pruebas» sin matices, separar la aprobación histórica y actualizar los estados obsoletos. Mantener a Windows y Word/LibreOffice como no validados, que hoy ya se hace.

## 9. Anexo: sondas reproducidas

Las sondas se ejecutaron en `/tmp/qa-writer-r3` con `writer-core` por path, sin tocar el repositorio.

```text
# Sidecar de metas
metas/verify-antes: errores=[] avisos=[".../metas.json: json: key must be a string ..."] escribible=true
metas/leer: cuarentena=["20261023T214449921Z-metas.json"]
metas/verify-despues: errores=[] avisos=[] escribible=true
metas/escritura-despues: escritura OK rev=4

# Sidecar de planificación
grid/verify-antes: errores=[".../plot-grid.json: json: key must be a string ..."] escribible=false
grid/escritura-antes: escritura ERR proyecto: el proyecto tiene errores...
grid/leer: cuarentena=["...plot-grid.json"]
grid/verify-despues: errores=[] escribible=true

# Sidecar de comentario
comentarios/verify-antes: errores=[".../01ARZ....json: json: key must be a string ..."] escribible=false
comentarios/escritura-antes: escritura ERR proyecto: el proyecto tiene errores...
comentarios/listar: n=0 cuarentena=["...01ARZ....json"]
comentarios/verify-despues: errores=[] escribible=true

# Reordenar
orden/antes: Primera escena=10, A=20, B=30
orden/swap: OK
orden/despues: Primera escena=10, B=20, A=30
orden/listar: ["Primera escena", "B", "A"]
orden/verify: errores=[] avisos=[] escribible=true

# PDF: orden tras el intercambio
orden 10: Primera escena
orden 20: B
orden 30: A
PDF: Primera escena / A / Contenido A / B / Contenido B

# PDF: caracteres
Capítulo
Encabezado
—¿Qué tal?— dijo. Añadí café, niño, ¡olé! «comillas» “tipográficas”…
guion — y más.
```

Antes de crear este informe, `git status --short` quedó vacío; al cierre solo aparece este archivo como no seguido. `git rev-parse HEAD` sigue siendo `a671a138bea963cef9b63fe592013cb30f040bf7`.

## 11. Correcciones posteriores

- **P1 sidecars:** `verify` ya no añade los sidecars corruptos a `errores`; los
  trata como avisos. Los lectores de metas, planificación, sesiones y
  comentarios los apartan a `.escritor/cuarentena/` y devuelven el valor por
  defecto. Prueba de integración: sidecar corrupto → `errores=[]`, proyecto
  escribible.
- **P2 orden del PDF:** `pdf::exportar` ordena por libro, capítulo y
  `escena.orden`, no por ruta. Prueba: tras intercambiar A/B, el PDF muestra B
  antes que A.

Pendiente: repetir los gates y esta verificación sobre el commit de corrección.
