# Fase 2 — Acta de congelación r13 (revalidación de release tras UX)

- **Fecha:** 2026-10-21
- **Commit de código congelado:** `c39dcae53fbee53f6340ec4a8ad09a660d41a150`
- **HEAD del pipeline:** `fe4dfa56ff31c33deee2357cac12d738ac2d44c4`
- **QA:** r3 aprobada con observaciones.
- **Seguridad:** r3 aprobada con observaciones.
- **UX:** r5 aprobada (0 P1/P2).
- **Release previo:** aprobado con observaciones sobre `b336467`; este acta
  revalida el release tras los cambios de UI y del núcleo de recuento.

## Cambios desde el release anterior

- UI: árbol por libro/capítulo, teclado, foco, zoom, banner de error, textos,
  E2E de accesibilidad y ajustes de historial/IME.
- Núcleo: `contar_palabras` cuenta el texto visible de las marcas.
- Tauri: `zoomHotkeysEnabled` y permiso de zoom.

## Evidencia

| Gate | Resultado |
|---|---|
| `cargo test --workspace --all-targets --locked` | 92 pruebas |
| `pnpm typecheck` | OK |
| `pnpm ui:test` | 57/57 |
| `pnpm e2e` | 9/9 |
| `python3 scripts/ci.py calidad` | OK |
| `python3 scripts/ci.py interfaz` | OK |
| `python3 scripts/ci.py producto` | OK |
| `python3 scripts/ci.py e2e-tauri` | OK (r9; repetir no es necesario para CSS) |

## Pendiente

- Release engineer: `empaquetado` sobre este commit, hashes y contenido.
- Product owner.
