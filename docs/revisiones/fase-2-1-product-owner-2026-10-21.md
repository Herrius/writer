# Fase 2.1 — Aceptación del product owner

- **Commit de código:** `e0bb3fc3f8f2cbc7a12a7c8ff5f586adac51a002`
- **HEAD de documentación:** `83f5d725ab7d115ac9e8bc73d576f5468b8b794e`
- **Fecha:** 2026-10-21
- **Decisión:** **aprobado con observaciones**
- **Fase 2.1:** **aprobada y cerrada** como hito de producto local Linux x86_64.
- **Release público:** **no autorizado**; ver §4.
- **Fases 3–5:** **pospuestas**; no se reactivan con esta firma.
- **Siguiente bloque autorizado:** **Bloque 2.2 — Pulido del panel de Proyectos, accesibilidad y validación en app real** (continuación de la Fase 2, no Fase 3); ver §5.

Árbol limpio antes de escribir este informe (`git status --porcelain` vacío). `git diff --stat e0bb3fc..HEAD` muestra 9 archivos, 785 inserciones y 7 borrados, todos Markdown. **No está limitado estrictamente a `docs/revisiones/`**: incluye también `HANDOFF.md`, como ya registró REL-07. No hay cambios de código, tests, configuración ni dependencias; lo acepto como observación de proceso, no como defecto de producto.

## 1. Qué se entrega

La Fase 2.1 entrega el panel de inicio y la creación de proyectos desde la GUI, sobre la Fase 2 ya aceptada:

- **Panel de Proyectos:** cabecera con marca, buscador, filtros/colecciones deshabilitados, pestañas Activo/Plantillas/Papelera, «Mis Proyectos», tarjeta «Crear nuevo proyecto», estado vacío, sidebar de bienvenida/estadísticas, meta diaria deshabilitada y cuaderno «Ideas».
- **Asistente de creación:** nombre de carpeta derivado del título, título, autor, idioma (`es`/`en`), carpeta padre mediante diálogo nativo, color de portada, notas iniciales de personajes/tramas/lugares y botones `Guardar` / `Guardar y abrir`.
- **Comando Tauri `crear_proyecto`:** sobre `Project::init`; crea manifiesto, `.escritor/`, Libro 1 / Capítulo 1 / Primera escena y, opcionalmente, las tres notas iniciales. Valida la ruta y el nombre en backend.
- **Recientes:** persistencia en `localStorage` (`writer.recientes.v1`), máximo 8, con título, autor, palabras y color; validación de tipos, longitudes, color y deduplicación al leer; la demo de navegador no contamina los recientes reales.
- **Integración:** `abrir_proyecto`, `listar`, `leer`, `cuarentena` y el editor de la Fase 2 siguen funcionando; `Guardar` deja el proyecto en recientes y `Guardar y abrir` abre el editor.
- **Pruebas y artefactos:** 94 pruebas Rust (34 unitarias + 35 de integración + 25 de CLI), 3 de la cáscara Tauri, 58 de UI y 10 flujos E2E de navegador. `.deb` de 5.252.084 B (SHA-256 `a15bee6b…ce94`) y `.rpm` de 5.252.306 B (SHA-256 `fabca278…b411`), con los assets `index-YQtoJIi0.js`, `index-BVfPg3Gh.css` y `window-CO1JT7GA.js` embebidos.

**Verificación propia de esta aceptación.** Ejecuté, con `nice -n 19`, `cargo test --workspace --all-targets --locked` (94/94), `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` (3/3), `pnpm ui:test` (58/58), `pnpm typecheck` y `pnpm e2e` (10/10). Revisé en headless el panel vacío, el panel con cuatro recientes, la vista de cuadrícula, la vista de lista y el modal a 1440×900 y 960×640. Creé un proyecto real por CLI con título, autor, idioma y tres notas, y comprobé `verify` y el árbol `Libro 1 / Capítulo 1 / Primera escena`. Comprobé los hashes de los paquetes. **No ejecuté `e2e-tauri`** porque abre la app y la restricción lo prohíbe; para el cierre de ventana y el diálogo nativo me apoyo en la evidencia del builder y en las limitaciones declaradas.

