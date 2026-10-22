# Fase 2 — Aceptación del product owner

- **Commit de código:** `c39dcae53fbee53f6340ec4a8ad09a660d41a150`
- **HEAD de documentación:** `df8229b4eca25b11359868a31133b22d1fce6151`
- **Fecha:** 2026-10-21
- **Decisión:** **aprobado con observaciones**
- **Fase 2:** **aprobada y cerrada** como hito de producto en Linux x86_64.
- **Release público:** **no autorizado** todavía; ver §4.2.
- **Fase 3:** **autorizado su inicio** con las condiciones de §6.

Árbol limpio antes de escribir este informe (`git status --porcelain` vacío). No se editó código, tests, configuración ni dependencias; este informe es la única escritura y no se commitea.

## 1. Qué se entrega

La Fase 2 entrega una aplicación de escritorio Tauri 2.12 utilizable para escribir en Linux x86_64, sobre el motor y la CLI de la Fase 1:

- **Aplicación:** `writer` 0.2.0, React 19 + Vite 7 + Tiptap 3, con árbol de manuscrito por libro/capítulo y notas por tipo.
- **Edición:** editor con autosave a los 900 ms, `Ctrl+S`/`Cmd+S`, toolbar de formato, marcas de personaje/trama/comentario y recuento de palabras.
- **Estados:** guardado, cambios sin guardar, guardando, conflicto por hash, recuperación, proyecto ocupado, error de guardado y solo lectura. El cierre de ventana espera al guardado y solo destruye si termina bien.
- **Búsqueda e historial:** FTS5 con fragmento resaltado y salto al documento; línea de tiempo por revisión, detalle del contenido de cada archivo y reversión con confirmación, que crea una revisión nueva.
- **Cuarentena:** copias de archivos inválidos visibles, con motivo, ruta original, fecha y descarte en dos pasos sin tocar el original.
- **DOCX:** exportación básica desde CLI y UI, con portada, títulos, párrafos, énfasis, listas, citas, código, enlaces, separadores y notas opcionales.
- **Contratos y tipos:** tipos TypeScript generados desde Rust con `ts-rs`; contratos Markdown, DOCX, cuarentena, marcas, esquema y protocolo de commit.
- **Empaquetado:** `writer_0.2.0_amd64.deb` (5.171.448 B, SHA-256 `5c6502d9…f05e83`) y `writer-0.2.0-1.x86_64.rpm` (5.171.439 B, SHA-256 `15a907b7…5a29f9`), con estructura, permisos, lanzador e iconos verificados.
- **Pruebas:** 92 pruebas Rust (34 unitarias + 33 de integración + 25 de CLI), 57 de UI, 2 de la cáscara Tauri, 9 flujos E2E de navegador y 2 escenarios E2E de escritorio Linux (guardar y cierre).

**Verificación propia de esta aceptación.** Ejecuté, con `nice -n 19`, `cargo test --workspace --all-targets --locked` (92/92), `pnpm ui:test` (57/57), `pnpm typecheck`, `pnpm e2e` (9/9), `python3 scripts/ci.py interfaz` y `producto`; comprobé hashes de los `.deb`/`.rpm`, la estructura ZIP/XML de un DOCX exportado y una revisión visual headless de la demo (editor, búsqueda e historial). No ejecuté `e2e-tauri` porque abriría la app Tauri y la restricción de entorno lo impide; para el cierre de ventana y los diálogos nativos me apoyo en la QA r3, que lo verificó 5/5 en la app real.

**Estado del diff de documentación.** `git diff --stat c39dcae..HEAD` muestra 8 archivos, 571 inserciones y 6 borrados, todos Markdown: `HANDOFF.md`, `README.md` y seis archivos de `docs/revisiones/`. **No está limitado estrictamente a `docs/revisiones/`**: incluye también `HANDOFF.md` y `README.md`, como ya registró REL-07. No hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD; lo acepto como observación de trazabilidad, no como defecto de producto.

## 2. Criterios de la fase

