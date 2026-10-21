# Iteración 11 — Correcciones de la QA r2

- **Fecha:** 2026-10-21
- **Alcance:** corrección de los hallazgos nuevos de la revisión QA r2.
- **Estado:** correcciones aplicadas; pendiente QA r3.
- **Informe de origen:** [`../revisiones/fase-2-qa-2026-10-21-r2.md`](../revisiones/fase-2-qa-2026-10-21-r2.md)

## 1. Resultado de la r2

La r2 confirmó los cinco P1 y los P2 de la r1, pero encontró dos defectos nuevos:

1. **P1:** la ventana guardaba los cambios, pero `destroy()` no estaba permitido
   por las capacidades de Tauri; la app no se podía cerrar.
2. **P2:** el observador de marcas mutaba el DOM que ProseMirror vigila y
   provocaba un bucle de repintado, CPU alta y títulos inestables.

También señaló documentación desactualizada sobre el corpus (17 → 18) y la
recomendación de ejecutar `reindex` tras una edición externa.

## 2. Correcciones

### Cierre de ventana

- Se añadió `core:window:allow-destroy` a
  `apps/desktop/src-tauri/capabilities/default.json`.
- `App.tsx` captura el rechazo de `destroy()`: si falla, vuelve a habilitar el
  cierre y muestra el error; si funciona, la ventana desaparece.
- El E2E de cierre ahora comprueba también que la ventana X11 y el proceso de la
  app desaparecen, no solo que el archivo se guardó.

### Títulos de marcas sin bucle

- Se eliminó el `MutationObserver` que escribía `title` y
  `data-referencia-desconocida` en el DOM.
- `ui/src/editor/referencias-plugin.ts` calcula decoraciones de ProseMirror para
  cada fragmento de personaje/trama. El título y el aviso de referencia
  desconocida forman parte del estado de la vista, no de una mutación externa.
- `ui/src/editor/referencias.ts` mantiene el mapa de títulos cargados; al
  cambiar las notas se actualiza y se fuerza un repintado con una transacción
  sin cambios.
- La prueba comprueba que el título es estable en diez muestras seguidas.

### Documentación

- Corpus actualizado a 18 parejas en `README.md` y `HANDOFF.md`.
- La búsqueda ya reindexa automáticamente por hash/`mtime`/tamaño; `reindex`
  queda como reconstrucción completa y así se documenta.

## 3. Ajustes tras la QA r3

La r3 aprobó la fase con observaciones. Se aplicaron los ajustes no bloqueantes:

- el E2E de cierre comprueba también que el proceso de la app termina, no solo
  que la ventana X11 desaparece;
- `HANDOFF.md` ya dice 18 parejas de corpus en todas sus menciones;
- la latencia con 1.000 marcas (~2,2–2,4×) queda documentada como limitación P3
  para que UX/rendimiento decida si es aceptable.

## 4. Evidencia

```text
pnpm typecheck                    OK
pnpm ui:test                      55/55
python scripts/ci.py e2e-tauri    OK (guardar + cerrar; ventana destruida)
```

## 5. Pendiente

- Repetir QA sobre el nuevo commit congelado (r3).
- Security reviewer, release engineer, UX/accesibilidad/rendimiento y product
  owner.
