# DOCX v1 — contrato de exportación

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Entrada:** cuerpo Markdown de escenas y notas
- **Salida:** archivo `.docx` (WordprocessingML dentro de un ZIP)

La exportación es **básica y deliberadamente conservadora**: prioriza que el
texto llegue completo y legible antes que reproducir cada detalle del Markdown.
No modifica el proyecto ni escribe dentro de él.

## 1. Uso

```bash
writer export docx ~/salida.docx ~/novela
writer export docx ~/salida.docx ~/novela --notas
writer export docx ~/salida.docx ~/novela --forzar
```

En la UI: **Exportar DOCX** en la barra superior, con la casilla **notas**. El
diálogo nativo elige el destino; la UI sobrescribe después de la confirmación
del diálogo.

Reglas del destino:

- no puede estar dentro del proyecto;
- si existe y no se pasa `--forzar`, la CLI falla;
- se escribe de forma atómica a través de `fsutil::escribir_atomico`.

## 2. Estructura del documento

1. Portada con título del proyecto y autor.
2. Por cada libro: título de nivel 1.
3. Por cada capítulo: título de nivel 2.
4. Por cada escena: título de nivel 3 y cuerpo Markdown.
5. Con `--notas`: salto de página, sección **Notas**, tipo de nota como nivel 2,
   título de nota como nivel 3 y cuerpo.

## 3. Markdown soportado

| Markdown | DOCX |
|---|---|
| Párrafos | Párrafos con espaciado posterior. |
| `#` … `######` | Títulos con tamaño y negrita directos. |
| `**negrita**`, `*cursiva*`, `~~tachado~~` | Runs con negrita, cursiva o tachado. |
| `` `código` `` | Run con fuente monoespaciada. |
| Enlaces inline y autolinks | Hipervínculo externo con relación en `document.xml.rels`. |
| Listas de viñetas y ordenadas | Prefijo `•` o `N.` con sangría por nivel. |
| Citas | Sangría izquierda y cursiva. |
| Bloques de código | Una línea por párrafo, fuente monoespaciada y sangría. |
| `---` | Línea centrada `* * *`. |
| Saltos duros | `<w:br/>` dentro del run. |
| Listas de tareas | Texto `[x]` / `[ ]`. |
| Imágenes | Texto `[imagen: alt]`; no se incrusta la imagen. |
| Marcas propias | Solo el texto marcado; los comentarios no se imprimen. |

## 4. Limitaciones

- Las listas usan prefijos y sangría, no numeración nativa de Word.
- No hay tablas, HTML, TOC, números de página, encabezados ni estilos de Word
  con navegación.
- Las notas al pie se conservan como texto literal.
- Las imágenes solo aportan su texto alternativo.
- El DOCX no es un formato de ida y vuelta: el proyecto sigue siendo la fuente de
  verdad Markdown.
- El ZIP se genera con `zip` 9 y solo se escribe; writer no extrae archivos de
  DOCX.
- La comprobación del destino resuelve enlaces simbólicos y `..` en el momento de
  validar. No elimina una carrera TOCTOU con un proceso local que cambie el
  enlace después de la validación.
- Los caracteres de control que XML 1.0 no permite (por ejemplo U+0001) se
  rechazan antes de escribir el destino; el archivo existente no se toca.

## 5. Pruebas

```bash
cargo test -p writer-core --test proyecto exportar_docx_incluye_manuscrito_y_notas
cargo test -p writer-cli --test cli exportar_docx_desde_cli
pnpm ui:test
```

La prueba del núcleo abre el ZIP, lee `word/document.xml` y
`word/_rels/document.xml.rels`, y comprueba títulos, texto, listas, cita, código,
enlace y notas. La de la CLI verifica el comando, el rechazo sin `--forzar` y la
sobrescritura con `--forzar`. La de UI verifica que la casilla de notas se envía
al comando.
