# Iteración 02 — Fase 1 (Motor + CLI, v0.1)

- **Fecha:** 2026-10-20
- **Alcance:** Fase 1 del plan (`Plan - Software de escritura personalizado.md`).
- **Estado:** **completada en local (Linux, Rust 1.99.1; MSRV 1.89.0
  verificada)** tras corregir seis fallos de secuencias/corrupción y un fallo de
  sincronización tras el reemplazo. Pendiente: matriz Windows/macOS y CI remoto.
- **Entregable:** CLI que crea, edita, indexa, versiona y recupera proyectos sin
  interfaz gráfica.

## 1. Resumen ejecutivo

La Fase 1 convierte el esqueleto de la Fase 0 en un motor usable. Un proyecto se
crea con `writer init`, se edita con `create`/`write`/`move`/`delete`, se consulta
con `list`/`read`/`history`/`show`, se revierte con `revert` y se exporta con
`export md`. Cada cambio pasa por una transacción con blobs, `PREPARE`, CAS,
`COMMIT` y recuperación al abrir.

La regla de durabilidad es la del esquema: **sin `COMMIT` no hay cambio**. Los
cortes en `prepare`, `after_write`, `commit`, `before_replace` y `before_check`
están cubiertos por pruebas que ejecutan procesos hijos y matan el proceso con
código 70.

## 2. Entregables

### 2.1 Modelo y formato

- `model.rs`: `Proyecto`, `Escena`, `Nota`, defaults del esquema, validación de
  tipos, listas de ULID, campos desconocidos conservados y conteo de palabras.
- `yaml.rs`: parser y escritor del subconjunto plano, con comillas dobles,
  escapes, listas en línea, rechazo de comentarios, comillas simples,
  anidamiento e indentación.
- `ulid.rs`: generación y validación de ULID (26 caracteres, Crockford).
- `time.rs`: RFC 3339 UTC con milisegundos, sin dependencias de fecha.
- `verify.rs`: escaneo, IDs duplicados, referencias por tipo, orden por carpeta,
  estructura de libros/capítulos/notas y cuarentena.

### 2.2 Escritura y blobs

- `fsutil.rs`: `atomic-write-file` (temporal + `fsync` + reemplazo atómico;
  `ReplaceFile`/`MoveFileExW` en Windows) y `fsync` del directorio en POSIX.
- `blob.rs`: `.escritor/objetos/<blake3>`, deduplicación, verificación de hash,
  listado y limpieza de huérfanos.

### 2.3 Log y recuperación

- `oplog.rs`: JSON canónico, checksum BLAKE3 por línea, validación de `seq`,
  descarte de la última línea truncada, corrupción en medio y tipos desconocidos.
- Efectos con `hash_anterior`/`hash_nuevo`; `hash_nuevo: null` representa una
  eliminación y un movimiento son dos efectos.
- `recovery.rs`: preflight de blobs, roll-back inverso, idempotencia, conflicto
  visible si el archivo cambió por fuera y `ABORT` solo después de la reversión.
- `lock.rs`: `File::try_lock` con espera acotada (`WRITER_LOCK_TIMEOUT_MS`).

### 2.4 Índice

- `index.rs`: SQLite WAL, `synchronous=FULL`, `user_version=1`, tablas de escenas,
  notas, referencias, etiquetas e historial, y FTS5 para escenas y notas.
- Un índice corrupto o de versión desconocida se aparta a
  `indice.sqlite.corrupto-<ts>` y se reconstruye.
- `reindex --limpiar` recoge blobs que no aparecen ni en el log ni en el estado
  actual.

### 2.5 CLI

Comandos implementados: `init`, `info`, `list`, `read`, `create escena`,
`create nota`, `write`, `move`, `delete`, `history`, `show`, `revert`, `verify`,
`reindex`, `export md`.

- Rutas relativas con `/` en el log y en el índice.
- IDs estables en `move`.
- `delete` elimina el archivo de trabajo y conserva el blob; `revert` lo
  restaura.
- `verify --cuarentena` copia (no mueve) los inválidos.

### 2.6 Inyección de fallos

`WRITER_FAULT` acepta `prepare`, `after_write:N`, `commit`, `before_replace` y
`before_check`. `WRITER_FAULT_EXTERNAL` fija el contenido de la escritura
externa. El corte termina con código 70 sin destructores.

