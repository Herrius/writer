# Documentación

- [`../HANDOFF.md`](../HANDOFF.md) — estado actual y cómo retomar tras mover la
  carpeta.
- [`plan-ejecucion-local-v1.md`](plan-ejecucion-local-v1.md) — modelo de
  ejecución por features, agentes, paralelismo, gates y estimaciones.
- [`esquema-v1.md`](esquema-v1.md) — contrato del proyecto en disco (escenas, notas,
  `proyecto.yml`, log de operaciones, blobs, índice, cuarentena).
- [`esquema-v2.md`](esquema-v2.md) — extensiones locales aditivas: comentarios,
  metas, planificación, plantillas, importación y backup.
- [`comentarios-v2.md`](comentarios-v2.md) — anclas, hilos, huérfanos y sticky
  notes.
- [`metas-v1.md`](metas-v1.md) — metas, sesiones, rachas y temporizador.
- [`planificacion-v1.md`](planificacion-v1.md) — Plot Grid, personajes, Story
  Notes, @menciones e Ideas.
- [`plantillas-v1.md`](plantillas-v1.md) — plantillas locales y creación desde
  plantilla.
- [`importacion-v1.md`](importacion-v1.md) — importación DOCX/Markdown/HTML/RTF
  y backup.
- [`exportacion-v2.md`](exportacion-v2.md) — Markdown, DOCX, PDF y formato de
  manuscrito.
- [`protocolo-commit.md`](protocolo-commit.md) — orden de la transacción, CAS,
  recuperación y pruebas de caída.
- [`markdown-v1.md`](markdown-v1.md) — contrato del editor: esquema aceptado,
  corpus golden, normalizaciones y construcciones bloqueadas.
- [`docx-v1.md`](docx-v1.md) — contrato de exportación DOCX: estructura,
  construcciones soportadas y limitaciones.
- [`cuarentena.md`](cuarentena.md) — copia de diagnóstico, comandos y reglas.
- [`marcas-v1.md`](marcas-v1.md) — sintaxis y modelo de personajes, tramas y
  comentarios.
- [`e2e.md`](e2e.md) — Playwright sobre la demo y límites frente a Tauri.
- [`empaquetado.md`](empaquetado.md) — configuración de bundle e instaladores.
- [`ci.md`](ci.md) — gates de calidad, interfaz, producto y seguridad; paridad
  local/CI.
- [`adr/0001-editor-tiptap.md`](adr/0001-editor-tiptap.md) — decisión del editor
  Tiptap frente a Milkdown.
- [`iteraciones/01-fase-0.md`](iteraciones/01-fase-0.md) — registro de la Fase 0 y
  su corrección 01.1.
- [`iteraciones/02-fase-1.md`](iteraciones/02-fase-1.md) — motor y CLI v0.1,
  correcciones 02.1 y 02.2.
- [`iteraciones/03-fase-2.md`](iteraciones/03-fase-2.md) — primera rebanada de la
  interfaz Tauri.
- [`iteraciones/04-fase-2.md`](iteraciones/04-fase-2.md) — corpus Markdown,
  autosave y conflictos por hash.
- [`iteraciones/05-fase-2.md`](iteraciones/05-fase-2.md) — búsqueda FTS5,
  historial y reversión.
- [`iteraciones/06-fase-2.md`](iteraciones/06-fase-2.md) — exportación DOCX
  desde CLI y UI.
- [`iteraciones/07-fase-2.md`](iteraciones/07-fase-2.md) — cuarentena visible y
  marcas propias.
- [`iteraciones/08-fase-2.md`](iteraciones/08-fase-2.md) — E2E de navegador y
  empaquetado deb/rpm.
- [`iteraciones/09-fase-2.md`](iteraciones/09-fase-2.md) — toolbar, recuento,
  E2E de escritorio y cierre de la Fase 2.
- [`iteraciones/10-fase-2.md`](iteraciones/10-fase-2.md) — correcciones de los
  hallazgos de la QA independiente.
- [`iteraciones/11-fase-2.md`](iteraciones/11-fase-2.md) — correcciones de la
  QA r2: permiso de cierre y títulos de marcas con decoraciones.
- [`iteraciones/12-fase-2.md`](iteraciones/12-fase-2.md) — correcciones de
  seguridad: confinamiento de rutas y `.escritor` real.
- [`iteraciones/13-fase-2.md`](iteraciones/13-fase-2.md) — correcciones de UX,
  accesibilidad y rendimiento.
- [`iteraciones/14-fase-2.md`](iteraciones/14-fase-2.md) — aceptación y cierre
  de la Fase 2; condiciones de publicación y de Fase 3.
- [`iteraciones/15-fase-2-1.md`](iteraciones/15-fase-2-1.md) — creación de
  proyectos desde la GUI y recientes.
- [`iteraciones/16-bloque-2-2.md`](iteraciones/16-bloque-2-2.md) — interfaz de
  escritura e historial adaptada de las referencias de Dabble.
- [`iteraciones/17-fase-1-rediseno-panel.md`](iteraciones/17-fase-1-rediseno-panel.md)
  — Fase 1: contrato visual, retirada de bloques y ajuste del panel de Proyectos.
- [`iteraciones/18-fase-2-0.md`](iteraciones/18-fase-2-0.md) — Fase 2.0:
  contratos de almacenamiento y modularización para el trabajo en paralelo.
- [`iteraciones/19-fase-2-4-metas.md`](iteraciones/19-fase-2-4-metas.md) —
  Fase 2.4: núcleo local de metas y sesiones.
- [`iteraciones/20-fase-2-1-manuscrito.md`](iteraciones/20-fase-2-1-manuscrito.md)
  — Fase 2.1: renombrar, duplicar, mover, archivo, metadatos y copia.
- [`iteraciones/21-fase-2-2-comentarios.md`](iteraciones/21-fase-2-2-comentarios.md)
  — Fase 2.2: comentarios anclados, sticky notes, respuestas y resolución.
- [`iteraciones/22-fase-2-3-planificacion.md`](iteraciones/22-fase-2-3-planificacion.md)
  — Fase 2.3: Plot Grid, notas nuevas, @menciones y personajes.
- [`iteraciones/23-fase-2-5-importacion.md`](iteraciones/23-fase-2-5-importacion.md)
  — Fase 2.5: importación Markdown/DOCX y backup.
- [`iteraciones/24-fase-2-6-plantillas.md`](iteraciones/24-fase-2-6-plantillas.md)
  — Fase 2.6: plantillas locales.
- [`iteraciones/25-fase-2-7-cierre.md`](iteraciones/25-fase-2-7-cierre.md) —
  Fase 2.7: integración, gates y cierre funcional de la Fase 2.
- [`inicio-v1.md`](inicio-v1.md) — contrato del panel de Proyectos y del
  asistente de creación.
- [`escritorio-v1.md`](escritorio-v1.md) — contrato de la interfaz de escritura
  e historial del Bloque 2.2.
- [`revisiones/README.md`](revisiones/README.md) — pipeline secuencial de
  revisión por roles y registro de la Fase 2. Incluye el informe QA que bloqueó
  la primera congelación y el acta r2 con las correcciones.

Pendiente por fase:

- Fase 2: pipeline de revisión por roles y validación externa (AppImage, firma,
  Windows/macOS y CI remoto).
- Fase 3: `protocolo-patches.md` y contrato del servidor MCP.
