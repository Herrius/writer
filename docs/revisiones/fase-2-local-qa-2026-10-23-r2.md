# Fase 2 local/offline — Revisión QA independiente (r2)

- **Commit revisado:** `5b6486df825bffc856c26dfd5b37dd306a26927c` (`HEAD`, `main`)
- **Fecha:** 2026-10-23
- **Entorno:** Linux x86_64; Rust 1.99.1; pnpm 12.14.0; Node 26.11.1; Chromium de Playwright; Python 3.14.8.
- **Árbol de trabajo:** limpio antes y después. `git diff --stat 5b6486d -- ui/src e2e crates` está vacío. No se modificó código, tests, contratos ni documentación de producto. Las sondas se hicieron en `/tmp/qa-writer-r2`, con `writer-core` como dependencia de path.
- **Decisión:** **no firmar todavía**. Los ocho P2 anteriores están corregidos en su síntoma principal salvo QA-04, que deriva en una regresión nueva: un sidecar corrupto deja **todo el proyecto en solo lectura** y no se aparta a cuarentena. Además hay dos defectos nuevos en flujos anunciados: reordenar escenas no funciona y el PDF nativo degrada caracteres habituales del español.

## 1. Resumen ejecutivo

Los gates del commit están en verde y los conteos coinciden con el cierre. La revisión del artefacto confirma que la mayoría de las correcciones son reales:

- el backup ya excluye lock, índice y cuarentena, y rechaza destinos relativos o dentro del proyecto;
- la carga de sidecars usa `Promise.allSettled` y resetea metas/planificación/sesiones;
- la exportación DOCX elimina las marcas v2 y las @menciones;
- DOCX/RTF de importación ya no inyectan OOXML ni destruyen acentos;
- el temporizador guarda el documento y las palabras del inicio;
- las plantillas eliminan la escena inicial y copian contenido de `manuscrito/` y `notas/`.

Pero QA-04 no quedó cerrado: `verify` detecta el sidecar corrupto y lo convierte en **error global**, y la UI usa cualquier error para desactivar la edición. El contrato pide apartarlo a cuarentena y avisar sin tocar el manuscrito. La sonda lo reproduce con un solo JSON inválido en `.escritor/comentarios/`.

Además:

- los botones **Subir/Bajar** de escenas llaman a `reordenar_escena` con el orden que ya ocupa la escena vecina; el núcleo devuelve `Conflict` y nunca se completa el intercambio. No hay drag & drop alternativo (está declarado como deuda), así que hoy no se puede reordenar;
- el **PDF nativo** convierte a `?` los caracteres fuera de Latin-1, incluido el guion largo `—` de los diálogos, y no renderiza encabezados Markdown (`# Capítulo` sale literal). El formato de manuscrito del contrato tampoco está completo (sangría de primera línea, formato de capítulos/escenas y encabezado por página);
- archivar un hilo anclado mueve el sidecar pero deja `[[comentario:...]]` en el Markdown; la operación de contrato «Borrar hilo» no existe y el ancla queda huérfana.

No hay P1 de pérdida de manuscrito ni de seguridad. El P1 que sí aparece es de **disponibilidad de escritura**: un sidecar corrupto bloquea la edición de todo el proyecto y no hay recuperación desde la UI.

## 2. Gates ejecutados

| Gate | Resultado | Evidencia |
|---|---|---|
| `pnpm typecheck` | **OK** | `tsc --noEmit`. |
| `pnpm ui:test` | **OK** | 63/63 en 5 archivos. |
| `pnpm e2e` | **OK** | 36/36 Chromium. |
| `cargo test --workspace --all-targets --locked` | **OK** | 69 unitarias del core + 37 de integración + 25 de CLI = 131; 0 fallos. |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | **OK** | 7/7. |
| `python3 scripts/ci.py interfaz` | **OK** | Instalación congelada, typecheck, 63 UI, build, Tauri, clippy y 7 pruebas. |
| `python3 scripts/ci.py producto` | **OK** | Autopruebas, fixtures, determinismo, 1.000 escenas y CLI. |
| `python3 scripts/ci.py seguridad` | **OK** | `cargo-deny`, `gitleaks`; `zizmor` omitido por no haber `.github/`. |
| `python3 scripts/ci.py calidad` | **OK** | `fmt`, `clippy -D warnings`, 131 pruebas. |

