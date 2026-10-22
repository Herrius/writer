# Fase 2.1 — Security review (r2)

- **Commit revisado:** `924d48568e17ca1d2a77caa4a32f7742965b699f`
- **HEAD del pipeline:** `efa859e0ce007a4f97f73a6765fc2fe99c4e3152`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64; Rust 1.99.1; clippy 0.1.99; `cargo-deny` 0.20.2; `zizmor` 1.31.1; gitleaks local (sin número de versión reportado); pnpm 12.14.0; Node 26.11.1; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). `git diff --stat 924d485..HEAD` solo toca `docs/revisiones/README.md` y `docs/revisiones/fase-2-1-11-congelado-seguridad-r2.md`; no hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD. Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **aprobado con observaciones** — P1: 0 · P2: 0 · P3 nuevos: 2 (SEC21-01-R, SEC21-07) · P3 residuales abiertos: 2 (SEC21-02, SEC21-03) · P4 residuales: 3. SEC21-01 (P2) **cerrado**. El gate `calidad` no está verde por clippy; debe corregirse antes de release.

## 1. Verificación de SEC21-01

Reproduje el escenario del informe r1 y las variantes de la corrección con el binario `target/debug/writer` recompilado desde el commit, una sonda Rust fuera del árbol para la ruta vacía y centinelas en `/tmp/opencode/sec21r2*`. No se tocó el repositorio.

### 1.1 Casos que ahora rechazan correctamente

| Caso | Resultado |
|---|---|
| `Project::new("").init(...)` en cwd con `proyecto.yml` centinela | **OK**: rc 1, `la ruta del proyecto no puede estar vacía`; el manifiesto centinela **no cambia** y no se crea nada. |
| `writer init .` en cwd **no vacío** con `proyecto.yml` centinela | **OK**: rc 1, `el directorio no está vacío`; manifiesto centinela intacto. |
| `writer init ..` | **OK**: rc 1, `la ruta no nombra una carpeta de proyecto`. |
| `writer init /` | **OK**: rc 1, `la ruta no nombra una carpeta de proyecto`. |
| `writer init 'a/..'` con `a` existente o inexistente | **OK**: rc 1, `la ruta no nombra una carpeta de proyecto`. |
| `writer init 'a/../b'` con `a` inexistente | **OK**: rc 1, `la carpeta padre no existe o no es un directorio`. |
| `writer init 'a/../b'` con `a` existente y `b` **no vacío** | **OK**: rc 1, `el directorio no está vacío: .../b`; el `proyecto.yml` centinela de `b` conserva bytes y hash. |
| `writer init 'a/../b'` con `a` existente y `b` vacío | **OK**: crea el proyecto en `b` (cwd/b); no sobrescribe nada existente. |
| `writer init '../outside/b'` con `b` **no vacío** | **OK**: rc 1, `el directorio no está vacío`; el centinela de `../outside/b` queda intacto. |
| `writer init 'a/../../outside/b'` con `b` **no vacío** | **OK**: rc 1, `el directorio no está vacío`; centinela intacto. |
| Destino existente no vacío (archivo cualquiera) | **OK**: rc 1, no escribe; el archivo centinela no cambia. |
| Destino existente vacío | **OK**: crea el proyecto. |
| Destino archivo, FIFO o socket | **OK**: rc 1, `el destino del proyecto no es un directorio`; sin cuelgue y sin escritura. |
| Destino symlink final (a directorio vacío, lleno o colgante) | **OK**: rc 1, `el destino del proyecto no puede ser un enlace`; el destino real no cambia. |
| Padre inexistente, archivo, FIFO o socket | **OK**: rc 1, `la carpeta padre no existe o no es un directorio`. |
| Padre de solo lectura / destino de solo lectura | **OK**: rc 1, `Permission denied`; no se crea ni se modifica nada. |
| Nombres `CON`, `AUX`, `NUL`, `COM1`, `LPT9`, `punto.`, `punto ` (espacio final), `C:\tmp\win`, `a\b` | **OK**: rc 1, `nombre de proyecto no válido`. |
| Nombre con carácter de control | **OK**: rc 1, nombre no válido; no se crea el destino. |
| `a/../b` con `b` no vacío en una copia del escenario r1 | **OK**: el `proyecto.yml` de `b` no se sobrescribe; el P2 de la r1 no se reproduce. |