## 2. Criterios de la fase 2.1

| Criterio | Resultado | Evidencia y límites |
|---|---|---|
| **1. Crear un proyecto desde la GUI** | **Cumplido con observaciones** | El formulario valida, llama a `crear_proyecto` y distingue `Guardar` de `Guardar y abrir`; la prueba Tauri 3/3 crea con y sin notas, y el arnés de IPC de QA/UX confirma argumentos y secuencia. **No se ejecutó el flujo completo con el diálogo nativo en la app real**: QA y UX usaron IPC simulado y el builder declara que el E2E de escritorio no prueba ese diálogo. Es una condición de validación, no un defecto reproducido. |
| **2. Estructura inicial: Libro 1 / Capítulo 1 / Primera escena** | **Cumplido** | Ruta `manuscrito/01-libro/01-capitulo/001-primera-escena.md`; manifiesto con título, autor e idioma; `verify` OK. Lo reproduje por CLI y coincide con la prueba Tauri. |
| **3. Notas iniciales opcionales** | **Cumplido** | Con la casilla activa se crean `personajes`, `tramas` y `lugares`; con ella desactivada, 0 notas. La prueba Tauri cubre ambos casos y la CLI reproduce 1 escena + 3 notas con `verify` OK. |
| **4. Abrir, guardar y recientes** | **Cumplido** | `Guardar` añade el proyecto al frente de recientes sin abrirlo; `Guardar y abrir` lo abre y muestra el banner; las tarjetas abren el proyecto; el máximo es 8; la demo no persiste; la lectura de `localStorage` valida tipos, color y longitudes. |
| **5. El panel replica razonablemente las capturas** | **Cumplido con observaciones** | La jerarquía, cabecera, buscador, pestañas, tarjeta de creación, estado vacío, sidebar, cuaderno y modal son fieles a la referencia. Las diferencias de marca, avatar, ilustración, portada con imagen, subtítulo y controles deshabilitados son cosméticas o límites declarados. La **vista de lista está rota** (QA21-01) y el **pie del modal queda fuera a 960×640 y 200 %** (QA21-03); ambas son deuda P3 prioritaria. |
| **6. Sin regresiones de la Fase 2** | **Cumplido** | 94/94 Rust, 3/3 Tauri, 58/58 UI, 10/10 E2E; editor, autosave, búsqueda, historial, DOCX, marcas y cuarentena siguen verdes. No hay P1/P2. |
| **7. Pipeline y release local** | **Cumplido con observaciones** | QA, seguridad r3, release y UX aprobaron con observaciones; seguridad y release revisaron `e0bb3fc`, y UX confirmó sobre el commit final los 14 hallazgos de QA sin regresiones nuevas. Release genera deb/rpm verificados. Sigue sin firma, updater, AppImage, Windows/macOS, CI remoto y prueba de app instalada. |

El objetivo central de 2.1 se cumple a nivel de implementación e integración: el formulario está conectado a un comando que crea proyectos reales, con la estructura inicial y notas opcionales, y los recientes permiten volver a abrirlo. La salvedad es de **verificación end-to-end en la app real** y de acabado, no de arquitectura ni de pérdida de datos.

## 3. Réplica y valor de producto

**La réplica es razonable y reconocible.** El panel conserva la estructura de la referencia: marca y barra superior, título «Proyectos», buscador, filtros y pestañas, «Mis Proyectos», tarjeta de creación, estado vacío, sidebar con bienvenida/estadísticas/meta y cuaderno «Ideas». El modal mantiene la composición de la referencia: pasos laterales, formulario, portada reactiva, paleta de 12 colores y pie con `Guardar` / `Guardar y abrir`. La adaptación a `writer` es deliberada y no copia marcas ni datos de Dabble.

**Lo que funciona bien.** La creación es real y conservadora: el motor vuelve a validar la ruta y el nombre, rechaza destinos no vacíos, enlaces y rutas con `.`/`..`, y deja un proyecto verificable. `Guardar` y `Guardar y abrir` tienen semánticas distintas y comprensibles. Los recientes persisten, se filtran por título y ruta, muestran autor, palabras y color, y no se contaminan con la demo. La interfaz es clara y utilizable a 1440×900 y 1280×820; el proyecto de 1.000 escenas carga en ~319 ms en el arnés de UX.

