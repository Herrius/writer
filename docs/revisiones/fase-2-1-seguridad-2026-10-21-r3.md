# Fase 2.1 — Security review (r3)

- **Commit revisado:** `e0bb3fc3f8f2cbc7a12a7c8ff5f586adac51a002`
- **HEAD del pipeline:** `a916bbec69781805e7c852d22b24dfc6841e1cbc`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64; Rust 1.99.1; clippy 0.1.99; `cargo-deny` 0.20.2; `zizmor` 1.31.1; gitleaks local (sin número de versión reportado); pnpm 12.14.0; Node 26.11.1; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). `git diff --stat e0bb3fc..HEAD` solo toca `docs/revisiones/README.md` y `docs/revisiones/fase-2-1-12-congelado-seguridad-r3.md`; no hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD. Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **aprobado con observaciones** — P1: 0 · P2: 0 · P3 residuales abiertos: 2 (SEC21-02, SEC21-03) · P4 residuales: 4. SEC21-01-R (los casos que bloqueaban) **cerrado**.

## 1. Verificación de SEC21-01-R

Recompilé `target/debug/writer` desde el commit y repetí los casos del informe r2 y los de la corrección. Usé centinelas y directorios desechables en `/tmp/opencode/sec21r3*`; no se tocó el repositorio.

### 1.1 Casos que ahora rechazan correctamente

| Caso | Resultado |
|---|---|
| `writer init .` en cwd vacío | **OK**: rc 1, `la ruta no puede contener '.' ni '..': .`; no se crea `proyecto.yml` ni ningún directorio del proyecto. |
| `writer init .` en cwd con `proyecto.yml` centinela | **OK**: rc 1, mismo rechazo; manifiesto intacto. |
| `writer init ..` | **OK**: rc 1, `la ruta no puede contener '.' ni '..': ..`. |
| `writer init 'a/..'` | **OK**: rc 1, `la ruta no puede contener '.' ni '..': a/..`. |
| `writer init 'a/../b'` | **OK**: rc 1, `la ruta no puede contener '.' ni '..': a/../b`; no se crea `b`, ni con `a` existente ni inexistente. |
| `writer init '../outside/b'` | **OK**: rc 1, `la ruta no puede contener '.' ni '..': ../outside/b`; el centinela de `outside/b` no cambia. |
| `writer init 'a/../../outside/b'` | **OK**: rc 1, `la ruta no puede contener '.' ni '..': a/../../outside/b`; el centinela no cambia. |
| `writer init /` | **OK**: rc 1, `la ruta no nombra una carpeta de proyecto`. |
| `writer init` con ruta vacía (sonda Rust) | **OK**: rc 1, `la ruta del proyecto no puede estar vacía`; el centinela no cambia. |
| Destino existente **no vacío** | **OK**: rc 1, `el directorio no está vacío`; el `proyecto.yml` centinela conserva bytes y hash. |
| Destino existente **vacío** | **OK**: crea el proyecto. |
| Destino archivo, FIFO o socket | **OK**: rc 1, `el destino del proyecto no es un directorio`; sin cuelgue y sin escritura. |
| Destino symlink final | **OK**: rc 1, `el destino del proyecto no puede ser un enlace`; el destino real no cambia. |
| Padre inexistente, archivo, FIFO o socket | **OK**: rc 1, `la carpeta padre no existe o no es un directorio`. |
| Padre o destino de solo lectura | **OK**: rc 1, `Permission denied`; no se crea ni se modifica nada. |
| Nombres `CON`, `AUX`, `NUL`, `COM1`, `LPT9`, `punto.`, `punto `, `C:\tmp\win`, `a\b` | **OK**: rc 1, `nombre de proyecto no válido`. |
| `CONIN$`, `CONOUT$` y nombre con espacio inicial | **OK**: rc 1, `nombre de proyecto no válido`. |
| Nombre con carácter de control | **OK**: rc 1, nombre no válido; no se crea el destino. |
| Ruta relativa normal `rel` | **OK**: crea `cwd/rel`; no hay regresión. |

Los casos que bloqueaban en la r2 (`../outside/b`, `a/../../outside/b`, `.` en cwd vacío, `a/../b`, `a/..`, `..`) ya no crean nada. El destino no vacío y el symlink final se rechazan antes de escribir, y los centinelas de `proyecto.yml`/`.escritor` permanecen intactos.

### 1.2 Residual P4: lo que sigue aceptándose