El P2 original —sobrescribir `proyecto.yml` o `.escritor/operaciones.log` de un proyecto existente con `a/..`, `a/../b` o ruta vacía— **está cerrado**: la corrección comprueba el destino canónico, rechaza destinos no vacíos y enlaces finales, y valida el nombre antes de crear. Los centinelas de manifiesto y log permanecieron intactos en todos los casos con datos preexistentes.

### 1.2 Casos aceptados que quedan como residual

| Caso | Resultado |
|---|---|
| `writer init .` en un cwd **vacío** | **Aceptado**: rc 0, crea el proyecto en el cwd. La documentación dice que `.` se rechaza; el código lo normaliza al directorio actual y solo lo rechaza si no está vacío. No sobrescribe datos. |
| `writer init '../outside/b'` con `b` **vacío o inexistente** | **Aceptado**: crea el proyecto **fuera del cwd**. No sobrescribe datos existentes, pero acepta un `..` relativo. |
| `writer init 'a/../../outside/b'` con `b` vacío | **Aceptado**: resuelve el padre canónico y crea fuera del cwd. No sobrescribe datos existentes. |
| `writer init 'link-ancestro/hijo'` con `link-ancestro` symlink a un directorio real | **Aceptado**: escribe en el destino del symlink. El destino final no es un enlace y el hijo no existía; no sobrescribe. |
| `writer init` absoluto en un padre arbitrario existente (por ejemplo, otro directorio vacío del usuario) | **Aceptado**: el backend valida que el padre exista, pero no que sea el padre elegido en el diálogo de la UI. No sobrescribe destinos no vacíos. |
| Nombres `CONIN$`, `CONOUT$`, `COM¹`, `LPT0`, `CLOCK$` | **Aceptados** en Linux; son formas reservadas o especiales en Windows que `validar_ruta_relativa` no cubre. No hay ejecución Windows. |
| Nombre con espacio inicial, bidi o longitud >255 | El backend no rechaza el espacio inicial ni el bidi; el límite de longitud lo impone el sistema de archivos. P4. |

Estos residuales no reproducen el P2: no se sobrescribe un `proyecto.yml`, un `.escritor` ni un archivo externo existente. Pero la ruta relativa con `..` y el padre absoluto arbitrario siguen permitiendo **crear** un proyecto fuera del cwd o del padre que el usuario eligió en la UI. La documentación dice «rechaza `.`/`..`»; el código solo rechaza el componente final y valida el padre resuelto.

### 1.3 Implementación observada

- `preparar_raiz_inicial` rechaza ruta vacía, resuelve la ruta contra `current_dir` si es relativa, exige `file_name`, valida el nombre con `validar_ruta_relativa`, exige que el padre sea un directorio, lo canonicaliza y construye `padre_real.join(nombre)`.
- Si el destino no existe, lo crea con `create_dir` (exclusivo); si existe, rechaza symlink, no-directorio y directorio no vacío; después canonicaliza y comprueba `real.parent() == padre_real`.
- `Project::init` valida el manifiesto **después** de `preparar_raiz_inicial` (por eso un título inválido deja el directorio creado), escribe `proyecto.yml`, crea dirs, llama a `validar_escritor`, y solo después `Log::crear_vacio`.
- `Log::crear_vacio` exige `directorio_real(padre)` y rechaza un log symlink o no regular; un log hardlink se acepta y la escritura atómica reemplaza la entrada, sin tocar el otro enlace.
- `crear_proyecto` sigue pasando `ruta` sin canonicalizar a `init`; las notas y `abrir_proyecto` no tienen rollback.

## 2. Residuales y superficie

### SEC21-02 — [P3, abierto] TOCTOU de `.escritor` en `Project::init`

La corrección añade `validar_escritor` antes de `Log::crear_vacio` y `directorio_real` dentro de `crear_vacio`, pero la ventana sigue existiendo. Reproduje la carrera:

```text
watcher: espera p/manuscrito; rm -rf p/.escritor; ln -s ext p/.escritor
writer init p --titulo X
```

Resultado en 10/300 intentos: `ext/operaciones.log` (centinela de 20 bytes) fue reemplazado por un log de 616 bytes y `ext/objetos/<hash>` fue creado; `writer init` terminó con rc 0. El `validar_escritor`/`directorio_real` no impide que el symlink se inserte después de la comprobación y antes de `escribir_atomico`. Es la misma clase TOCTOU que la r1; requiere un atacante local con escritura en el directorio del proyecto y una carrera. P3, no bloquea por sí solo, pero sigue siendo una escritura fuera.

