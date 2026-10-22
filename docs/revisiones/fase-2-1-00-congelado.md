# Fase 2.1 — Acta de congelación inicial

- **Fecha:** 2026-10-21
- **Commit congelado:** `376c0b8a07c4381f36c56f9a0620ea150580b352`
- **Alcance:** panel de inicio tipo Proyectos y asistente de creación de proyectos.
- **Referencias visuales:**
  `referencias-interfaz/Captura de pantalla_20261021_214741.png` y
  `..._214835.png` (adaptadas a writer; no se copian marcas ni datos de Dabble).

## Qué incluye

- Panel de Proyectos con buscador, pestañas Activo/Plantillas/Papelera, tarjeta
  “Crear nuevo proyecto”, estado vacío, sidebar de bienvenida/estadísticas y
  cuaderno “Ideas”.
- Asistente modal: nombre de carpeta, título, autor, idioma, carpeta padre,
  portada por color, notas iniciales y botones Guardar / Guardar y abrir.
- Comando Tauri `crear_proyecto` sobre `Project::init`, con notas opcionales.
- Recientes en `localStorage` con autor, palabras y color; apertura directa.
- En navegador, la creación explica que requiere la app de escritorio.

## Evidencia del builder

| Gate | Resultado |
|---|---|
| `cargo test --workspace --all-targets --locked` | 92/92 |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | 3/3 |
| `pnpm typecheck` | OK |
| `pnpm ui:test` | 58/58 |
| `pnpm e2e` | 10/10 Chromium |
| `python3 scripts/ci.py e2e-tauri` | OK (guardar y cerrar) |
| `python3 scripts/ci.py calidad` | OK |
| `python3 scripts/ci.py interfaz` | OK |
| `python3 scripts/ci.py producto` | OK |

## Límites declarados

- No hay importación de DOCX/Markdown, plantillas, renombrar/borrar/mover
  proyectos ni portada con imagen.
- El color de portada es una preferencia local de recientes, no un campo del
  manifiesto.
- El E2E de escritorio no prueba el diálogo nativo de carpeta; la creación real
  se cubrió con la prueba del comando Tauri y la UI con Vitest/Playwright.

## Pipeline 2.1

1. QA independiente.
2. Security reviewer (nueva superficie IPC y creación de archivos).
3. Release engineer (assets de UI y empaquetado).
4. UX/accesibilidad/rendimiento.
5. Product owner.