- `writer init link-ancestro/hijo` con `link-ancestro` symlink a un directorio real: **aceptado**; escribe en el destino del symlink si `hijo` no existe o está vacío. No sobrescribe datos existentes.
- `writer init` con un padre absoluto arbitrario y vacío: **aceptado**; el backend valida que el padre exista, pero no que sea el padre elegido en el diálogo de la UI. No sobrescribe destinos no vacíos.
- `COM¹`, `COM²`, `LPT0` y `CLOCK$`: **aceptados** en Linux; son formas reservadas/especiales en Windows que `validar_ruta_relativa` no cubre. No hay ejecución Windows.
- Nombres con bidi o longitud >255: el backend no rechaza el bidi; la longitud la limita el sistema de archivos.

Estos residuales no reproducen el P2: no se sobrescribe un `proyecto.yml`, un `.escritor` ni un archivo externo existente. Quedan como deuda P4 de alcance/binding al diálogo y paridad Windows.

### 1.3 Corrección observada

- `preparar_raiz_inicial` rechaza cualquier componente `CurDir` o `ParentDir` antes de resolver la ruta; eso cubre `.`, `..`, `a/..`, `a/../b`, `../outside/b` y `a/../../outside/b`.
- `validar_ruta_relativa` rechaza espacios iniciales, `CONIN$` y `CONOUT$`, además de los reservados y controles anteriores.
- `Project::init` mantiene la validación de destino no vacío, symlink final y padre real, y `Log::crear_vacio` mantiene la comprobación de padre real y log no enlazado.
- La validación de título sigue ocurriendo después de `preparar_raiz_inicial`; por eso un título inválido deja un directorio vacío (SEC21-03).

## 2. Residuales y superficie

### SEC21-02 — [P3, abierto] TOCTOU de `.escritor` en `Project::init`

La carrera sigue reproduciéndose con el commit congelado:

```text
watcher: espera p/manuscrito; rm -rf p/.escritor; ln -s ext p/.escritor
writer init p --titulo X
```

Resultado en **25/300** intentos: `ext/operaciones.log` (centinela) fue reemplazado por un log de 616 bytes y `ext/objetos/<hash>` fue creado; `writer init` terminó con rc 0. `validar_escritor`/`directorio_real` no eliminan la ventana entre la comprobación y `escribir_atomico`. Requiere un atacante local con escritura en el directorio del proyecto; es la misma clase TOCTOU ya registrada, ahora en `init`. P3, no bloquea por sí sola.

### SEC21-03 — [P3, abierto] Creación parcial sin rollback

- `writer init vacio --titulo ""` → rc 1 y deja `vacio/` vacío; `writer init control --titulo <control>` → rc 1 y deja `control/` vacío.
- `init` + `chmod 0555 notas` + `create nota` → rc 1, el proyecto permanece con 1 nota y `verify` OK; el comando Tauri devolvería error y el reintento fallaría por «directorio no está vacío».
- Si falla un directorio, el log o la primera escena, queda un proyecto parcial; si falla `abrir_proyecto`, el proyecto queda creado y el comando devuelve error.

No hay pérdida de datos existentes; es deuda de robustez P3. La corrección pendiente es validar el título antes de crear, crear en temporal + rename y limpiar las notas/el proyecto si falla la operación completa.

### SEC21-04 — [P4, corregido en lo principal] `localStorage` de recientes

`ui/src/recientes.ts` valida ahora:

- `ruta` no vacía y ≤ 4096 caracteres;
- `titulo` no vacío y ≤ 200;
- `autor` string ≤ 200 o ausente;
- `palabras` número finito no negativo o ausente;
- `color` `#rrggbb`;
- deduplicación por ruta y recorte a 8.

Lo comprobé ejecutando la función real compilada con `tsc` y un `localStorage` simulado:

| Entrada | Resultado |
|---|---|
| Entrada válida | aceptada. |
| `autor: {}` / `palabras: {}` / `palabras: "1"` | rechazadas; ya no rompen el render. |
| `color` arbitrario (`red); ...`) | rechazado; solo hex de 6 dígitos. |
| `color` `#4D6B8A` | aceptado. |
| `titulo`/`autor` >200 o `ruta` >4096 | rechazados. |
| `titulo` vacío | rechazado. |
| 9 entradas | 8 tras deduplicar/recortar. |

Residual P4: `ruta` se guarda en claro y se usa para abrir un proyecto, por lo que una entrada manipulada puede mostrar una tarjeta señuelo con una ruta arbitraria; `titulo`/`autor` no sanean controles/bidi; no se comprueba que la ruta siga existiendo. Requiere acceso al `localStorage` local o ejecución de script; no hay XSS (`dangerouslySetInnerHTML` ausente; React escapa).

