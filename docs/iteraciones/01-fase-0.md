# Iteración 01 — Fase 0 (Preparación)

- **Fecha:** 2026-10-20
- **Alcance:** Fase 0 del plan (`Plan - Software de escritura personalizado.md`).
- **Commit base:** `db4c71c` — *Fase 0: workspace, esquema v1 y fixtures*.
- **Estado:** entregables completos y corregidos; gate verificado en local
  (Windows, Rust 1.99.1). **Pendiente:** ejecutar el CI real (Linux/macOS) y el
  job de MSRV 1.89.
- **Corrección 01.1:** aplicada en el árbol de trabajo, **sin commit** (decisión
  pendiente del autor).

## 1. Resumen ejecutivo

La Fase 0 dejó preparado el proyecto para escribir el motor sin improvisar: un
workspace de Rust con dos crates, un esquema en disco normativo, un generador de
fixtures y un verificador independiente en Python que actúa como implementación
de referencia.

La revisión previa a la Fase 1 encontró cuatro defectos reales: el verificador
aceptaba datos inválidos, el bloqueo por vencimiento permitía dos escritores, la
recuperación prometía dos resultados incompatibles y los ejemplos del esquema no
coincidían con sus propias reglas. La iteración 01.1 los corrigió y añadió pruebas
que impiden su reaparición.

| Métrica | Valor |
|---|---|
| Crates | 2 (`writer-core`, `writer-cli`) |
| Tests de Rust | 5 (3 de contrato de CLI + 2 de core) |
| Fixture versionado | 3 escenas · 18 notas · 21 ids |
| Fixture de rendimiento | 1.000 escenas · 23 notas · 1.023 ids |
| Casos negativos del verificador | 11 + 2 tolerancias (BOM, CRLF) |
| Comprobaciones de documentación | ULID y bloques YAML/Markdown/JSON |
| Determinismo | 22 archivos idénticos byte a byte |

## 2. Alcance y criterio de salida

Del plan, la Fase 0 debía entregar:

- [x] Repositorio, workspace de Cargo, licencia y README.
- [x] CI mínimo: `fmt`, `clippy`, `test`.
- [x] `docs/esquema-v1.md` con `proyecto.yml`, frontmatter, log y `.escritor/`.
- [x] Fixtures base: proyecto pequeño y proyecto de 1.000 escenas.
- [x] Corrección 01.1 antes de empezar la Fase 1.

**Gate:** `cargo test` verde en CI en Windows y Linux. **Cumplido en local; el CI
real sigue pendiente** porque el repositorio aún no tiene remoto.

## 3. Entregables

### 3.1 Repositorio y workspace

- Repositorio Git con licencia MIT, README, `.editorconfig`, `.gitattributes`,
  `rustfmt.toml` y `rust-toolchain.toml` (stable + `rustfmt`/`clippy`).
- Workspace de Cargo con `resolver = "2"` y lints de workspace:
  `unsafe_code = forbid`, `missing_debug_implementations = warn`,
  `clippy::all`, `dbg_macro`, `todo`, `unwrap_used`.
- Dos crates: `writer-core` (motor, aún esqueleto) y `writer-cli` (binario
  `writer`).

### 3.2 CLI

- `writer --version` → `writer 0.1.0 (esquema 1)`.
- `writer --help` → ayuda con los comandos previstos.
- Comando desconocido → código de salida 2 y mensaje en `stderr`.
- Contrato cubierto por `crates/cli/tests/cli.rs` (3 tests sobre el binario real).

### 3.3 CI

`.github/workflows/ci.yml`:

- Matriz `windows-latest`, `ubuntu-latest`, `macos-latest`.
- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- `cargo test --workspace --all-targets`.
- Generación y verificación de fixtures (pequeño y 1.000 escenas).
- **Añadido en 01.1:** `--autoprueba`, `--verificar-docs` y job `msrv` con Rust
  1.89.