No se repitieron `e2e-tauri` ni `empaquetado` en esta ronda; el alcance pedido no los incluía y no hay cambios en el empaquetado desde la ronda anterior. Windows, la app instalada y Word/LibreOffice siguen sin validar, como declaran el cierre y `HANDOFF`.

## 3. Los ocho P2 de la QA anterior

| ID | Estado | Comprobación |
|---|---|---|
| QA-01 backup excluye lock/índice/cuarentena | **Corregido** | `import.rs:307-315` compara las rutas normalizadas correctas; la sonda abrió el ZIP y no encontró `.escritor/bloqueo`, `.escritor/indice.sqlite*` ni `.escritor/cuarentena/**`. La restauración salta las mismas entradas (`import.rs:280-283`). Test `backup_excluye_lock_indice_y_cuarentena`. |
| QA-02 destino del backup | **Corregido** | `crear_backup` exige ruta absoluta, canonicaliza el padre y rechaza cualquier destino bajo la raíz real (`import.rs:174-196`). Sonda: `crear_backup(raiz, "proyecto.yml")` devuelve error y el manifiesto sigue empezando por `esquema:`. Test `backup_relativo_dentro_del_proyecto_se_rechaza`. |
| QA-03 sidecars y estado entre proyectos | **Corregido en lo principal** | `cargarMetas` usa `Promise.allSettled`, resetea metas/sesiones/grid y limpia los campos (`App.tsx:557-606`); `cargar` resetea antes de abrir (`App.tsx:608-626`). `guardarMetasPanel`, `crearTramaGrid` y `anadirPuntoGrid` bloquean escritura si `errorMetas`. No hay test de UI de contaminación entre proyectos. |
| QA-04 comentario corrupto y `verify` | **Parcial / regresión** | `listar_comentarios` ya omite el sidecar inválido y devuelve el resto (`comments.rs:285-312`), pero `verify` lo añade a `escaneo.errores` (`verify.rs:491-512`) y no a `invalidos`/cuarentena. La UI convierte cualquier error en solo lectura (`App.tsx:508-519` y `681-693`). Sonda: `verificar()` devuelve error, `escribible=false`, cuarentena vacía. Ver §4.1. |
| QA-05 exportación DOCX v2/@menciones | **Corregido** | `parsear_marca` trata comentario v2 y `nota` (`docx.rs:576-613`). Sonda DOCX: `Antes texto marcado y ficha de Marta después.`, sin ULID. Tests `quitar_marcas_v2_y_menciones`. |
| QA-06 importación DOCX/RTF | **Corregido en lo principal** | DOCX exige `<w:t>`/`<w:t ` exactos y cierra párrafos en `</w:p>` (`import.rs:347-384`); RTF interpreta `\'hh`, `\uN` y escapes (`import.rs:443-543`). Tests de tabla, runs y acentos. Queda sin aviso explícito de tabla y sin los límites de párrafos/imágenes del contrato (P3). |
| QA-07 temporizador y documento inicial | **Corregido en lo principal** | `documentoInicioRef` guarda id, ruta y palabras al iniciar (`App.tsx:814-834`); al finalizar usa ese documento (`App.tsx:856-895`). Queda el borde de palabras sin autosave al pulsar Finalizar (P3); no hay test de este flujo. |
| QA-08 plantillas con contenido | **Corregido** | Elimina la escena de `init`, copia `manuscrito/` y `notas/` (`templates.rs:180-265`). Sonda con plantilla 1×2×1: dos escenas, una con «Contenido personalizado», y una nota con «Ficha de Marta». Test `crea_proyecto_desde_plantilla`. |

