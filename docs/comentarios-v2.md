# Comentarios v2 — contrato local

- **Fecha:** 2026-10-23
- **Estado:** propuesta normativa para la Fase 2.2.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §3 y
  [`marcas-v1.md`](marcas-v1.md).

## 1. Alcance

Comentarios personales anclados al texto, con hilo, autor local, estado,
historial y notas adhesivas. No hay comentarios compartidos, cuentas ni red.

## 2. Ancla

```markdown
[[comentario:01K...|revisar el ritmo|—No puede ser— dijo.]]
```

- `01K...` es el ULID del hilo.
- `revisar el ritmo` es el mensaje inicial.
- `—No puede ser— dijo.` es el texto marcado.
- El comentario se pinta como una marca sobre el texto; no bloquea la edición.
- No se anidan comentarios.
- Los comentarios v1 de dos campos quedan como legado de solo lectura. La UI los
  muestra como «comentario heredado» y no los reescribe.

## 3. Modelo

El hilo vive en `.escritor/comentarios/<ULID>.json` con el esquema de
`esquema-v2.md` §3.2.

Estados:

| Estado | Significado |
|---|---|
| `abierto` | Tiene mensajes pendientes. |
| `resuelto` | El hilo se cerró; el ancla sigue visible si el usuario quiere. |

Operaciones:

- **Responder:** añade un mensaje `tipo: respuesta`.
- **Editar:** añade un mensaje `tipo: edicion` con `reemplaza` y el texto nuevo.
- **Borrar mensaje:** añade `tipo: borrado`; no elimina el historial.
- **Resolver / reabrir:** añade `tipo: resolucion` y cambia `estado`.
- **Borrar hilo:** elimina el ancla del Markdown y archiva el sidecar en
  `.escritor/comentarios/archivados/`; no se destruye el historial.

El autor es `local` o el autor del proyecto. No hay identidad remota.

## 4. Anclaje y huérfanos

- Al abrir el documento se busca el texto del ancla.
- Si se encuentra una vez, se actualizan offsets y hash.
- Si aparece varias veces, se usa el offset más cercano al anterior y se avisa.
- Si no aparece, `huerfano: true`; el hilo se muestra en una sección de
  huérfanos y no se borra.
- Si el usuario restaura el texto, el hilo puede volver a anclarse.
- `verify` comprueba el hash y reporta huérfanos como aviso, no como error.

## 5. Sticky notes

- Son comentarios sin selección (`ancla: null`, `tipo: sticky`).
- Se muestran en el panel de comentarios y en el documento si el usuario elige
  un punto de anclaje visual.
- No se imprimen en DOCX ni se exportan al Markdown del manuscrito.
- Se incluyen en el backup.

## 6. Copiar y pegar

- **Con comentarios:** copia el Markdown con las marcas completas, incluidos los
  ULID.
- **Sin comentarios:** elimina las marcas `[[comentario:...]]` y conserva el
  texto marcado.
- Al pegar un comentario con un ULID que ya existe en el documento, se genera un
  ULID nuevo y se copia el hilo como una conversación nueva.
- Al pegar entre proyectos, los ULID se regeneran y el hilo se marca como
  local al proyecto destino.

## 7. Previsualización e historial

- Al previsualizar una revisión, los comentarios se muestran en solo lectura.
- Un comentario cuyo ancla no existe en esa revisión aparece como huérfano.
- El historial de comentarios es el propio `mensajes` append-only; no se mezcla
  con el historial de versiones del manuscrito.

## 8. UI

- Panel lateral con lista de hilos, filtros `abiertos`/`resueltos`/`huérfanos`.
- Hilo con mensajes, autor, fecha, responder, editar, resolver y borrar.
- Al pulsar un hilo se salta al ancla; si es huérfano, se muestra el documento
  original y la última posición conocida.
- Sticky notes en una lista separada.
- Estados vacíos claros, sin controles deshabilitados sin acción.

## 9. Límites

- Sin comentarios compartidos ni notificaciones.
- Sin adjuntos ni imágenes en comentarios.
- Sin anidamiento de comentarios.
- Sin edición colaborativa.
- El texto de un comentario no puede contener `|` ni `]]`.

## 10. Pruebas

- Ida y vuelta Markdown → Tiptap → Markdown con comentarios v2.
- Comentario v1 legado no se reescribe.
- Responder, editar, resolver y borrar conservan historial.
- Ancla borrada → huérfano; texto restaurado → se reancla.
- Copiar/pegar con y sin comentarios.
- Previsualización de revisión en solo lectura.
- `verify` detecta sidecar corrupto y lo aparta sin tocar el manuscrito.