## 3. Decisiones nuevas

| # | Decisión | Motivo |
|---|---|---|
| 1 | `hash_nuevo: null` = eliminación | Permite borrar y mover con el mismo algoritmo de reversión, sin campos especiales. |
| 2 | Movimiento = dos efectos | El roll-back restaura el origen y elimina el destino sin lógica adicional. |
| 3 | `delete` reversible | El blob queda en el historial; el archivo de trabajo desaparece. |
| 4 | `write` cambia el cuerpo | La CLI v0.1 no edita frontmatter; los campos desconocidos se conservan. |
| 5 | Índice reconstruido al escribir | Correctitud antes que microoptimización; 1.000 escenas siguen siendo rápidas. |
| 6 | `ABORT` terminal | Solo se escribe después de persistir la reversión; no se reprocesa. Si el proceso muere antes, el `PREPARE` sigue pendiente y se reintenta. |

## 4. Evidencia

```text
$ cargo fmt --all --check
(sin salida)                                              # OK

$ cargo clippy --workspace --all-targets --all-features -- -D warnings
Finished `dev` profile                                    # 0 avisos

$ cargo test --workspace --all-targets --locked
65 tests pasan                                            # 29 unitarios + 14 core + 22 CLI

$ cargo +1.89.0 test --workspace --all-targets --locked
65 tests pasan                                            # MSRV verificada en Linux

$ cargo deny --all-features --manifest-path Cargo.toml check
advisories ok, bans ok, licenses ok, sources ok

$ python scripts/ci.py producto
autopruebas OK · docs OK · fixture pequeño OK
determinismo OK · 12 hallazgos esperados (Python y Rust)
1.000 escenas generadas y verificadas en 0,3 s
CLI Rust: fixture pequeño OK · 12 hallazgos esperados en el roto
enlaces OK

# Copia temporal de fixtures/proyecto-1000 (no se modifica el original)
$ cargo run --bin writer -- reindex <copia> --limpiar
índice reconstruido: 1000 escenas, 23 notas, revisión 0     # 0,8 s
$ cargo run --bin writer -- list <copia>
1000 líneas                                                  # 0,1 s
```

Pruebas de caída verificadas:

| Prueba | Resultado |
|---|---|
| `prepare` | Archivo intacto; recuperación revierte y anexa `ABORT`. |
| `after_write:1` (un archivo) | Archivo nuevo en disco; al abrir se restaura el anterior. |
| `after_write:1` (movimiento, dos efectos) | Origen restaurado; destino inexistente. |
| `commit` | Cambio confirmado; no se revierte. |
| `before_replace` | Gana el motor; se documenta la pérdida del cambio externo. |
| `before_check` | Conflicto; se conserva el cambio externo y se anexa `ABORT`. |
| Blob faltante | Modo recuperación; `verify` falla y no se escribe. |
| Blob corrupto en preflight | No se modifica ningún archivo; el destino del movimiento se conserva. |
| Lock ocupado | `proyecto ocupado` sin espera indefinida. |

## 4.1 Corrección 02.1 — revisión externa (6 fallos)

La revisión encontró seis casos que las pruebas iniciales no cubrían. Se
convirtieron en regresiones y se corrigieron antes de cerrar la fase.

| # | Fallo | Corrección | Regresión |
|---|---|---|---|
| 1 | Un `ABORT` antiguo se reprocesaba y deshacía un guardado posterior. | `pendientes()` excluye operaciones con `ABORT` válido; `ABORT` es terminal. | `un_abort_antiguo_no_deshace_un_guardado_posterior` |
| 2 | Un checksum corrupto en la última línea completa se trataba como truncamiento y borraba una revisión confirmada. | Solo se descarta una línea final **incompleta** (sin salto). Una línea completa inválida detiene la recuperación. | `checksum_corrupto_en_la_ultima_linea_completa_no_borra_la_revision` |
| 3 | El preflight comprobaba existencia, no integridad; un blob corrupto podía dejar el movimiento sin origen ni destino. | El preflight verifica el hash de todos los blobs necesarios antes de tocar archivos. | `blob_corrupto_no_elimina_el_destino_del_movimiento` |
| 4 | Anexar a un log válido sin salto final introducía un NUL. | El offset de escritura se calcula antes de añadir el salto pendiente. | `anexar_a_un_log_sin_salto_final_no_introduce_nul` |
| 5 | Borrar una nota referenciada bloqueaba su propia reversión por los errores existentes. | `revert` parte con errores permitidos y valida el estado resultante; la CLI informa y sale 1 si quedan errores. | `revertir_nota_borrada_repara_referencias` |
| 6 | Una página SQLite corrupta con cabecera legible no se apartaba. | La reconstrucción detecta corrupción, aparta el índice y reintenta una vez. | `indice_con_pagina_corrupta_se_aparta_y_se_reconstruye` |

