# Iteración 17 — Fase 1: rediseño del panel de Proyectos

- **Fecha:** 2026-10-22
- **Estado:** en curso
- **Plan de producto:** `../../Plan - Rediseño del panel de Proyectos.md`
- **Plan de ejecución:** [`../plan-ejecucion-local-v1.md`](../plan-ejecucion-local-v1.md)
- **Baseline de código:** `fca190f` (sin cambios de UI desde `773cc2c`)

## F1.0 — Contrato visual y línea base

### Objetivo

Fijar las medidas reales del panel antes de modificarlo, para que F1.2 y F1.3
tengan un contrato verificable y no se confunda la portada 2:3 con la tarjeta
completa.

### Mediciones

Chromium headless sobre la demo, con 8 proyectos recientes y sin conexión.

| Escenario | Tarjeta crear | Tarjeta proyecto | Portada | Datos | Sidebar |
|---|---:|---:|---:|---:|---:|
| 1440×900, cuadrícula | 155,27 × 297,11 | 155,27 × 297,11 | 155,27 × 232,89 | 56,19 | 300 × 738,13 |
| 1440×900, vacío | 155,27 × 205,00 | — | — | — | 300 × 601,59 |
| 960×640, cuadrícula | 191,75 × 351,81 | 191,75 × 351,81 | 191,75 × 287,63 | 56,19 | 250 × 1215,38 |
| 960×640, raíz 32 px (200 %) | 237,22 × 481,20 | 237,22 × 481,20 | 237,22 × 355,83 | 109,38 | 250 × 2807,97 |
| 700×800, cuadrícula | 143,31 × 279,16 | 143,31 × 279,16 | 143,31 × 214,97 | 56,19 | 661,63 (apilada) |
| 1440×900, lista | 1011,63 × 64,59 | 1011,63 × 89,59 | 54 × 70 | 38,39 | 300 × 1037,28 |

Comprobaciones:

- ratio de portada en cuadrícula: `232,89 / 155,27 = 1,4999` (2:3 correcto);
- sin overflow horizontal a 960×640 ni a 960×640 con raíz de 32 px
  (`scrollWidth = clientWidth = 960`);
- la barra lateral mide lo mismo que el panel a 1440 cuando hay proyectos;
- el estado vacío deja la tarjeta de creación en 205 px y rompe la referencia
  de 300 px completos;
- la vista de lista conserva su altura compacta (89,59 px por proyecto).

### Defectos que debe corregir la fase

1. **Estado vacío:** la tarjeta de creación baja a 205 px cuando no hay
   proyectos. Debe conservar la altura de una tarjeta completa.
2. **Alineación estructural:** la tarjeta de creación debe medir lo mismo que
   una tarjeta de proyecto (portada 2:3 + datos), no solo que la portada.
3. **Barra lateral:** tras retirar meta diaria e Ideas, el espacio liberado debe
   quedar reservado y la columna debe seguir ocupando la altura del panel.
4. **Controles sin acción:** se retira solo «Ayuda» del panel; el resto de
   controles deshabilitados se conserva como excepción aprobada por el product
   owner y se documenta en `inicio-v1.md`.

### Contrato de medidas

| Elemento | Referencia a 1440 px | Regla responsive |
|---|---:|---|
| Portada | 160 × 240 px (2:3) | Ancho `w`, alto `1,5w`; nunca se estira |
| Zona de datos | ~60 px | Se mantiene la altura del contenido |
| Tarjeta de proyecto | 160 × 300 px | `w × (1,5w + datos)` |
| Tarjeta de creación | 160 × 300 px | Iguala la tarjeta completa, no la portada |

La referencia de 160 px es aproximada: la cuadrícula actual reparte 155,27 px
por columna a 1440 con 8 proyectos. La prueba de aceptación comprobará la
igualdad entre tarjeta de creación y tarjeta de proyecto, y el ratio 2:3 de la
portada; no exigirá un píxel exacto de ancho si la cuadrícula responsive no lo
permite.

### Evidencia

- Capturas y medidas: `/tmp/fase-1/` (`1440-proyectos.png`, `1440-vacio.png`,
  `1440-lista.png`, `960-proyectos.png`, `960-200.png`, `700-proyectos.png`,
  `medidas.json`).
