# Fase 2 — Acta de congelación r3 (correcciones de QA r2)

- **Fecha:** 2026-10-21
- **Commit congelado:** `6a0a55c1c6f3ddc281edc1afd0ace31b517eae15`
- **Commit revisado por QA r2:** `06a158105919cdc394a47a5acb827443bdda25f1`
- **Informe que motivó las correcciones:**
  [`fase-2-qa-2026-10-21-r2.md`](fase-2-qa-2026-10-21-r2.md)
- **Estado del árbol:** limpio al congelar.
- **Alcance:** P1 de cierre de ventana, P2 del observer de marcas y P4 de
  documentación detectados en la r2.

## Cambios principales

| Área | Corrección |
|---|---|
| Cierre de ventana | `core:window:allow-destroy` en las capacidades; el fallo de `destroy()` se captura y se puede reintentar |
| Marcas | Decoraciones de ProseMirror en lugar de mutar el DOM; títulos estables y sin bucle |
| E2E | El escenario `cerrar` comprueba que la ventana X11 y el proceso desaparecen |
| Documentación | Corpus 18 parejas; `reindex` descrito como reconstrucción completa, no obligatoria tras edición externa |

## Evidencia del builder sobre el commit congelado

| Gate | Resultado |
|---|---|
| `python3 scripts/ci.py calidad` | OK — 86 pruebas Rust |
| `python3 scripts/ci.py interfaz` | OK — typecheck, 55 pruebas UI, build Vite, 2 pruebas Tauri |
| `python3 scripts/ci.py producto` | OK — esquema, determinismo, 12 hallazgos, 1.000 escenas, enlaces, CLI |
| `python3 scripts/ci.py e2e` | OK — 7/7 flujos Chromium |
| `python3 scripts/ci.py e2e-tauri` | OK — `guardar` y `cerrar`; la ventana se destruye |
| `python3 scripts/ci.py empaquetado` | OK — deb y rpm verificados |

## Pendiente

- Repetir QA independiente (r3) sobre este commit.
- Security reviewer, release engineer, UX/accesibilidad/rendimiento y product
  owner.
- Validación externa: AppImage, firma, Windows/macOS, CI remoto y app instalada.
