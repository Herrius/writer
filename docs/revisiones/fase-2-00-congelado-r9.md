# Fase 2 — Acta de congelación r9 (UX corregida)

- **Fecha:** 2026-10-21
- **Commit congelado:** `4484f78eb1f1757756fb13abae42e8ab5f452766`
- **Informe que motivó las correcciones:**
  [`fase-2-ux-2026-10-21.md`](fase-2-ux-2026-10-21.md)
- **Estado del árbol:** limpio al congelar.

## Correcciones P2

| Hallazgo | Corrección |
|---|---|
| UX-01 | El árbol agrupa por `libro/capítulo` con etiquetas «Libro 1 / Capítulo 2» |
| UX-02 | `tabindex` itinerante, flechas/Home/End, enlace «Saltar al contenido» y `Ctrl+Alt+E` |
| UX-03 | Anillo de foco `:focus-visible` en el editor |
| UX-04 | `zoomHotkeysEnabled`, permiso de zoom y texto auxiliar mínimo ~12 px |
| UX-05 | Recuento sobre texto visible; el núcleo elimina la sintaxis de marcas |
| UX-06 | Banner de error sticky, `role="alert"` y foco automático |

También se corrigieron P3 baratos: `aria-pressed`, nombre accesible del editor,
guardas IME, orden del historial de demo, contraste del distintivo Demo,
`prefers-reduced-motion`, toolbar sticky y E2E de accesibilidad.

## Evidencia

| Gate | Resultado |
|---|---|
| `cargo test --workspace --all-targets --locked` | OK — 92 pruebas |
| `pnpm typecheck` | OK |
| `pnpm ui:test` | OK — 57 pruebas |
| `pnpm e2e` | OK — 9/9 flujos Chromium, incluidos salto/foco/árbol |
| `python3 scripts/ci.py e2e-tauri` | OK — guardar y cerrar, ventana/proceso terminados |
| `python3 scripts/ci.py calidad` | OK |
| `python3 scripts/ci.py interfaz` | OK |
| `python3 scripts/ci.py producto` | OK |

## Pendiente

- Repetir UX/accesibilidad/rendimiento sobre este commit.
- Revalidar release (los assets de UI cambiaron) y después product owner.