Corrección pendiente: crear el proyecto en un directorio temporal y renombrarlo; usar `openat`/`O_NOFOLLOW`/`dirfd`; o comprobar el inodo de `.escritor` inmediatamente antes de cada escritura. Añadir la carrera como regresión (`#[cfg(unix)]`).

### SEC21-03 — [P3, abierto] Creación parcial sin rollback

- `Project::init` valida el título **después** de `preparar_raiz_inicial`; un título vacío o con control deja un directorio vacío (reproducido: `titulo-vacio` y `titulo-control`, 0 entradas).
- Si falla un directorio, el log o la primera escena, queda manifiesto/dirs/log parciales.
- `crear_proyecto` crea las notas después de `init` y llama a `abrir_proyecto` después; si una nota falla, el proyecto permanece con las notas anteriores. Reproducido: `init` + `chmod 0555 notas` + `create nota` → rc 1, proyecto con 1 nota y `verify` OK; el comando Tauri devolvería error y el reintento fallaría por «directorio no está vacío».
- Si `abrir_proyecto` falla, el proyecto queda creado y el comando devuelve error.

No hay pérdida de datos existentes; es deuda de robustez. P3. La corrección debería validar el título antes de crear, crear en temporal + rename y limpiar lo creado si falla una nota o la apertura.

### SEC21-04 — [P4, parcialmente corregido] `localStorage` de recientes

`ui/src/recientes.ts` ahora valida `autor` string/undefined, `palabras` finito no negativo, `color` como `#rrggbb`, `ruta`/`titulo` no vacíos, deduplica por ruta y recorta a 8. Con eso quedan cerrados el crash por objetos, el CSS arbitrario y el exceso de tarjetas.

Residual P4:

- `ruta` se guarda en claro y se usa para abrir un proyecto; una entrada manipulada puede mostrar una tarjeta con título señuelo y una ruta arbitraria. Requiere acceso de escritura a `localStorage` o ejecución de script local.
- `titulo`/`autor` no tienen límite de longitud ni saneado de controles; un valor enorme o bidi puede degradar/engañar la UI. React escapa el HTML; no hay XSS.
- No hay validación de que la ruta reciente siga existiendo o sea el mismo proyecto; el usuario puede abrir otra ruta.
- Las rutas y títulos quedan en claro en el perfil del WebView.

No reproduje XSS: no hay `dangerouslySetInnerHTML`/`innerHTML`/`eval` en `ui/src`.

### SEC21-05 — [P4, parcialmente corregido] Windows, nombres y permisos

