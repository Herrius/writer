# Iteración 32 — Filtros, colecciones, plantillas y papelera de proyectos

- **Fecha:** 2026-10-24
- **Estado:** implementado en Linux x86_64; Windows pendiente de Fase 4.
- **Contratos:** [`../organizacion-v1.md`](../organizacion-v1.md),
  [`../papelera-v1.md`](../papelera-v1.md),
  [`../plantillas-v1.md`](../plantillas-v1.md).

## 1. Qué se implementó

### Filtros

- Autor, periodo (7/30 días/año), palabras mínimas y orden.
- Persistencia en `writer.filtros.v1`.
- Combinación con búsqueda y colección activa.
- Botón con contador `Filtros (n)` y limpieza desde el vacío.

### Colecciones

- CRUD local en `writer.colecciones.v1`.
- Nombre, color y rutas; máximo 50 colecciones y 1.000 rutas por colección.
- Crear desde la cabecera, asignar con `⋯`, filtrar con chips, gestionar
  renombrado/eliminación con confirmación.
- Al mover un proyecto a la papelera se retira de las colecciones.

### Plantillas

- Pestaña **Plantillas** operativa.
- Lista nombre, descripción, estructura y notas.
- Crear plantilla desde la UI (núcleo + Tauri, app data).
- Usar plantilla preseleccionada en el asistente.
- Eliminar con confirmación y validar que sea una plantilla real.
- Límite de 500 escenas para evitar plantillas desmesuradas.

### Papelera

- `crates/core/src/trash.rs`: mover, restaurar y purgar.
- Movimiento atómico a `<padre>/.writer-papelera/<ULID>/proyecto` con
  `papelera.json`.
- Comandos Tauri y UI con confirmación.
- La pestaña **Papelera** lista, restaura y purga; no sobrescribe destinos.
- Rechaza symlinks, directorios que no son proyectos y metadata ajena.

## 2. Archivos principales

- `ui/src/organizacion.ts`
- `ui/src/Inicio.tsx`
- `ui/src/features/inicio/ModalFiltros.tsx`
- `ui/src/features/inicio/ModalColeccion.tsx`
- `ui/src/features/inicio/ModalGestionColecciones.tsx`
- `ui/src/features/inicio/ModalAccionesProyecto.tsx`
- `ui/src/features/inicio/ModalPlantilla.tsx`
- `ui/src/features/inicio/ModalConfirmar.tsx`
- `crates/core/src/templates.rs`
- `crates/core/src/trash.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `e2e/fase-3-7-organizacion.spec.ts`

## 3. Pruebas

```text
cargo test --workspace --all-targets --locked   91 unit + 49 integración + 25 CLI + 1 perf
cargo test desktop                              9/9
pnpm typecheck                                  OK
pnpm ui:test                                    88/88
pnpm e2e                                        58/58
```

Casos nuevos destacados:

- filtros combinados y persistencia;
- colecciones crear/asignar/renombrar/eliminar y persistir al recargar;
- plantillas crear/listar/usar/eliminar y límite de escenas;
- papelera mover/restaurar/purgar;
- núcleo: symlinks, destino ocupado, metadata ajena y directorios arbitrarios.

## 4. Límites

- Filtros y colecciones viven en `localStorage`, no en el proyecto.
- La papelera es local y hermana del proyecto; no usa la papelera del SO.
- Si se pierde `localStorage`, los archivos de papelera siguen en disco pero la
  UI no los lista automáticamente.
- No hay purga automática ni restauración de colecciones al recuperar.
- La validación Windows de `rename`, rutas y permisos sigue pendiente.

## 5. Correcciones de la revisión QA/UX r1

- **Papelera:** la metadata se escribe antes de mover el proyecto; si falla, el
  slot vacío se elimina y el proyecto no se toca. Se valida que `ruta_original`
  sea hermana de `.writer-papelera` y que no apunte dentro del slot.
- **Plantillas:** se rechazan raíces enlazadas, FIFOs y archivos no regulares;
  se limitan archivos y tamaño; se deduplican notas; el límite baja a 500
  escenas; la creación corre en `spawn_blocking`.
- **Límites locales:** colecciones 50/1.000 y papelera 100 se comprueban al
  escribir y muestran error; ya no se recortan al recargar.
- **UI:** foco inicial/retorno tras `inert`, trampa Tab, pie de modal fijo,
  mensajes de error dentro de la confirmación, botones de peligro deshabilitados
  durante la operación, menú `⋯` de 24×24 y papelera responsive a 960/720/200 %.
- **Colecciones:** se guardan antes de Abrir, Mover o crear otra colección.
- **Contraste:** metadatos y “Crear desde plantilla” corregidos.
- **Docs:** conteos 91/49/25+1, 88 UI y 58 E2E; límites y validaciones
  actualizados.

## 6. Correcciones de la revisión QA/UX r2

- **Plantillas:** el límite se interpreta sobre la estructura declarada; los
  archivos de manuscrito solo rellenan escenas existentes y se rechazan si
  sobran. Presupuesto único de 500 archivos y 64 MiB entre `manuscrito/` y
  `notas/`; se probó `500 escenas + 1 archivo` y `1 escena + archivos extra`.
- **Foco apilado:** al abrir una confirmación o crear una colección desde el
  menú `⋯`, el foco vuelve al disparador original; al borrar filas de
  colecciones se enfoca el siguiente control; al borrar la última colección se
  enfoca la pestaña activa.
- **Límite de colecciones:** el error de 1.000 rutas se muestra dentro del
  modal y bloquea Abrir/Mover/Nueva hasta corregirlo.
- **Papelera:** restaurar usa el estado de proceso y deshabilita los botones;
  el error de purga no se duplica en el fondo; el botón `×` se deshabilita
  mientras se procesa.
- **Plantillas corruptas:** se pueden eliminar por API aunque el manifiesto sea
  inválido; el listado sigue mostrando solo las válidas.
- **Docs:** 91 unitarias + 49 integración + 25 CLI + 1 rendimiento, 88 UI y
  58 E2E.

## 7. Últimos ajustes P3

- Las plantillas corruptas se pueden eliminar por API/slug aunque no se listen.
- Los errores de colecciones no se duplican detrás del modal y se limpian al
  cerrarlo; al borrar una fila se enfoca el siguiente campo.
- Los conteos finales son 91 unitarias + 49 integración + 25 CLI + 1
  rendimiento, 88 UI y 58 E2E.
