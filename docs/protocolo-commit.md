# Protocolo de commit — v1

> Normativo para la Fase 1. Detalla el orden exacto de una transacción, qué se
> considera confirmado y cómo se recupera un corte. El resumen vive en
> [`esquema-v1.md`](esquema-v1.md) §9.

## 1. Principio

**`COMMIT` es el único punto de confirmación.** Antes de `COMMIT` no hay cambio
confirmado; los archivos de trabajo pueden contener efectos transitorios que se
revierten al reabrir con el motor. Después de `COMMIT`, el cambio está confirmado
y no se revierte. Un `PREPARE` sin `COMMIT` y sin `ABORT` se revierte al abrir el
proyecto; un `ABORT` válido es terminal.

## 2. Forma de una operación

Una transacción tiene un `op_id` ULID, una `base_rev` (revisión confirmada
actual) y una `target_rev = base_rev + 1`. Sus efectos son una lista de cambios
de contenido sobre rutas relativas:

| Operación | Efectos |
|---|---|
| Crear | `{ruta, hash_anterior: null, hash_nuevo: H}` |
| Reescribir | `{ruta, hash_anterior: H₁, hash_nuevo: H₂}` |
| Eliminar | `{ruta, hash_anterior: H, hash_nuevo: null}` |
| Mover | `{ruta_vieja, hash_anterior: H, hash_nuevo: null}` + `{ruta_nueva, hash_anterior: null, hash_nuevo: H}` |

`hash_nuevo: null` significa que el archivo ya no existe después del efecto.
Un movimiento son dos efectos, no un campo especial: así la reversión usa el
mismo algoritmo que cualquier otra operación.

Las rutas de efecto se validan antes de cargar el log y antes de aplicar cada
efecto:

- relativas, con separadores `/`, dentro de `manuscrito/` o `notas/`;
- sin `\`, `:`, prefijos de unidad, UNC, componentes `.` o `..`, controles ni
  nombres reservados de Windows;
- sin enlaces simbólicos en ningún componente existente;
- `.escritor`, el log, el lock, `objetos`, `indice.sqlite` y `cuarentena` deben
  ser archivos o directorios reales, no enlaces.

## 3. Orden de una transacción

1. **Lock del SO** sobre `.escritor/bloqueo` (`File::try_lock`). Si está ocupado,
   se espera hasta `WRITER_LOCK_TIMEOUT_MS` (5 s por defecto) y se falla con
   `proyecto ocupado`.
2. **Recuperación** de cualquier `PREPARE` pendiente antes de empezar.
3. **Lectura del estado anterior** de cada ruta y cálculo de `hash_anterior`.
   Si `crear_solo` y el archivo existe, se aborta.
4. **Blobs**: se guardan los bytes del estado anterior y del nuevo en
   `.escritor/objetos/<hex>`, con temporal + `fsync` + `rename`. Los anteriores
   son obligatorios para poder revertir.
5. **`PREPARE`** al log + `fsync`.
6. **Efectos**, uno a uno, en el orden de la lista:
   - se comprueba que el hash actual sigue siendo `hash_anterior` (CAS
     optimista);
   - se escribe el temporal en el mismo directorio, `fsync`, y se reemplaza
     (`ReplaceFile`/`MoveFileExW` en Windows, `rename` + `fsync` del directorio
     en POSIX);
   - si `hash_nuevo` es `null`, se elimina el archivo y se sincroniza el
     directorio.
7. **`COMMIT`** al log + `fsync`. Aquí, y solo aquí, el cambio está guardado.
8. **Índice** en una transacción SQLite. Si el índice falla, se reconstruye sin
   tocar los archivos; el commit ya está confirmado.

## 4. Fallo antes de `COMMIT`

Si un CAS falla, una escritura falla o el proceso se corta después de `PREPARE`,
se revierten los efectos ya aplicados:

- se recorren en orden inverso;
- se incluye también el efecto que devolvió error: el reemplazo o borrado pudo
  completarse antes de que fallara el `fsync` del directorio; el rollback lo
  decide por hash;
- solo se restaura un efecto si el archivo actual coincide con `hash_nuevo`;
- se restaura el blob de `hash_anterior`, o se elimina si era `null`;
- si el archivo cambió por fuera, se deja intacto y se registra conflicto;
- **solo cuando la reversión terminó y está persistida** se anexa `ABORT`
  (`motivo: "conflicto"` o `"error"`);
- si falta un blob necesario, está corrupto o falla la sincronización de la
  reversión, no se toca nada más, **no se anexa `ABORT`** y el `PREPARE` queda
  pendiente para el siguiente arranque.

La reversión es idempotente: si se corta a su vez, el siguiente arranque vuelve
a comprobar los hashes y termina.

## 5. Recuperación al abrir

Al abrir un proyecto con `.escritor/operaciones.log`:

1. Se validan checksums y secuencias. Una última línea **incompleta** (sin salto
   final) se descarta; una línea completa con checksum o JSON inválido es un
   error de recuperación, también si es la última.
2. Se recogen los `PREPARE` sin `COMMIT` **y sin `ABORT`**. Un `ABORT` válido es
   terminal: solo se escribió después de persistir la reversión, y no se vuelve a
   aplicar en aperturas posteriores. Si el proceso murió antes de escribirlo, el
   `PREPARE` sigue pendiente.
3. **Preflight**: se leen y verifican por hash todos los blobs que harán falta
   para restaurar un `hash_anterior`. Si falta o está corrupto alguno, no se
   modifica ningún archivo y se devuelve modo recuperación.
4. Se revierte cada operación pendiente con el algoritmo de §4.
5. Se anexa `ABORT` si faltaba y la reversión terminó.
6. Se reindexa.

Un `ABORT` con `motivo: "conflicto"` se conserva y `verify` lo muestra, pero no
se reprocesa.

## 6. Ventana del CAS

El CAS es **optimista, no atómico**. Entre la comprobación del hash y el
reemplazo hay una ventana en la que otro proceso puede escribir. Qué garantiza
v1:

- **Procesos cooperativos** (CLI, GUI, MCP) con lock del SO en sistemas locales:
  no hay dos escritores; el hash detecta cambios previos.
- **Editores externos que ignoran el lock**: si escriben antes de la
  comprobación, la operación aborta con conflicto visible y conserva su cambio;
  si escriben dentro de la ventana, el resultado es *last-writer-wins* y el
  cambio externo puede perderse. No se promete ausencia de pérdida.
- **SMB/NFS**: no soportados para escritura en v1.

La prueba `WRITER_FAULT=before_replace` introduce una escritura externa dentro de
la ventana y fija el resultado observable: gana el motor, el contenido externo se
pierde. `WRITER_FAULT=before_check` introduce el cambio antes de la comprobación y
fija el otro resultado: conflicto, se conserva el cambio externo y se anexa
`ABORT`.

## 7. Inyección de fallos

| Valor | Punto de corte |
|---|---|
| `prepare` | Después de `PREPARE` + `fsync`, antes de aplicar efectos. |
| `after_write:N` | Después de aplicar `N` efectos. |
| `commit` | Después de `COMMIT` + `fsync`. |
| `before_replace` | Después del CAS y antes del reemplazo; escribe contenido externo. |
| `before_check` | Antes del CAS; escribe contenido externo para forzar conflicto. |

El proceso termina con código 70 sin ejecutar destructores, como un `SIGKILL`
controlado. `WRITER_FAULT_EXTERNAL` permite fijar el contenido externo.

## 8. Log

- JSON canónico: claves ordenadas por bytes, sin espacios, enteros decimales.
- Línea: `<json>\t<blake3-hex>\n`.
- `seq` empieza en 1 y crece de uno en uno.
- El checksum se calcula sobre los bytes exactos del JSON.
- `PREPARE`, `COMMIT` y `ABORT` comparten el mismo `op_id`.
- `target_rev = base_rev + 1`; las revisiones empiezan en 1.
- Los tipos desconocidos con checksum válido se ignoran con aviso.

## 9. Lock

- Archivo `.escritor/bloqueo`; el contenido JSON es informativo. El directorio
  `.escritor` y el archivo de lock deben ser reales: un enlace se rechaza antes
  de abrirlos.
- Lock exclusivo del SO, sin vencimiento. Si el proceso muere, el SO lo libera.
- Se toma por transacción y también para recuperar al abrir.
- En red, el lock puede no excluir; el CAS sigue protegiendo y `verify` avisa.

## 10. Índice

- `.escritor/indice.sqlite`, WAL, `synchronous=FULL`.
- `user_version = 1`; una versión desconocida, un archivo ilegible o una
  corrupción detectada al reconstruir se aparta a
  `indice.sqlite.corrupto-<ts>` y se reconstruye desde los archivos y el log.
- Se reconstruye desde los archivos y el log; nunca gana al archivo.
- `reindex --limpiar` elimina blobs que no aparecen en ningún efecto del log ni
  en el estado actual.

## 11. Comandos y alcance

| Comando | Efectos | Scope |
|---|---|---|
| `init` | Crea `proyecto.yml`, `.escritor/` y la primera escena (rev 1). | `estructura` |
| `create escena` | Crea un archivo nuevo. | `estructura` |
| `create nota` | Crea un archivo nuevo. | `notas` |
| `write` | Reescribe el cuerpo; conserva frontmatter y campos desconocidos. | `prosa` / `notas` |
| `move` | Dos efectos: elimina origen, crea destino con el mismo contenido. | `estructura` |
| `delete` | Elimina el archivo; el blob queda en el historial. | `estructura` |
| `revert` | Restaura una revisión concreta si el estado actual coincide. | `revert` |

`delete` es lógico en el sentido de que es **reversible**: el contenido queda en
`.escritor/objetos` y se puede recuperar con `revert`. El archivo de trabajo se
elimina para que la carpeta refleje el proyecto actual.

`revert` puede partir de un proyecto con errores porque su objetivo puede ser
repararlos (por ejemplo, restaurar una nota referenciada que se borró). Tras el
commit vuelve a validar el estado; si quedan errores, los muestra y la CLI sale
con código 1. No se permite revertir si el archivo actual ya no coincide con el
`hash_nuevo` de la revisión.

## 12. Limitaciones conocidas

- La ventana del CAS no es atómica (§6).
- El lock no es fiable en SMB/NFS.
- El historial no se compacta en v1; `reindex --limpiar` solo recoge blobs
  huérfanos.
- La edición del frontmatter desde la CLI aún no existe; `write` cambia el
  cuerpo y `create` fija los metadatos iniciales.
