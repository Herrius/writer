# Fase 2 — Acta de congelación r10 (UX-06 corregido)

- **Fecha:** 2026-10-21
- **Commit congelado:** `7daa92e08b7f27f5b557e0ae2716cfd3f94fa73e`
- **Informe que motivó la corrección:**
  [`fase-2-ux-2026-10-21-r2.md`](fase-2-ux-2026-10-21-r2.md)
- **Estado del árbol:** limpio al congelar.

## Corrección

El banner de error del panel ahora es `flex: 0 0 auto`, con
`max-height: min(60vh, 24rem)` y scroll interno. El banner conserva su altura,
el motivo y las acciones aunque el documento sea largo y el panel esté
desplazado; sigue siendo sticky, `role="alert"` y recibe el foco.

## Evidencia

| Gate | Resultado |
|---|---|
| `pnpm typecheck` | OK |
| `pnpm ui:test` | 57/57 |
| `pnpm e2e` | 9/9 Chromium |
| Gates de QA previos | calidad/interfaz/producto/e2e-tauri verdes sobre la r9 |

## Pendiente

- Repetir UX (r3) sobre este commit.
- Revalidar release por el cambio de CSS y después product owner.
