# Iteración 39 — Nota flotante al seleccionar texto

- **Fecha:** 2026-11-12
- **Estado:** implementado y cubierto por 87 tests de UI y 59 flujos E2E.

## 1. Interfaz

- Al seleccionar texto en el editor aparece un globo flotante junto a la
  selección con la acción **Comentar**, al estilo de Word.
- Al pulsar **Comentar**, el globo se convierte en una caja para escribir la
  nota, con **Añadir nota** y **Cancelar**. Enter guarda y Escape cierra.
- La nota se guarda como comentario anclado: crea el hilo en
  `.escritor/comentarios/` y aplica la marca `[[comentario:…]]` sobre el texto.
- El globo se oculta al plegar la selección, al hacer scroll, al cambiar el
  tamaño de la ventana o al pulsar fuera del editor.
- El botón **Comentario** de la barra de formato se mantiene como acceso
  alternativo.
- En solo lectura no se muestra el globo.

## 2. Detalles técnicos

- La posición se calcula con `editor.view.coordsAtPos` y se pinta con
  `position: fixed`, de modo que acompaña a la selección sin depender del
  contenedor del editor.
- La selección se captura al abrir la caja; así escribir en el campo o pulsar
  los botones no pierde el rango sobre el que se aplica la nota.
- La validación de la nota y de la selección ocurre antes de crear el hilo, para
  no dejar comentarios huérfanos si el texto contiene `|` o `]]`.

## 3. Pruebas

- `ui/src/EditorMarkdown.test.tsx`: la selección muestra **Comentar**, la caja
  acepta texto y el Markdown resultante conserva la marca del comentario.
- `e2e/fase-2-2-comentario-anclado.spec.ts`: crea una nota desde el globo
  flotante y comprueba el `[[comentario:ULID|texto|…]]` en la vista Markdown.