### 3.4 Esquema en disco

`docs/esquema-v1.md` (450 líneas) es normativo y cubre:

- Principios: archivos como verdad, índice derivado, IDs estables, YAML mínimo.
- Estructura: `manuscrito/libro/capitulo/escena.md` y `notas/<tipo>/`.
- `proyecto.yml`: campos, tipos, enums de política y retención.
- Escena y nota: frontmatter, campos del motor y reglas de referencia.
- Subconjunto YAML admitido.
- Convenciones de texto: UTF-8 sin BOM, LF, ULID, hashes, fechas.
- `.escritor/`: blobs, `operaciones.log`, bloqueo, cuarentena.
- Índice SQLite/FTS5 y su carácter reconstruible.
- Protocolo de commit y recuperación.
- Invariantes de `verify` y exportación Markdown.
- Compatibilidad y decisiones de la versión.

### 3.5 Verificador de referencia

`scripts/generar-fixtures.py` (1.217 líneas, sin dependencias externas):

- Genera proyectos deterministas con ULID reproducibles.
- Implementa el subconjunto YAML y los invariantes del esquema.
- Informa errores y avisos con ruta y línea.
- **01.1:** `--autoprueba`, `--verificar-docs` y validación estricta de tipos,
  comillas, referencias y finales de línea.

### 3.6 Fixtures

| Carpeta | Versionada | Contenido |
|---|---|---|
| `fixtures/proyecto-pequeno/` | sí | 3 escenas, 18 notas, semilla 1; golden test |
| `fixtures/proyecto-1000/` | no | 1.000 escenas, 23 notas; se regenera |

El fixture pequeño se regenera byte a byte con la misma semilla. Los fixtures no
incluyen `.escritor/`: son proyectos limpios.

### 3.7 Toolchain

- Visual Studio Build Tools 2022 (17.14.42, workload C++).
- Rust stable 1.99.1 (`x86_64-pc-windows-msvc`) con `rustfmt` y `clippy`.
- **01.1:** MSRV declarada **1.89** por `File::try_lock`.

## 4. Estructura del repositorio

```text
writer/
  Cargo.toml              # workspace + MSRV 1.89
  crates/
    core/                 # writer-core (motor)
    cli/                  # writer-cli (binario `writer`)
  docs/
    esquema-v1.md         # normativo
    iteraciones/
      01-fase-0.md        # este informe
  fixtures/
    proyecto-pequeno/     # versionado
    proyecto-1000/        # generado, ignorado
  scripts/
    generar-fixtures.py   # generador + verificador de referencia
  .github/workflows/ci.yml
```

## 5. Decisiones normativas de la Fase 0

| # | Decisión | Consecuencia |
|---|---|---|
| 1 | **YAML plano, sin anidamiento** | Parser pequeño, ida y vuelta verificable; políticas en claves planas. |
| 2 | **`palabras` no se persiste** | El índice lo calcula; el archivo queda limpio y `verify` avisa si aparece. |
| 3 | **`orden` único por carpeta de capítulo** | Huecos de 10 para insertar sin renumerar; el orden global usa prefijos de carpeta + `orden`. |
| 4 | **Log `<json-canónico>\t<blake3>`** | Checksum fuera del JSON: sin circularidad y detecta líneas truncadas. |
| 5 | **Blobs con bytes completos** | Revertir es restaurar bytes, no reconstruir texto. |
| 6 | **Verificador en Python** | Independiente del motor; segunda opinión y especificación ejecutable. |
| 7 | **Una escena = un archivo; una novela por proyecto** | Sin CRDT ni sync propio en v1. |
| 8 | **Recuperación por roll-back** | `COMMIT` es el único punto de confirmación (01.1). |

## 6. Qué comprueba el verificador y qué no

