# Plan de ejecución — etapa local/offline (v1)

- **Fecha:** 2026-10-22
- **Estado:** activo. La codificación de la Fase 1 espera autorización expresa.
- **Alcance:** ejecución de la etapa local/offline del plan de producto.
- **Plan de producto:** `../../Plan - Rediseño del panel de Proyectos.md`
  (fuera del repositorio; define alcance y fases).
- **Estado del repositorio:** [`../HANDOFF.md`](../HANDOFF.md).

## 0. Relación con el plan de producto

Este documento no sustituye al plan de producto. Lo operacionaliza:

- El plan de producto define **qué** se construye y **qué queda fuera**.
- Este documento define **cómo se trabaja**: features, commits, agentes,
  paralelismo, contratos, gates y estimaciones.
- Si hay conflicto de alcance, manda el plan de producto. Si hay conflicto de
  proceso, manda este documento.
- La **Fase 2.0** de preparación es una adición de este plan de ejecución, no
  una función nueva del producto. Debe aprobarse antes de arrancar la Fase 2.

La etapa sigue siendo **local-first y offline**: sin cuentas, sin servidor, sin
sincronización y sin servicios externos. Windows y Linux son el objetivo; el
estado validado actual es Linux x86_64.

## 1. Reglas de ejecución

### 1.1 Un feature = un commit documentado

Cada feature termina con este checklist, sin excepciones:

1. Código y tests del feature funcionando.
2. Documentación en el mismo commit:
   - sección en `docs/iteraciones/17-fase-1-rediseno-panel.md` o en la
     iteración que corresponda;
   - actualización del contrato afectado (`docs/inicio-v1.md`,
     `docs/esquema-v2.md`, etc.);
   - medidas antes/después, comandos ejecutados y resultado;
   - límites o deuda que deja el feature.
3. Gate mínimo: `pnpm typecheck`, `pnpm ui:test`, `pnpm ui:build`. Si toca
   geometría o responsive, además el E2E correspondiente.
4. Commit atómico, en español, sin `WIP` ni mezclas de features.
5. `git status` limpio y hash registrado en la tabla de congelación.
6. No se empieza el siguiente feature hasta cumplir 1–5.

`HANDOFF.md` se actualiza al congelar y al cerrar la fase, no en cada feature.

### 1.2 Convención de commits

Se mantiene el estilo existente:

```text
docs: definir contrato visual del panel
feat: retirar meta diaria, proyectos personales, Ideas y Ayuda
style: ajustar tarjeta de creación y portadas
style: ampliar la barra lateral a toda la altura
test: cubrir geometría y responsive del panel
docs: congelar la fase 1 del panel
```

Los informes de revisión se commitean aparte como documentación. El
orquestador es el único que toca el índice de Git; ningún agente commitea por su
cuenta.

### 1.3 Congelación y revisión

- Se congela un **hash de código** concreto (`git rev-parse HEAD` o tag
  `fase-X-congelado`).
- Los revisores reciben el hash, los contratos y los criterios; no reciben la
  justificación del builder.
- Los revisores no editan código ni tests. Sus pruebas viven en `/tmp`; si
  proponen una prueba permanente, la implementa el builder.
- Secuencia formal: QA → seguridad → release → UX/accesibilidad/rendimiento →
  product owner.
- Un P1/P2 bloquea. Se corrige en un commit nuevo, se recongela y se repiten el
  rol afectado y los posteriores.
- Los P3/P4 se registran; no bloquean.

## 2. Agentes y propiedad de archivos

| Rol | Escribe | No toca | Entregable |
|---|---|---|---|
| Orquestador | `.git`, `HANDOFF.md` al congelar, actas | Código de features | Plan, contexto, commits, freeze, integración |
| Builder de núcleo | `crates/core/`, `apps/desktop/src-tauri/` | `ui/`, `docs/`, `e2e/` | Motor, comandos, DTOs, tests de núcleo |
| Builder de UI | `ui/src/features/`, `ui/src/editor/` | `crates/`, `docs/`, `e2e/` | Componentes, estados, accesibilidad |
| Docs | `docs/` | Código, tests | Contratos, iteraciones, guías |
| Test author | `e2e/`, tests de UI nuevos | Código de producción | E2E, corpus, sondas permanentes |
| Medición | `/tmp/` | Repo | Medidas, capturas, inventario |
| Revisores | Su informe en `docs/revisiones/` | Código, tests, docs de producto | Informe con severidad y decisión |
| Product owner | Acta de aceptación | Código | Firma |

