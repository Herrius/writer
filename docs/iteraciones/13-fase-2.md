# Iteración 13 — Correcciones de UX, accesibilidad y rendimiento

- **Fecha:** 2026-10-21
- **Alcance:** P2 de la revisión UX r1.
- **Estado:** correcciones aplicadas; pendiente repetir UX.
- **Informe de origen:** [`../revisiones/fase-2-ux-2026-10-21.md`](../revisiones/fase-2-ux-2026-10-21.md)

## 1. Correcciones P2

### Árbol por libro y capítulo

`App.tsx` ya no usa la etiqueta humanizada como clave. Agrupa por la ruta
completa (`libro/capítulo`) y muestra «Libro 1 / Capítulo 2». Las notas se
agrupan por tipo con etiquetas legibles.

### Navegación por teclado

- Enlace «Saltar al contenido» al principio de la aplicación.
- `tabindex` itinerante en el árbol: solo el documento activo es tabulable.
- Flechas arriba/abajo, `Home` y `End` mueven el foco entre documentos.
- `aria-current="page"` en el documento activo.
- `Ctrl+Alt+E` enfoca el editor desde cualquier vista.

### Foco visible y zoom

- `.ProseMirror:focus-visible` tiene un anillo de 3 px.
- `zoomHotkeysEnabled` en la ventana Tauri y permiso
  `core:webview:allow-set-webview-zoom`.
- El texto auxiliar sube a ~12 px.

### Recuento de palabras

El contador usa el texto visible (`getText()`), no la sintaxis de las marcas.
`writer_core::model::contar_palabras` elimina la sintaxis de
`[[personaje|trama|comentario:...]]` antes de contar, para que índice y editor
coincidan en el caso de las marcas.

### Errores y conflictos visibles

- El banner de error del panel es sticky y tiene `role="alert"`.
- Al aparecer un error de guardado, el foco va al banner.
- Los banners de búsqueda, historial, detalle y error global también son
  alertas.
- La toolbar del editor es sticky.

### Otros ajustes

- `aria-pressed` en los botones de formato.
- El editor (`role="textbox"`) tiene nombre accesible.
- Guardas de composición IME en buscador y comentario.
- Orden del historial de la demo corregido (ascendente en la API, inverso en la
  UI, igual que la app real).
- Contraste del distintivo Demo y `prefers-reduced-motion`.
- E2E nuevos para salto al contenido, flechas del árbol y anillo de foco.

## 2. Evidencia

```text
pnpm typecheck                    OK
pnpm ui:test                      57/57
pnpm e2e                          9/9
pnpm check                        OK
cargo test --workspace            92/92
```

## 3. Pendiente

- Repetir UX/accesibilidad/rendimiento sobre el commit congelado.
- Product owner.
- P3/P4 restantes: tabs ARIA completos, historial virtualizado, cuarentena
  plegable, búsqueda con truncamiento, latencia de 1.000 marcas.
