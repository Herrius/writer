# Esquema en disco — v1

> Normativo. Define qué hay en disco, qué significa cada campo y qué invariantes
> debe poder comprobar `writer verify`. Todo lo que no esté aquí es derivado,
> interno o queda para una versión futura del esquema.
>
> Cambiar este documento sin subir `esquema` en `proyecto.yml` es un error.

## 0. Principios

1. **Los archivos son la verdad.** `proyecto.yml` y los `.md` son el proyecto.
   `indice.sqlite` es divergente por diseño: se puede borrar y reconstruir.
2. **El motor no adivina.** Un archivo inválido no se corrige: se pone en
   cuarentena y se informa.
3. **Nada binario en el manuscrito.** UTF-8 sin BOM, finales de línea LF.
4. **Los IDs son estables.** Renombrar, mover o reorganizar carpetas no cambia
   ningún `id` ni rompe referencias.
5. **YAML mínimo.** Se admite un subconjunto plano y predecible, para que el ida
   y vuelta con otras herramientas no destruya datos.

## 1. Estructura

```text
mi-novela/
  proyecto.yml
  manuscrito/
    01-libro/
      01-capitulo/
        001-escena.md
        002-otra-escena.md
  notas/
    tramas/
    personajes/
    lugares/
    cronologia/
    objetos/
  .escritor/
    indice.sqlite
    objetos/<hash>
    operaciones.log
    bloqueo
    cuarentena/
```

- `manuscrito/` contiene **solo** escenas, en tres niveles fijos:
  libro → capítulo → escena. Un nivel de más o de menos es un error de
  estructura, no una escena.
- `notas/` contiene **solo** notas, un nivel: `notas/<tipo>/<archivo>.md`, con
  `tipo` ∈ `tramas`, `personajes`, `lugares`, `cronologia`, `objetos`.
- Los nombres de carpeta y archivo son para humanos y para el sistema de
  archivos. **No definen el orden de lectura** (eso lo hace `orden`).
- Convención de nombre recomendada: `NNN-slug.md` con slug ASCII en minúsculas,
  sin acentos, separado por guiones. La CLI la aplica al crear; `verify` no la
  exige.

## 2. `proyecto.yml`

```yaml
esquema: 1
id: 01KDVDNAZ84TFF59TDWH9EDD1R
titulo: "El umbral"
autor: "Autora"
idioma: es
creado: "2026-10-20T10:00:00.000Z"
retencion: completa
guardar_prompts: false
politica_estructura: auto
politica_metadatos: auto
politica_notas: auto
politica_prosa: proponer
politica_exportar: proponer
```

| Campo | Tipo | Obligatorio | Regla |
|---|---|---|---|
| `esquema` | entero | sí | Exactamente `1` para este documento. Otro valor: el motor avisa y abre en solo lectura. |
| `id` | ULID | sí | Identidad del proyecto. No cambia nunca. |
| `titulo` | texto | sí | No vacío. |
| `autor` | texto | no | Vacío permitido. |
| `idioma` | texto | no | Código BCP 47 corto (`es`, `en`…). Por defecto `es`. |
| `creado` | texto | no | RFC 3339 UTC con milisegundos. Informativo. |
| `retencion` | enum | no | `completa` (por defecto), `diaria`, `por_tamano`. Nunca borra sin confirmación. |
| `guardar_prompts` | booleano | no | Por defecto `false`. La autoría se guarda igual. |
| `politica_estructura` | enum | no | `auto` por defecto. |
| `politica_metadatos` | enum | no | `auto` por defecto. |
| `politica_notas` | enum | no | `auto` por defecto. |
| `politica_prosa` | enum | no | `proponer` por defecto; `proponer` y `bloquear` prohibidos como `auto` para prosa. |
| `politica_exportar` | enum | no | `proponer` por defecto. |

Los enums de política son `auto`, `proponer`, `bloquear`. Un campo desconocido se
conserva al reescribir y `verify` lo marca como aviso (compatibilidad hacia
delante). Un campo obligatorio ausente o inválido es error de proyecto: la app
abre en **modo recuperación** y no escribe.