| Criterio | Resultado | Evidencia y límites |
|---|---|---|
| **1. App usable para escribir: árbol, editor y autosave** | **Cumplido** | Árbol por libro/capítulo y notas; edición con autosave y `Ctrl+S`; conflicto real por hash; cierre con y sin cambios, conflicto y error de E/S verificados en la app real por QA r3. Revisión visual propia: el flujo de escritura es claro y la UI no estorba. Quedan deudas P3 de latencia con 1.000 marcas y de layout al 200 % de texto. |
| **5. Búsqueda e historial** | **Cumplido** | FTS5 en CLI y UI, búsqueda fresca tras edición externa sin `reindex`, IDs duplicados sin romper `buscar`/`reindex`/`revert`, línea de tiempo, detalle por archivo y reversión con confirmación. |
| **6. Estados, tipos generados, cuarentena, marcas e ida y vuelta** | **Cumplido con observaciones** | Estados visibles; tipos TS generados desde Rust; cuarentena con descarte seguro; marcas estables sin bucle de DOM; corpus Markdown de 18 parejas; HTML, imágenes, tablas, tareas y notas al pie se abren en solo lectura para no perder contenido. Persisten 15 P3 y 4 P4 de UX, ninguno P1/P2. |
| **14. DOCX** | **Cumplido con limitaciones documentadas** | Exportación OOXML válida: ZIP íntegro, XML bien formado, texto y formato básicos, marcas limpiadas en prosa. Sin tablas nativas, TOC ni imágenes incrustadas, como declara el contrato. **No se probó en Word real**; sí la estructura del paquete. |
| **E2E** | **Cumplido en local** | 9/9 flujos de navegador reproducidos por mí; 2 escenarios de escritorio Linux (guardar y cierre) reportados 5/5 por QA r3, no reejecutados por la restricción de no abrir la app. |
| **Empaquetado Linux** | **Cumplido en local** | `.deb` y `.rpm` generados y verificados; hashes coincidentes con el release r2. AppImage, firma, updater, Windows/macOS y app instalada en máquina limpia siguen fuera. |
| **Pipeline §9** | **Cumplido** | QA r3, seguridad r3 y release r2: aprobados con observaciones y 0 P1/P2. UX r5: aprobado y 0 P1/P2. Esta firma cierra la secuencia. |

Los cuatro criterios del gate se cumplen en el alcance declarado. La fase no se acepta como “sin observaciones”: se acepta como **hito de producto local Linux**, con las deudas P3/P4 y las carencias externas registradas en §4.

## 3. Valor de producto y alcance

**Lo que ya vale.** El bucle principal de escritura está completo y es coherente con la promesa del producto: archivos como fuente de verdad, motor que no pierde cambios, búsqueda útil, historial con recuperación, marcas para organizar personajes y tramas, exportación a DOCX y una app que se puede empaquetar. La interfaz revisada en headless es limpia, legible y adecuada para escribir: el árbol, el editor, la barra de formato, el estado de guardado y el historial se entienden sin explicación. La app no es una demo: usa el motor real, conserva el frontmatter, bloquea construcciones que perderían contenido y ofrece recuperación ante conflicto.

**Lo que todavía no es.** No es un release para usuario final:

- solo está construida y probada en Linux x86_64;
- no hay creación de proyecto ni proyectos recientes desde la GUI (UX-23), aunque el CLI `writer init` existe;
- el historial muestra contenido por archivo y reversión, pero no una vista de diferencias línea a línea;
- el DOCX es básico y no se validó en Word;
- hay deudas de accesibilidad y layout (tabs ARIA, foco, cuarentena no plegable, historial a 960×640, toolbar al 200 % de texto, búsqueda truncada);
- con 1.000 marcas en una escena la latencia de tecleo es ~2,2–2,4× la de un documento sin marcas, sin bucle ni CPU alta.

**¿Es suficiente para cerrar la Fase 2 y empezar IA/MCP?** Sí. El objetivo de la fase —una app usable para escribir con árbol, editor, autosave, búsqueda, historial, estados, cuarentena, marcas, DOCX, E2E y empaquetado— está cumplido en Linux. El motor y la CLI son una base estable para que la Fase 3 construya patch semántico, políticas y MCP sin rehacer la persistencia. No es suficiente para **publicar** el producto ni para prometer paridad Windows/macOS; eso queda como condición de release.

**P1/P2 abiertos.** Ninguno en los informes finales de QA r3, seguridad r3, release r2 y UX r5. Las observaciones P3/P4 y las carencias externas **no impiden la aceptación de la fase**; se aceptan como deuda registrada y como condiciones de publicación. La única salvedad de proceso es que QA r3 y seguridad r3 revisaron commits anteriores a `c39dcae`; release r2 y UX r5 sí revisaron el commit final, y verifiqué de forma acotada el delta (permiso de zoom de la ventana principal, recuento de palabras y cambios de UI/E2E). No hay nueva superficie de `fs`, `shell` o red.