Resultado: **siete P2 cerrados en su síntoma principal; QA-04 no puede darse por cerrado** por la regresión de solo lectura y la falta de cuarentena.

## 4. Hallazgos nuevos

### QA-R2-01 — [P1] Un sidecar corrupto deja todo el proyecto en solo lectura y no se aparta

- **Esperado:** `docs/comentarios-v2.md` §10: «`verify` detecta sidecar corrupto y lo aparta sin tocar el manuscrito». `docs/metas-v1.md` §5: «Si un archivo está corrupto, se aparta a `.escritor/cuarentena/` y se avisa». `docs/planificacion-v1.md` §9: «Sidecar corrupto se aparta sin tocar notas/escenas».
- **Real:** `verify::verificar_sidecars` añade el error a `escaneo.errores` (`verify.rs:410-513`). `abrir_proyecto` devuelve esos errores (`apps/desktop/src-tauri/src/lib.rs:1093-1126`); `App.tsx:508` define `soloLecturaProyecto` con cualquier error; `puedeEditar` queda en `false` (`App.tsx:514-519`) y `guardarUnaVez` rechaza el guardado (`App.tsx:687-693`). `cuarentenar` solo recorre `escaneo.invalidos`, que únicamente contiene Markdown inválido (`verify.rs:785-839`).
- **Reproducción (sonda):** proyecto válido con una escena; se escribe `{no es json` en `.escritor/comentarios/01ARZ3NDEKTSV4RRFFQ69G5FAV.json`; `Project::verificar()` devuelve:
  ```text
  errores: [".../.escritor/comentarios/01ARZ...json: json: key must be a string at line 1 column 2"]
  avisos: []
  escribible: false
  cuarentena: 0 archivos
  ```
  El mismo patrón se aplica a `metas.json`, a `plot-grid.json` y a una línea inválida de `sesiones/<mes>.jsonl`.
- **Impacto:** el manuscrito está intacto, pero el usuario no puede seguir escribiendo ni desde la UI ni con `escribir_cuerpo_con_base`; no hay botón para apartar el sidecar. Es una regresión del arreglo de QA-04: antes un comentario corrupto inutilizaba los comentarios; ahora inutiliza la edición completa.
- **Corrección:** validar cada sidecar de forma aislada, mover/copiar el inválido a `.escritor/cuarentena/` con informe y aviso, y no convertirlo en error bloqueante del proyecto. Añadir pruebas de `verify` + UI con comentarios, metas, sesiones y grid corruptos.

### QA-R2-02 — [P2] Reordenar escenas con Subir/Bajar siempre falla

- **Esperado:** poder reordenar escenas dentro del capítulo; el cierre solo declara pendiente el drag & drop, no que los botones existentes estén rotos.
- **Real:** `reordenarActivo` intenta intercambiar dos órdenes con dos llamadas consecutivas (`App.tsx:1022-1047`). La primera llama a `reordenar_escena(A, orden_de_B)` y el núcleo rechaza cualquier orden ya usado por otra escena del capítulo (`project.rs:679-707`). Como la primera lanza `Conflict`, la segunda nunca se ejecuta.
- **Reproducción (sonda con dos escenas, órdenes 10 y 20):**
  ```text
  swap 01...HDK (orden 10) <-> 01...M2DB (orden 20)
  first reorder result: Err(Conflict("'orden' 20 ya está usado en el capítulo"))
  second reorder result: Err(Conflict("'orden' 10 ya está usado en el capítulo"))
  ```
- **Impacto:** no hay ninguna forma de reordenar escenas en la app: los botones fallan y el drag & drop está declarado como deuda. Es una función nueva anunciada que no funciona.
- **Corrección:** una operación atómica de intercambio/renumeración en el núcleo (o aceptar un orden temporal) y una sola llamada desde la UI; añadir E2E de Subir/Bajar en un capítulo con dos escenas.

### QA-R2-03 — [P2] El PDF nativo degrada caracteres habituales del español