### SEC21-05 — [P4, parcialmente corregido] Windows, nombres y permisos

Se corrigieron espacios iniciales, `CONIN$` y `CONOUT$`. Quedan:

- `COM¹`, `COM²`, `LPT0` y `CLOCK$` aceptados; no hay ejecución Windows para confirmar el efecto.
- Padre symlink/junction aceptado y canonicalizado; el proyecto se crea en el destino si el hijo no existe o está vacío.
- Rutas UNC/`\\?\`/ADS solo están cubiertas por la validación del nombre final, no por una política Windows completa.
- `umask 000` sigue produciendo directorios `0777` y archivos `0666`; `.escritor`, manifiesto y log quedan accesibles a otros usuarios locales. No se fijan `0700`/`0600`.

P4; no se probó Windows en runtime.

### SEC21-06 — [P4, parcialmente corregido] Pruebas y documentación

Los tests nuevos añaden `.` y `..` a las rutas inválidas y cubren ruta vacía, `raiz/otra/..`, padre inexistente/`..`, destino no vacío y symlink final. Siguen sin cubrir:

- `../outside/b` y `a/../../outside/b` (ahora rechazados, pero sin regresión explícita);
- symlink de ancestro (hoy aceptado y escribe en el destino);
- `COM¹`/`LPT0`/`CLOCK$`/bidi;
- TOCTOU de `.escritor`;
- notas fallidas/rollback y `abrir_proyecto` fallido;
- `umask`/permisos y los residuales de `localStorage`.

`docs/inicio-v1.md` sigue teniendo **dos pasos numerados «4»** y afirma que se rechazan los reservados de Windows en general; `COM¹`/`LPT0`/`CLOCK$` no se rechazan. La afirmación sobre `.`/`..` ya es correcta. `HANDOFF.md`/`README.md` mantienen 94 pruebas.

### Vigentes (no nuevos)

- **SEC-04-B — [P3]:** FIFO en `.escritor/objetos/<hash>` referenciado cuelga `verify`/`info`/`list`/`write`/`show`.
- **SEC-04-C — [P3]:** FIFO en `.escritor/indice.sqlite-journal` cuelga `write`/`reindex`/`buscar`.
- **SEC-02-H Windows — [P3 residual]:** enlaces duros de Windows no rechazados.
- **SEC-04 límites — [P3]:** lecturas completas sin límite; títulos/autor/ruta nuevos tampoco tienen límite.
- **TOCTOU general — [P3 residual]:** SEC21-02 es la manifestación concreta en `init`.
- **SEC-05 — [P4]:** CSP/capacidades/IPC sin endurecer; `crear_proyecto` no está ligado a un padre concesionado por el diálogo.
- **SEC-06 — [P4]:** deriva documental; ver SEC21-06.
- **`RUSTSEC-2024-0370` — [P4]:** excepción documentada en `deny.toml`.

**Resumen de severidades:** P1: 0 · **P2: 0** · P3 residuales abiertos: 2 (SEC21-02, SEC21-03) · P4 residuales: 4 (SEC21-01-R parcial, SEC21-04, SEC21-05, SEC21-06) · vigentes: SEC-04-B/C, SEC-02-H Windows, límites, SEC-05, SEC-06 y `RUSTSEC-2024-0370`.

## 3. Gate y pruebas

| Comprobación | Comando | Resultado |
|---|---|---|
| Tests Rust | `nice -n 19 cargo test --workspace --all-targets --locked` | **OK**: **94** (25 CLI + 34 unitarias + 35 de integración), 0 fallos. |
| Tests Tauri | `nice -n 19 cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | **OK**: **3/3**. |
| TypeScript | `nice -n 19 pnpm typecheck` | **OK**. |
| UI | `nice -n 19 pnpm ui:test` | **OK**: **58/58**. |
| Formato | `nice -n 19 cargo fmt --all --check` | **OK**. |
| Calidad | `nice -n 19 python3 scripts/ci.py calidad` | **OK**: fmt, clippy `-D warnings` y tests; clippy 0.1.99 ya no reporta los `redundant_closure` de la r2. |
| Producto | `nice -n 19 python3 scripts/ci.py producto` | **OK**: fixture, determinismo, 12 errores/4 avisos, 1.000 escenas en 0,3 s, CLI y enlaces. |
| Seguridad | `nice -n 19 python3 scripts/ci.py seguridad` | **OK**: `cargo-deny` núcleo/desktop, gitleaks historial/árbol, zizmor omitido. |
| gitleaks historial | incluido | **48 commits**, ~1,68 MB, 0 hallazgos. |
| gitleaks árbol | incluido | ~1,65 MB, 0 hallazgos; solo `git ls-files`. |
| `.gitleaks.toml` | — | Ausente; configuración por defecto. |
| `.gitleaksignore` | — | Dos fingerprints exactos; sin allowlist amplia. |
| zizmor | incluido | Omitido: no existe `.github/`. |
| `cargo-deny` núcleo/desktop | incluido | `advisories ok, bans ok, licenses ok, sources ok`. |
| Creación normal | CLI + Tauri | **OK**: 1 escena/0 notas sin notas y 1 escena/3 notas con notas; `verify` OK. |
| `localStorage` | función real compilada + `localStorage` simulado | **OK**: tipos, color, longitudes, dedup y 8 entradas. |

