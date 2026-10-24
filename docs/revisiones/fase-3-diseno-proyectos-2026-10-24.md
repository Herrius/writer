# Revisión visual — panel de Proyectos (2026-10-24)

- **Alcance:** diseño de «Mis Proyectos», portadas, tarjetas y estado vacío.
- **Referencia:** captura aportada por el usuario (2026-10-24).
- **Resultado:** aprobado visualmente; composición y medidas equivalentes.
- **Captura:** [`fase-3-diseno-proyectos-2026-10-24.png`](fase-3-diseno-proyectos-2026-10-24.png).

## Comprobaciones

| Comprobación | Resultado |
|---|---|
| Cabecera con chevron y «Mis Proyectos» | OK |
| Crear tarjeta discontinua con `+` y texto en una línea | OK |
| Crear desde plantilla debajo de la tarjeta | OK; abre el asistente |
| Portada ilustrada con título y autor dentro | OK |
| Borde interior de portada | OK |
| Título, fecha y `⋯` debajo de la portada | OK |
| Fecha completa sin truncar | OK |
| Acciones de cabecera ocultas en reposo y accesibles al enfocar | OK |
| Vacío y lista compacta | OK |
| Sin overflow a 1440, 960 y 700 | OK |
| 960 px con texto al 200 % | OK |

Medidas en Chromium a 807 px de ventana (panel 768 px): panel 768,6 × 503,0;
crear 162 × 265,6 en x=47,4; proyectos en x=295,5 y x=543,5; portada 162 ×
265,6. La referencia era 768 × 505, crear 161 × 264 en x=46 y proyectos en
x=295 y x=541.

El arte de la portada es la ilustración por defecto de writer (CC0, StockSnap),
no la fotografía exacta de la referencia; la estructura, proporción, jerarquía
y comportamiento sí se corresponden.

```text
pnpm typecheck                                       OK
pnpm exec playwright test e2e/fase-1-panel.spec.ts   12/12
```
