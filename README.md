# writer

Motor de escritura con archivos como fuente de verdad: Markdown + frontmatter para
el proyecto, índice SQLite reconstruible y un log de operaciones que permite
recuperarse de un corte sin perder ni medio cambio.

Este repositorio está en **Fase 2 (Interfaz, v0.2)** del plan. El motor y la CLI
(Fase 1) están completos; la interfaz Tauri abre proyectos, muestra el árbol,
permite leer y **editar con autosave**, busca con FTS5 y ofrece un historial con
reversión. La exportación DOCX y el empaquetado son las siguientes rebanadas.

> **Estado para retomar:** [`HANDOFF.md`](HANDOFF.md).
>
> **Gestor de paquetes: pnpm.** `npm` y `npx` están prohibidos en este
> repositorio. El workspace se instala y se ejecuta con `pnpm`.

## Estado

| Fase | Contenido | Estado |
|---|---|---|
| 0 | Workspace, CI, esquema v1, fixtures | Completada en local; falta CI remoto |
| 1 | Motor + CLI (v0.1) | **Completada en local (Linux, Rust 1.99.1; MSRV 1.89.0 verificada; corrección 02.2)** |
| 2 | Interfaz Tauri (v0.2) | **En curso: árbol, lectura/edición con autosave, conflictos, corpus Markdown, búsqueda FTS5 e historial con reversión; DOCX, cuarentena visible, marcas y empaquetado pendientes** |
| 3 | IA + MCP (v0.3) | Pendiente |
| 4 | Licencias + Pro (v0.4) | Pendiente |

La Fase 1 incluye modelo, escritura atómica, blobs, op log con checksum,
recuperación idempotente, lock del SO, CAS optimista, índice SQLite/FTS5,
exportación Markdown e inyección de fallos. El conjunto Rust acumula 70 pruebas
(29 unitarias, 18 de integración y 23 de CLI). La Fase 2 añade 2 pruebas de la
cáscara Tauri y 33 pruebas de UI (corpus Markdown, compatibilidad, autosave,
búsqueda, historial y regresiones de borrador/reversión).

## Requisitos

- Rust estable (el repo fija `stable` en `rust-toolchain.toml`).
  En Windows hace falta el toolchain MSVC y Visual Studio Build Tools con el
  workload **Desarrollo para el escritorio con C++**. El núcleo mantiene MSRV
  1.89; la cáscara Tauri (`apps/desktop`) requiere Rust 1.90 o superior.
- **pnpm 12** y Node 22 o superior para la interfaz. `npm` y `npx` no se usan.
  Instalación de pnpm sin npm: `curl -fsSL https://get.pnpm.io/install.sh | sh -`
  (o el binario oficial de pnpm).
- En Linux, las dependencias de Tauri: `webkit2gtk-4.1`, `gtk3`,
  `libappindicator-gtk3`, `librsvg` y `patchelf` (los nombres cambian según la
  distribución).
- Un compilador de C para compilar SQLite embebido (`rusqlite` con `bundled`).
  En Linux: `build-essential` o `gcc`; en macOS: Xcode Command Line Tools.
- Python 3.9+ para generar y verificar fixtures y para ejecutar los gates
  (sin dependencias).
- Git.
- Solo para el gate de seguridad en local: `cargo-deny`, `gitleaks` y `zizmor`
  (instrucciones en [`docs/ci.md`](docs/ci.md)). En CI se instalan con sus
  acciones.

## Puesta en marcha

```bash
cargo build --workspace
cargo test --workspace
cargo run --bin writer -- --help
cargo run --bin writer -- --version
```

### Primer proyecto

```bash
# Crea proyecto.yml, .escritor/ y una primera escena (revisión 1)
cargo run --bin writer -- init ~/novela --titulo "El umbral" --autor "Autora"

# Crear, leer y escribir
cargo run --bin writer -- create escena --titulo "El faro" ~/novela
cargo run --bin writer -- create nota --tipo personajes --titulo "Marta" ~/novela
cargo run --bin writer -- list ~/novela --notas
cargo run --bin writer -- read <id> ~/novela
cargo run --bin writer -- write <id> ~/novela --texto "Una luz sobre el puerto."
cargo run --bin writer -- buscar puerto ~/novela --limite 10

# Historial, revisión y exportación
cargo run --bin writer -- history ~/novela
cargo run --bin writer -- show 3 ~/novela
cargo run --bin writer -- revert 3 ~/novela
cargo run --bin writer -- export md ~/salida.md ~/novela --notas

# Verificación e índice
cargo run --bin writer -- verify ~/novela
cargo run --bin writer -- reindex ~/novela --limpiar
```

