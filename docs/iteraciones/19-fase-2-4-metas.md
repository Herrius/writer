# Iteración 19 — Fase 2.4: núcleo de metas y sesiones

- **Fecha:** 2026-10-23
- **Estado:** núcleo implementado; Tauri y UI pendientes.
- **Contrato:** [`../metas-v1.md`](../metas-v1.md) y
  [`../esquema-v2.md`](../esquema-v2.md) §4.
- **Fase:** 2.4 (metas, estadísticas y temporizador).

## 1. Alcance de este pase

Se implementó la capa de datos local del proyecto, sin conectar todavía la
interfaz:

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

## 3. Verificación

```text
cargo fmt --all                                         OK
cargo clippy -p writer-core --all-targets -- -D warnings OK
cargo test -p writer-core --all-targets --locked        41 unitarias + 35 integración
cargo test -p writer-core goals                         7/7
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

- DTOs ts-rs y comandos Tauri `leer_metas`, `guardar_metas`,
  `agregar_sesion` y `leer_sesiones`.
- UI del panel Metas: metas, rachas, gráfico y temporizador.
- Cálculo de palabras por sesión y estadísticas.
- Integración con backup/exportación.
- Pruebas de UI/E2E y revisión de accesibilidad.

## 5. Decisión

El sidecar no entra en el log de operaciones del manuscrito: es una métrica
local, con escritura atómica y `fsync`. El manuscrito sigue siendo la verdad y
un sidecar corrupto no lo toca.

El siguiente pase conecta Tauri y la UI. Hasta entonces, la funcionalidad no se
anuncia como disponible.