- **Esperado:** `docs/exportacion-v2.md` §4/§5: PDF generado desde el manuscrito renderizado, con el formato configurado, sin perder texto.
- **Real:** `escapar_pdf` solo conserva ASCII y codepoints ≤ 255; cualquier otro carácter se sustituye por `?` (`pdf.rs:219-239`). El guion largo `—` (U+2014), las comillas tipográficas `“ ”` y los puntos suspensivos `…` caen en ese caso. Son habituales en diálogos y prosa española.
- **Reproducción:** escena con `—¿Qué tal?— dijo. Añadí café, niño, ¡olé! «comillas» — guion.` y `# Capítulo`. `pdftotext` devuelve:
  ```text
  # Capítulo
  ?¿Qué tal?? dijo. Añadí café, niño, ¡olé! «comillas» ? guion.
  ```
  `pdfinfo` y `qpdf --check` confirman que el archivo es un PDF 1.4 válido; el problema es de contenido, no de estructura.
- **Impacto:** el PDF exportado no es fiel al manuscrito en un flujo normal (diálogos con raya). El DOCX sí conserva esos caracteres.
- **Corrección:** mapear la puntuación Unicode a WinAnsi (0x97 para `—`, etc.), incrustar una fuente con `ToUnicode`, o rechazar/avisar los caracteres no representables. Añadir prueba de PDF con diálogo español y `pdftotext`.

### QA-R2-04 — [P3] El PDF no aplica el formato de manuscrito del contrato

- **Esperado:** `docs/exportacion-v2.md` §5: fuente/tamaño, interlineado, márgenes, **sangría de primera línea**, numeración, **encabezado con título/autor** y **formato de capítulos y escenas**.
- **Real:** el formato solo tiene `fuente`, `interlineado` y `margen` (`pdf.rs:12-31`); `maquetar` imprime el título del proyecto, autor y título de escena, pero no renderiza Markdown: un `# Capítulo` sale literal (`pdf.rs:93-138`); no hay sangría de primera línea ni encabezado repetido por página. El título/autor solo aparecen al principio del documento.
- **Impacto:** el PDF es válido y legible, pero no cumple el «formato de manuscrito» que el cierre da por implementado; no se puede usar como PDF editorial sin retoques.
- **Corrección:** completar el formato o enmendar el contrato y registrar la deuda explícitamente. Como mínimo, renderizar encabezados y aplicar sangría de primera línea.

### QA-R2-05 — [P3] Archivar un hilo anclado deja el ancla en el Markdown

- **Esperado:** `docs/comentarios-v2.md` §3: «Borrar hilo: elimina el ancla del Markdown y archiva el sidecar en `.escritor/comentarios/archivados/`».
- **Real:** `archivar_comentario` mueve el sidecar y su propio comentario dice que la UI debe quitar el ancla (`comments.rs:504-523`), pero `archivarHilo` solo actualiza las listas (`App.tsx:1177-1189`). No existe una operación «Borrar hilo» en la UI.
- **Reproducción (sonda):** escena `Antes [[comentario:revisar|texto marcado]] después`; tras archivar, el cuerpo sigue igual y `listar_comentarios` devuelve 0. El `[[comentario...]]` queda como marca muerta.
- **Impacto:** el hilo desaparece del panel y no se puede recuperar desde la UI, pero el ancla sigue en el documento; el Markdown conserva un ULID sin sidecar. No hay pérdida del texto, y «Quitar marca» permite limpiarlo manualmente.
- **Corrección:** al archivar, eliminar del cuerpo la marca concreta conservando el texto marcado (operación de editor o núcleo), o separar «Archivar» de «Borrar hilo» y dar a este último la semántica del contrato.

### QA-R2-06 — [P3] Duplicar/copiar documentos con comentarios deja marcas sin hilo

