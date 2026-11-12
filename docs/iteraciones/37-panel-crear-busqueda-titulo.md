# Iteración 37 — Panel Crear, búsqueda y título editable

- **Fecha:** 2026-11-12
- **Estado:** implementado y cubierto por 86 tests de UI y 58 flujos E2E.

## 1. Interfaz

- Se retiraron las pestañas **Plan** y **Cambios** del panel lateral y el botón
  de control de cambios de la barra superior. El backend de cambios y sus
  sidecars se conservan para la CLI y trabajo futuro, pero el editor guarda
  directamente.
- El botón `+` abre el nuevo panel **Crear**, con selector de:
  - libro, capítulo, sección, escena o nota;
  - libro/capítulo de destino;
  - título de escena o nota.
- La creación de capítulos ya no depende del antiguo panel Plan: se puede
  crear un capítulo desde el panel Crear o desde el menú del libro en el árbol.
- El título de la hoja es editable con un clic (Enter guarda, Escape cancela).
- Se añadió **Buscar en esta hoja** (`Ctrl+F`) con contador y navegación entre
  coincidencias.
- La búsqueda lateral queda para **Buscar texto en todas las hojas**
  (`Ctrl+Shift+F`), y se añadió un filtro por título para la lista de hojas.
- La línea de tiempo del historial recorre únicamente las revisiones existentes,
  con anterior/siguiente y contador `Revisión X · n/N`.

## 2. Pruebas

- `e2e/fase-4-creacion-titulo.spec.ts`: crea un capítulo desde Crear, edita el
  título, busca dentro de la hoja y filtra la lista por título.
- `e2e/fase-3-1-cambios.spec.ts`: comprueba la línea de tiempo y la comparación
  de una revisión.
- Se retiraron los flujos E2E de Plan, Plot Grid y propuestas de cambio, porque
  esas secciones ya no forman parte de la interfaz.
