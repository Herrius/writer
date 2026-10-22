# Fase 2 — Acta de congelación r11 (UX-28 corregido)

- **Fecha:** 2026-10-21
- **Commit congelado:** `33365553391148e6f1f8c1324cf935bdc9ddc182`
- **Informe que motivó la corrección:**
  [`fase-2-ux-2026-10-21-r3.md`](fase-2-ux-2026-10-21-r3.md)
- **Estado del árbol:** limpio al congelar.

## Corrección

Con `max-height: 520px` (equivalente a 200 % de zoom en la ventana mínima), el
banner de error deja de ser sticky y la toolbar también; el banner vuelve al
flujo normal del panel y el panel puede desplazarse para mostrar motivo,
acciones y contenido. En tamaños normales sigue siendo sticky.

## Evidencia

| Gate | Resultado |
|---|---|
| `pnpm typecheck` | OK |
| `pnpm e2e` | 9/9 Chromium |
| `python3 scripts/ci.py producto` | OK |
| Gates previos | UI 57/57, Rust 92, calidad/interfaz/producto verdes |

## Pendiente

- Repetir UX (r4) sobre este commit.
- Revalidar release por CSS y después product owner.