### Comandos

| Comando | Qué hace |
|---|---|
| `init` | Crea el proyecto y la primera escena. |
| `info` | Resumen de proyecto, revisión y recuentos. |
| `list` | Escenas en orden de lectura; `--notas` añade notas. |
| `buscar` | Busca texto en escenas y notas con FTS5; `--limite` acota resultados. |
| `read` | Cuerpo por ID o ruta; `--completo` incluye frontmatter. |
| `create escena` / `create nota` | Crea un documento nuevo. |
| `write` | Reemplaza el cuerpo (`--texto`, `--archivo` o `--stdin`). |
| `move` | Mueve sin cambiar el ID. |
| `delete` | Elimina; el contenido queda en el historial y `revert` lo restaura. |
| `history` / `show` | Historial confirmado y contenido de una revisión. |
| `revert` | Revierte una revisión concreta en una revisión nueva. |
| `verify` | Comprueba IDs, frontmatter, referencias, orden y estructura. |
| `reindex` | Reconstruye SQLite/FTS5; `--limpiar` recoge blobs huérfanos. |
| `export md` | Exporta el manuscrito (y notas con `--notas`). |

Todos los comandos aceptan la ruta del proyecto como argumento posicional; el
valor por defecto es el directorio actual. `--json` está disponible en `info`,
`list`, `buscar`, `read`, `history`, `show` y `verify`.

La búsqueda usa el índice FTS5 y lo reconstruye si falta o está desactualizado
por revisión. Si editas archivos por fuera del motor, ejecuta `reindex` para que
la búsqueda y el índice vuelvan a reflejar el disco.

### Interfaz Tauri (Fase 2)

```bash
pnpm install        # instala ui y apps/desktop en el workspace pnpm
pnpm dev            # Tauri + Vite
pnpm typecheck      # comprobación TypeScript
pnpm ui:test        # corpus Markdown, compatibilidad y autosave
pnpm ui:build       # build estático de la UI
pnpm check          # Tauri build --no-bundle
```

- `apps/desktop/src-tauri`: cáscara Tauri. Expone `abrir_proyecto`, `listar`,
  `leer`, `verificar`, `buscar`, `historial`, `mostrar`, `revertir` y `escribir`;
  toda la lógica vive en `writer-core`.
- `ui/`: React + Vite + Tiptap. Sin Tauri, `pnpm ui:dev` y el botón **Ver demo**
  permiten trabajar con datos de ejemplo en el navegador.
- Búsqueda FTS5 en la barra lateral (`Ctrl+F`), con fragmento resaltado y salto
  al documento.
- Pestaña **Historial**: línea de tiempo por revisión, vista del contenido de
  cada archivo y reversión con confirmación; la reversión crea una revisión
  nueva y no borra el historial.
- Autosave a los 900 ms y con `Ctrl+S` / `Cmd+S`; estados `guardado`,
  `cambios sin guardar`, `guardando`, `conflicto`, `recuperación`, `ocupado`,
  `error` y `solo lectura`.
- El guardado envía el hash del archivo leído; `writer-core` lo compara dentro de
  la sesión de escritura y devuelve conflicto si cambió por fuera.
- Los documentos con HTML, imágenes, tablas, tareas o notas al pie se abren en
  solo lectura para no perder contenido. Contrato:
  [`docs/markdown-v1.md`](docs/markdown-v1.md).
- Los tipos TypeScript se generan desde Rust:

  ```bash
  cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml exportar_tipos_typescript
  ```

- Decisión de editor: [`docs/adr/0001-editor-tiptap.md`](docs/adr/0001-editor-tiptap.md).

### Pruebas de caída

```bash
# Corte después de PREPARE: al volver a abrir se revierte
WRITER_FAULT=prepare cargo run --bin writer -- write <id> . --texto "no confirmado"

# Corte después de aplicar un efecto
WRITER_FAULT=after_write:1 cargo run --bin writer -- write <id> . --texto "tampoco"

# Corte después de COMMIT: el cambio queda confirmado
WRITER_FAULT=commit cargo run --bin writer -- write <id> . --texto "confirmado"

# Carrera en la ventana del CAS: gana el motor (last-writer-wins)
WRITER_FAULT=before_replace WRITER_FAULT_EXTERNAL="externo" \
  cargo run --bin writer -- write <id> . --texto "motor"

# Cambio externo antes del CAS: conflicto visible, se conserva lo externo
WRITER_FAULT=before_check WRITER_FAULT_EXTERNAL="externo" \
  cargo run --bin writer -- write <id> . --texto "motor"
```