- **Esperado:** `docs/comentarios-v2.md` §6: al pegar/copiar un comentario con un ULID existente se genera un ULID nuevo y se copia el hilo como conversación nueva; entre proyectos se regeneran.
- **Real:** `Project::duplicar` copia el cuerpo tal cual (`project.rs:968-1017`) y no copia ni regenera los sidecars. Sonda: la copia contiene `[[comentario:revisar|texto]]`, `listar_comentarios(copia)` devuelve 0 y el sidecar original sigue apuntando al documento original. `copiarTexto(false)` además deja `revisar|texto marcado` para un comentario v1 (`App.tsx:1241-1253`).
- **Impacto:** una escena duplicada muestra marcas de comentario que no tienen hilo; el usuario no puede verlos y el Markdown queda con ULID obsoletos.
- **Corrección:** al duplicar/copiar, o bien clonar los hilos con ULID nuevos y reescribir las marcas, o bien eliminar las marcas conservando el texto. Añadir prueba de duplicado con comentario v2.

### QA-R2-07 — [P3] Editar/borrar mensajes no se refleja como operación de UI

- **Real:** el backend añade eventos `edicion`/`borrado` (`comments.rs:456-502`), pero el panel renderiza **todos** los mensajes, incluido el original y el evento, como filas separadas (`App.tsx:3462-3500`). «Borrar mensaje» no oculta el texto original; solo añade «Mensaje borrado.».
- **Impacto:** el usuario ve una conversación duplicada y el botón «Borrar» no borra visualmente nada. El historial append-only es correcto en disco, pero la UI no lo presenta como historial.
- **Corrección:** renderizar el estado vigente de cada mensaje (última `edicion`/`borrado` que lo reemplaza) y dejar el historial en un desplegable; probar borrado y edición en UI.

### QA-R2-08 — [P3] La racha no se recalcula al cambiar la meta diaria

- **Real:** `racha` lee `metas?.diaria?.objetivo` pero su `useMemo` solo depende de `[sesiones]` (`App.tsx:2274-2296`). Al guardar una meta nueva sin crear sesiones, la racha mostrada sigue calculada con la meta anterior.
- **Impacto:** el panel puede mostrar una racha que ya no corresponde a la meta vigente hasta que se registre otra sesión.
- **Corrección:** añadir `metas?.diaria?.objetivo` a las dependencias y probarlo con fechas controladas.

### QA-R2-09 — [P3] Estadísticas y metas siguen incompletas respecto al contrato

- `palabrasHoy` solo suma sesiones (`App.tsx:2255-2258`); escribir sin temporizador no progresa la meta diaria, aunque `metas-v1` §4 pide derivar de sesiones **y manuscrito**.
- No hay «Limpiar» temporizador, «mejor racha», archivado de metas ni edición de la fecha `desde` (`docs/metas-v1.md` §3/§4).
- El gráfico de 30 días usa claves UTC (`toISOString`), no la fecha local; una sesión cerca de medianoche puede caer en el día equivocado.
- La sincronización de huérfanos no se repite cuando cambia `activo.cuerpo`: el efecto depende de `[activo?.id, activo?.ruta, proyecto?.raiz]` (`App.tsx:2188-2208`), no del cuerpo. Tras borrar el texto anclado, el hilo sigue apareciendo como abierto hasta cambiar de documento.
- El salto al ancla solo se ejecuta una vez por texto (`EditorMarkdown.tsx:69-90`); el segundo clic en «Ir al ancla» no vuelve a saltar si el usuario se ha movido.
- No hay lista separada de sticky notes, etiqueta «comentario heredado» para v1 ni resolución/clic de `[[nota:...]]` (`referencias-plugin.ts` solo decora personaje/trama).

### QA-R2-10 — [P3] Plot Grid e importación: detalles de contrato

- El contrato de Plot Grid pide «etiquetar» puntos (`planificacion-v1.md` §2); la UI permite mover/quitar/añadir, pero no editar `punto.etiqueta`.
- Crear una nota de tipo `tramas` con «Nueva nota» no la incorpora al grid; solo «Crear trama» lo hace.
- La importación detecta el formato solo por extensión (`App.tsx:1556-1565` y `import.rs:113-139`), no por contenido; la previsualización muestra solo el primer archivo de una selección múltiple.
- No se aplican los límites de párrafos/imágenes del DOCX ni el aviso explícito de tabla; HTML con entidades nombradas (`&eacute;`) e `<iframe>` conserva fallback (QA-13, P3).

