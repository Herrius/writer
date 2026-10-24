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
- Límite de 10.000 escenas para evitar plantillas desmesuradas.

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
cargo test --workspace --all-targets --locked   83 unit + 49 integración + 25 CLI + 1 perf
cargo test desktop                              9/9
pnpm typecheck                                  OK
pnpm ui:test                                    83/83
pnpm e2e                                        52/52
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
