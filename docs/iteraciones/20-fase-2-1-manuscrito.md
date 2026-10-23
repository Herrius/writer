# Iteración 20 — Fase 2.1: renombrar y duplicar documentos

- **Fecha:** 2026-10-23
- **Estado:** renombrar, duplicar, arrastrar/soltar y archivo/papelera
  implementados; resto de 2.1 pendiente.
- **Commits:** `2a8dd8c` (renombrar/duplicar), `a423c8e` (arrastrar/soltar) y el
  commit de archivo/papelera.
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
pnpm e2e                                                                   29/29
```

## 5. Pendiente de 2.1

- Estados, etiquetas y filtros por etiqueta.
- Copiar/pegar documentos completos.
- Navegación y filtro de archivos por texto.
- Revisión de accesibilidad y E2E de las operaciones nuevas.

El drag & drop mueve escenas entre capítulos; no reordena dentro del capítulo
todavía.

## 6. Límites

- Renombrar no cambia la ruta ni el nombre del archivo.
- Duplicar no copia comentarios, metas ni historial.
- Las operaciones usan el lock y el log del proyecto; no hay edición
  colaborativa.