### QA-R2-11 — [P3] Backup/restauración: consistencia y validación

- `crear_backup` no adquiere el lock ni sincroniza el log; el autosave pendiente puede quedar fuera del ZIP (QA-16).
- `restaurar_backup` escribe en un temporal y solo renombra al final, lo que mejora QA-12, pero no valida manifiesto/hashes/versión de extensiones y **ignora `escaneo.errores`**: `Project::verificar()?` devuelve `Ok` aunque el proyecto restaurado tenga errores (`import.rs:299`). La restauración puede declararse correcta y abrirse en solo lectura.
- El temporal `<destino>.restaurando` se borra recursivamente si existe; puede eliminar una carpeta del usuario con ese nombre.

### QA-R2-12 — [P4] Documentación desactualizada o contradictoria

- `HANDOFF.md:5` dice que «los P2 de QA» están corregidos; QA-04 no lo está y hay un P1 nuevo.
- `HANDOFF.md:84-89` conserva 59 pruebas UI, 12 E2E y 3 Tauri; la realidad es 63, 36 y 7 (la sección §6 sí tiene los conteos nuevos).
- `HANDOFF.md:180` y `docs/iteraciones/25-fase-2-7-cierre.md:18` hablan de gráfico de 7 días; la UI ya usa 30.
- `HANDOFF.md:184-194` sigue diciendo que faltan edición/borrado de comentarios, previsualización, PDF nativo, selección parcial y copia de contenido de plantillas, cuando ya existen; y presenta @menciones como implementadas sin la resolución/clic que sigue faltando.
- `docs/iteraciones/25-fase-2-7-cierre.md:76-77` afirma que los ocho P2 se corrigieron «con pruebas»; no hay prueba del temporizador (QA-07), de la carga aislada de sidecars (QA-03) ni de la cuarentena de sidecars (QA-04).
- `HANDOFF.md:45-47` mantiene la frase «La Fase 2 está aprobada… y el product owner firmó», aunque las líneas 7-8 aclaran que son las fases anteriores; conviene separar explícitamente la aprobación histórica de esta Fase 2 local.

## 5. Contraste con `HANDOFF` y el cierre

| Afirmación | Estado real |
|---|---|
| Ocho P2 corregidos | **Siete en su síntoma principal; QA-04 no**: el sidecar corrupto bloquea la edición y no se aparta. |
| Backup excluye lock, índice y cuarentena | **Cierto**; probado abriendo el ZIP. |
| Restauración validada | **Parcial**: valida rutas, destino vacío y escribe en temporal, pero no manifiesto/hashes/versiones ni errores de `verify`. |
| PDF nativo con formato configurable | **Parcial**: existe y es válido, pero pierde `—`/comillas tipográficas y no aplica el formato completo del contrato. |
| Comentarios: archivar hilo | **Parcial**: archiva el sidecar, no elimina el ancla. |
| Comentarios: editar/borrar con historial | **Parcial en UI**: el historial append-only existe, pero la UI muestra original + evento. |
| @menciones implementadas | **Solo ida y vuelta**; no hay resolución a título, clic ni diagnóstico de referencia desconocida. |
| Reordenar escenas | **Deuda declarada solo para drag & drop**; los botones Subir/Bajar están rotos. |
| Windows / Word / LibreOffice | **Correctamente declarados como no validados** en cierre y `HANDOFF`; no se promete lo contrario. |
| Conteos de gates | **Coinciden** con los ejecutados en esta ronda. |

## 6. Cobertura que falta

