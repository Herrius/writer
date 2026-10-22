# Bloque 2.2 — Acta de congelado

- **Commit congelado:** `76b139f` (`style: conservar el panel lateral a 960 px`)
- **Base:** `cd9683e` (Fase 2.1 cerrada)
- **Fecha:** 2026-10-22
- **Alcance:** sustituir la interfaz de escritura e historial por la adaptación de
  las referencias de Dabble y cerrar la deuda P3 prioritaria del Bloque 2.2.
- **Release público:** no autorizado; ver el informe de release.

## 1. Qué entra en el congelado

### Interfaz de escritura e historial

- Cabecera de proyecto con vuelta a Proyectos, pestañas, estado de guardado,
  exportación, Markdown, historial y avatar.
- Árbol lateral con Manuscritos por libro/capítulo, Tramas, Personajes, Cuaderno
  y Utilidades.
- Página blanca central con título, metadatos, toolbar, editor y recuento.
- Panel derecho Metas/Notas con gráfico y temporizador visuales y metadatos.
- Historial con versiones a la derecha, contenido de la revisión en el centro y
  línea de tiempo sticky con Recuperar / Salir de la máquina del tiempo.
- Cuarentena plegable para no tapar el editor.

### Deuda del Bloque 2.2

- Lista de proyectos real (miniatura + fila compacta), no una cuadrícula de una
  columna con portadas gigantes.
- Modal: pie siempre visible a 960×640 y 200 %, trampa de foco, Escape global,
  retorno de foco, orden de tabulación reducido en la paleta y nombres
  accesibles.
- “Abrir proyecto” siempre visible; estadísticas etiquetadas como recientes.
- Vuelta al panel desde el editor guardando antes de salir.
- `Guardar` limpia el filtro, limpia el error anterior y muestra confirmación.
- Recuento de palabras a 0 correcto (`??`, no `||`).
- Controles sin acción deshabilitados o con feedback; Ideas ya no abre el
  asistente de proyecto.
- Creación: manifiesto validado antes de crear, rechazo de proyectos anidados,
  rollback de la carpeta nueva y del comando si falla una nota.
- Documentación de inicio, iteración 15, README, E2E y este bloque.

## 2. Evidencia de gates

Ejecutados con `nice -n 19`, sin ventanas interactivas salvo el E2E de Tauri
(que arranca la app real y se ejecuta en segundo plano):

| Gate | Resultado |
|---|---|
| `python3 scripts/ci.py calidad` | OK — **94** pruebas Rust (34 unitarias + 35 integración + 25 CLI) |
| `python3 scripts/ci.py interfaz` | OK — tipos, build, cáscara Tauri y **3/3** pruebas Tauri |
| `python3 scripts/ci.py producto` | OK — fixtures, CLI y enlaces |
| `python3 scripts/ci.py e2e` | OK — **12/12** Chromium |
| `python3 scripts/ci.py e2e-tauri` | OK — 2 escenarios Linux (guardado y cierre) |
| `python3 scripts/ci.py seguridad` | OK — cargo-deny, gitleaks, sin fugas |
| `python3 scripts/ci.py empaquetado` | OK — `.deb` y `.rpm` verificados |

## 3. Artefactos

Los paquetes se regeneran después de congelar el código. Los tamaños y SHA-256
exactos quedan en `fase-2-2-release-2026-10-22.md`.

## 4. Capturas propias

En `/tmp/opencode/` (fuera del árbol):

- `ux22-editor-1440.png`
- `ux22-history-1440.png`
- `ux22-editor-960-normal-v2.png`
- `ux22-editor-960-200-v2.png`
- `ux22-modal-960-fixed.png`
- `ux22-modal-200-fixed.png`
- `ux22-list-1440.png`

## 5. Límites declarados

- El E2E de escritorio de creación con el diálogo nativo GTK no se ejecutó; el
  flujo de creación se cubre con el modal de navegador, la prueba Tauri del
  comando y la CLI. Sigue siendo condición de validación de la app real.
- El historial muestra instantáneas por archivo, no tarjetas visuales de escenas
  con cambios resaltados.
- Metas y temporizador son visuales; sus controles están deshabilitados.
- “Hoy día” se muestra como 0; no hay contador diario persistente.
- SEC21-02 (TOCTOU de `.escritor`) sigue como riesgo residual P3 aceptado.
- El release público sigue bloqueado por versión, firma, updater, AppImage,
  Windows/macOS, app instalada, DOCX en Word y CI remoto.
