# Documentación

- [`../HANDOFF.md`](../HANDOFF.md) — estado actual y cómo retomar tras mover la
  carpeta.
- [`esquema-v1.md`](esquema-v1.md) — contrato del proyecto en disco (escenas, notas,
  `proyecto.yml`, log de operaciones, blobs, índice, cuarentena).
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
- [`revisiones/README.md`](revisiones/README.md) — pipeline secuencial de
  revisión por roles y registro de la Fase 2. Incluye el informe QA que bloqueó
  la primera congelación y el acta r2 con las correcciones.

Pendiente por fase:

- Fase 2: pipeline de revisión por roles y validación externa (AppImage, firma,
  Windows/macOS y CI remoto).
- Fase 3: `protocolo-patches.md` y contrato del servidor MCP.