## 4. Observaciones aceptadas y condiciones

### 4.1 Observaciones aceptadas (no bloquean la fase)

| Origen | Observación | Tratamiento |
|---|---|---|
| QA r3 | QA3-01: el E2E de cierre comprobaba la ventana, no el proceso. | **Corregido después de la r3**: el test actual comprueba `/proc/<pid>` y el estado del proceso. Condición: reejecutar `e2e-tauri` antes de publicar. |
| QA r3 / UX | QA3-02 / UX-16: latencia de ~203–225 ms con 1.000 marcas frente a 95 ms sin marcas. | Aceptado como P3 de rendimiento. No hay pérdida de datos ni bucle. Condición: perfilar y optimizar, o documentar el límite, antes de un release 1.0. |
| QA r3 | QA3-03: `HANDOFF.md` decía “16 parejas”. | **Corregido** en el árbol actual (dice 18). Queda el desglose erróneo de `README.md` (33+28+25 no suma 92; el real es 34+33+25), P4 documental. |
| Seguridad r3 | SEC-04-B y SEC-04-C: un FIFO en un blob referenciado o en `indice.sqlite-journal` puede colgar `verify`, `write`, `reindex` o `buscar`. | Aceptado como P3 de robustez para proyectos de terceros. **No bloquea la Fase 2**, pero debe cerrarse antes de que MCP opere sobre proyectos no confiables. |
| Seguridad r3 | Límites de tamaño, enlaces duros en Windows, TOCTOU residual, `parece_corrupto` por subcadenas, CSP/capacidades P4. | Deuda registrada. Los límites y el FIFO son condición de entrada de la Fase 3; Windows y TOCTOU, de publicación. |
| Release r2 | REL-01/02/03/09: binario sin strip, paquetes no reproducibles, toolchain flotante y versión `0.2.0` repetida con contenido distinto. | Deuda de release. Condiciones de publicación en §4.2. |
| UX r5 | 15 P3 y 4 P4: tabs ARIA, árbol sin plegado/virtualización, cuarentena no plegable, historial a 960×640, búsqueda truncada, foco, regiones vivas, toolbar al 200 %, onboarding, copy. | Aceptados. No impiden escribir ni exportar; se priorizan antes de publicación. |
| Proceso | QA r3 y seguridad r3 revisaron `6a0a55c` y `b336467`, no el commit final `c39dcae`; el delta final es UI, recuento de palabras y permiso de zoom. | Verificado de forma acotada: no añade `fs`, `shell`, red ni permisos generales; el zoom se limita a la ventana `main`. Condición: security review del commit exacto que se publique. |
| Proceso | `git diff c39dcae..HEAD` incluye `HANDOFF.md` y `README.md`, no solo `docs/revisiones/`. | Todo es documentación; no hay código ni configuración. Se registra para que el commit de release congele también estos documentos y declare el HEAD exacto. |
| Proceso | `docs/revisiones/README.md` tiene dos filas “Product owner / pendiente”. | P4 de bookkeeping; el builder debe dejar una sola fila y enlazar este informe al commitearlo. |

### 4.2 Condiciones para publicar el release (no para cerrar la Fase 2)