### 2.1 Regla de paralelismo

**Paralelo por propiedad de archivo; serial por aprobación.**

- Cada archivo tiene un solo escritor a la vez.
- Máximo **2 builders de código** simultáneos, siempre con archivos disjuntos.
- El builder de núcleo puede ir un bloque por delante del builder de UI si los
  DTOs y fixtures están congelados.
- La UI puede trabajar contra DTOs y datos demo; no espera al motor.
- El motor no espera a la UI para sus tests de integración.
- Los gates pesados se ejecutan de uno en uno: `pnpm`, `cargo` y
  `scripts/ci.py` compiten por CPU y locks.
- Los worktrees se usan solo si dos builders necesitan `target/` o
  `node_modules/` separados. En Fase 1 no hacen falta.
- MEGA debe pausarse durante la fase paralela o el repositorio debe moverse
  fuera de la carpeta sincronizada.

### 2.2 Qué no se paraleliza

- Migraciones de esquema.
- Dos escritores sobre `App.tsx`, `styles.css`, `project.rs` o cualquier
  archivo compartido.
- Las firmas formales de QA, seguridad, release, UX y product owner.
- El ajuste de rendimiento antes de que las features estén estabilizadas.
- La decisión de release de Windows y Linux sobre artefactos distintos.

### 2.3 Puntos de sincronización

1. Fin de cada feature: código, tests y documentación listos; gate mínimo en
   verde; commit único.
2. Fin de cada bloque: freeze, informe de congelación y revisión.
3. Fin de cada fase: integración, gates completos, pipeline formal y firma del
   product owner.
4. Cada semana: checkpoint de integración entre carriles.

## 3. Fase 1 — Rediseño del panel

### 3.1 Contrato de la tarjeta

Referencia base a 1440 px:

| Elemento | Medida | Regla |
|---|---:|---|
| Portada | 160 × 240 px (2:3) | Ancho `w`, alto `1,5w`; nunca se estira |
| Zona de datos | ~60 px | Título, autor y palabras |
| Tarjeta de proyecto | 160 × 300 px | Portada + datos |
| Tarjeta «Crear nuevo proyecto» | 160 × 300 px | Iguala la tarjeta completa, no solo la portada |

Al reducir el ancho se mantiene la portada 2:3 y se recalcula la altura. La
tarjeta de creación iguala la altura de la tarjeta completa del proyecto. No se
estira la portada para rellenar.

Prueba E2E de F1.2:

- ratio de portada ≈ 0,6667;
- altura de la tarjeta de creación ≈ altura de la tarjeta de proyecto ±2 px;
- zona de datos entre 54 y 66 px a 1440.

### 3.2 Decisiones de producto de la fase

- **Meta diaria, «Proyectos personales» e «Ideas»:** se eliminan del panel.
  La meta y el cuaderno pueden seguir existiendo dentro del proyecto.
- **«Ayuda»:** se retira solo el botón «Ayuda» del panel de Proyectos.
- **Resto de controles deshabilitados:** Notificaciones, Filtros, Colección,
  Plantillas, Papelera, Más opciones y Crear desde plantilla se conservan.
  Quedan documentados como **excepción aprobada por el product owner** al
  criterio «no hay controles visibles sin acción». Se revisarán en fases
  posteriores.
- **Barra derecha:** ocupa toda la altura del panel. El espacio liberado queda
  como área vacía con estructura visual discreta, sin texto ni controles.
- **Cuadrícula, vista de lista, búsqueda y apertura de proyectos:** se
  conservan.

### 3.3 Features

