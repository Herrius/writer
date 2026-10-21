# Fase 2 — Acta de congelación r5 (seguridad corregida)

- **Fecha:** 2026-10-21
- **Commit congelado:** `7b7734418b11fca4f2f29fd14639679b23e2cf90`
- **Informe que motivó las correcciones:**
  [`fase-2-seguridad-2026-10-21.md`](fase-2-seguridad-2026-10-21.md)
- **Estado del árbol:** limpio al congelar.

## Cambios

| Hallazgo | Corrección |
|---|---|
| SEC-01 (P1) | Rutas de efecto confinadas a `manuscrito/`/`notas/`; validación de componentes de plataforma y rechazo de enlaces en cualquier ancestro |
| SEC-02 (P2) | `.escritor`, lock, log, objetos, índice y cuarentena deben ser reales; validación antes de leer/escribir |
| SEC-03 (P3) | `.gitleaks.toml`, escaneo de árbol solo con archivos versionados y `zizmor` omitido si no hay workflows |

## Evidencia

| Gate | Resultado |
|---|---|
| `cargo test --workspace --all-targets --locked` | OK — 88 pruebas |
| `python3 scripts/ci.py calidad` | OK |
| `python3 scripts/ci.py interfaz` | OK |
| `python3 scripts/ci.py producto` | OK |
| `python3 scripts/ci.py seguridad` | OK — cargo-deny, gitleaks historial/árbol, zizmor omitido sin `.github/` |

## Limitaciones declaradas

- La validación de Windows (`\`, `C:`, UNC) está implementada y probada en
  Linux a nivel de cadena, pero falta ejecutarla en Windows.
- Queda una ventana TOCTOU si un atacante local cambia un enlace después de la
  validación y antes de la escritura.
- No se repitieron los gates con GUI; el release engineer debe ejecutar
  `e2e-tauri` y `empaquetado`.

## Pendiente

- Repetir security review sobre este commit.
- Release engineer, UX/accesibilidad/rendimiento y product owner.