**Comprueba:** estructura de carpetas, UTF-8/BOM/CRLF/CR, frontmatter y
subconjunto YAML, ULID y duplicados, tipos de campos, referencias por tipo,
`orden` por carpeta, `tipo` de nota, campos desconocidos, `palabras` manual,
cuarentena estructural y coherencia de los ejemplos del esquema.

**No comprueba:** el log real, los blobs, el índice SQLite ni la recuperación;
eso pertenece al motor de la Fase 1. Tampoco sustituye los tests de Rust: es la
especificación ejecutable que Rust deberá reproducir.

**Regla de conflicto:** si el documento y el script discrepan, gana el documento
y se corrige el script.

## 7. Iteración 01.1 — Corrección antes de la Fase 1

### 7.1 Hallazgos

Se reprodujeron con mutaciones en copias temporales; los fixtures no se tocaron.

| Caso | Antes | Ahora |
|---|---|---|
| `pov` apunta al id del proyecto | 0 errores | error: no es personaje (`proyecto`) |
| `titulo: true` | 0 errores | error: debe ser texto, no `bool` |
| `guardar_prompts: "false"` | 0 errores | error: debe ser booleano |
| Comillas interiores sin escapar | 0 errores | error: comilla interior sin escapar |
| Texto con comillas simples | 0 errores | error: comillas simples no admitidas |
| CR aislado | 0 errores | error: se exige LF |
| ULID de 24 caracteres | error | error (sin cambios) |

**Causas:**

1. Las referencias al proyecto se saltaban con `continue` antes de comprobar el
   tipo de destino.
2. El parser aceptaba cualquier texto entre la primera y la última comilla, sin
   revisar comillas interiores ni texto posterior.
3. Faltaban validaciones de tipo en `titulo`, `guardar_prompts` y campos de
   proyecto.
4. Los finales de línea se normalizaban antes de comprobar, así que el CR suelto
   desaparecía sin aviso.
5. Los ejemplos del esquema usaban ULID de 24 caracteres y mezclaban reglas de
   comillas con la práctica de los fixtures.

### 7.2 Cambios por archivo

| Archivo | Cambio |
|---|---|
| `scripts/generar-fixtures.py` | Parser estricto; tipos; referencias por tipo; BOM/CRLF/CR; `--autoprueba`; `--verificar-docs`; rutas relativas en errores |
| `docs/esquema-v1.md` | ULID válidos y distintos; hashes completos; reglas YAML reales; `ABORT`; roll-back; lock por SO; invariantes y §14 de pruebas |
| `Cargo.toml` | `rust-version = "1.89"` |
| `.github/workflows/ci.yml` | Autopruebas, verificación de docs y job `msrv` |
| `.gitignore` | `__pycache__/` |
| `docs/iteraciones/01-fase-0.md` | Este informe |

### 7.3 Regla única de recuperación

Antes había dos resultados incompatibles: completar la operación si existían los
blobs, o restaurar el estado anterior. Se fijó una sola regla:

1. **`COMMIT` es el único punto de confirmación.** Un `PREPARE` sin `COMMIT` no
   está confirmado y **siempre se revierte**; nunca se completa.
2. Los blobs del estado anterior se guardan **antes** de aplicar efectos, de modo
   que restaurar no depende de leer el archivo actual.
3. El roll-back restaura `hash_anterior` solo si el archivo sigue coincidiendo con
   `hash_nuevo`; si cambió por fuera, se deja intacto y se registra conflicto. Si
   `hash_anterior` es `null`, se elimina el archivo creado.
4. Un `ABORT` cierra la operación. La reversión es idempotente: si se corta, se
   retoma al siguiente arranque.
5. Si falta un blob necesario, el proyecto abre en **modo recuperación** y no
   escribe.
6. Si un CAS falla a mitad, se revierte lo aplicado y se respeta el cambio
   externo: nunca queda una operación parcial.

Consecuencia en los criterios de aceptación: el crash multiarchivo deja **las
tres escenas o ninguna**, y un cambio no confirmado no se hace visible.