## 3. Escena

`manuscrito/<libro>/<capitulo>/<archivo>.md`:

```markdown
---
id: 01KDVDNBYG1HEAD8X4A1JH69RE
titulo: "El umbral"
orden: 10
estado: borrador
pov: 01KDVDNCXRH4QX4FR84G98PBSK
tramas: [01KDVDNDX0JMRNV7E9Z0C1HT0H, 01KDVDNEW8G7KYHRJXJG7DJ13N]
personajes: [01KDVDNCXRH4QX4FR84G98PBSK]
fecha_narrativa: "1998-06"
---

Texto de la escena.
```

| Campo | Tipo | Obligatorio | Regla |
|---|---|---|---|
| `id` | ULID | sí | Único en todo el proyecto (escenas y notas comparten espacio de IDs). |
| `titulo` | texto | sí | No vacío. Se puede repetir entre escenas. |
| `orden` | entero | sí | `1..99999`. Único **dentro de su carpeta**. Se recomiendan huecos de 10 para insertar sin renumerar. |
| `estado` | enum | no | `borrador` (por defecto), `revision`, `final`. Otro valor: aviso, no cuarentena. |
| `pov` | ULID | no | Debe existir y ser una nota de tipo `personajes`. |
| `tramas` | lista de ULID | no | Cada una debe existir y ser nota de tipo `tramas`. |
| `personajes` | lista de ULID | no | Cada uno debe existir y ser nota de tipo `personajes`. |
| `fecha_narrativa` | texto | no | Texto libre (`1998-06`, `invierno de 1998`). No se interpreta. |
| `etiquetas` | lista de texto | no | Libres; se indexan. |

Reglas adicionales:

- `id`, `orden` y `palabras` son **campos del motor**. `orden` se valida; `id`
  nunca se reasigna. `palabras` **no se escribe** en el archivo en v1: lo calcula
  el índice. Si aparece, `verify` avisa y la siguiente reescritura lo elimina.
- Un archivo sin `id`, con `id` mal formado, con frontmatter sin cerrar o con
  YAML fuera del subconjunto entra en **cuarentena**: no se le asigna un ID en
  silencio.
- Duplicar `id` es error de integridad: las dos copias quedan en cuarentena y el
  resto del proyecto sigue operativo.

### 3.1 Orden de lectura

El orden global se obtiene de forma determinista:

1. Carpetas de libro y capítulo ordenadas por su **prefijo numérico** si existe
   (`01-`, `02-`…); si no, por nombre en orden de bytes.
2. Dentro de cada capítulo, escenas por `orden` ascendente.
3. Empates de `orden` (no debería haberlos): error de `verify`; el índice
   desempata por ruta para no romper la lectura.

## 4. Nota

`notas/<tipo>/<archivo>.md`:

```markdown
---
id: 01KDVDNFVG2R00K6AGV0VFCXEC
tipo: personajes
titulo: "Marta"
fija: true
etiquetas: [protagonista]
refs: [01KDVDNBYG1HEAD8X4A1JH69RE]
---

Ficha libre en Markdown.
```

| Campo | Tipo | Obligatorio | Regla |
|---|---|---|---|
| `id` | ULID | sí | Único en todo el proyecto. |
| `tipo` | enum | sí | Debe coincidir con la carpeta: `tramas`, `personajes`, `lugares`, `cronologia`, `objetos`. |
| `titulo` | texto | sí | No vacío. |
| `fija` | booleano | no | `true` = la IA no la sobrescribe nunca (solo propone). |
| `etiquetas` | lista de texto | no | Libres. |
| `refs` | lista de ULID | no | Referencias libres; deben existir. |

Una nota con `tipo` distinto al de su carpeta es error de estructura: cuarentena.

## 5. Subconjunto YAML admitido

Válido en `proyecto.yml` y en el frontmatter:

- Claves planas `clave: valor`, una por línea. **Sin anidamiento.** Las líneas en
  blanco se ignoran.
- **Texto plano:** sin comillas. No puede empezar por un indicador YAML (`-`,
  `?`, `#`, `&`, `*`, `!`, `|`, `>`, `@`, `` ` ``, `%`, `[`, `{`) ni contener
  comillas simples o dobles. Los espacios interiores se conservan; los exteriores
  se recortan.
- **Texto entre comillas dobles:** obligatorio si el valor empieza por un
  indicador, si contiene `#`, `,`, `[`, `]`, `{`, `}` o comillas, o si parece
  número, `true`, `false` o `null`. Escapes admitidos: `\"`, `\\`, `\n`, `\t`.
  Una comilla interior sin escapar es error; nada puede seguir a la comilla de
  cierre.
- **Comillas simples: prohibidas.**
- Enteros decimales (`-?[0-9]+`), `true`/`false`, `null` y listas en línea
  `[a, b, c]` con elementos escalares. Sin listas ni mapas anidados.
- **Comentarios: prohibidos.** Un `#` fuera de comillas dobles inicia un comentario
  en YAML; aquí es un error de subconjunto. Para usar `#` como texto hay que
  entrecomillar (`titulo: "Escena # 1"`). Tampoco se admiten líneas de comentario
  dentro del frontmatter.
- Las líneas en blanco dentro del frontmatter se ignoran: no son clave, no
  cierran el bloque y no se conservan al reescribir.
- Sin bloques `|`/`>`, anclas, alias, tags, mapas ni comentarios.
- UTF-8 sin BOM y finales LF. El BOM y el CRLF se toleran al leer (aviso); un CR
  suelto es error.

Cualquier otra construcción es **fuera de subconjunto** → cuarentena (escena o
nota) o modo recuperación (`proyecto.yml`). El motor nunca reinterpreta: si no
puede leerlo tal cual, no lo toca.

## 6. Convenciones de texto

- **Codificación:** UTF-8 estricto, sin BOM.
- **Finales de línea:** LF. Un CR suelto es error de `verify`; CRLF se tolera al
  leer y se normaliza al reescribir.
- **Palabras:** secuencias maximales de caracteres alfanuméricos Unicode. El
  frontmatter no cuenta. El cuerpo sí. Se guarda en el índice, nunca en disco.
- **IDs:** ULID de 26 caracteres, alfabeto Crockford base32 en mayúsculas
  (`0123456789ABCDEFGHJKMNPQRSTVWXYZ`), primera letra `0`–`7`.
- **Hashes:** `blake3:<hex minúsculas>` en documentos y logs; sin prefijo en los
  nombres de blob.
- **Fechas de sistema:** RFC 3339 UTC con milisegundos y `Z`.

## 7. `.escritor/`

| Ruta | Papel | Si falta |
|---|---|---|
| `operaciones.log` | Autoridad del historial y de las revisiones. Append-only. | Modo recuperación. Sin historial no se inventa: se reindexa el presente y se avisa. |
| `objetos/<hex>` | Blobs inmutables con el contenido exacto (bytes) de cada versión de archivo. | Se puede perder contenido histórico; el estado actual vive en los `.md`. `verify` informa qué revisiones quedan incompletas. |
| `indice.sqlite` | Derivado. Búsqueda, listas, hashes y `mtime`. | Se reconstruye. |
| `bloqueo` | Archivo sobre el que se toma el lock del sistema operativo, uno por transacción. | Se crea al abrir el proyecto; su contenido es informativo. |
| `cuarentena/` | Archivos apartados con su motivo. | Se crea al primer caso. |

Nada más vive en `.escritor/`. Un archivo desconocido ahí no es del motor y no
se toca.

### 7.1 Blobs

- Nombre: hash blake3 en hexadecimal minúsculas, sin extensión.
- Contenido: **los bytes completos del archivo** (frontmatter incluido), tal como
  quedaron en esa revisión. Revertir es restaurar bytes, no reconstruir texto.
- Deduplicación por contenido: dos revisiones idénticas comparten blob.
- Escritura: temporal en `objetos/` → `fsync` → `rename`. Un blob huérfano es
  basura inofensiva; se recoge con `reindex --limpiar`.

### 7.2 `operaciones.log`

JSON Lines, una operación por línea, con checksum:

```text
<json-canónico>\t<blake3-hex-del-json>
```

- `json-canónico`: UTF-8, claves ordenadas por bytes, sin espacios, enteros
  decimales (sin flotantes), `true`/`false`/`null`, escapes mínimos.
- El checksum se calcula sobre los **bytes exactos** del JSON tal como aparecen
  antes del tabulador, y no incluye el propio checksum (no hay circularidad).
- Una línea final **incompleta** (sin salto de línea final, sin tabulador o con
  JSON/checksum cortado) se descarta. Una línea **completa** (con su salto de
  línea) con checksum inválido o JSON inválido es corrupción, también si es la
  última: no se descarta y el proyecto abre en modo recuperación. Un checksum
  inválido en medio del log también es corrupción.
- `seq` empieza en 1 y crece de uno en uno. Un salto es corrupción.

`PREPARE`:

```json
{
  "seq": 41,
  "tipo": "PREPARE",
  "ts": "2026-10-20T10:15:32.412Z",
  "op_id": "01KDVDNGTR27H6P3AMKDQG6STT",
  "task_id": "ia-estructura-42",
  "base_rev": 118,
  "target_rev": 119,
  "autor": "ia",
  "scope": "estructura",
  "modelo": "proveedor/modelo",
  "efectos": [
    {
      "ruta": "manuscrito/01-libro/01-capitulo/003-escena.md",
      "hash_nuevo": "blake3:238b4429a65d9a87985b33497d8368e0847b52437595018c604af92a154f3e98",
      "hash_anterior": "blake3:1cc40893501a6a30f02e8b24ee70ba2dff7115af4783a67fda2dc4cba4c895d5"
    }
  ]
}
```

`COMMIT`:

```json
{
  "seq": 42,
  "tipo": "COMMIT",
  "ts": "2026-10-20T10:15:32.780Z",
  "op_id": "01KDVDNGTR27H6P3AMKDQG6STT",
  "target_rev": 119
}
```

`ABORT`:

```json
{
  "seq": 43,
  "tipo": "ABORT",
  "ts": "2026-10-20T10:15:32.780Z",
  "op_id": "01KDVDNGTR27H6P3AMKDQG6STT",
  "motivo": "conflicto"
}
```

Reglas:

- `autor` ∈ `humano`, `ia`, `externo`. `task_id` y `modelo` pueden ser `null`.
- `hash_anterior` es `null` cuando el archivo no existía.
- `hash_nuevo` es `null` cuando el efecto elimina el archivo. Un movimiento se
  representa con dos efectos: el origen con `hash_nuevo: null` y el destino con
  `hash_anterior: null`.
- `efectos` es la lista de cambios; `hash_anterior` describe el estado previo que
  hay que restaurar en un roll-back.
- `target_rev = base_rev + 1`. Revisiones por proyecto, empiezan en 1.
- Un `PREPARE` sin `COMMIT` y sin `ABORT` es una operación pendiente. Al
  recuperar **se revierte**, nunca se completa: sin `COMMIT` no hay cambio
  confirmado.
- El roll-back restaura `hash_anterior` solo si el archivo actual coincide con
  `hash_nuevo`; si cambió por fuera, se deja intacto y se registra conflicto. Si
  `hash_anterior` es `null` (archivo nuevo), se elimina el archivo creado.
- **`ABORT` es terminal.** Solo se anexa después de que la reversión haya
  terminado y esté persistida (cada archivo restaurado y sincronizado). Una
  operación con `ABORT` válido está cerrada y **no se vuelve a aplicar** en
  aperturas posteriores, aunque un guardado posterior produzca el mismo
  `hash_nuevo`. Si falla la sincronización de la reversión, no se anexa `ABORT`
  y el `PREPARE` queda pendiente.
- Si el proceso muere después de revertir y antes de anexar `ABORT`, el siguiente
  arranque vuelve a revertir (idempotente: los archivos ya coinciden con
  `hash_anterior`), verifica los hashes y anexa el `ABORT` que falta.
- Cuando la reversión no puede completarse porque un archivo cambió por fuera, se
  deja constancia del conflicto y **después** se anexa `ABORT` con
  `motivo: "conflicto"`; el contenido externo no se toca. `verify` muestra los
  `ABORT` con conflicto. La recuperación es idempotente y se puede retomar tras
  otro corte.
- Los blobs de `hash_anterior` se guardan antes de aplicar cualquier efecto
  (§9, paso 1), así que restaurar nunca depende de leer el archivo actual. Antes
  de modificar nada, la recuperación **verifica la integridad** de todos los
  blobs que necesitará. Si falta o está corrupto alguno, el proyecto abre en
  **modo recuperación**, no se modifica ningún archivo y no se escribe: no se
  adivina ni se reconstruye a mano.
- Tipos desconocidos se ignoran con aviso si el checksum es válido
  (compatibilidad hacia delante).

### 7.3 `bloqueo`

```json
{"pid": 4312, "host": "portatil", "inicio": "2026-10-20T10:15:32.400Z", "op_id": "01KDVDNHT0HM8HEE7QV4YSR5S4"}
```

El bloqueo efectivo es un **lock del sistema operativo** sobre este archivo
(`File::try_lock` de Rust: `flock` en Unix, `LockFileEx` en Windows). Se toma al
iniciar la transacción y se libera al terminarla. El contenido es informativo
para diagnóstico; no decide nada.

- **Sin vencimiento ni reclamación por tiempo.** Si el proceso muere, el sistema
  operativo libera el lock; nadie puede "heredar" una transacción a medias.
- Si ya está tomado, la operación espera hasta un límite configurable y falla con
  `proyecto ocupado`. Nunca hay dos escritores.
- En sistemas de archivos de red (SMB, NFS) los locks no son fiables: **no están
  soportados para escritura en v1**. Al abrir se avisa y `verify` lo registra; se
  puede leer y exportar. El alcance exacto del CAS está en §7.5.
- Requiere Rust 1.89 o superior (MSRV del proyecto).

### 7.4 Cuarentena

- Copia (no movimiento destructivo) a
  `.escritor/cuarentena/<AAAAMMDDTHHMMSSZ>-<slug>-<motivo>.md`.
- Una línea por caso en `.escritor/cuarentena/informe.jsonl`:
  `{"ts": "...", "archivo": "...", "ruta_original": "...", "motivo": "...", "hash": "blake3:..."}`.
  Las líneas antiguas sin `archivo` se listan con metadatos desconocidos.
- El archivo original **no se modifica**. `verify` lo sigue listando.

### 7.5 Alcance del CAS y concurrencia externa

El «compare-and-swap» de §9 es **optimista, no atómico**: se lee el archivo, se
compara su hash y después se reemplaza. No existe una primitiva portátil que
compare y reemplace contenido en una sola operación, así que entre la
comprobación y el reemplazo hay una ventana en la que otro proceso puede escribir.

Qué garantiza v1:

- **Procesos cooperativos** (CLI, GUI y MCP, todos con el lock del SO) en sistemas
  de archivos locales con locks fiables (NTFS, APFS, ext4, XFS): no hay dos
  escritores. El lock excluye y, una vez tomado, el hash detecta cambios previos.
- **Editores externos que ignoran el lock**: el motor detecta el cambio si ocurre
  antes de la comprobación y aborta con conflicto visible. Si ocurre en la ventana
  entre comprobación y reemplazo, el cambio externo puede perderse: en ese
  intervalo la garantía es last-writer-wins. No se promete lo contrario.
- **Sistemas de archivos de red (SMB, NFS)**: no soportados para escritura en v1
  (§7.3); solo lectura y exportación.

Ninguna de estas garantías se declara «sin pérdida» hasta que exista una prueba
que introduzca una escritura externa **dentro de la ventana** (gancho de
inyección `WRITER_FAULT=before_replace`). Esa prueba pertenece al gate de la
Fase 1 y debe fijar el resultado observable: qué se conserva, qué se pierde y qué
se informa. Si la pérdida no se puede evitar, se documenta como limitación
conocida en lugar de prometer exclusión total.

## 8. Índice SQLite (derivado)

- WAL, `synchronous=FULL`, `foreign_keys=ON`, `user_version` = versión de esquema.
- Tablas conceptuales:
  - `meta(clave, valor)`: versión de esquema del índice, revisión actual, hash del
    último `COMMIT` aplicado.
  - `escenas(id, ruta, hash, mtime_ns, tamano, titulo, estado, pov, orden, fecha_narrativa, palabras)`
  - `escena_tramas(escena_id, trama_id)`, `escena_personajes(escena_id, personaje_id)`
  - `notas(id, ruta, tipo, titulo, fija, hash, mtime_ns, tamano)`
  - `notas_refs(origen_id, destino_id)`, `etiquetas(objeto_id, etiqueta)`
  - `historial(rev, op_id, task_id, autor, scope, ts)` — caché del log.
  - `fts_escenas(titulo, cuerpo)`, `fts_notas(titulo, cuerpo)` — FTS5.
- **Nunca gana al archivo.** Al leer se comprueba `hash` + `mtime_ns` + `tamano`;
  si no cuadran, se reindexa ese archivo y se devuelve el contenido real.
- Si el índice no se puede abrir o su `user_version` no se entiende: se renombra
  a `indice.sqlite.corrupto-<ts>` y se reconstruye.
- `reindex` completo debe producir el mismo resultado que la reconstrucción desde
  cero (propiedad verificada en tests).

## 9. Protocolo de commit (resumen normativo)

Detalle completo en [`protocolo-commit.md`](protocolo-commit.md) (Fase 1).

1. Guardar los blobs del estado anterior (`hash_anterior`) y del nuevo:
   temporal → `fsync` → `rename`. Los anteriores son obligatorios: sin ellos no
   se puede revertir.
2. `PREPARE` al log → `fsync`.
3. Efectos por archivo: temporal en el mismo directorio → `fsync` → reemplazo
   atómico (`ReplaceFile`/`MoveFileExW` en Windows, `rename` + `fsync` del
   directorio en POSIX). Antes de cada reemplazo se comprueba que el hash actual
   sigue siendo `hash_anterior`. Es un CAS **optimista** con una ventana entre la
   comprobación y el reemplazo; su alcance exacto está en §7.5.
4. `COMMIT` → `fsync`. **Aquí, y solo aquí, está "guardado".** Si la comprobación
   previa falla o la operación se detiene, se revierten los efectos ya aplicados
   que sigan coincidiendo con su `hash_nuevo`; los archivos cambiados por fuera se
   dejan intactos. Solo cuando la reversión terminó y está persistida se anexa
   `ABORT` (§7.2).
5. Índice en una transacción. Si falla, se reindexa sin tocar archivos.

Al reiniciar, un `PREPARE` sin `COMMIT` y sin `ABORT` se **revierte** y nunca se
completa. Un `ABORT` válido es terminal: solo se escribió tras persistir la
reversión y no se reprocesa. La reversión es idempotente: si se corta a su vez,
se retoma al siguiente arranque sin perder cambios confirmados ni pisar cambios
externos.

## 10. Invariantes verificables (`writer verify`)

Errores (bloquean la operación, pueden llevar a cuarentena):

1. `proyecto.yml` ilegible, fuera de subconjunto, con `esquema` ≠ 1 o con tipos
   incorrectos (`titulo` no textual, `guardar_prompts` no booleano, etc.).
2. Escena sin `id`, con `id` mal formado o duplicado; nota igual.
3. Frontmatter sin cerrar o con YAML fuera de subconjunto: comillas simples,
   comillas interiores sin escapar, texto tras el cierre o CR aislado.
4. `tipo` de nota distinto de su carpeta.
5. `pov`, `tramas`, `personajes` o `refs` que apuntan a IDs inexistentes o de
   tipo incorrecto. El `id` del proyecto existe, pero no es personaje ni trama.
6. `orden` ausente, no entero o repetido dentro de la carpeta.
7. Archivo que no está en `manuscrito/` ni `notas/` pero pretende serlo
   (frontmatter con `id` en otra ruta).
8. `titulo` ausente, vacío o no textual en escena, nota o proyecto.

Avisos (no bloquean, se informan):

9. `estado` o `tipo` fuera del enum correspondiente.
10. `palabras` escrito a mano.
11. Campos desconocidos (se conservan).
12. CRLF, BOM o espacios al final de línea.
13. Blobs faltantes para revisiones pasadas.
14. `id` del proyecto repetido en una escena o nota.

`fixtures/proyecto-roto/` contiene un caso deliberado por cada error de esta
lista. `fixtures/proyecto-roto/esperados.txt` fija los hallazgos exactos y el CI
falla si `verify` deja de detectar uno o inventa otro. No se regenera ni se
"arregla": es la prueba negativa del contrato.

## 11. Exportación Markdown

- Destino explícito; nunca sobrescribe el proyecto ni un archivo existente sin
  `--forzar`.
- Sin frontmatter. Títulos de libro/capítulo desde las carpetas (`01-libro` →
  `Libro`), escenas separadas por `***` en línea propia, orden de lectura de §3.1.
  La implementación de la Fase 1 emite `# Libro`, `## Capítulo` y `### Título de
  la escena`, con el cuerpo limpio debajo.
- Notas excluidas por defecto; `--notas` las añade al final, agrupadas por tipo.
- El original nunca se modifica: la exportación solo lee.

## 12. Compatibilidad

- `esquema` mayor distinto: no se escribe, se avisa.
- Campos nuevos desconocidos: se conservan al reescribir.
- Cambios incompatibles: nueva versión de esquema + migrador explícito con copia
  previa. Nunca migración silenciosa.

## 13. Decisiones de esta versión

- **Una escena = un archivo.** Escenas largas siguen siendo un archivo.
- **Una novela por proyecto.** Varias novelas = varios proyectos.
- **Sin CRDT ni sync propio en v1.**
- **`palabras` no se persiste.** Índice derivado; el archivo queda limpio.
- **YAML plano.** Sin anidamiento, para que el ida y vuelta sea verificable.

## 14. Pruebas del esquema

- `python scripts/generar-fixtures.py --autoprueba` ejecuta los casos negativos
  (tipos, comillas, comentarios, CR aislado, ULID corto, referencias cruzadas,
  orden duplicado) y los positivos y de tolerancia (nota → escena, `#` entre
  comillas, línea en blanco, BOM, CRLF). Si un caso deja de fallar —o de pasar—,
  la suite falla.
- `python scripts/generar-fixtures.py --verificar-docs docs/esquema-v1.md`
  comprueba que todos los ULID de ejemplo son válidos y que los bloques YAML,
  Markdown y JSON parsean con estas mismas reglas.
- Fase 1: la escritura externa inyectada entre la comprobación y el reemplazo
  (`WRITER_FAULT=before_replace`) está cubierta por las pruebas de CLI. También
  se prueba `before_check`, que fuerza un conflicto visible. Sin esas pruebas no
  se declara ausencia de pérdida frente a editores externos (§7.5).
- Fase 1: las pruebas de secuencias cubren `ABORT` terminal, checksum corrupto en
  una línea completa, blob corrupto en el preflight, log sin salto final,
  reversión que repara referencias e índice con una página corrupta.
- Las suites de Python y las pruebas Rust se ejecutan en CI antes de verificar
  los fixtures.
