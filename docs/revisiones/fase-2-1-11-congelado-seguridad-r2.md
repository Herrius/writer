# Fase 2.1 — Congelación r2 para seguridad (P2 corregido)

- **Fecha:** 2026-10-21
- **Commit congelado:** `924d48568e17ca1d2a77caa4a32f7742965b699f`
- **Informe que motivó la corrección:**
  [`fase-2-1-seguridad-2026-10-21.md`](fase-2-1-seguridad-2026-10-21.md)
- **Estado del árbol:** limpio.

## Corrección SEC21-01

`Project::init` ya no confía en la ruta del webview:

- resuelve la ruta contra el directorio actual y el padre canónico;
- exige que el destino sea una carpeta (no vacía) o cree una nueva bajo un
  padre existente y real;
- rechaza `..`, `.`, ruta vacía, nombres inválidos/reservados, destinos
  enlazados y escapes del padre;
- `Log::crear_vacio` exige un padre real y rechaza un log enlazado o no regular.

Regresiones: rutas inválidas, destino no vacío, destino enlazado y creación
normal con/sin notas.

## Evidencia

```text
cargo test --workspace --all-targets --locked   94/94
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml   3/3
pnpm typecheck                                  OK
pnpm ui:test                                    58/58
pnpm e2e                                        10/10
python3 scripts/ci.py calidad                   OK
```

## Pendiente

- Security review r2 de la Fase 2.1.
- Release, UX y product owner.