### 7.4 Bloqueo por sistema operativo

Se descartó el lock por vencimiento: un proceso lento o suspendido podía seguir
escribiendo cuando otro ya había reclamado el lock. Ahora:

- `File::try_lock` durante la transacción (`flock`/`LockFileEx`).
- Sin vencimiento: el SO libera el lock si el proceso muere.
- Si está ocupado: espera acotada y error `proyecto ocupado`.
- El contenido de `.escritor/bloqueo` es informativo, no autoridad.
- En red, el lock puede no excluir; el CAS por hash sigue protegiendo y `verify`
  lo advierte.
- MSRV 1.89 y job `msrv` en CI.

### 7.5 Pruebas nuevas

`--autoprueba` cubre 11 casos negativos y 2 tolerancias:

1. `pov` al proyecto.
2. `pov` a una trama.
3. `titulo` booleano.
4. `titulo` vacío.
5. `guardar_prompts` de texto.
6. Comillas interiores sin escapar.
7. Comillas simples.
8. CR aislado.
9. ULID de 24 caracteres.
10. Referencia inexistente.
11. `orden` duplicado.
12. BOM → aviso, no error.
13. CRLF → aviso, no error.

Además, el fixture base debe verificar sin errores y la generación con la misma
semilla debe ser idéntica byte a byte.

## 8. Evidencia de validación

```text
$ cargo fmt --all --check
(sin salida)                                              # OK

$ cargo clippy --workspace --all-targets --all-features   # RUSTFLAGS=-D warnings
Finished `dev` profile                                    # 0 avisos, 0 errores

$ cargo test --workspace --all-targets
3 passed (contrato de CLI) · 2 passed (core)              # 5/5

$ python scripts/generar-fixtures.py --autoprueba
Autopruebas: OK (casos negativos y de tolerancia)

$ python scripts/generar-fixtures.py --verificar-docs docs/esquema-v1.md
Documentación: docs/esquema-v1.md · OK

$ python scripts/generar-fixtures.py --verificar fixtures/proyecto-pequeno
escenas: 3 · notas: 18 · ids: 21
resultado: OK

$ python scripts/generar-fixtures.py --verificar fixtures/proyecto-1000
escenas: 1000 · notas: 23 · ids: 1023
resultado: OK
```

Determinismo: dos generaciones consecutivas de `proyecto-pequeno` con semilla 1
producen 22 archivos idénticos byte a byte (SHA-256 comparado).

## 9. Archivos tocados en 01.1

| Archivo | Estado |
|---|---|
| `scripts/generar-fixtures.py` | modificado |
| `docs/esquema-v1.md` | modificado |
| `Cargo.toml` | modificado |
| `.github/workflows/ci.yml` | modificado |
| `.gitignore` | modificado |
| `docs/iteraciones/01-fase-0.md` | reescrito |

En el vault, la definición y el plan se actualizaron con la regla de roll-back,
el lock por SO y el criterio 15.

## 10. Deuda y pendientes

- **Commit:** la corrección 01.1 está en el árbol de trabajo, sin commit.
- **CI real:** no se ha ejecutado; falta remoto en GitHub (u otro).
- **MSRV:** el job de Rust 1.89 no se ha ejecutado; en local hay 1.99.1.
- **Identidad de Git:** marcador local `writer <writer@localhost>`; cambiar antes
  de publicar.
- **MEGA:** excluir `writer/` o al menos `writer/target/` de la sincronización.
- **`docs/protocolo-commit.md`:** se escribirá al inicio de la Fase 1; el esquema
  lo resume en §9.
- **Paridad de pruebas:** los 11 casos negativos deben pasar a tests de
  `writer-core` cuando exista.
- **`fixtures/proyecto-roto/`:** pendiente para probar `writer verify`.
- **Lock en Rust:** especificado, no implementado (Fase 1).

## 11. Riesgos conocidos