1. **Identidad de versión.** Subir a `0.2.1` (o añadir `Release`/build metadata) antes de publicar; no volver a distribuir un `0.2.0` con contenido distinto. Asociar cada SHA-256 al commit y al artefacto.
2. **Seguridad del commit publicado.** Revisión de seguridad del commit exacto de release, incluido el permiso `core:webview:allow-set-webview-zoom`, CSP/capacidades, los FIFO SEC-04-B/C y los límites de tamaño. Publicar `SHA256SUMS`; firmar `.deb`/`.rpm` o documentar formalmente la excepción. Decidir updater y notarización.
3. **Matriz de plataformas.** O se construyen y prueban Windows y macOS, o se declara que el release soportado es solo Linux. En Linux, o se genera AppImage o se documenta que los canales soportados son `.deb`/`.rpm` (la configuración `tauri.linux.conf.json` ya limita a esos dos). Probar la app **instalada** desde el paquete en una máquina limpia: lanzar, abrir proyecto, editar, buscar, historial, exportar DOCX y diálogos nativos.
4. **DOCX en Word real.** Abrir en Word o LibreOffice un manuscrito representativo y comprobar portada, títulos, énfasis, listas, citas, código, enlaces, separadores y notas. Si Word no lo abre o pierde formato, es un P2 de release, no una observación.
5. **CI remoto y reproducibilidad.** Publicar `.github/` con los gates `calidad`, `interfaz`, `e2e`, `e2e-tauri`, `empaquetado`, `producto` y `seguridad` en la matriz declarada; fijar toolchain, usar `--locked`, aplicar strip/remap de rutas y normalizar timestamps o documentar la no reproducibilidad de los paquetes.
6. **Documentación y bookkeeping.** Corregir el desglose de pruebas de `README.md` (34+33+25), enlazar este informe en `docs/revisiones/README.md`, eliminar la fila duplicada de product owner y actualizar el estado de la Fase 2 en el plan y el `HANDOFF.md`.
7. **Pulido de producto antes de llamarlo “release”.** Decidir el backlog P3 de UX; como mínimo, onboarding/creación de proyecto o declarar el flujo CLI-first, cuarentena plegable, historial usable a 960×640, toolbar al 200 % de texto y búsqueda con truncamiento visible. No es necesario cerrar los 15 P3 para publicar una beta, pero sí elegir cuáles se aceptan.
8. **Regla de bloqueo.** Cualquier P1/P2 que aparezca durante estas comprobaciones bloquea la publicación hasta corrección y regresión.

## 5. Decisión y firma

**Decisión: `aprobado con observaciones`.**

La Fase 2 cumple los criterios 1, 5, 6 y 14 en el alcance declarado, los gates locales están verdes y el pipeline §9 está completo con QA r3, seguridad r3, release r2 y UX r5, sin P1/P2 abiertos. La app es usable para escribir, el motor conserva las garantías de la Fase 1, la búsqueda y el historial funcionan, la cuarentena y las marcas son visibles, el DOCX es válido y el empaquetado Linux está verificado.

Las observaciones P3/P4 y las carencias externas no impiden cerrar la fase: son deuda acotada y condiciones de publicación. La Fase 2 queda **aprobada y cerrada como hito de producto en Linux**, no como release público.

**Firma:** Product owner del proyecto `writer` — 2026-10-21.  
**Fase 2:** aprobada con observaciones.  
**Release público de `0.2.0`:** no autorizado.

## 6. Autorización de la Fase 3

**Se autoriza iniciar la Fase 3 (IA y MCP)** sobre `c39dcae` o un descendiente que mantenga verdes los gates de la Fase 2, con estas condiciones:

1. **Prerrequisito de robustez.** Antes de que MCP opere sobre proyectos no confiables, cerrar SEC-04-B y SEC-04-C (FIFO en blobs y en `indice.sqlite-journal`), centralizar la lectura de archivos regulares con límite de tamaño y añadir regresiones. Un proyecto manipulado no debe poder colgar el servidor MCP.
2. **Rol de AI safety/privacy.** Constituir la revisión exigida por §9 para la Fase 3: modelo de amenazas de BYOK, endpoints, fuga de prompts y documentos, claves en logs, inyección de prompts, límites de coste, permisos de herramientas MCP y ausencia de telemetría por defecto.
3. **Política en el servidor.** Las políticas `auto`/`proponer`/`bloquear` deben aplicarse en el servidor MCP y en el motor, no solo en la UI. La IA no escribe archivos directamente: usa las operaciones del núcleo, `base_rev`, rebase, conflictos visibles, actividad y undo grupal.
4. **Alcance de proveedores.** Implementar la capa OpenAI-compatible + Ollama con streaming por eventos, sin ampliar la superficie de red de la app fuera de los proveedores configurados. Mantener el modo local-first y el funcionamiento sin red.
5. **Gate de Fase 3.** Respetar los criterios 7, 8, 9, 12 y 13, y repetir el pipeline completo (builder, QA, seguridad/AI safety, release, UX y product owner) sobre el commit congelado de la fase.
6. **Separación de release.** El trabajo de Fase 3 puede comenzar ya, pero no se publicará un instalador público mientras estén abiertas las condiciones de §4.2.

Con esta autorización, la Fase 2 queda cerrada y la Fase 3 puede comenzar sin reabrir el alcance de la interfaz. Las deudas P3/P4 de la Fase 2 se gestionan en el backlog de publicación, no como bloqueo de la siguiente fase.
