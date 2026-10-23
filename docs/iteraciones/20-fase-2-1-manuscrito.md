# Iteración 20 — Fase 2.1: renombrar y duplicar documentos

- **Fecha:** 2026-10-23
- **Estado:** **2.1 completa** en funcionalidad; pendiente revisión formal de
  QA/UX.
- **Commits:** `2a8dd8c` (renombrar/duplicar), `a423c8e` (arrastrar/soltar),
  `cba0040` (archivo/papelera) y los commits de metadatos/copia.
- **Contrato:** [`../esquema-v1.md`](../esquema-v1.md) (escenas y notas) y
  [`../esquema-v2.md`](../esquema-v2.md) (extensiones).

## 1. Alcance de este pase

- Renombrar una escena o nota sin cambiar su ID ni su ruta.
- Duplicar una escena o nota con ID nuevo, título «Copia de …» y sin historial.
- Conservar metadatos y cuerpo en la copia.
- Asignar el siguiente `orden` libre en escenas.
- Validar títulos vacíos o con caracteres de control.
- Mover una escena entre capítulos con arrastrar y soltar, con resaltado del
  destino y refresco del árbol.
- Archivar, enviar a papelera y restaurar escenas/notas con el campo lógico
  `archivo`, sin mover ni renombrar el Markdown.
- Filtrar el árbol por activos, archivo y papelera.
- Editar estado y etiquetas de escenas; etiquetas de notas.
- Filtrar por etiqueta.
- Copiar un documento y pegarlo como copia nueva.

## 2. Archivos

| Archivo | Cambio |
|---|---|
| `crates/core/src/project.rs` | `renombrar` y `duplicar` con transacción y CAS |
| `crates/core/src/model.rs` | Campo `archivo` en escenas/notas; `validar_texto` pasa a `pub(crate)` |
| `crates/core/src/project.rs` | `cambiar_archivo` para archivo/papelera/restaurar |
| `crates/core/tests/proyecto.rs` | Prueba de renombrar/duplicar escena y nota |
| `apps/desktop/src-tauri/src/lib.rs` | Comandos `renombrar`, `duplicar` y `mover` + pruebas |
| `ui/src/api.ts` | Puente Tauri y demo |
| `ui/src/App.tsx` | Formulario de título, Renombrar/Duplicar, Archivar/Papelera/Restaurar y filtros |
| `ui/src/styles.css` | Estilos del editor de título y acciones |
| `ui/src/App.test.tsx` | Prueba de UI de renombrar/duplicar |
| `e2e/fase-2-1-manuscrito.spec.ts` | Flujo E2E de renombrar/duplicar |
| `e2e/fase-2-1-arrastrar.spec.ts` | Flujo E2E de arrastrar/soltar entre capítulos |
| `e2e/fase-2-1-archivo.spec.ts` | Flujo E2E de archivar, filtrar y restaurar |
| `e2e/fase-2-1-metadatos.spec.ts` | Flujo E2E de estado, etiquetas y filtro |
| `e2e/fase-2-1-copiar.spec.ts` | Flujo E2E de copiar y pegar documento |

## 3. Decisiones

- **No se renombra el archivo:** el título es metadato y el nombre de archivo
  sigue siendo estable para no romper referencias ni el historial.
- **La copia no comparte ID ni historial:** se crea una escena/nota nueva y se
  copian cuerpo, estado, etiquetas, referencias y campos desconocidos.
- **El orden de la copia** es el siguiente libre del capítulo.
- **La copia de una nota** conserva el tipo y el prefijo de carpeta.

## 4. Verificación

```text
cargo test -p writer-core --test proyecto renombrar_y_duplicar_documentos  OK
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
  renombrar_y_duplicar_sobre_un_proyecto_real                              OK
pnpm typecheck                                                             OK
pnpm ui:test                                                               61/61
pnpm exec playwright test e2e/fase-2-1-manuscrito.spec.ts                  1/1
pnpm exec playwright test e2e/fase-2-1-arrastrar.spec.ts                  1/1
pnpm exec playwright test e2e/fase-2-1-archivo.spec.ts                    1/1
pnpm exec playwright test e2e/fase-2-1-metadatos.spec.ts                  1/1
pnpm exec playwright test e2e/fase-2-1-copiar.spec.ts                     1/1
pnpm e2e                                                                   31/31
```

## 5. Pendiente

- Revisión formal de QA/UX del bloque 2.1.
- Reordenar dentro del mismo capítulo por drag & drop; hoy se mueve entre
  capítulos.
- Copiar/pegar con el portapapeles del sistema; hoy la copia es interna del
  proyecto.
- Navegación por árbol con teclado para las nuevas acciones.

El resto de 2.1 está implementado y probado.

## 6. Límites

- Renombrar no cambia la ruta ni el nombre del archivo.
- Duplicar no copia comentarios, metas ni historial.
- Las operaciones usan el lock y el log del proyecto; no hay edición
  colaborativa.