| Riesgo | Estado |
|---|---|
| Diferencias de `fsync`/reemplazo entre Windows y POSIX | Se abordará con un módulo de escritura atómica y tests por plataforma |
| Locks poco fiables en red | Mitigado por CAS; se advertirá en `verify` |
| Watcher de cambios externos | Fase 1–2 |
| Ida y vuelta Markdown ↔ Tiptap | Fase 2, con corpus y golden tests |
| Deriva entre esquema y motor | `--autoprueba` y `--verificar-docs` en CI |

## 12. Cómo reproducir

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
python scripts/generar-fixtures.py --autoprueba
python scripts/generar-fixtures.py --verificar-docs docs/esquema-v1.md
python scripts/generar-fixtures.py --verificar fixtures/proyecto-pequeno
python scripts/generar-fixtures.py --destino fixtures/proyecto-1000 --escenas 1000
python scripts/generar-fixtures.py --verificar fixtures/proyecto-1000
```

## 13. Siguiente fase

La Fase 1 empieza por el documento, no por el código:

1. `docs/protocolo-commit.md`: orden exacto, `fsync`, CAS, `ABORT`, recuperación
   y modo recuperación.
2. Modelo (`Project`, `Scene`, `Note`, `Revision`, `Operation`) con `serde` y
   tipos de ID validados.
3. Escritura atómica y blobs.
4. Op log y recuperación con inyección de fallos.
5. Lock del SO y CAS.
6. Índice SQLite/FTS5.
7. `init` y `verify` con `fixtures/proyecto-roto/`.

## Anexo A — Criterios de aceptación afectados

| Criterio | Estado tras 01.1 |
|---|---|
| 2. Crash en guardado sin cambios no confirmados visibles | Regla definida; test en Fase 1 |
| 3. Crash multiarchivo: tres o ninguna | Regla definida; test en Fase 1 |
| 4. Índice corrupto se reconstruye | Sin cambios; Fase 1 |
| 15. Lock: nunca dos escritores | Especificado y MSRV fijada; test en Fase 1 |

## Anexo B — Lo que esta fase no demuestra

- Que el motor Rust implemente el esquema: todavía no existe.
- Que la recuperación funcione: solo está especificada y probada en el
  verificador de estructura, no en persistencia real.
- Que el CI pase en Linux/macOS: no se ha ejecutado.
- Que el rendimiento con 1.000 escenas sea el definitivo: el fixture existe, la
  medición corresponde al motor futuro.

## Ampliación 01.2 — CI de seguridad y de producto

Petición: que el CI no solo valide el código, sino también **temas de
ciberseguridad** y **características propias del software**.

### Qué se añadió

- `scripts/ci.py`: orquestador único de gates, ejecutable en local y en CI.
  Grupos `calidad`, `producto` y `seguridad`.
- `deny.toml`: política de dependencias (advisories, licencias, comodines,
  orígenes). Solo crates.io; `wildcards = "deny"` obligó a versionar la
  dependencia interna `writer-cli → writer-core`.
- `fixtures/proyecto-roto/` + `esperados.txt`: 12 defectos deliberados con
  contrato exacto de hallazgos (prueba negativa de `verify`). Incluye CR aislado
  marcado con `-text` en `.gitattributes` para que Git no lo normalice.
- `--json` en el verificador y `Informe.resumen` para automatizar comparaciones.
- `.github/workflows/ci.yml` reorganizado: jobs `calidad` (3 SO), `producto`
  (Windows y Linux), `seguridad` y `msrv`; permisos mínimos, `concurrency`,
  `persist-credentials: false` y todas las acciones fijadas por SHA.
- `.github/dependabot.yml` para Cargo y GitHub Actions con cooldown de 7 días.
- `docs/ci.md`: qué cubre cada gate, paridad local/CI y qué queda para fases
  posteriores (fuzzing, mutation testing, CodeQL, SBOM).
- Corrección de un aviso espurio: cuando `proyecto.yml` no tenía un id válido,
  todas las escenas "coincidían" con el proyecto (`None == None`).

### Evidencia (local, Windows)

```text
$ python scripts/ci.py calidad
fmt OK · clippy OK (-D warnings) · 5/5 tests          # 4,9 s

