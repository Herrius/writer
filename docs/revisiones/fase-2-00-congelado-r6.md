# Fase 2 — Acta de congelación r6 (seguridad r2 corregida)

- **Fecha:** 2026-10-21
- **Commit congelado:** `b336467726373ea863eb960796f817ae7b8d0b48`
- **Informe que motivó las correcciones:**
  [`fase-2-seguridad-2026-10-21-r2.md`](fase-2-seguridad-2026-10-21-r2.md)
- **Estado del árbol:** limpio al congelar.

## Cambios

| Hallazgo | Corrección |
|---|---|
| SEC-02-R (P2) | `reindex` ya no decide por el texto del error; `parece_corrupto` exige `Error::Index` y el índice se aparta solo desde `.escritor` validado |
| SEC-02-H (P3) | Se rechazan archivos operativos con `nlink > 1` (enlaces duros) en Unix |
| SEC-04 FIFO (P3) | `ruta_del_proyecto` exige archivo regular en el componente final; `hash_archivo` lo revalida |
| SEC-03-R (P4) | `.gitleaksignore` con fingerprints exactos; `zizmor` solo se exige si existe `.github/` |

## Evidencia

| Gate | Resultado |
|---|---|
| `cargo test --workspace --all-targets --locked` | OK — 92 pruebas (34 unitarias + 33 integración + 25 CLI) |
| `python3 scripts/ci.py calidad` | OK |
| `python3 scripts/ci.py interfaz` | OK |
| `python3 scripts/ci.py producto` | OK |
| `python3 scripts/ci.py seguridad` | OK — cargo-deny, gitleaks historial/árbol, zizmor omitido sin `.github/` |

## Residuales declarados

- TOCTOU entre validación y escritura; no hay `openat`/`O_NOFOLLOW`.
- Límites de tamaño de documentos/log/proyecto (SEC-04) siguen pendientes.
- Windows no se ha ejecutado; la validación léxica está probada en Linux.
- No se ejecutaron GUI, `e2e-tauri` ni empaquetado en esta congelación.

## Pendiente

- Repetir security review (r3) sobre este commit.
- Release engineer, UX/accesibilidad/rendimiento y product owner.
