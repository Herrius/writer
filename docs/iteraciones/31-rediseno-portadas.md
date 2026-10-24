# Iteración 31 — Rediseño de portadas y tarjetas de proyecto

- **Fecha:** 2026-10-24
- **Estado:** implementado y validado visualmente contra la referencia.
- **Evidencia:** [`../revisiones/fase-3-diseno-proyectos-2026-10-24.png`](../revisiones/fase-3-diseno-proyectos-2026-10-24.png)
- **Contrato actualizado:** [`../inicio-v1.md`](../inicio-v1.md) §1.

## 1. Objetivo

Sustituir la portada de degradado y la ficha de texto por una tarjeta de libro:

- portada ilustrada con título y autor dentro;
- título y fecha de modificación debajo;
- tarjeta de creación con la misma altura que la portada y borde discontinuo;
- enlace «Crear desde plantilla» debajo;
- cabecera «Mis Proyectos» limpia, con las acciones disponibles al pasar el
  ratón o al enfocar.

La referencia visual es la captura de producto aportada por el usuario el
2026-10-24. La composición y las medidas se validan con una prueba de geometría
en Chromium.

## 2. Cambios

- `ui/src/Inicio.tsx`:
  - nueva portada con arte, título centrado y autor al pie;
  - tarjeta de proyecto con título, «Modificado el …» y `⋯`;
  - fecha `modificado` en recientes;
  - tarjeta de creación y enlace de plantilla en una columna propia;
  - nombre accesible `Abrir <título> de <autor>`.
- `ui/src/recientes.ts`: validación de `modificado`.
- `ui/src/App.tsx`: se actualiza la fecha al abrir o crear un proyecto.
- `ui/src/assets/portada-arbol.jpg`: portada por defecto.
- `ui/src/styles.css`: nueva rejilla, proporción 0.61, sombras, tinte, estado
  vacío y vista de lista.
- `e2e/fase-1-panel.spec.ts`: contrato visual y de accesibilidad actualizado.

## 3. Medidas de la referencia frente al resultado

Medición a 807 px de ventana, que deja el panel en 768 px CSS.

| Elemento | Referencia | Resultado |
|---|---:|---:|
| Panel | 768 × 505 | 768,6 × 503,0 |
| Cabecera | 57 px | 56,2 px |
| Crear: x / ancho / alto | 46 / 161 / 264 | 47,4 / 162 / 265,6 |
| Proyecto 1: x | 295 | 295,5 |
| Proyecto 2: x | 541 | 543,5 |
| Portada: ancho / alto | 161 / 265 | 162 / 265,6 |
| Proporción | ≈0,61 | 0,61 |
| Fecha visible completa | sí | sí |
| Acciones de cabecera en reposo | ocultas | ocultas |

La portada usa `Árbol de invierno`, una imagen CC0 de StockSnap.io
(`EBNAMJ57QL`), recortada y compuesta en sepia. No requiere atribución; el
crédito se conserva en `ui/src/assets/portada-arbol.credito.txt`.

## 4. Validación

```text
pnpm typecheck                                  OK
pnpm exec playwright test e2e/fase-1-panel.spec.ts   12/12
```

La prueba `la composición reproduce la referencia de Mis Proyectos` comprueba
las posiciones, el tamaño de las tarjetas, la altura del panel, la cabecera y la
proporción de la portada. También se comprueba que la fecha no se recorte, que
la tarjeta no muestre el recuento de palabras, que las acciones aparezcan al
enfocar y que la vista de lista siga siendo compacta.

## 5. Límites

- La imagen de portada es un arte por defecto de writer; no se reproduce la
  fotografía exacta de la referencia. No hay todavía subida de portada ni
  selector de arte.
- El color elegido tiñe la portada al 16 %; no se guarda en el manifiesto.
- La fecha es la última apertura/creación registrada en recientes, no el mtime
  del sistema de archivos.
- El indicador `⋯` es visual; el menú de acciones de proyecto sigue fuera de
  alcance.