**Lo que todavía no es.** No es un gestor completo de proyectos ni un release público:

- la vista de lista es una cuadrícula de una columna con portadas gigantes (1.012×1.582 px), no una lista;
- a 960×640 —el tamaño mínimo de la ventana— el pie del modal queda por debajo del área visible y los botones no se descubren sin scroll interno;
- el modal no atrapa el foco, `Escape` falla si el foco salió al fondo y no devuelve el foco al abridor;
- «Mis Proyectos» son 8 recientes, no todos los proyectos; no hay botón visible para abrir uno fuera de la lista;
- tras abrir un proyecto no hay vuelta al panel;
- con un filtro activo, `Guardar` puede crear el proyecto sin que se vea y sin mensaje; un error anterior puede seguir visible;
- el recuento de palabras no baja a 0 y algunos controles parecen activos pero no hacen nada;
- «Ideas» abre el asistente de creación; hay diferencias de copy, plurales, contraste y nombres accesibles;
- no hay importación, plantillas, subtítulo, portada con imagen, renombrar/borrar/mover ni un registro completo de proyectos (límites declarados).

**Valor de producto.** 2.1 cierra el gap más importante de la Fase 2: ya no hace falta la CLI para empezar a escribir. El panel y el asistente convierten el motor en un producto que se puede abrir y usar. Es suficiente para cerrar 2.1 como hito local Linux, pero no para llamarlo release: los defectos de lista, foco, tamaño mínimo y navegación afectan al uso diario y deben resolverse antes de publicar.

**P1/P2 abiertos.** Ninguno en QA, seguridad r3, release ni UX. La QA revisó `376c0b8` y la UX revisó `e0bb3fc`; la UX confirmó que los 14 hallazgos de QA siguen presentes y no hay regresiones nuevas de los P2 de la Fase 2. No reabro revisiones técnicas: acepto la deuda P3/P4 como deuda de producto y la convierto en condiciones del siguiente bloque.

## 4. Observaciones aceptadas y condiciones

### 4.1 Observaciones aceptadas (no bloquean la fase)