$ python scripts/ci.py producto
autopruebas OK · docs OK · fixture pequeño OK
determinismo OK · 12 hallazgos esperados exactos
1.000 escenas generadas y verificadas en 13 s (presupuesto 60 s)
enlaces OK                                             # 15,3 s

$ python scripts/ci.py seguridad
advisories ok · bans ok · licenses ok · sources ok
gitleaks: no leaks (historial y árbol)
zizmor: no findings (3 suprimidos)                     # 2,8 s
```

Herramientas instaladas en la máquina para verificar el gate de seguridad:
`cargo-deny 0.20.2`, `gitleaks 8.30.1`, `zizmor 1.31.1`.

### Pendiente

- Ejecutar el workflow real en GitHub (sigue sin remoto).
- `advanced-security` de zizmor queda en `false`: activar SARIF requiere GHAS en
  repos privados. Los hallazgos ya hacen fallar el job.
- El presupuesto de rendimiento (60 s) mide el verificador, no el motor: se
  sustituirá por `criterion` en la Fase 1.

## Corrección 01.3 — revisión externa (4 puntos)

Revisión sobre `bb65166`: cuatro problemas concretos, corregidos en este commit.

1. **Alta — `ABORT` antes de terminar la reversión.** El esquema anexaba `ABORT`
   antes del roll-back; un corte entre ambos dejaba cambios parciales que el
   arranque ignoraba. Ahora `ABORT` se anexa **solo** con la reversión terminada y
   persistida, y la recuperación revierte todo `PREPARE` sin `COMMIT` **haya o no
   `ABORT`** (idempotente). Alineado en la Definición §3.2–3.3 y en el plan
   (Fase 1).
   > **Actualizado en 02.1:** un `ABORT` válido es terminal; no se reprocesa. La
   > recuperación solo revisa `PREPARE` sin `COMMIT` **y sin `ABORT`**. Un `ABORT`
   > antiguo no debe deshacer un guardado posterior con el mismo hash. Si falta el
   > `ABORT`, el `PREPARE` sigue pendiente y se reintenta.
2. **Alta — CAS no atómico.** Se retiró la promesa de que la comprobación de hash
   impide la sobrescritura incluso sin locks fiables. Nuevo §7.5 del esquema:
   alcance real (procesos cooperativos en sistemas de archivos locales), ventana
   last-writer-wins con editores externos, SMB/NFS no soportados para escritura, y
   **prueba de carrera obligatoria** (`WRITER_FAULT=before_replace`) antes de
   prometer ausencia de pérdida. Alineado en Definición §3.4 y §6.
3. **Media — notas que referencian escenas.** El mapa de destinos omitía las
   escenas, así que un `refs` válido a una escena daba «id inexistente». Ahora
   incluye todos los IDs y mantiene las restricciones de `pov`, `personajes` y
   `tramas`. Caso positivo añadido a `--autoprueba`.
4. **Media — parser contra contrato.** Se rechaza `#` en texto plano (comentario)
   y se ignoran las líneas en blanco del frontmatter, como dice §5. Casos de
   aceptación y rechazo añadidos a `--autoprueba`.

Evidencia tras la corrección:

```text
--autoprueba: OK (negativos, positivos y tolerancia)
ci.py producto: 12/12 hallazgos exactos · determinismo OK · enlaces OK
ci.py calidad: fmt, clippy y 5/5 tests OK
ci.py seguridad: deny, gitleaks y zizmor OK
```

Pendiente: CI remoto y la prueba de carrera de la Fase 1 (sin ella no se declara
«sin pérdida»).