- No hay E2E de backup/restauración real, importación, PDF, reordenar escenas ni temporizador; los E2E de comentarios solo editan/archivan una sticky.
- No hay prueba de UI de `cargarMetas` con un sidecar corrupto ni de la contaminación entre proyectos.
- `cargo test` cubre backup, DOCX/RTF y plantillas, pero no la cuarentena de sidecars, la asociación temporizador-documento ni la racha contra la meta.
- No hay prueba de PDF con texto español fuera de Latin-1 ni de formato (sangría/encabezados).
- El E2E de escritorio y el empaquetado no se repitieron en esta ronda; la app instalada, los diálogos nativos, Word/LibreOffice y Windows siguen sin validar.

## 7. Decisión

**No firmar la Fase 2 local en `5b6486d`.** La base es sólida y siete de los ocho P2 están corregidos, pero:

1. **QA-R2-01 (P1):** un sidecar corrupto deja el proyecto en solo lectura y no se aparta; incumple `comentarios-v2` §10, `metas-v1` §5 y `planificacion-v1` §9.
2. **QA-R2-02 (P2):** reordenar escenas no funciona con la UI actual.
3. **QA-R2-03 (P2):** el PDF pierde caracteres habituales del español.

Condiciones mínimas para reabrir la firma:

1. Corregir la cuarentena/aislamiento de sidecars y demostrar que el manuscrito sigue editable; añadir tests de `verify` y UI para comentarios, metas, sesiones y grid corruptos.
2. Implementar el intercambio atómico de orden de escenas y un E2E de Subir/Bajar.
3. Corregir la codificación del PDF y decidir explícitamente si el formato completo (sangría/encabezados/capítulos) se implementa o se enmienda el contrato; añadir prueba con diálogo español.
4. Cerrar o registrar como deuda con dueño y criterio de aceptación: ancla al archivar, duplicado con comentarios, presentación de edición/borrado, racha/estadísticas, etiquetas de Plot Grid, @menciones, límites de importación y validación de restauración.
5. Actualizar `HANDOFF` y el cierre para que no digan «P2 corregidos», «PDF con formato» o «7 días» sin matices.

Los P3/P4 restantes pueden aceptarse como deuda si el product owner los registra explícitamente; hoy varios no aparecen en el cierre.

## 8. Anexo: sondas reproducidas

Las sondas se ejecutaron en `/tmp/qa-writer-r2/probe` con `writer-core` por path, sin tocar el repositorio.

```text
# Reordenar
first reorder result: Err(Conflict("'orden' 20 ya está usado en el capítulo"))
second reorder result: Err(Conflict("'orden' 10 ya está usado en el capítulo"))

# Sidecar corrupto
verify errors: [".../.escritor/comentarios/01ARZ...json: json: key must be a string at line 1 column 2"]
escribible: false
cuarentena files: Some(0)

# Archivar hilo
body after archive: "Antes [[comentario:revisar|texto marcado]] después\n"
listar comments after archive: 0

# PDF
# Capítulo
?¿Qué tal?? dijo. Añadí café, niño, ¡olé! «comillas» ? guion.
```

Los gates completos y sus conteos están en §2; el árbol quedó limpio y el diff con el commit congelado sigue vacío.

## 10. Correcciones posteriores

Se corrigieron los bloqueos de esta re-revisión:

- **P1 sidecars:** un sidecar inválido de metas, planificación o sesiones se
  aparta a `.escritor/cuarentena/` y se devuelve el valor por defecto; ya no
  bloquea el proyecto. La lista de comentarios ya lo hacía y `verify` los
  reporta como aviso.
- **P2 reordenar:** `Project::intercambiar_orden` cambia ambas escenas en una
  sola transacción; la UI usa ese comando.
- **P2 PDF:** se mapean guion largo, comillas tipográficas, puntos suspensivos
  y espacio duro a WinAnsi, y se limpian encabezados Markdown.

Además se completaron comentarios (editar/borrar/reabrir/archivar, copiar,
salto al ancla), PDF nativo configurable, racha contra meta y 30 días,
Plot Grid editable, selección/previsualización de importación y reordenar
escenas.

Pendiente: repetir los gates y esta revisión sobre el commit final.