| Origen | Observación | Tratamiento |
|---|---|---|
| QA/UX | **QA21-01:** vista de lista rota; tarjetas de 1.012×1.582 px y página de 5.276 px. | P3 visible. Aceptada como deuda, pero **prioridad 1** del Bloque 2.2: implementar una lista real (miniatura + fila) o retirar el conmutador hasta hacerlo. |
| QA/UX | **QA21-02:** sin trampa de foco, `Escape` parcial, sin retorno de foco, 12 colores antes de Título. | P3 de accesibilidad. Prioridad 1 del Bloque 2.2; añadir E2E de teclado. |
| QA/UX | **QA21-03:** pie del modal fuera del viewport a 960×640 y 200 %; acciones no descubribles. | P3 de layout. Prioridad 1 del Bloque 2.2; pie sticky o layout que quepa en el mínimo soportado. |
| QA/UX | **QA21-04:** los 8 recientes se presentan como «todos los proyectos»; no hay abrir fuera de la lista. | P3 de producto. Prioridad 2: botón «Abrir proyecto» siempre visible y etiquetar las estadísticas como recientes, o mantener un registro completo. |
| QA/UX | **QA21-05:** no hay vuelta al panel desde el editor. | P3 de navegación. Prioridad 2: «Proyectos»/«Inicio» que guarde y vuelva al panel. |
| QA/UX | **QA21-06:** filtro activo oculta el proyecto recién creado; error anterior persiste. | P3 de feedback. Prioridad 2: limpiar filtro/error o mostrar confirmación con «Ver proyecto». |
| QA/UX | **QA21-07:** el recuento de palabras no baja a 0. | P3 de datos de UI. Prioridad 2: usar `??` en lugar de `||` y actualizar el total. |
| QA/UX | **QA21-08:** Notificaciones, Ayuda, chevron de «Mis Proyectos» y «Más opciones» son botones sin acción. | P3 de acabado. Prioridad 3: implementar o deshabilitar con «próximamente». |
| QA/UX | **QA21-09:** documentación imprecisa: iteración 15 dice que `Guardar` abre el proyecto; README dice 2 pruebas Tauri y hay 3; `inicio-v1` dice «número de proyectos» y son recientes. | P3 documental. Corregir en el mismo bloque; no bloquea el producto. |
| QA/UX | **QA21-10 a QA21-14 y UX21-15:** Ideas abre el asistente; «1 proyectos» y formatos; nombres accesibles/landmarks; slugs no latinos; creación anidada/carpetas vacías; contraste 4,31:1. | P4 de acabado, internacionalización, accesibilidad y robustez. Aceptados como deuda; corregir los de bajo coste en 2.2 y el resto antes de release. |
| Seguridad r3 | **SEC21-02:** TOCTOU de `.escritor` en `Project::init` (25/300 intentos con atacante local). | P3 residual. Aceptado para 2.1; endurecer con temporal + rename o `openat`/inodo antes de release. |
| Seguridad r3 | **SEC21-03:** creación parcial sin rollback (título inválido deja carpeta vacía; fallo de notas deja proyecto parcial). | P3 de robustez. **Prioridad 1 del Bloque 2.2**: validar antes de crear y limpiar/rollback si falla la operación completa. |
| Seguridad r3 | **SEC21-04/05/06:** rutas señuelo en `localStorage`, nombres Windows no cubiertos, `umask`, cobertura de tests y documentación. | P4 residuales. Aceptados; añadir tests y documentar antes de release. |
| Release | **REL-01/02/03/09:** binario sin strip, paquetes no reproducibles, toolchain flotante y versión `0.2.0` repetida con contenido distinto. | P3 de release. Condiciones de publicación en §4.2. |
| Release | **REL-04 a REL-08:** baseline glibc, metadatos incompletos, gate de empaquetado parcial, trazabilidad y restos de AppImage. | P4 heredados. Aceptados como deuda de release. |
| Fase 2 | **SEC-04-B/C, SEC-02-H Windows, límites de tamaño, TOCTOU general y P3/P4 de UX de la Fase 2.** | Deuda ya aceptada en la firma de la Fase 2. Sigue vigente; no es una regresión de 2.1. Se prioriza antes de un release público o de reactivar IA/MCP. |
| Proceso | El flujo completo de creación con el diálogo nativo en la app real no lo ejecutó ningún revisor independiente; el `e2e-tauri` del builder cubre guardar y cierre, no creación. | **Condición de validación**, no defecto: añadir un E2E real de creación con diálogo nativo en el Bloque 2.2. |
| Proceso | `git diff e0bb3fc..HEAD` incluye `HANDOFF.md`, no solo `docs/revisiones/`; todo es documentación. | Observación de trazabilidad (REL-07). El commit de release debe congelar también `HANDOFF.md` y declarar el HEAD exacto. |

### 4.2 Condiciones para publicar el release (siguen abiertas)

1. **Versión e identidad.** Subir a `0.2.1` (o añadir `Release`/build metadata) antes de publicar; no volver a distribuir un `0.2.0` con contenido distinto. Asociar cada SHA-256 al commit y al artefacto.
2. **Seguridad del commit publicado.** Revisión de seguridad del commit exacto de release, incluidas las rutas de creación, `SEC21-02/03`, los FIFO `SEC-04-B/C` y los límites de tamaño. Publicar `SHA256SUMS`; firmar `.deb`/`.rpm` o documentar formalmente la excepción; decidir updater y notarización.
3. **Matriz de plataformas.** Construir y probar Windows/macOS o declarar que el release soportado es solo Linux; decidir AppImage o documentar deb/rpm como canales oficiales. Probar la app **instalada** en máquina limpia: abrir, crear proyecto con diálogo nativo, editar, buscar, historial, exportar DOCX y recientes.
4. **DOCX en Word real.** Validar en Word o LibreOffice un manuscrito representativo; si Word no lo abre o pierde formato, es un P2 de release.
5. **CI remoto y reproducibilidad.** Publicar los gates en CI remoto con la matriz declarada; fijar toolchain, usar `--locked`, aplicar strip/remap y normalizar timestamps o documentar la no reproducibilidad.
6. **Documentación.** Corregir el desglose de pruebas del `README.md` (3 pruebas Tauri, no 2), la numeración duplicada de `docs/inicio-v1.md`, la afirmación de «número de proyectos» y el estado de la Fase 2.1 en `HANDOFF.md`/plan; enlazar este informe en `docs/revisiones/README.md`.
7. **Pulido de producto.** Cerrar o aceptar explícitamente el backlog P3 de 2.1; como mínimo, la lista, el modal a 960/200 %, el foco, la vuelta al panel y el feedback de `Guardar`. No es necesario convertir 2.1 en perfecta para publicar una beta, pero no se debe publicar con controles visibles que no funcionan.
8. **Regla de bloqueo.** Cualquier P1/P2 que aparezca en estas comprobaciones bloquea la publicación hasta corrección y regresión.

