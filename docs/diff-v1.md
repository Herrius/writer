# Diferencias y comparación de versiones — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado en la Fase 3.2.
- **Ámbito:** local/offline.

## 1. Modelo

Una diferencia es una lista ordenada de líneas:

```json
{
  "tipo": "igual | insertada | borrada",
  "numeroBase": 1,
  "numeroPropuesta": 1,
  "texto": "línea sin salto"
}
```

- `numeroBase` es la línea en el texto base/antiguo; puede ser `null` en una
  inserción.
- `numeroPropuesta` es la línea en el texto nuevo/actual; puede ser `null` en un
  borrado.
- Los números empiezan en 1.
- `insertadas` y `borradas` resumen el número de líneas de cada tipo.

La presentación es **lado a lado**: una fila muestra el número y el texto de
cada lado; las inserciones dejan vacío el lado base y los borrados dejan vacío el
lado propuesto.

## 2. Algoritmo

1. Normaliza `CRLF`/`CR` a `LF`.
2. Elimina un único salto final para que `"a\n"` y `"a"` sean iguales.
3. Recorta el prefijo y el sufijo comunes.
4. Calcula la subsecuencia común máxima del tramo central con programación
   dinámica.
5. Si el tramo central supera 2.000.000 de celdas, evita la matriz y emite un
   bloque de borrados seguido de inserciones. El resultado sigue siendo correcto;
   solo es menos granular.

No usa red, no ejecuta diff externo y no modifica archivos.

## 3. Comparación con una revisión

`Project::comparar_documento(objetivo, revision)`:

- localiza el documento actual y la operación de la revisión;
- exige que la revisión haya modificado ese archivo;
- toma el contenido **posterior** a la revisión (`hash_nuevo`) del blob store;
- extrae el cuerpo Markdown sin frontmatter;
- lo compara con el cuerpo actual.

Si el archivo fue eliminado en esa revisión, la base es vacía y
`baseExiste = false`. Si la revisión no tocó el documento, la operación devuelve
un error de entrada: no se inventa el contenido intermedio.

La comparación no cambia la revisión actual. Para recuperar una versión se usa
la reversión existente, que crea una revisión nueva.

## 4. DTO y comandos

Tauri:

- `comparar_documento(ruta, objetivo, revision)` devuelve
  `ComparacionDto { revision, ruta, baseExiste, insertadas, borradas, lineas }`.

La UI ofrece **Comparar con el documento actual** en cada archivo de una
revisión del historial y muestra el diff en el mismo panel.

## 5. Límites de v1

- Diff por líneas, no por caracteres ni palabras.
- El fallback de documentos desproporcionados pierde granularidad.
- Solo se compara un documento contra una revisión que lo haya modificado; el
  diff de proyecto completo queda fuera de esta entrega.
- No hay edición del diff ni fusión de tres vías.
