# Iteración 20 — Fase 2.1: renombrar y duplicar documentos

- **Fecha:** 2026-10-23
- **Estado:** renombrar y duplicar implementados; resto de 2.1 pendiente.
- **Commit:** `2a8dd8c`.
- **Contrato:** [`../esquema-v1.md`](../esquema-v1.md) (escenas y notas) y
  [`../esquema-v2.md`](../esquema-v2.md) (extensiones).

## 1. Alcance de este pase

- Renombrar una escena o nota sin cambiar su ID ni su ruta.
- Duplicar una escena o nota con ID nuevo, título «Copia de …» y sin historial.
- Conservar metadatos y cuerpo en la copia.
- Asignar el siguiente `orden` libre en escenas.
- Validar títulos vacíos o con caracteres de control.

## 2. Archivos

| Archivo | Cambio |
|---|---|
| `crates/core/src/project.rs` | `renombrar` y `duplicar` con transacción y CAS |
| `crates/core/src/model.rs` | `validar_texto` pasa a `pub(crate)` |
| `crates/core/tests/proyecto.rs` | Prueba de renombrar/duplicar escena y nota |
| `apps/desktop/src-tauri/src/lib.rs` | Comandos `renombrar` y `duplicar` + prueba |
| `ui/src/api.ts` | Puente Tauri y demo |
| `ui/src/App.tsx` | Formulario de título y botones Renombrar/Duplicar en Notas |
| `ui/src/styles.css` | Estilos del editor de título y acciones |
| `ui/src/App.test.tsx` | Prueba de UI de renombrar/duplicar |
| `e2e/fase-2-1-manuscrito.spec.ts` | Flujo E2E en la demo |

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
```

## 5. Pendiente de 2.1

- Archivar y papelera con restauración.
- Mover por drag & drop en el árbol.
- Estados, etiquetas y filtros.
- Copiar/pegar documentos completos.
- Navegación y filtro de archivos.
- Revisión de accesibilidad y E2E de las operaciones nuevas.

## 6. Límites

- Renombrar no cambia la ruta ni el nombre del archivo.
- Duplicar no copia comentarios, metas ni historial.
- Las operaciones usan el lock y el log del proyecto; no hay edición
  colaborativa.