## 5. Firma y siguiente bloque

**Decisión: `aprobado con observaciones`.**

La Fase 2.1 cumple el objetivo de producto: el panel de Proyectos replica razonablemente las capturas, el asistente está conectado a un comando que crea proyectos reales con manifiesto, Libro 1 / Capítulo 1 / Primera escena y notas opcionales, `Guardar` y `Guardar y abrir` se distinguen, y los recientes persisten y permiten reabrir. Los gates están verdes, seguridad y release revisaron el commit final y no hay P1/P2. Las observaciones P3/P4 son deuda de acabado, accesibilidad, navegación, robustez y documentación; se aceptan, con prioridades claras.

**Firma:** Product owner del proyecto `writer` — 2026-10-21.  
**Fase 2.1:** aprobada con observaciones y cerrada como hito local Linux.  
**Release público:** no autorizado.  
**Fases 3, 4 y 5:** no autorizadas; siguen pospuestas por decisión del usuario.

**Siguiente bloque autorizado: Bloque 2.2 — Pulido del panel de Proyectos, accesibilidad y validación en app real.** Es continuación de la Fase 2, no una reactivación de las fases 3–5. Alcance autorizado:

1. **Corregir los P3 de producto prioritarios:** QA21-01 (lista real o retirar el conmutador), QA21-02 (trampa de foco, `Escape`, retorno y orden de tabulación), QA21-03 (pie del modal a 960×640 y 200 %), QA21-04 (abrir proyecto fuera de recientes/etiquetar estadísticas), QA21-05 (vuelta al panel), QA21-06 (feedback de `Guardar`), QA21-07 (palabras 0) y QA21-08 (controles muertos).
2. **Accesibilidad y copy:** UX21-15 (contraste), QA21-10 a QA21-12 (Ideas, plurales/formatos, nombres accesibles y landmarks) y QA21-13 (fallback de slug no latino).
3. **Robustez de creación:** cerrar SEC21-03 (validar antes de crear y rollback/limpieza si falla la operación completa), QA21-14 (rechazar proyectos dentro de otro y carpetas vacías), añadir las regresiones de SEC21-04/05/06 y cerrar SEC21-02 (temporal + rename/`openat` o mitigación equivalente) o aceptarlo formalmente con riesgo documentado antes de release.
4. **Validación real:** añadir E2E de escritorio para crear un proyecto con el diálogo nativo, `Guardar`, `Guardar y abrir`, abrir una tarjeta reciente y volver al panel; añadir E2E del modal a 960×640/200 % con trampa de foco.
5. **Documentación:** corregir `docs/inicio-v1.md` (dos pasos «4», «número de proyectos»), iteración 15, `README.md` (3 pruebas Tauri) y actualizar el estado en `HANDOFF.md`/plan.
6. **Gate del bloque:** QA, UX y seguridad si se tocan rutas; sin P1/P2; con las pruebas nuevas y los gates verdes. Después se decidirá el bloque de preparación de release, que tampoco está autorizado todavía.

**No se autoriza** añadir en este bloque importación, plantillas, subtítulo, portada con imagen, renombrar/borrar/mover proyectos ni ninguna función de las fases 3–5. Primero se cierra la deuda de la GUI y se valida el flujo real; después se decide el release.