| # | Feature | Código | Documentación en el mismo commit | Commit |
|---|---|---|---|---|
| F1.0 | Contrato visual y línea base | — | `iteraciones/17` §F1.0 + medidas y capturas 1440/960/200 % | `docs: definir contrato visual del panel` |
| F1.1 | Retirar meta diaria, Proyectos personales, Ideas y Ayuda | `Inicio.tsx`, `styles.css`, tests | §F1.1 + `inicio-v1.md` §1 + excepción §6 | `feat: retirar meta diaria, proyectos personales, Ideas y Ayuda` |
| F1.2 | Tarjeta de creación y portadas | `styles.css`, `Inicio.tsx`, E2E | §F1.2 + medidas antes/después + contrato 160×300 y 2:3 | `style: ajustar tarjeta de creación y portadas` |
| F1.3 | Barra derecha a toda altura | `styles.css`, `Inicio.tsx`, E2E | §F1.3 + `inicio-v1.md` §1 + geometría 1440/960 | `style: ampliar la barra lateral a toda la altura` |
| F1.4 | QA visual y responsive | Fixes si aparecen | §F1.4 + capturas 1440/960/200 % + foco/teclado | `test: cubrir geometría y responsive del panel` |
| F1.5 | Gates y congelación | — | `HANDOFF.md`, `fase-1-00-congelado.md`, tabla de hashes | `docs: congelar la fase 1 del panel` |

Cada feature cumple el checklist de §1.1. Al cerrar F1.5 se crea el tag
`fase-1-congelado` sobre el último commit de código y se registra el hash exacto
en el acta de congelación.

### 3.4 Trabajo en paralelo de Fase 1

```text
OLEADA 0 — análisis (paralelo, read-only)
  A. Medición base 1440/960/200 %        ─┐
  B. Inventario de tests y selectores     ├─> F1.0: contrato + commit
  C. Auditoría de docs y criterios       ─┘

OLEADA 1 — features (paralelo por feature)
  Builder UI       ──────────────┐
  Docs redacta §F1.x             ├─> sync -> gate -> commit F1.x
  Test author prepara spec       ─┘

OLEADA 2 — congelación
  Pre-flight en paralelo (máx. 2):
    QA gates | seguridad | release | UX capturas
  Formal, secuencial:
    QA -> seguridad -> release -> UX -> product owner

OLEADA 3 — correcciones
  Builder fix + Docs nota + Test author regresión
  -> nuevo commit -> nuevo freeze -> repetir rol afectado y posteriores
```

### 3.5 Estimación

| Tramo | Horas de agente |
|---|---:|
| F1.0 | 0,5–1 |
| F1.1 | 1,5–2 |
| F1.2 | 2–3 |
| F1.3 | 2–2,5 |
| F1.4 | 1–1,5 |
| F1.5 | 0,5–1 |
| QA | 0,75–1,5 |
| Seguridad | 0,5–0,75 |
| Release | 0,5–0,75 |
| UX/accesibilidad/rendimiento | 1–2 |
| Product owner | 0,5 |
| Correcciones | 2–4 |
| **Total** | **13–20 h** |

Calendario: 2–3 días; 4 días si UX repite ciclo.

## 4. Fase 2 — Núcleo local prioritario

### 4.1 Fase 2.0 — Preparación

**Duración: 6–10 días. Serial, un solo builder.**

| Trabajo | Salida |
|---|---|
| Esquema v2 y migración | `docs/esquema-v2.md`, fixtures v1, prueba de migración |
| Comentarios v2 | `docs/comentarios-v2.md`: ancla, hilo, autor, estado, huérfanos, sticky notes |
| Metas y temporizador | `docs/metas-v1.md`: almacenamiento, rachas, sesiones, presupuesto |
| Planificación | `docs/planificacion-v1.md`: Plot Grid, personajes, Story Notes, @menciones, Ideas |
| Plantillas | `docs/plantillas-v1.md`: formato local, creación desde plantilla |
| Importación/exportación | `docs/importacion-v1.md`, `docs/exportacion-v2.md` |
| Modularizar núcleo | `manuscript.rs`, `comments.rs`, `goals.rs`, `planning.rs`, `templates.rs`, `import.rs`; `project.rs` como fachada |
| Modularizar UI | `ui/src/features/{manuscript,comments,planning,goals,importexport,templates}/`; `App.tsx` como shell |
| DTOs y fixtures | Tipos ts-rs congelados y datos demo para la UI |
| Fixtures de riesgo | DOCX/HTML/RTF malformados, zip bomb, rutas `..`, 1.000 escenas, 1.000 marcas |