- Script temporal de medición: `medir-fase1.mjs` (no versionado).
- Comando: `node medir-fase1.mjs` con el servidor de `pnpm ui:dev` en
  `http://127.0.0.1:1420`.

### Decisión de F1.0

Se mantiene la cuadrícula responsive actual y se corrige la alineación con una
medida estructural compartida entre la tarjeta de creación y la tarjeta de
proyecto. No se fija un ancho rígido de 160 px que rompa la cuadrícula a 1440.

## F1.1 — Retirada de meta diaria, Proyectos personales, Ideas y Ayuda

### Cambios

- `ui/src/Inicio.tsx`:
  - se eliminaron la tarjeta de meta diaria y la sección «Proyectos
    personales» con el cuaderno «Ideas»;
  - se eliminó el estado `aviso` y su banner, que solo servía al clic de
    «Ideas»;
  - se eliminó el botón deshabilitado «Ayuda» de la barra superior;
  - se conservaron Notificaciones, Filtros, Colección, Plantillas, Papelera,
    Más opciones y Crear desde plantilla como excepción documentada.
- `ui/src/styles.css`: se retiraron `.tarjeta-meta`, `.meta-punto`,
  `.proyectos-personales`, `.cuaderno-ideas`, `.cuaderno-espiral` y
  `.cuaderno-bombillas`.
- `ui/src/App.test.tsx`: nueva prueba de regresión que comprueba la ausencia de
  los cuatro bloques y la permanencia de los controles acordados.
- `docs/inicio-v1.md` §1: contrato actualizado.

### Verificación

```text
pnpm typecheck       OK
pnpm ui:test         60/60
```

### Límites

- La barra lateral queda con la tarjeta de bienvenida; el espacio reservado se
  implementa en F1.3.
- El resto de controles deshabilitados sigue visible por decisión del product
  owner; no se añaden acciones nuevas.

## F1.2 — Tarjeta de creación y portadas

### Cambios

- `ui/src/Inicio.tsx`: la tarjeta de creación contiene una medida invisible
  (`crear-medida`) con la misma estructura que una tarjeta de proyecto
  (portada 2:3 + `.tarjeta-datos`) y un contenido visible superpuesto.
- `ui/src/styles.css`: la tarjeta usa un borde discontinuo superpuesto
  (`::after`) que no altera el layout; la medida invisible fija la altura. La
  vista de lista oculta la medida y conserva el formato compacto.
- `e2e/fase-1-panel.spec.ts`: pruebas de igualdad de alturas, ratio 2:3,
  estado vacío, 960/200 % y lista compacta.
- `docs/inicio-v1.md` §1: contrato de la tarjeta actualizado.

### Medidas después del cambio

| Escenario | Tarjeta crear | Tarjeta proyecto | Portada | Datos | Ratio |
|---|---:|---:|---:|---:|---:|
| 1440×900 | 155,27 × 297,11 | 155,27 × 297,11 | 155,27 × 232,89 | 56,19 | 0,6667 |
| 1440×900, vacío | 155,27 × 297,08 | — | — | 56,19 (medida) | — |
| 960×640 | 191,75 × 351,81 | 191,75 × 351,81 | 191,75 × 287,63 | 56,19 | 0,6667 |
| 960×640, 200 % | 237,22 × 481,20 | 237,22 × 481,20 | 237,22 × 355,83 | 109,38 | 0,6667 |
| 700×800 | 143,31 × 279,16 | 143,31 × 279,16 | 143,31 × 214,97 | 56,19 | 0,6667 |
| 1440×900, lista | 1011,63 × 64,00 | 1011,63 × 89,59 | 54 × 70 | — | — |

La igualdad es exacta (±0,03 px) en cuadrícula. El estado vacío pasa de 205 px
a 297,08 px. El ancho de 155,27 px a 1440 se mantiene porque la cuadrícula
reparte seis columnas; la referencia de 160 px es aproximada.

### Verificación

```text
pnpm typecheck                         OK
pnpm ui:test                           60/60
pnpm exec playwright test e2e/fase-1-panel.spec.ts   4/4
```

### Límites

- No se fija un ancho rígido de 160 px: se conserva la cuadrícula responsive.
- La vista de lista mantiene su altura compacta y no usa la medida de
  cuadrícula.