Además, `verify` muestra los `ABORT` con `motivo: "conflicto"`, aunque no se
reprocesan.

## 4.2 Corrección 02.2 — fallo de fsync después del efecto

| # | Fallo | Corrección | Regresión |
|---|---|---|---|
| 7 | Un fallo de `fsync` después del reemplazo o borrado dejaba el efecto fuera de `aplicados`; el rollback lo omitía y se escribía un `ABORT` terminal. El archivo quedaba sin `COMMIT` y la reapertura lo daba por bueno. | El efecto que devuelve error entra también en el rollback; este comprueba el hash actual y restaura si el disco ya cambió. Si la reversión no se puede sincronizar, no se anexa `ABORT` y el `PREPARE` queda pendiente. | `fallo_fsync_tras_reemplazo_se_revierte`, `fallo_fsync_tras_borrado_se_revierte`, `si_falla_el_fsync_del_rollback_no_se_persiste_abort` |

La inyección usa `WRITER_FAULT_FSYNC_DIR` y `WRITER_FAULT_FSYNC_ONCE=1`; la
primera permite fallar todas las sincronizaciones del directorio para probar que
un rollback incompleto no cierra la operación.

## 5. Criterios del gate

| Criterio | Estado |
|---|---|
| 1. Crear/editar/indexar/versionar por CLI | Cumplido. |
| 2. Corte en guardado sin cambios no confirmados visibles | Cumplido: los archivos de trabajo pueden contener el cambio antes del `COMMIT`, pero al reabrir con el motor se revierte si no está confirmado. |
| 3. Crash multiarchivo: todo o nada | Cumplido con movimiento de dos efectos. |
| 4. Índice corrupto se reconstruye | Cumplido, incluida corrupción parcial detectada al reconstruir. |
| 14. Markdown | Ida y vuelta de modelo y exportación cubiertos; corpus amplio queda para Fase 2. |
| 15. Lock: nunca dos escritores cooperativos | Cumplido con `File::try_lock` y prueba de lock ocupado. |

## 6. Deuda y pendientes

- **CI remoto:** sigue sin repositorio remoto; no se ha ejecutado Windows/macOS.
- **gitleaks/zizmor:** no ejecutados en esta copia (herramientas ausentes y
  directorio sin `.git`). `cargo-deny` sí está en verde.
- **MSRV 1.89:** verificada en local con `cargo +1.89.0 test`; falta el job en
  CI remoto.
- **Rendimiento del motor:** el gate mide el generador Python; falta `criterion`
  con 1.000 escenas para apertura, escritura y búsqueda.
- **Frontmatter editable:** `write` solo cambia el cuerpo; falta un comando para
  metadatos (título, POV, tramas, etiquetas).
- **Historial:** no se compacta; `reindex --limpiar` solo recoge blobs huérfanos.
- **Carrera CAS:** *last-writer-wins* en la ventana; documentado y probado, no
  eliminado.
- **Búsqueda CLI:** el índice FTS5 existe y tiene API, pero no hay comando
  `search` en la v0.1.
- **Archivos ocultos del repo:** `.gitignore`, `.editorconfig` y
  `.gitattributes` ya existen. En el momento de este informe faltaban `.github/`
  y el repositorio Git; desde el 2026-10-21 el repositorio local existe en
  `main`. `.github/` sigue pendiente.

## 7. Siguiente fase

Fase 2: cáscara Tauri, tipos TypeScript generados, editor con autosave,
búsqueda, línea de tiempo, estados visibles y exportación DOCX. Antes de la GUI,
la CLI ya cubre el ciclo completo y las pruebas de caída.