Sin esta fase, dos builders chocan en `project.rs`, `App.tsx` y `styles.css`.
La modularización es un refactor sin cambio de comportamiento y tiene su propio
freeze y revisión.

### 4.2 Bloques funcionales

| Bloque | Contenido | Depende de | Días |
|---|---|---|---:|
| 2.1 Manuscrito | Libros, partes, capítulos y escenas; renombrar, duplicar, mover, archivar, papelera; drag & drop; estados, etiquetas, filtros; copiar/pegar documentos; navegación y filtro | 2.0 | 8–12 |
| 2.2 Comentarios | Comentarios anclados, hilos, respuestas, edición, borrado, resolución; historial; huérfanos; sticky notes; copiar/pegar con o sin comentarios; solo lectura en previsualización | 2.0, 2.1 | 8–12 |
| 2.3 Planificación | Plot Grid con líneas, puntos, colores y etiquetas; escenas vinculadas; personajes principales y secundarios; casts y fichas; Story Notes, cuaderno y mundo; @menciones; Ideas | 2.0, 2.1 | 10–15 |
| 2.4 Metas | Meta diaria, de libro y desafío con fecha; gráfico, estadísticas, rachas, metas archivadas; temporizador 5/10/15/25/30; iniciar, pausar, finalizar, limpiar; laps, palabras por sesión, chime, historial | 2.0 | 5–7 |
| 2.5 Importación/exportación | Importar DOCX, Markdown, HTML, RTF y Word; todo o parte; exportar DOCX/Markdown; imprimir/PDF; backup e importación de base de datos; formato de manuscrito | 2.0, 2.1 | 7–10 |
| 2.6 Plantillas e Ideas | Plantillas locales, creación desde plantilla, cuaderno de Ideas local y conversión a proyecto | 2.0, 2.1 | 4–6 |
| 2.7 Integración | Rendimiento, accesibilidad, E2E, documentación y gate de fase | 2.1–2.6 | 6–10 |

**Total: 54–82 días de builder.** Calendario: 12–18 semanas con un carril;
8–12 semanas con dos.

### 4.3 Carriles

| Carril | Dueño | Trabajo |
|---|---|---|
| A — Núcleo | Builder A | 2.1 core, 2.2 persistencia, 2.4 almacenamiento, 2.5 import/export, 2.6 plantillas |
| B — UI/editor | Builder B | 2.1 árbol, 2.2 UI de comentarios, 2.3 planificación, 2.4 metas, 2.5 diálogos, 2.6 plantillas |
| C — Docs/tests | Docs + test author | Contratos, iteraciones, E2E, accesibilidad, fixtures |
| D — Integración | Orquestador | Commits, freeze, merges, revisiones, HANDOFF |

El builder A va un bloque por delante del builder B. Ejemplo:

```text
Semanas 1–2   Fase 2.0 serial
Semana 3      A: 2.1 core        B: 2.1 UI       C: tests 2.1
Semana 4      A: 2.2 core        B: 2.2 UI       C: tests 2.2
Semana 5      A: 2.3 refs core   B: 2.3 UI       C: tests 2.3
Semana 6      A: 2.4 storage     B: 2.4 UI       C: tests 2.4
Semana 7      A: 2.5 core        B: 2.5 UI       C: tests 2.5
Semana 8      A: 2.6 core        B: 2.6 UI       C: tests 2.6
Semanas 9–10  Integración, rendimiento, gate de Fase 2
```

No es un calendario rígido: es el orden de carriles. Cada semana hay un
checkpoint de integración y un commit por feature.

### 4.4 Revisiones y salida

- **2.1, 2.2, 2.5:** pipeline completo; seguridad obligatoria en comentarios e
  importación.
- **2.3, 2.4, 2.6:** QA + UX por bloque; seguridad y release en el gate de
  fase.
- **Gate de Fase 2:** pipeline completo sobre el integrado: QA → seguridad →
  release → UX → product owner.

Criterios de salida:

- migración v1→v2 probada con fixtures y proyectos reales;
- comentarios con hilo, autor, estado y huérfanos;
- Plot Grid, personajes y Story Notes enlazados a escenas;
- metas, rachas y temporizador persistentes y sin red;
- importación malformada a cuarentena, sin path traversal, zip bomb ni XXE;
- backup/restore round-trip;
- rendimiento con 1.000 escenas y 1.000 marcas dentro del presupuesto;
- cero controles online y cero dependencias de red.

## 5. Fase 3 — Local avanzado

| Bloque | Contenido | Depende de | Días |
|---|---|---|---:|
| 3.1 Control de cambios | Inserciones/borrados locales, aceptar/rechazar, autor y fecha | 2.2 | 6–9 |
| 3.2 Comparación de versiones | Diff visual, lado a lado, restauración, por documento y proyecto | Historial + 3.1 | 6–9 |
| 3.3 Formato y preferencias | Formato de manuscrito, fuentes, interlineado, párrafo, temas, tipografía, atajos, Quick Open, paleta de comandos, accesibilidad | 2.0 | 5–8 |
| 3.4 Read to Me y escritura | Voz del sistema, corrector, diccionario, tesauro, gramática local si el motor es offline, recuentos y modo enfoque | 2.0 | 6–10 |
| 3.5 Rendimiento | Proyectos grandes, búsqueda, historial, arranque, marcas | 3.1–3.4 | 4–6 |
| 3.6 Integración | E2E, accesibilidad, documentación, gate | 3.1–3.5 | 4–6 |

**Total: 30–45 días.** Calendario: 6–10 semanas con un carril; 4–7 con dos.

Paralelismo:

- Builder A: control de cambios y diff.
- Builder B: formato/preferencias y Read to Me.
- Docs/tests: contratos, corpus de diff, voces y accesibilidad.
- Rendimiento se mide al final, con las features estabilizadas.

Contratos: `cambios-v1.md`, `diff-v1.md`, `formato-v1.md`, `tts-v1.md`,
`rendimiento-v1.md`.

## 6. Fase 4 — Windows

| Bloque | Contenido | Días |
|---|---|---:|
| 4.1 Toolchain y CI | MSVC, WebView2, Rust 1.90, scripts de build, runner Windows | 3–4 |
| 4.2 Integración nativa | Diálogos, rutas, cierre de ventana, asociaciones, rutas largas, nombres reservados | 3–5 |
| 4.3 Instalador | MSI/NSIS, firma, instalación/desinstalación, máquina limpia | 2–4 |
| 4.4 E2E y validación | tauri-driver, DOCX en Word/LibreOffice, app instalada | 2–3 |

**Total: 10–15 días.** Calendario: 2–4 semanas, condicionado por hardware y CI.

Paralelismo:

- 4.1 y 4.2 pueden solaparse con documentación.
- 4.3 espera al build.
- 4.4 espera al instalador.
- El cuello de botella es tener una máquina Windows o un runner CI disponible.

Salida: build, instalación limpia, abrir/guardar/exportar, diálogos nativos,
cierre, desinstalación y DOCX validado.

## 7. Fase 5 — Documentación y cierre

| Bloque | Contenido | Días |
|---|---|---:|
| 5.1 Documentación | README, HANDOFF, contratos, límites offline, capturas | 2–3 |
| 5.2 QA/UX final | Gates, accesibilidad, rendimiento, capturas 1440/960/200 % | 2–3 |
| 5.3 Cierre | Aceptación, tag, backlog, archivo del plan | 1–2 |

**Total: 4–6 días.** Calendario: 1–2 semanas.

La documentación puede redactarse durante Fase 3/4; el cierre formal va después
de Windows.

## 8. Resumen de calendario

| Fase | Días de builder | 1 carril | 2 carriles | Confianza |
|---|---:|---:|---:|---|
| Fase 1 — Panel | 7–9 | 2–3 días | — | Alta |
| Fase 2 — Núcleo local | 54–82 | 12–18 semanas | 8–12 semanas | Media-baja |
| Fase 3 — Local avanzado | 30–45 | 6–10 semanas | 4–7 semanas | Media |
| Fase 4 — Windows | 10–15 | 2–4 semanas | 2–3 semanas | Baja sin hardware/CI |
| Fase 5 — Cierre | 4–6 | 1–2 semanas | 1 semana | Alta |
| **Total** | **105–157** | **23–37 semanas** | **17–26 semanas** | |