El código de salida 70 indica un corte simulado. La siguiente apertura ejecuta la
recuperación.

## Arquitectura

```text
writer/
  Cargo.toml                 # workspace Rust (núcleo + CLI)
  package.json               # workspace pnpm y scripts de interfaz
  pnpm-workspace.yaml        # ui + apps/desktop
  deny.toml                  # política de dependencias (cargo-deny)
  crates/
    core/                    # writer-core: motor
      src/
        model.rs             # Proyecto, Escena, Nota, YAML de ida y vuelta
        yaml.rs              # subconjunto YAML v1
        fsutil.rs            # escritura atómica y fsync
        blob.rs              # objetos/<blake3>
        oplog.rs             # PREPARE/COMMIT/ABORT y checksum
        recovery.rs          # roll-back idempotente
        lock.rs              # File::try_lock
        index.rs             # SQLite + FTS5
        verify.rs            # escaneo e invariantes
        project.rs           # transacciones y comandos
    cli/                     # writer-cli: binario `writer`
  apps/
    desktop/src-tauri/       # cáscara Tauri y comandos
  ui/                        # React + Vite + Tiptap
  docs/
    markdown-v1.md           # esquema del editor, corpus y pérdidas
    esquema-v1.md            # contrato de lo que hay en disco
    protocolo-commit.md      # orden, CAS, recuperación y fallos
    ci.md                    # gates de calidad, producto y seguridad
    adr/                     # decisiones de arquitectura
    iteraciones/             # informes por fase
  fixtures/
    proyecto-pequeno/        # versionado y determinista
    proyecto-roto/           # 12 defectos deliberados
```

El crate de Tauri queda fuera del workspace Rust para que `cargo test --workspace`
no dependa de WebKit/WebView2; tiene su propio `Cargo.lock` y se compila con
`pnpm check` o `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`.

## Garantías de la v0.1

- **Archivos como verdad:** el índice se puede borrar y reconstruir.
- **`COMMIT` como único punto de confirmación:** los archivos de trabajo pueden
  contener un cambio no confirmado antes del `COMMIT`; la garantía es que, al
  reabrir con el motor, un `PREPARE` sin `COMMIT` se revierte. Un `ABORT` válido
  es terminal y no se vuelve a aplicar.
- **Recuperación idempotente:** un corte durante la reversión se retoma; antes de
  tocar archivos se verifica la integridad de todos los blobs necesarios. Si un
  efecto falla después de reemplazar o borrar (por ejemplo, en el `fsync` del
  directorio), también entra en el rollback.
- **`ABORT` solo tras reversión sincronizada:** si la reversión falla, el
  `PREPARE` queda pendiente y se reintenta al reabrir; no se cierra en falso.
- **Conflictos visibles:** si un archivo cambió por fuera, no se pisa; se informa
  y `verify` muestra los `ABORT` con conflicto.
- **Lock del SO:** dos procesos cooperativos no escriben a la vez.
- **CAS optimista:** se detecta un cambio externo anterior al reemplazo.
- **Corrupción del log:** una línea completa con checksum inválido detiene la
  recuperación y conserva los archivos; solo se descarta una línea final
  incompleta.
- **Índice reconstruible:** una corrupción detectada al reconstruir se aparta a
  `indice.sqlite.corrupto-<ts>` y se genera uno nuevo.
- **Sin pérdida de campos desconocidos:** se conservan al reescribir.

Limitaciones conocidas (documentadas en [`docs/protocolo-commit.md`](docs/protocolo-commit.md)):

- La ventana entre la comprobación del hash y el reemplazo no es atómica. Un
  editor externo que escriba justo ahí puede perder su cambio
  (*last-writer-wins*). Está cubierto por la prueba `before_replace`.
- SMB/NFS no están soportados para escritura.
- `write` cambia el cuerpo; la edición completa del frontmatter desde la CLI aún
  no existe (los campos desconocidos se conservan al reescribir).
- El historial no se compacta en v1.
- La búsqueda FTS5 usa el índice; si editas archivos por fuera del motor, ejecuta
  `reindex` para que los resultados reflejen el disco.
- Sin `.escritor`, la búsqueda reconstruye un FTS5 en memoria en cada consulta;
  es correcto para fixtures y proyectos de solo lectura.
- Una reversión falla con conflicto si alguno de los archivos afectados cambió
  después de esa revisión.

## Pruebas y gates

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --locked

pnpm install --frozen-lockfile
pnpm typecheck
pnpm ui:test
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml --locked

# Todos los gates que también corre el CI
python scripts/ci.py todo

