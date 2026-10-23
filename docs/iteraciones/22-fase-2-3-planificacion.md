# Iteración 22 — Fase 2.3: planificación

- **Fecha:** 2026-10-23
- **Estado:** Plot Grid, tipos de nota nuevos, @menciones y campos de personaje
  implementados; falta revisión de UX.
- **Contrato:** [`../planificacion-v1.md`](../planificacion-v1.md).

## 1. Alcance

- `crates/core/src/planning.rs`: Plot Grid local en
  `.escritor/planificacion/plot-grid.json`, validación de ULID, colores,
  puntos y duplicados.
- Notas nuevas: `cuaderno` e `ideas`; el árbol las separa de tramas y
  personajes.
- Ficha de personaje: `rol`, `cast` y `alias` como campos opcionales.
- @menciones `[[nota:ULID|texto]]` en el editor, con ida y vuelta.
- UI: pestaña Plan con creación de tramas, asignación de la escena activa,
  creación de escenas y notas.
- Comandos Tauri y DTOs para el Plot Grid y las fichas.

## 2. Verificación

```text
cargo test -p writer-core planning                         5/5
cargo test -p writer-core templates                        4/4
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
  plot_grid_sobre_un_proyecto_real                         OK
pnpm typecheck                                            OK
pnpm ui:test                                              63/63
pnpm exec playwright test e2e/fase-2-3-planificacion.spec.ts 1/1
```

## 3. Pendiente

- Vista visual del Plot Grid (cuadrícula/líneas) más allá de la lista.
- Editar/eliminar puntos y tramas desde la UI.
- Resolver @menciones a títulos y salto a la nota.
- Conversión de Ideas a proyecto.
- Revisión de accesibilidad.

## 4. Límites

- El Plot Grid es una vista derivada: no cambia el orden del manuscrito.
- Las referencias desconocidas se conservan.
- No hay tableros compartidos ni colaboración.
