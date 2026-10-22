# Fase 2.1 — Congelación r3 para seguridad

- **Fecha:** 2026-10-21
- **Commit congelado:** `e0bb3fc3f8f2cbc7a12a7c8ff5f586adac51a002`
- **Informe que motivó la corrección:**
  [`fase-2-1-seguridad-2026-10-21-r2.md`](fase-2-1-seguridad-2026-10-21-r2.md)
- **Estado del árbol:** limpio.

## Correcciones

- Rechazo de componentes `.` y `..` en la ruta de creación.
- Rechazo de espacios iniciales y `CONIN$`/`CONOUT$`.
- Validación de tipos, color, rutas y longitudes en `localStorage` de recientes.
- `clippy -D warnings` corregido; `calidad` vuelve a verde.

## Evidencia

```text
cargo test --workspace --all-targets --locked   94/94
python3 scripts/ci.py calidad                   OK
```

## Pendiente

- Security review r3 de la Fase 2.1.
- Release, UX y product owner.
