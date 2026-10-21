# ADR 0001 — Editor Tiptap para la interfaz

- **Fecha:** 2026-10-20
- **Estado:** aceptado
- **Fase:** 2
- **Decisión relacionada:** `Plan - Software de escritura personalizado.md` §6

## Contexto

La interfaz necesita editar el cuerpo Markdown de escenas y notas sin tocar el
frontmatter. El motor ya es la autoridad: la GUI solo envía el cuerpo y llama a
`writer-core`; no reimplementa IDs, versiones ni recuperación.

Requisitos que pesan en la decisión:

- ida y vuelta Markdown verificable con un corpus (énfasis, listas, citas,
  código, enlaces, separadores);
- esquema de documento controlado, sin HTML arbitrario;
- undo/redo, selección, búsqueda y extensiones;
- integración React y empaquetado con Tauri sin servidor;
- posibilidad de añadir marcas propias más adelante (personajes, tramas).

## Opciones

### Tiptap 3 + ProseMirror

- Editor basado en ProseMirror, con React.
- Extensión oficial `@tiptap/markdown` para parsear y serializar Markdown.
- Esquema declarativo y extensiones maduras.
- Bundle mayor y necesidad de fijar el esquema para que el Markdown no derive.

### Milkdown

- Basado en ProseMirror, con enfoque Markdown-first y plugins.
- Buena integración Markdown, pero menor ecosistema de extensiones propias.
- Requiere probar el mismo corpus y fijar igualmente el esquema.

## Decisión

Se adopta **Tiptap 3** con `@tiptap/markdown`, React 19 y Vite.

Reglas de integración:

1. **El frontmatter nunca entra al editor.** La GUI recibe el cuerpo y el
   frontmatter por separado desde `writer-core`.
2. El esquema inicial se limita a: párrafo, título, énfasis, negrita, código en
   línea, lista de viñetas, lista ordenada, cita, bloque de código, enlace,
   separador y salto de línea.
3. El Markdown que produce el editor se normaliza en el núcleo antes de
   persistirlo; el editor no escribe archivos directamente.
4. Antes de conectar autosave se implanta el corpus de ida y vuelta; si una
   construcción no vuelve idéntica, se rechaza o se documenta como pérdida.
5. Las extensiones futuras (personajes, tramas, comentarios) se añaden como
   marcas del esquema, no como HTML embebido.

## Resultado del corpus (2026-10-20)

El corpus golden está en `ui/src/markdown/corpus/` y se ejecuta con
`pnpm ui:test`. Resultado:

- El esquema del ADR se conserva con normalizaciones documentadas: setext → ATX,
  referencias → enlaces inline, salto con barra → dos espacios, código indentado
  → cercado y escapado de literales.
- Se aceptan también tachado y autolinks, que StarterKit ya soporta.
- Se rechazan (documento en solo lectura) las construcciones que pierden
  contenido: HTML embebido, imágenes, tablas, listas de tareas y notas al pie.
- El autosave envía el hash del archivo leído; `writer-core` rechaza la escritura
  como conflicto si cambió por fuera.

El contrato completo, incluida la tabla de normalizaciones y pérdidas, está en
[`../markdown-v1.md`](../markdown-v1.md).

## Consecuencias

- Se acepta un bundle mayor a cambio de un esquema controlado y extensiones.
- La Fase 2 debe incluir pruebas golden de Markdown antes de habilitar el
  autosave.
- Milkdown queda descartado para v0.2; cambiar de editor exigiría otro ADR y
  repetir el corpus.
