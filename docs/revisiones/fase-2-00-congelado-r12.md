# Fase 2 — Acta de congelación r12 (UX-28 corregido en flujo)

- **Fecha:** 2026-10-21
- **Commit congelado:** `c39dcae53fbee53f6340ec4a8ad09a660d41a150`
- **Informe que motivó la corrección:**
  [`fase-2-ux-2026-10-21-r4.md`](fase-2-ux-2026-10-21-r4.md)
- **Estado del árbol:** limpio al congelar.

## Corrección

El banner de error ya no es sticky. Al aparecer un error de guardado, el panel
vuelve al inicio (`scrollTop = 0`) y el foco va al aviso, que tiene
`role="alert"`. Así el banner conserva su altura natural en cualquier zoom,
incluido el 200 % de texto, y no tapa permanentemente el documento: el panel
puede recorrer el aviso y el contenido sin solapamientos.

## Evidencia

| Gate | Resultado |
|---|---|
| `pnpm typecheck` | OK |
| `pnpm ui:test` | 57/57 |
| `pnpm e2e` | 9/9 Chromium |
| Gates previos | Rust 92, calidad/interfaz/producto verdes |

## Pendiente

- Repetir UX (r5) sobre este commit.
- Revalidar release por CSS/UI y después product owner.
