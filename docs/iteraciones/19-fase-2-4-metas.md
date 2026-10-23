# Iteración 19 — Fase 2.4: núcleo de metas y sesiones

- **Fecha:** 2026-10-23
- **Estado:** implementado de extremo a extremo; pendiente revisión y
  estadísticas avanzadas.
- **Commits:** `7b30de7` (núcleo), `9b8919f` (Tauri/UI), `330f62d` (formato).
- **Contrato:** [`../metas-v1.md`](../metas-v1.md) y
  [`../esquema-v2.md`](../esquema-v2.md) §4.
- **Fase:** 2.4 (metas, estadísticas y temporizador).

## 1. Alcance de este pase

Se implementó la capa de datos local y se conectó con Tauri y la UI:

- `Metas` con meta diaria, de libro, desafío y archivadas.
- `Sesion` con preset, pausas, palabras, documento y autor.
- Lectura y escritura atómica de `.escritor/metas/metas.json`.
- Append con `fsync` de `.escritor/sesiones/<YYYY-MM>.jsonl`.
- Validación de ULID, fechas, presets, palabras, pausas y rutas.
- Rechazo de directorios de extensión enlazados.

## 2. Archivos

| Archivo | Cambio |
|---|---|
| `crates/core/src/goals.rs` | Nuevo módulo con modelo, validación, lectura/escritura y 7 pruebas |
| `crates/core/src/lib.rs` | Expone `pub mod goals` |
| `crates/core/src/fsutil.rs` | `validar_escritor` comprueba `metas`, `sesiones`, `comentarios` y `planificacion` si existen |
| `apps/desktop/src-tauri/src/lib.rs` | DTOs y comandos `leer_metas`, `guardar_metas`, `agregar_sesion`, `leer_sesiones`; exportación ts-rs |
| `ui/src/api.ts` | Puente Tauri y modo demo para metas y sesiones |
| `ui/src/App.tsx` | Panel de metas, presets, temporizador, pausa/reanudar/finalizar y resumen del día |
| `ui/src/styles.css` | Campos, error, resumen y estados del temporizador |
| `e2e/fase-2-metas.spec.ts` | Flujo real de guardar meta y ejecutar temporizador |

## 3. Verificación

```text
cargo fmt --all                                         OK
cargo clippy -p writer-core --all-targets -- -D warnings OK
cargo test -p writer-core --all-targets --locked        41 unitarias + 35 integración
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
  metas_y_sesiones_sobre_un_proyecto_real               OK
pnpm typecheck                                          OK
pnpm ui:test                                            60/60
pnpm exec playwright test e2e/fase-2-metas.spec.ts     1/1
pnpm e2e                                                26/26
```

Pruebas nuevas:

- metas por defecto si no existen;
- ida y vuelta de metas;
- objetivo negativo rechazado;
- ida y vuelta de sesión;
- preset/fin inválidos rechazados;
- sidecar corrupto devuelve error;
- directorio de metas enlazado rechazado (Unix).

## 4. Pendiente

- Estadísticas históricas fuera del mes actual; la racha y el gráfico de 7 días
  ya se calculan desde las sesiones.
- Integración con backup/exportación.
- Revisión de accesibilidad del panel y del temporizador.
- Prueba de escritorio con Tauri (la UI se probó en demo y con el comando
  Tauri aislado).

## 5. Decisión

El sidecar no entra en el log de operaciones del manuscrito: es una métrica
local, con escritura atómica y `fsync`. El manuscrito sigue siendo la verdad y
un sidecar corrupto no lo toca.

El temporizador y las metas ya están conectados. La fase 2.4 queda funcional,
pero no se declara cerrada hasta la revisión y las estadísticas restantes.
