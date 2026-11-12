# Iteración 38 — Compatibilidad con proyectos antiguos y menús del árbol

- **Fecha:** 2026-11-12
- **Estado:** implementado y cubierto por las pruebas del motor, 86 tests de UI y
  58 flujos E2E.

## 1. Problema

Un proyecto creado por una versión anterior guardaba las escenas directamente
bajo el libro:

```text
manuscrito/01-libro/001-escena-sin-titulo.md
```

El verificador exigía exactamente `manuscrito/libro/capitulo/escena.md` y
marcaba ese archivo como error de estructura. Como cualquier error deja el
proyecto en solo lectura, el proyecto antiguo no se podía editar.

Además, el menú de un capítulo ofrecía crear **Capítulo** y **Sección**, pero no
**Escena**, que es la única operación útil en ese nivel del árbol.

## 2. Cambios

### 2.1 Compatibilidad en el motor

- `verify::escanear` acepta las escenas en:
  - `manuscrito/<libro>/<escena>.md` (formato antiguo, sin capítulo);
  - `manuscrito/<libro>/<capítulo>/<escena>.md` (formato canónico);
  - carpetas de agrupación adicionales dentro del libro.
- Un Markdown suelto directamente en `manuscrito/` sigue siendo error de
  estructura.
- El orden de lectura y la comprobación de `orden` duplicado se agrupan por la
  carpeta real que contiene cada escena, sin asumir una profundidad fija.
- `verify::libro_de_escena`, `carpeta_de_escena`, `capitulo_de_escena` y
  `titulo_capitulo_de_escena` centralizan la interpretación de la ruta.
- La exportación a Markdown, DOCX y PDF usa esas funciones: un libro con escenas
  antiguas exporta sin encabezados de capítulo vacíos y conserva el orden.
- Duplicar y mover escenas respeta la profundidad original; mover acepta una
  carpeta de destino con varios niveles (por ejemplo, una sección existente).
- No se mueven ni renombran archivos al abrir: abrir un proyecto sigue siendo
  una operación de solo lectura.

### 2.2 Menús del árbol

- **Libro:** Agregar nuevo → Capítulo o Escena.
- **Capítulo:** Agregar nuevo → Escena.
- **Carpetas anidadas existentes:** Agregar nuevo → Escena.
- Se retiró **Sección** de los menús del manuscrito y del panel **Crear**. Las
  carpetas de sección que ya existan se siguen mostrando, renombrando,
  eliminando y admiten escenas.
- El arrastrar y soltar de escenas usa la carpeta completa de destino, no solo
  los dos primeros niveles.

## 3. Pruebas

- `crates/core/tests/proyecto.rs::acepta_escenas_de_proyectos_antiguos_colgadas_del_libro`:
  verifica que el proyecto es escribible, que se puede editar y duplicar, y que
  la copia queda en el mismo libro.
- `e2e/fase-3-8-arbol.spec.ts`: crea capítulo y escena desde los menús, comprueba
  que un capítulo no ofrece **Capítulo** ni **Sección**, y renombra/elimina el
  capítulo con la escena.
- `fixtures/proyecto-roto`: el caso de estructura inválida se trasladó a
  `manuscrito/suelto.md` para seguir cubriendo el único nivel realmente
  prohibido.
