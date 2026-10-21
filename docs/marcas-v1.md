# Marcas v1 — contrato de personajes, tramas y comentarios

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Esquema del editor:** marcas de Tiptap, no HTML embebido

Las marcas son anotaciones sobre el texto que se serializan al propio Markdown.
No son nodos ni HTML: el archivo sigue siendo Markdown legible y el motor sigue
siendo la única autoridad del proyecto.

## 1. Sintaxis reservada

```text
[[personaje:ULID|texto marcado]]
[[trama:ULID|texto marcado]]
[[comentario:texto del comentario|texto marcado]]
```

Ejemplos:

```markdown
[[personaje:01KDVDNBYG20KW71NVRK6P2FHG|Marta]] dejó la maleta.

La luz del [[trama:01KDVDNFVGFKJ1G1SEHGTVZ69D|faro]] giraba cada doce segundos.

[[comentario:revisar el ritmo|—No puede ser— dijo.]]
```

Reglas:

- `personaje` y `trama` exigen un **ULID válido**; si no lo es, la construcción
  queda como texto literal y el serializador la escapa.
- El texto del comentario no puede contener `|`.
- El texto marcado no puede contener `]]`.
- No se admiten marcas anidadas dentro de marcas.
- Se admite formato inline dentro del texto marcado:
  `[[personaje:ULID|**Marta** y *Lucía*]]`.
- El resto de `[[...]]` que no case con el patrón se conserva como texto (con el
  escapado normal del serializador).

## 2. Modelo

| Marca | Atributo | Significado |
|---|---|---|
| `personaje` | `id` | ULID de una nota de tipo `personajes`. |
| `trama` | `id` | ULID de una nota de tipo `tramas`. |
| `comentario` | `texto` | Nota de revisión sobre el fragmento. |

- Las marcas se pintan en el editor con subrayado/color y no bloquean la
  escritura.
- La UI ofrece aplicar/quitar marcas sobre la selección.
- Si el ULID no existe en las notas del proyecto, la marca sigue siendo válida
  (el proyecto puede haberse movido); la UI la muestra como “referencia
  desconocida”.

## 3. Ida y vuelta

- El corpus golden incluye un caso de marcas.
- La prueba de ida y vuelta comprueba que
  `markdown → Tiptap → markdown` conserva la sintaxis exacta.
- El escapado normal de Tiptap sigue aplicándose al texto no marcado.

## 4. Exportación

- **Markdown:** conserva la sintaxis tal cual; el archivo es la fuente.
- **DOCX:** muestra solo el texto marcado. Las marcas de personaje y trama
  aportan el texto; los comentarios no se imprimen en el manuscrito.
- La exportación no modifica el proyecto.

## 5. Limitaciones

- Sin marcas anidadas.
- El comentario no puede contener `|`.
- El texto marcado no puede contener `]]`.
- Los ULID no se resuelven a títulos en el Markdown; la UI los resuelve contra
  las notas cargadas.
- Los comentarios no tienen hilo, autor ni estado resuelto en la v1.

## 6. Pruebas

```bash
pnpm ui:test
cargo test -p writer-core --test proyecto exportar_docx
```