La validación del backend ahora rechaza los reservados comunes (`CON`, `AUX`, `NUL`, `COM1`…`LPT9`), puntos/espacios finales, controles, `:` y `\`. Quedan:

- `CONIN$`, `CONOUT$`, `COM¹`, `LPT0`, `CLOCK$` y nombres con espacio inicial/bidi son aceptados; no hay ejecución Windows para confirmar el efecto.
- El padre puede ser un **symlink** o **junction** existente: se canonicaliza y el proyecto se crea en el destino; no se rechaza el enlace de ancestro.
- Las rutas UNC/`\\?\`/ADS y las formas de dispositivo solo están cubiertas por la validación del nombre final, no por una política Windows completa.
- `umask 000` sigue produciendo directorios `0777` y archivos `0666`; `.escritor`, manifiesto y log quedan accesibles a otros usuarios locales. No se fijan modos `0700`/`0600`.

P4. Si se quiere paridad Windows, validar la lista completa de dispositivos y probar en Windows antes de release.

### SEC21-06 — [P4, parcialmente corregido] Pruebas y documentación

Se añadieron dos tests de integración (`init_rechaza_rutas_invalidas_y_destinos_no_vacios` y `init_rechaza_un_destino_enlazado`) y la suite pasa a **94** pruebas. Cubren ruta vacía, `raiz/otra/..`, padre inexistente/`..`, destino no vacío y symlink final, pero **no** cubren:

- `writer init .` en cwd vacío (hoy aceptado);
- `../outside/b` y `a/../../outside/b` con destino vacío (hoy aceptados fuera del cwd);
- symlink en ancestro (hoy aceptado y escribe en el destino);
- `CONIN$`/`CONOUT$`/`COM¹`/espacio inicial;
- TOCTOU de `.escritor`;
- notas fallidas/rollback y `abrir_proyecto` fallido;
- `umask`/permisos y `localStorage`.

`docs/inicio-v1.md` se actualizó, pero tiene dos pasos numerados «4» y afirma que se rechazan `.`/`..` y los reservados de Windows; `.` en cwd vacío se acepta, los `..` intermedios se aceptan y varios reservados Windows no se cubren. `HANDOFF.md` y `README.md` actualizan el conteo a 94 correctamente.

## 3. Gate y pruebas

| Comprobación | Comando | Resultado |
|---|---|---|
| Tests Rust | `nice -n 19 cargo test --workspace --all-targets --locked` | **OK**: **94** (25 CLI + 34 unitarias + 35 de integración), 0 fallos. |
| Tests Tauri | `nice -n 19 cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | **OK**: **3/3**. |
| TypeScript | `nice -n 19 pnpm typecheck` | **OK**. |
| UI | `nice -n 19 pnpm ui:test` | **OK**: **58/58**. |
| Formato | `nice -n 19 cargo fmt --all --check` | **OK**. |
| Calidad | `nice -n 19 python3 scripts/ci.py calidad` | **FALLA**: `cargo clippy --workspace --all-targets --all-features -- -D warnings` reporta dos `redundant_closure` en `crates/core/src/project.rs:1476` y `:1497` (`map_err(|error| Error::Io(error))`). El acta decía «calidad OK»; no es reproducible con clippy 0.1.99. |
| Producto | `nice -n 19 python3 scripts/ci.py producto` | **OK**: fixture, determinismo, 12 errores/4 avisos, 1.000 escenas en 0,3 s, CLI y enlaces. |
| Seguridad | `nice -n 19 python3 scripts/ci.py seguridad` | **OK**: `cargo-deny` núcleo/desktop, gitleaks historial/árbol, zizmor omitido. |
| gitleaks historial | incluido | **48 commits**, ~1,68 MB, 0 hallazgos. |
| gitleaks árbol | incluido | ~1,65 MB, 0 hallazgos; solo `git ls-files`. |
| `.gitleaks.toml` | — | Ausente; el gate usa la configuración por defecto. |
| `.gitleaksignore` | — | Dos fingerprints exactos de falsos positivos históricos; sin allowlist amplia. |
| zizmor | incluido | Omitido: no existe `.github/`. |
| `cargo-deny` núcleo/desktop | incluido | `advisories ok, bans ok, licenses ok, sources ok`. |

No ejecuté `interfaz`, `e2e`, `e2e-tauri` ni `empaquetado`: la instrucción prohíbe abrir la app/ventanas. No se abrió la GUI ni se probó Windows en runtime.

## 4. Hallazgos nuevos

### SEC21-07 — [P3] El gate `calidad` no está verde: clippy falla en el commit congelado

