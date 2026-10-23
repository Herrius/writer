# Iteración 21 — Fase 2.2: comentarios v2

- **Fecha:** 2026-10-23
- **Estado:** comentarios anclados, sticky notes, respuestas, resolución,
  huérfanos y filtros implementados; faltan edición/borrado y previsualización.
- **Contrato:** [`../comentarios-v2.md`](../comentarios-v2.md).

## 1. Alcance de este pase

- Sidecar `.escritor/comentarios/<ULID>.json` con hilo, autor, estado y
  mensajes append-only.
- Creación de comentarios anclados desde el editor con sintaxis
  `[[comentario:ULID|texto|texto marcado]]`.
- Sticky notes sin ancla.
- Listado por documento, respuestas y resolución.
- Compatibilidad con comentarios v1 de dos campos.

## 2. Archivos

| Archivo | Cambio |
|---|---|
| `crates/core/src/comments.rs` | Modelo, validación, creación, lectura, mensajes y resolución |
| `crates/core/src/lib.rs` | Expone `comments` |
| `crates/core/src/fsutil.rs` | Valida `.escritor/comentarios` real |
| `apps/desktop/src-tauri/src/lib.rs` | DTOs y comandos de comentarios + prueba |
| `ui/src/editor/marcas.ts` | Parser/serializador v2 y compatibilidad v1 |
| `ui/src/EditorMarkdown.tsx` | Crea el hilo antes de aplicar la marca |
| `ui/src/api.ts` | Puente Tauri y demo |
| `ui/src/App.tsx` | Pestaña Comentarios, sticky, respuesta y resolución |
| `ui/src/styles.css` | Estilos del panel de hilos |
| `e2e/fase-2-2-comentarios.spec.ts` | Sticky, respuesta y resolución |
| `e2e/fase-2-2-comentario-anclado.spec.ts` | Comentario anclado con ULID |

## 3. Decisiones

- El ancla vive en el Markdown; el hilo vive en el sidecar.
- Los mensajes no se destruyen: resolver o editar añade eventos.
- `sticky` no tiene ancla y no se imprime en DOCX.
- Los comentarios v1 se siguen leyendo; no se reescriben automáticamente.

## 4. Verificación

```text
cargo test -p writer-core comments                         4/4
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
  comentarios_sobre_un_proyecto_real                       OK
pnpm typecheck                                            OK
pnpm ui:test                                              62/62
pnpm exec playwright test e2e/fase-2-2-comentarios.spec.ts 1/1
pnpm exec playwright test e2e/fase-2-2-comentario-anclado.spec.ts 1/1
```

## 5. Pendiente de 2.2

- Editar y borrar mensajes con historial visible.
- Copiar/pegar texto con y sin comentarios.
- Comentarios de solo lectura en la previsualización de una revisión.
- Filtros abiertos/resueltos/huérfanos y salto al ancla.
- Revisión de accesibilidad del panel.

La detección de huérfanos y los filtros abiertos/resueltos/huérfanos ya están
implementados.

## 6. Límites

- El comentario v2 exige ULID; los comentarios v1 no se convierten.
- No hay comentarios compartidos ni notificaciones.
- El editor aplica la marca después de crear el sidecar; si el sidecar falla,
  no se aplica la marca.
