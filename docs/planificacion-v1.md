# Planificación v1 — contrato local

- **Fecha:** 2026-10-23
- **Estado:** propuesta normativa para la Fase 2.3.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §5.

## 1. Alcance

Plot Grid, personajes, casts, Story Notes, cuaderno, construcción del mundo,
@menciones e Ideas. Todo local y sin servicios externos.

## 2. Plot Grid

` .escritor/planificacion/plot-grid.json` con el esquema de `esquema-v2.md`
§5.1.

- Cada trama es una nota de `notas/tramas/`.
- Cada punto enlaza una escena por ULID.
- La UI permite crear/renombrar tramas, asignar color, mover puntos y etiquetar.
- Una escena puede pertenecer a varias tramas.
- Una referencia desconocida se muestra como «escena desconocida» y no se
  borra.
- El orden visual no cambia el orden del manuscrito.

## 3. Personajes y casts

Los personajes siguen siendo notas Markdown en `notas/personajes/`.

Campos opcionales nuevos:

```yaml
rol: principal
cast: [01K..., 01K...]
alias: [Marta, Martita]
```

- `rol`: `principal` o `secundario`; ausente = sin clasificar.
- `cast`: lista de ULID de otras notas; no implica jerarquía.
- `alias`: lista de textos para búsqueda y detección.
- El editor de fichas muestra los campos existentes y los nuevos sin romper el
  Markdown.
- Un `cast` o alias inválido se reporta; no se borra el campo.

## 4. Story Notes, cuaderno y mundo

Se añaden dos tipos de nota:

```text
notas/cuaderno/<archivo>.md
notas/ideas/<archivo>.md
```

- `tipo` debe coincidir con la carpeta.
- `notas/cuaderno/` agrupa Story Notes, construcción del mundo y notas de
  continuidad.
- `notas/ideas/` es el cuaderno local de Ideas; no hay tarjeta en el panel de
  Proyectos.
- Las notas se editan con el mismo editor Markdown y el mismo autosave.
- El árbol lateral agrupa por tipo y permite filtrar.

## 5. @menciones

- Sintaxis: `[[nota:ULID|texto]]`.
- Se resuelven contra las notas del proyecto.
- Si el ULID no existe, se muestra «referencia desconocida» y se conserva.
- Al hacer clic se abre la nota; si no existe, se muestra el diagnóstico.
- No se permiten menciones anidadas.
- La exportación DOCX/Markdown conserva el texto visible; el ULID no se imprime
  como texto crudo en DOCX.

## 6. Ideas

- Viven en `notas/ideas/`.
- Se pueden crear, editar y convertir en proyecto local.
- Convertir en proyecto copia el contenido y genera IDs, fechas y estructura
  nuevos; no copia historial, comentarios ni índice.
- La conversión no borra la idea original salvo confirmación explícita.

## 7. UI

- Vista de planificación dentro del proyecto, con pestañas o secciones:
  Plot Grid, personajes, cuaderno, ideas.
- El Plot Grid se puede ver en cuadrícula o líneas; no requiere canvas externo.
- Las fichas de personaje se editan en el panel, no en un modal bloqueante.
- Los estados vacíos explican qué crear y no muestran controles sin acción.
- El árbol lateral conserva manuscrito, tramas, personajes, cuaderno, ideas y
  utilidades.

## 8. Límites

- Sin tableros compartidos, comentarios en el grid ni colaboración.
- Sin imágenes remotas ni mapas externos.
- Sin sincronización.
- El Plot Grid no sustituye al manuscrito: es una vista derivada y local.

## 9. Pruebas

- Crear trama/punto, mover y borrar sin tocar escenas.
- Referencia desconocida se conserva.
- Campos nuevos de personaje sobreviven a la edición.
- @mención resuelta y no resuelta.
- Ideas → proyecto conserva contenido y genera IDs nuevos.
- Sidecar corrupto se aparta sin tocar notas/escenas.