- **Real:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` (clippy 0.1.99, Rust 1.99.1) falla con `clippy::redundant_closure` en `crates/core/src/project.rs:1476` y `:1497`. Son dos `map_err(|error| Error::Io(error))` que clippy sugiere sustituir por `map_err(Error::Io)`. El resto del gate (fmt, tests) pasa; `calidad` termina rojo.
- **Impacto:** el acta de congelación afirma `calidad OK` y la evidencia no es reproducible; el pipeline no puede considerar el gate de calidad verde. No es una vulnerabilidad de seguridad, pero bloquea el release hasta corregirlo y repetir el gate.
- **Corrección:** aplicar la sugerencia de clippy en las dos líneas, ejecutar `python3 scripts/ci.py calidad` y actualizar la evidencia si cambia el conteo. No requiere cambio de comportamiento.

### SEC21-01-R — [P3] Rutas relativas con `..`, `.` en cwd vacío y padres absolutos/symlink siguen permitiendo crear fuera del padre elegido

- **Real:** `preparar_raiz_inicial` valida el **último** componente y el padre canónico, pero no rechaza `..` intermedios ni exige que la ruta venga del diálogo. Reproducido:
  - `writer init ../outside/b` con `b` vacío: crea el proyecto fuera del cwd.
  - `writer init a/../../outside/b` con `b` vacío: crea fuera del cwd.
  - `writer init .` en un cwd vacío: crea el proyecto en el cwd (la documentación dice que `.` se rechaza).
  - `writer init link-ancestro/hijo` con `link-ancestro` symlink: escribe en el destino del enlace.
  - Un padre absoluto arbitrario y vacío es aceptado.
- **Impacto:** creación de proyectos en directorios vacíos fuera de la carpeta que el usuario eligió, y discrepancia con el contrato; no se sobrescriben datos existentes (los destinos no vacíos se rechazan). Requiere una ruta construida por IPC, no la UI normal, o un padre enlazado preexistente. P3, no bloquea por sí solo.
- **Corrección:** rechazar cualquier componente `..` y `.` en la ruta de entrada; exigir ruta absoluta si el comando viene del diálogo; pasar padre y nombre por separado y validar ambos; rechazar symlinks/junctions en cualquier ancestro; canonicalizar y comprobar que el destino está directamente bajo el padre permitido. Alinear `docs/inicio-v1.md` con el comportamiento real.

### Vigentes (no nuevos)

- **SEC-04-B — [P3]:** FIFO en `.escritor/objetos/<hash>` referenciado cuelga `verify`/`info`/`list`/`write`/`show`.
- **SEC-04-C — [P3]:** FIFO en `.escritor/indice.sqlite-journal` cuelga `write`/`reindex`/`buscar`.
- **SEC-02-H Windows — [P3 residual]:** enlaces duros de Windows no rechazados.
- **SEC-04 límites — [P3]:** lecturas completas sin límite en archivos, log, escaneo, cuarentena y DOCX; títulos/autor/ruta nuevos tampoco tienen límite.
- **TOCTOU general — [P3 residual]:** SEC21-02 es la manifestación concreta en `init`.
- **SEC-05 — [P4]:** CSP/capacidades/IPC sin endurecer; `crear_proyecto` no está ligado a un padre concesionado por el diálogo.
- **SEC-06 — [P4]:** deriva documental; ver SEC21-06.
- **`RUSTSEC-2024-0370` — [P4]:** excepción documentada en `deny.toml`.

**Resumen de severidades:** P1: 0 · **P2: 0** (SEC21-01 cerrado) · P3 nuevos: 2 (SEC21-01-R, SEC21-07) · P3 residuales abiertos: 2 (SEC21-02, SEC21-03) · P4 residuales: 3 (SEC21-04, SEC21-05, SEC21-06) · vigentes: SEC-04-B/C, SEC-02-H Windows, límites, SEC-05, SEC-06 y `RUSTSEC-2024-0370`.

## 5. Conclusión y decisión

**Decisión: aprobado con observaciones.**

SEC21-01 **queda cerrado en el punto que bloqueaba**: ya no se sobrescribe `proyecto.yml`, `.escritor/operaciones.log` ni un destino existente mediante `a/..`, `a/../b`, ruta vacía o symlink final. Verifiqué los centinelas en todos los casos con datos preexistentes; el P2 de la r1 no se reproduce.

Quedan residuales que no bloquean la seguridad por sí solos:

- **SEC21-01-R [P3]:** `../b`/`a/../../b` con destino vacío crean fuera del cwd; `.` en cwd vacío se acepta; un padre absoluto arbitrario o un symlink de ancestro se acepta. No sobrescribe datos existentes, pero contradice el contrato «rechaza `.`/`..`» y permite crear fuera del padre elegido.
- **SEC21-02 [P3]:** la carrera de `.escritor` sigue escribiendo `operaciones.log`/blobs fuera; reproducida en 10/300 intentos con un watcher.
- **SEC21-03 [P3]:** creación parcial sin rollback (título inválido, notas fallidas, `abrir_proyecto` fallido).
- **SEC21-07 [P3]:** `calidad` no está verde por dos `redundant_closure` en `project.rs:1476/1497`; corregir antes de release y repetir el gate.
- **P4:** `localStorage` residual (rutas señuelo/plaintext/longitudes), Windows/permissions residual y cobertura/documentación incompletas.

Recomendaciones para el siguiente pase (no bloqueantes para la seguridad, salvo que el release exija gates verdes):

1. Corregir los dos `map_err` de clippy y dejar `calidad` verde.
2. Rechazar `..`/`.` en cualquier componente y exigir que la ruta venga de un padre validado; o recibir padre+nombre separados y comprobar el destino directamente bajo ese padre; rechazar symlinks/junctions de ancestro.
3. Crear el proyecto en un temporal y renombrar; validar `.escritor` antes de log/blobs y añadir la carrera como test.
4. Añadir rollback/limpieza para notas y `abrir_proyecto`; validar el título antes de crear.
5. Completar la validación Windows, los modos restrictivos y los tests negativos; corregir la numeración/afirmaciones de `docs/inicio-v1.md`.
6. Repetir security review tras las correcciones; después release, UX y product owner.

No hay P1 ni P2 abiertos. La seguridad de la ruta corregida puede avanzar con las observaciones anteriores; el gate `calidad` debe corregirse antes de publicar. No se abrió la app ni se ejecutaron ventanas; Windows no se probó en runtime.