No ejecuté `interfaz`, `e2e`, `e2e-tauri` ni `empaquetado`: la instrucción prohíbe abrir la app/ventanas. No se abrió la GUI ni se probó Windows en runtime.

## 4. Hallazgos nuevos

**No hay P1, P2 ni P3 nuevos.** La corrección de SEC21-01-R cierra los casos que quedaban abiertos y `calidad` vuelve a verde. Los únicos hallazgos son residuales ya registrados:

| ID | Severidad | Estado |
|---|---|---|
| SEC21-02 | P3 residual | Abierto: TOCTOU de `.escritor`; reproducida escritura externa de `operaciones.log`/blobs en 25/300 intentos. |
| SEC21-03 | P3 residual | Abierto: creación parcial sin rollback (título inválido, notas fallidas, `abrir_proyecto` fallido). |
| SEC21-01-R | P4 residual | Los casos `../outside/b`, `a/../../outside/b`, `.`, `..`, `a/..` y `a/../b` se rechazan; quedan padre absoluto arbitrario y symlink de ancestro aceptados, sin sobrescritura. |
| SEC21-04 | P4 residual | Tipos/color/longitudes/dedup/8 corregidos; quedan rutas señuelo/plaintext y controles en títulos. |
| SEC21-05 | P4 residual | Espacios iniciales, `CONIN$` y `CONOUT$` corregidos; quedan `COM¹`/`LPT0`/`CLOCK$`, junction y `umask 000` (`0777`/`0666`). |
| SEC21-06 | P4 residual | Cobertura de tests incompleta y `docs/inicio-v1.md` con dos pasos «4» y afirmación amplia sobre reservados Windows. |

No hay un nuevo P2 que bloquee. SEC21-02 es la deuda P3 más relevante: sigue permitiendo una escritura fuera bajo una carrera local; SEC21-03 es robustez de creación. Ambas deben registrarse como residuales aceptados si el product owner decide avanzar.

## 5. Conclusión y decisión

**Decisión: aprobado con observaciones.**

- **SEC21-01-R:** **cerrado** para los casos que motivaron la corrección. `../outside/b`, `a/../../outside/b`, `.`, `..`, `a/..` y `a/../b` se rechazan por componentes `.`/`..`; no se crea nada ni se modifican los centinelas. Los destinos no vacíos, archivos/FIFO/socket, symlink final, padre inexistente y permisos insuficientes también se rechazan con error visible.
- **`calidad`:** **verde**; clippy 0.1.99 y los tests pasan. No hay regresión en creación normal con/sin notas.
- **`localStorage`:** tipos, color, longitudes, deduplicación y límite de 8 verificados con la función real; quedan residuales P4 de rutas señuelo/plaintext/controles.
- **Residuales:** SEC21-02 (P3, TOCTOU de `.escritor`) y SEC21-03 (P3, creación parcial) siguen abiertos; SEC21-04/05/06 y el resto de P4 quedan registrados. No hay P1/P2.

Recomendaciones no bloqueantes:

1. Cerrar SEC21-02 creando el proyecto en un temporal y renombrando, o usando `openat`/`O_NOFOLLOW`/`dirfd` y comprobando el inodo de `.escritor`.
2. Cerrar SEC21-03 validando el título antes de crear y limpiando las notas/proyecto si falla la operación completa.
3. Completar nombres Windows (`COM¹`, `LPT0`, `CLOCK$`), modos `0700`/`0600` y los tests negativos que faltan.
4. Corregir la numeración y las afirmaciones de `docs/inicio-v1.md`; mantener el contrato alineado con el código.
5. Repetir security review si se tocan estas rutas; después release, UX y product owner.

No se abrió la app ni se ejecutaron ventanas; Windows no se probó en runtime. El informe queda en `docs/revisiones/fase-2-1-seguridad-2026-10-21-r3.md` sin commitear.