# Por grupos (el de seguridad necesita cargo-deny, gitleaks y zizmor)
python scripts/ci.py calidad
python scripts/ci.py interfaz
python scripts/ci.py producto
python scripts/ci.py seguridad
```

La suite cubre, entre otras cosas:

- ida y vuelta de YAML y del modelo;
- los 12 hallazgos del fixture roto;
- fixture pequeño sin modificarlo;
- transacciones, historial, movimiento, borrado y reversión;
- índice corrupto (cabecera y página) que se aparta y se reconstruye;
- cortes en `prepare`, `after_write`, `commit`, `before_replace` y
  `before_check`;
- secuencia fallo → recuperación → guardado posterior → reapertura, con `ABORT`
  terminal;
- checksum corrupto en una última línea completa: error, sin borrar la revisión;
- blob corrupto en el preflight: no se modifica ningún archivo;
- anexar a un log sin salto final sin introducir NUL;
- fallo de `fsync` después de un reemplazo y después de un borrado: rollback y
  `ABORT` solo si la reversión se sincronizó;
- reversión que repara referencias (nota borrada) y valida el estado resultante;
- lock ocupado;
- falta de blob → modo recuperación;
- escritura con hash base: un archivo cambiado por fuera produce conflicto y no
  se toca;
- guardar el cuerpo conserva el frontmatter;
- búsqueda FTS5 en CLI y UI, incluido el fallback sin `.escritor`;
- historial, detalle de revisión y reversión con confirmación;
- UI: corpus golden Markdown (16 casos), compatibilidad conservadora, autosave
  con `Ctrl+S`, conflicto y solo lectura.

### Fixtures

```bash
# Proyecto pequeño, determinista (se versiona en el repo)
python scripts/generar-fixtures.py --destino fixtures/proyecto-pequeno --escenas 3 --semilla 1

# Proyecto de rendimiento: 1.000 escenas (no se versiona)
python scripts/generar-fixtures.py --destino fixtures/proyecto-1000 --escenas 1000

# Comprobar invariantes del esquema
python scripts/generar-fixtures.py --verificar fixtures/proyecto-pequeno

# Fixture con defectos deliberados: 12 hallazgos esperados
python scripts/generar-fixtures.py --verificar fixtures/proyecto-roto --json

# Autopruebas del verificador
python scripts/generar-fixtures.py --autoprueba
```

El script Python es la implementación de referencia del subconjunto YAML y de las
comprobaciones de `docs/esquema-v1.md`. El verificador Rust reproduce los mismos
12 hallazgos; la prueba está en `crates/core/tests/proyecto.rs`.

## Decisiones ya cerradas

- Una escena = un archivo.
- Una novela por proyecto.
- `palabras` no se escribe en el archivo: lo calcula el índice.
- YAML plano (sin anidamiento) para que el ida y vuelta sea verificable.
- Movimiento = dos efectos (eliminar origen, crear destino), sin campos
  especiales en el log.
- `hash_nuevo: null` representa una eliminación.
- Comprobación optimista del hash antes de reemplazar: la carrera con editores
  externos se documenta, no se oculta.
- El autosave envía el hash del archivo leído; el motor lo comprueba otra vez
  dentro de la sesión de escritura.
- El editor bloquea las construcciones Markdown que perderían contenido
  (HTML, imágenes, tablas, tareas y notas al pie).
- CLI-first: ninguna función de la GUI existe antes en la CLI.

## CI

Tres familias de gates, ejecutables también en local con `python scripts/ci.py`:

| Grupo | Qué comprueba |
|---|---|
| `calidad` | `fmt`, `clippy -D warnings`, `cargo test --locked` en Windows, Linux y macOS |
| `interfaz` | `pnpm install --frozen-lockfile`, tipos TS, corpus/autosave, build de Vite y `cargo check` de la cáscara Tauri |
| `producto` | esquema, autopruebas, determinismo de fixtures, 12 hallazgos esperados, presupuesto de 1.000 escenas, enlaces y CLI Rust |
| `seguridad` | `cargo-deny` (advisories, licencias, bans, orígenes), `gitleaks` (historial y árbol) y `zizmor` (workflows) |

Además: acciones fijadas por SHA, permisos mínimos, Dependabot con cooldown y job
de MSRV. Detalle en [`docs/ci.md`](docs/ci.md).

## Nota sobre MEGA

El repositorio vive dentro de una carpeta sincronizada por MEGA. Para no sincronizar
`target/` (miles de archivos, cambios constantes) conviene excluir esa carpeta en
la app de MEGA o mover el repo fuera de la sincronización. `.gitignore` ya excluye
`target/`.

## Licencia

MIT. Ver [`LICENSE`](LICENSE).