No es un compromiso de fecha: es un orden de magnitud con el alcance actual.
La Fase 2 es la que más puede moverse porque incluye esquema, comentarios e
importación.

## 9. Solapamientos controlados

- **Fase 1 en revisión + Fase 2.0 en papel:** mientras QA/UX revisan el panel,
  Docs puede redactar `esquema-v2.md` y los contratos de comentarios. No se
  toca código.
- **Fase 2.0 terminando + Fase 2.1 contratos:** los contratos de manuscrito
  pueden empezar antes de que acabe la modularización.
- **Fase 2 en gate + Fase 3 contratos:** se redactan `cambios-v1.md` y
  `diff-v1.md` sin codificar.
- **Fase 3 + preparación Windows:** se instala MSVC/WebView2 y se configura el
  runner mientras se termina Fase 3.
- **Fase 5 docs durante Fase 3/4:** README y HANDOFF se actualizan en vivo; el
  cierre espera a Windows.

## 10. Riesgos

- **Esquema v2:** la migración puede romper proyectos v1. Mitigación:
  compatibilidad de lectura, fixtures v1, migración explícita y pruebas.
- **Comentarios:** hilos, autoría y huérfanos no caben en la sintaxis actual de
  marcas. Mitigación: contrato 2.0 y decisión de almacenamiento antes de
  codificar.
- **Importación:** DOCX/HTML/RTF añaden parsers y superficie de seguridad.
  Mitigación: cuarentena, límites de tamaño, pruebas de zip bomb y path
  traversal.
- **Monolitos:** `project.rs`, `App.tsx` y `styles.css` impiden paralelizar.
  Mitigación: Fase 2.0 de modularización.
- **Windows:** sin máquina o runner, la fase queda bloqueada. Mitigación:
  preparar toolchain y CI durante Fase 3.
- **MEGA:** la sincronización concurrente genera ruido. Mitigación: pausar MEGA
  o mover el repo fuera de la carpeta sincronizada.
- **Rendimiento:** 1.000 escenas y 1.000 marcas. Mitigación: fixtures y
  presupuestos desde 2.0.
- **Alcance:** la Fase 2 es amplia. Mitigación: bloques con freeze y revisión,
  no un único cierre monolítico.
- **Read to Me:** la calidad y disponibilidad de voces depende del sistema.
  Mitigación: interfaz de voz y pruebas manuales por plataforma.

## 11. Decisiones abiertas para Fase 2.0

Las decisiones iniciales ya están registradas en
[`iteraciones/18-fase-2-0.md`](iteraciones/18-fase-2-0.md). Este apartado queda
como checklist hasta que los contratos de 2.0.2 estén aprobados.

1. **«Partes» del manuscrito:** ¿cuarto nivel `libro → parte → capítulo →
   escena`, o agrupación de libros? Cambia el esquema v2 y la migración.
2. **Almacenamiento de comentarios:** ¿ancla con ID en Markdown y metadatos de
   hilo en `.escritor/comentarios/`, o todo dentro del Markdown? Los hilos,
   autor y estado no caben en la sintaxis actual.
3. **Metas y estadísticas:** ¿por proyecto en `.escritor/` o globales de la
   app? Afecta backup y exportación.
4. **Prioridad de importación:** DOCX y Markdown primero; HTML y RTF después.
5. **Modularización:** aprobar los 6–10 días de 2.0 antes de arrancar Fase 2.
6. **Windows:** confirmar máquina o runner CI para Fase 4.

## 12. Próximo paso

1. Autorizar el arranque de Fase 1.
2. Ejecutar F1.0–F1.5 con el checklist de §1.1.
3. Congelar y lanzar el pipeline de revisión.
4. En paralelo con las revisiones, redactar los contratos de Fase 2.0 sin tocar
   código.
5. Abrir Fase 2.0 solo con la firma del product owner.

No se toca código de producto hasta la autorización expresa.
