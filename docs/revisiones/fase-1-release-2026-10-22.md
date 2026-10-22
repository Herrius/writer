# Fase 1 — Release del rediseño del panel de Proyectos

- **Commit revisado:** `744aaa5` (`test: completar la QA visual y de teclado del panel`)
- **Tag:** `fase-1-congelado` (tag anotado `aef3372`, resuelve a `744aaa5`)
- **Baseline:** `fca190f`
- **HEAD al revisar:** `6e8f920` (`docs: añadir informe de seguridad de la fase 1`). Solo añade documentación; no toca `ui/src` ni `e2e`.
- **Fecha:** 2026-10-22
- **Entorno:** Linux x86_64, EndeavourOS (Arch), kernel 7.2.9; Node 26.11.1; pnpm 12.14.0; Rust/cargo 1.99.1; Python 3.14.8; Tauri 2.12.4 y Tauri CLI 2.11.0; WebKitGTK 4.1 2.54.1; GTK 3.24.52; RPM 6.0.1; GNU ar 2.47; `tauri-driver` 2.0.6, `WebKitWebDriver` y `xdotool` presentes. Sesión Wayland con Xorg en `DISPLAY=:1`; el E2E usó `GDK_BACKEND=x11`. `dpkg-deb` no está instalado; no fue necesario: Tauri generó el `.deb` y el gate lo verificó con `ar`/`tar`.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). Al terminar solo queda este informe sin versionar; no hay cambios en código, tests, contratos ni documentación de producto.
- **Decisión:** **aprobado con observaciones** — P1: 0 · P2: 0 · P3: 2 · P4: 2. La fase puede continuar a UX y product owner. El **release público sigue bloqueado** por deuda heredada de release, no por una regresión de Fase 1.

No edité código, tests, contratos ni documentación de producto. Las sondas y los artefactos de trabajo viven en `/tmp/fase-1-release/` y no se versionan.

## 0. Inmutabilidad del artefacto

```text
$ git diff --stat 744aaa5 -- ui/src e2e
(vacío)

$ git diff --name-status 744aaa5..HEAD
M  HANDOFF.md
M  docs/iteraciones/17-fase-1-rediseno-panel.md
M  docs/revisiones/README.md
A  docs/revisiones/fase-1-00-congelado.md
A  docs/revisiones/fase-1-qa-2026-10-22.md
A  docs/revisiones/fase-1-seguridad-2026-10-22.md
```

El diff posterior al hash congelado es exclusivamente documental. El árbol de trabajo no tiene cambios de código.

## 1. Alcance del diff y configuración

El diff funcional congelado es de siete archivos y no toca manifiestos, lockfiles, permisos, CSP ni configuración de bundle:

```text
$ git diff --stat fca190f..744aaa5
 docs/README.md                               |   2 +
 docs/inicio-v1.md                            |  14 +-
 docs/iteraciones/17-fase-1-rediseno-panel.md | 235 +++++++++++++++++++++++++++
 e2e/fase-1-panel.spec.ts                     | 137 ++++++++++++++++
 ui/src/App.test.tsx                          |  18 ++
 ui/src/Inicio.tsx                            |  62 ++-----
 ui/src/styles.css                            | 147 ++++++++---------
 7 files changed, 483 insertions(+), 132 deletions(-)
```

Comprobación archivo por archivo contra `fca190f` (todos idénticos en `744aaa5`):

| Archivo | Resultado |
|---|---|
| `package.json`, `ui/package.json`, `apps/desktop/package.json` | idéntico |
| `pnpm-lock.yaml`, `pnpm-workspace.yaml` | idéntico |
| `Cargo.toml`, `Cargo.lock`, `apps/desktop/src-tauri/Cargo.lock` | idéntico |
| `deny.toml`, `rust-toolchain.toml` | idéntico |
| `apps/desktop/src-tauri/tauri.conf.json` | idéntico |
| `apps/desktop/src-tauri/tauri.linux.conf.json` | idéntico |
| `apps/desktop/src-tauri/capabilities/default.json` | idéntico |
| `ui/index.html`, `ui/vite.config.ts` | idéntico |

En particular:

- **Versión:** `0.2.0` en `package.json`, `ui/package.json`, `apps/desktop/package.json`, `tauri.conf.json` y `apps/desktop/src-tauri/Cargo.toml`; idéntica en `fca190f`, `744aaa5` y `HEAD`. La versión de biblioteca del workspace (`crates/core`, `crates/cli`) sigue siendo `0.1.0`, como en el baseline.
- **CSP y `devCsp`:** sin cambios. Producción sigue con `default-src 'self'`, `connect-src ipc: http://ipc.localhost`, `script-src 'self'`; no se añadió `unsafe-eval` ni orígenes remotos.
- **Capabilities:** sin cambios; no hay permisos nuevos de `fs`, `http`, `shell` ni comandos IPC nuevos.
- **Bundle:** `tauri.conf.json` y `tauri.linux.conf.json` sin cambios; los targets Linux siguen siendo `deb` y `rpm`.
- **Dependencias:** no hay cambios en manifiestos ni en los tres lockfiles. No se añadió ninguna dependencia.

## 2. Gates ejecutados

| Comando | Resultado |
|---|---|
| `python scripts/ci.py empaquetado` | **OK** en 19,5 s (primera ejecución) y 18,9 s (segunda) |
| `python scripts/ci.py e2e-tauri` | **OK** en 23,0 s; los dos escenarios de escritorio pasaron |

Detalle de `empaquetado`:

```text
Empaquetado: writer_0.2.0_amd64.deb verificado (estructura, permisos, md5)
Empaquetado: writer-0.2.0-1.x86_64.rpm verificado (estructura, permisos, digests)
== empaquetado: OK en 18.9 s ==
```

Detalle de `e2e-tauri`:

```text
E2E Tauri: Ctrl+S guarda en disco y la escena sigue visible
E2E Tauri: cerrar la ventana guarda, destruye la ventana y termina el proceso
== e2e-tauri: OK en 23.0 s ==
```

`e2e-tauri` ejecutó `pnpm install --frozen-lockfile` y compiló la app con la feature `e2e`, que solo existe en esa compilación. El entorno Linux permitió ejecutar Tauri/WebDriver sin limitaciones; no hubo que omitir el gate. Los logs completos están en `/tmp/fase-1-release/empaquetado.log`, `empaquetado2.log` y `e2e-tauri.log`.

La segunda ejecución de `empaquetado` se hizo para dejar `target/release/writer-desktop` en estado de producción después de que `e2e-tauri` lo recompilara con la feature de prueba, y para comparar el build y los binarios entre dos ejecuciones.

No se usó `npm` ni `npx`; los gates invocan `pnpm`. No tomé como válidos los resultados del builder: ejecuté de nuevo los dos gates sobre el árbol congelado.

## 3. Artefactos y contenido

### 3.1 Build y assets de UI

El build de Vite (109 módulos) produjo exactamente estos assets, sin restos de builds anteriores:

| Asset | Tamaño | SHA-256 |
|---|---:|---|
| `ui/dist/assets/index-BvBwub77.js` | 753.297 B | `61e842b862062cdeea4e5b4dddb016760bf08d142764869a481f10315e534fd1` |
| `ui/dist/assets/index-DuUTl7bv.css` | 42.052 B | `a7b0e4022527ab227d17214d3ddf9b2c8462f33b6d799ec2b7ab23fd96ebee52` |
| `ui/dist/assets/window-C30PvoBI.js` | 15.163 B | `f211a29ca3b6cc36e7d5a3e28773a92ba28a15e54c1d5155ab501ab74f50283f` |
| `ui/dist/index.html` | 440 B | `c969895d8ccc1bb00fd99de9e0f7f1289ed7fc5bc3b8f2c4532dd0833354dcf5` |

`ui/dist/index.html` referencia `index-BvBwub77.js` e `index-DuUTl7bv.css`; `window-C30PvoBI.js` es el chunk de ventana importado dinámicamente por el bundle principal. No hay rutas absolutas ni `file://` en `ui/dist`.

### 3.2 Paquetes finales

| Artefacto | Tamaño | SHA-256 |
|---|---:|---|
| `bundle/deb/writer_0.2.0_amd64.deb` | 5.259.060 B | `64e4026f49699dededc727e23c5a80b099f0574f32d1f7e0b6629674c5271c77` |
| `bundle/rpm/writer-0.2.0-1.x86_64.rpm` | 5.259.430 B | `5b5f996b44350655d8dde7a2ede8b74c69e1e3a58ba29e26300b0dfa9cb44a8c` |
| `writer-desktop` empaquetado | 16.306.776 B | `.deb`: `03a9832a...` · `.rpm`: `95faf857...` |

Los dos binarios empaquetados solo difieren en **3 bytes** (posiciones 1.607.062–1.607.064), los que Tauri parchea con el tipo de bundle (`DEB`/`RPM`); el resto es idéntico.

Contenido de ambos paquetes (cinco archivos, todos bajo `/usr`):

```text
/usr/bin/writer-desktop
/usr/share/applications/writer.desktop
/usr/share/icons/hicolor/32x32/apps/writer-desktop.png
/usr/share/icons/hicolor/128x128/apps/writer-desktop.png
/usr/share/icons/hicolor/256x256@2/apps/writer-desktop.png
```

- `.deb`: `Version: 0.2.0`, `Architecture: amd64`, `Installed-Size: 15994`; los cinco `md5sums` coinciden con `data.tar.gz`; no hay `preinst`/`postinst`/`prerm`/`postrm`, `triggers` ni `conffiles`.
- `.rpm`: `writer-0.2.0-1.x86_64`, `License: MIT`, `Build Date: 2026-10-22 18:15`; payload gzip y digests internos válidos; no hay scripts.
- `.desktop`: `Exec=writer-desktop`, `Icon=writer-desktop`, `Terminal=false`, `StartupWMClass=writer-desktop`, `Categories=Office;`; `desktop-file-validate` termina 0.
- No hay rutas `/home`, `/tmp` ni archivos fuera de `/usr` en los nombres de archivo de ninguno de los dos paquetes; no hay enlaces simbólicos, setuid ni dispositivos.

### 3.3 La UI nueva está embebida

Verificaciones sobre el binario extraído del `.deb` y del `.rpm`:

- El binario referencia exactamente `assets/index-BvBwub77.js`, `assets/index-DuUTl7bv.css` y `assets/window-C30PvoBI.js`, los tres assets de la build congelada.
- No contiene los nombres antiguos `index-CMWN1yFN.css` ni `index-Dqs-46R1.js` que aparecen en los restos de AppImage de una build anterior.
- Ambos binarios empaquetados carecen de la cadena `WRITER_E2E_PROJECT`; la feature de prueba no se filtró a los paquetes.
- Sonda independiente con `tauri-driver` + `WebKitWebDriver` sobre el binario del `.deb` (X11, `/tmp/fase-1-release/probe-paquete.mjs`): el panel `.inicio` se mostró, contenía «Crear nuevo proyecto» y la reserva lateral; no aparecieron «Proyectos personales», «Ideas», «Ayuda» ni «meta diaria». El título de la ventana fue `writer`.
- `ui/dist/` contiene solo los tres assets nuevos; Vite limpia el directorio de salida y no quedan assets obsoletos dentro de los paquetes.

### 3.4 Restos de AppImage en el directorio de bundle

El directorio `bundle/` **no está limpio**: conserva `bundle/appimage/` y `bundle/appimage_deb/` de una ejecución anterior (21 oct, 13:50), aunque el build actual solo genera `deb` y `rpm`. Ocupan 275.417.163 B y 16.084.738 B respectivamente (291,5 MB, 139 archivos) y contienen un binario antiguo de 16.066.240 B con los assets `index-CMWN1yFN.css` e `index-Dqs-46R1.js`, es decir, la UI anterior. No hay ningún `*.AppImage` final. Es un riesgo de inventario, no de los paquetes revisados; ver REL-F1-03.

## 4. Versión, tag y release

- **Versión:** `0.2.0` coherente en los manifiestos, `tauri.conf.json`, `control` del `.deb` y metadatos del `.rpm`. No hubo cambio de versión entre `fca190f`, `744aaa5` y `HEAD`.
- **Tag de congelación:** `fase-1-congelado` es un tag anotado (`aef337204113114837b5ffebf032773d47f011a1`) que resuelve al commit `744aaa5d4398accdecdba305b9d69849c9697c3f`. `git describe --tags --exact-match 744aaa5` devuelve `fase-1-congelado`.
- **Tags existentes:** solo los de fases/bloques (`fase-1-congelado`, `fase-2-aprobada`, `fase-2-1-aprobada`, `bloque-2-2-congelado`, `bloque-2-2-aprobado`). No existe `v0.2.0`, `0.2.0` ni ningún tag de release.
- **Remoto:** el repositorio no tiene remoto configurado (`git remote -v` vacío); `gh release list` responde `no git remotes found`. No se ha emitido un release público desde este árbol.
- **Rama:** solo `main`, en `6e8f920`; el diff `744aaa5..HEAD` es exclusivamente documental.

## 5. Documentación de congelación y HANDOFF

- El acta `docs/revisiones/fase-1-00-congelado.md` identifica correctamente el commit `744aaa5`, el tag y la rama; declara Linux x86_64 y advierte de que **Windows no se ha probado**.
- El acta dice expresamente: «El release público sigue sin autorización. Windows sigue pendiente».
- `HANDOFF.md` declara el objetivo Windows/Linux pero mantiene «Windows pendiente de build y pruebas» y «El release público de `0.2.0` **no** está autorizado», con la lista de lo que falta (firma/checksums, AppImage/updater, Windows, app instalada, CI remoto).
- No hay promesa de soporte Windows ni de release público en el acta ni en `HANDOFF.md`. La frase «Windows y Linux» describe el objetivo de plataforma, no un resultado validado.
- Observación menor: `HANDOFF.md` conserva la frase «conserva 297 px en estado vacío» sin acotar a 1440; QA ya la registró como QA-F1-04 (P4). No afecta al release ni contradice la decisión de congelación.

## 6. Reproducibilidad, tamaño y warnings

### 6.1 Reproducibilidad

Se ejecutó `empaquetado` dos veces sobre el mismo commit:

- Los **assets de UI** son byte a byte idénticos entre las dos builds (mismos SHA-256 de §3.1).
- Los **binarios empaquetados** son byte a byte idénticos entre las dos builds: `.deb` `03a9832a...` y `.rpm` `95faf857...`.
- Los **paquetes** no son byte-reproducibles: `.deb` pasó de `24ebff624ae3ae765b48eeb9e4ec57eee90696c2daf3d77d8cbf81e137d3f407` a `64e4026f...` (4 bytes de diferencia), y `.rpm` de `91728eeb...` a `5b5f996b...`. La diferencia observable está en los mtimes de `ar`/tar y en `BUILDTIME` del RPM, no en el binario ni en los assets. No hay `SOURCE_DATE_EPOCH` ni normalización de timestamps.

Esto reproduce la deuda REL-02 del release anterior: el binario y los assets son deterministas, el contenedor del paquete no. Para un release público hacen falta checksums publicados o paquetes normalizados.

### 6.2 Tamaño

- Binario empaquetado: 16.306.776 B (16,31 MB).
- `.deb`: 5.259.060 B (5,26 MB); `.rpm`: 5.259.430 B (5,26 MB).
- `bundle/` completo: 310 MB, de los cuales 291,5 MB son restos de AppImage ajenos al build actual.
- Frente a la referencia del release anterior (`0.2.0`, UI anterior: `.deb` 5.171.448 B, `.rpm` 5.171.439 B), el paquete crece unos 88 KB. Es coherente con el aumento de los assets de UI (JS +19.037 B; CSS +24.031 B) y del binario; no hay un salto anómalo.

### 6.3 Warnings

- **Vite:** `index-BvBwub77.js` mide 753,30 kB (236,62 kB gzip) y supera el umbral de 500 kB. Es deuda previa, no rompe el build ni el empaquetado. El chunk de ventana está separado (15,16 kB).
- **Tauri:** `warning: writer-desktop@0.2.0: STATIC_VCRUNTIME is deprecated; use build.windows.staticVCRuntime...`. Es un aviso de configuración para Windows; no afecta al binario Linux. Conviene corregirlo antes de la fase Windows.
- **`empaquetado`:** el gate verifica estructura, permisos, `md5sums` y digests del último `.deb` y `.rpm`, pero no compara la versión con los manifiestos ni comprueba que el binario embebido sea exactamente el de `ui/dist`. Lo verifiqué manualmente en §3.3; la cobertura automática sigue siendo deuda.

### 6.4 Efecto secundario de `e2e-tauri` en `target/`

`e2e-tauri` compila `target/release/writer-desktop` con `--features e2e` y deja esa compilación en el árbol de trabajo. Los paquetes ya generados no se contaminan, y una nueva ejecución de `empaquetado` reconstruye el binario de producción. El orden del acta (`e2e-tauri` antes de `empaquetado`) deja el estado correcto; si se ejecutan en orden inverso, hay que repetir `empaquetado` antes de publicar. Lo hice así y el binario final ya no contiene `WRITER_E2E_PROJECT`.

## 7. Hallazgos

| ID | Severidad | Hallazgo | Origen |
|---|---|---|---|
| REL-F1-01 | **P3** | El binario de release no está strip-eado y contiene rutas absolutas del entorno de build | Heredado de REL-01 (release anterior) |
| REL-F1-02 | **P3** | Los paquetes no son byte-reproducibles; el binario y los assets de UI sí son deterministas | Heredado de REL-02 |
| REL-F1-03 | **P4** | `bundle/appimage/` y `bundle/appimage_deb/` conservan una UI antigua y 291,5 MB de restos | Heredado de REL-08 |
| REL-F1-04 | **P4** | Warnings de Vite (`chunk >500 kB`) y `STATIC_VCRUNTIME`; el gate de empaquetado no valida versión ni identidad de los assets embebidos | Deuda de release |

No hay P1 ni P2.

### REL-F1-01 — P3: rutas absolutas y símbolos en el binario

`file` reporta `not stripped` y el ELF conserva `.symtab`. `strings -a` encuentra **289** cadenas con `/home/enrique`:

- 6 en `crates/core/src/`: `docx.rs`, `fsutil.rs`, `index.rs`, `model.rs`, `oplog.rs` y `yaml.rs`;
- 283 en `/home/enrique/.cargo/registry/src/...`.

No aparecen rutas de `ui/src` ni del código nuevo de Fase 1; el problema es de compilación/empaquetado, no de la UI. Divulga el usuario y la estructura del proyecto, impide reproducibilidad entre rutas de build y añade tamaño. La corrección sigue siendo la del informe anterior: `[profile.release] strip = "symbols"` y `RUSTFLAGS=--remap-path-prefix=<home>=/build` (o build en ruta neutral), más una comprobación de `strings` en el gate. No bloquea la fase local; sí debe cerrarse antes de un release público.

### REL-F1-02 — P3: paquetes no reproducibles byte a byte

Dos builds del mismo commit producen los mismos assets y el mismo binario empaquetado, pero hashes de paquete distintos por mtimes y `BUILDTIME`. Sin `SOURCE_DATE_EPOCH`, `SHA256SUMS` publicados o normalización de timestamps, un usuario no puede verificar que el paquete distribuido corresponde al build revisado. Es deuda heredada; no impide la revisión local.

### REL-F1-03 — P4: restos de AppImage con UI antigua

`bundle/appimage/` y `bundle/appimage_deb/` existen desde el 21 de octubre y contienen un binario de 16.066.240 B con `index-CMWN1yFN.css` e `index-Dqs-46R1.js`. No hay `.AppImage` final y los paquetes deb/rpm revisados no los usan, pero el inventario de `bundle/` parece contener un artefacto publicable que ya no corresponde a la UI congelada. Limpiar `bundle/` antes de publicar o borrar esos directorios.

### REL-F1-04 — P4: warnings y cobertura del gate

Vite mantiene el aviso de chunk >500 kB (753,30 kB; deuda previa) y Tauri emite el aviso de `STATIC_VCRUNTIME` (configuración Windows). Ninguno afecta al binario Linux. Además, `_comprobar_deb`/`_comprobar_rpm` no contrastan la versión con `package.json`/`tauri.conf.json` ni verifican que el binario embebido sea el de `ui/dist`; la comprobación manual de §3.3 cubre esta revisión, pero conviene automatizarla.

## 8. Decisión

**Aprobado con observaciones.** No hay P1 ni P2. Los dos paquetes Linux se generan desde `744aaa5`, contienen la UI nueva embebida, la versión `0.2.0`, el lanzador y los iconos correctos, y pasan estructura, permisos, md5 y digests. `e2e-tauri` pasa los dos escenarios de escritorio. El tag y el acta son coherentes y el árbol no tiene cambios de código.

La fase puede continuar a **UX/accesibilidad/rendimiento** y a la firma del product owner. El **release público no queda autorizado**: antes deben cerrarse REL-F1-01 (strip/rutas), REL-F1-02 (reproducibilidad/checksums), REL-F1-03 (limpieza de AppImage), la firma/checksums, la prueba de la app instalada en máquina limpia, Windows y la CI remota. Los P3/P4 no bloquean la fase según el plan, pero sí el release público.

**Firma:** Release engineer del proyecto `writer` — 2026-10-22.

## 9. Evidencia

- Logs: `/tmp/fase-1-release/empaquetado.log`, `empaquetado2.log`, `e2e-tauri.log`.
- Artefactos extraídos: `/tmp/fase-1-release/extract/`, `/tmp/fase-1-release/build2/`, `/tmp/fase-1-release/rpmx/`.
- Sonda de UI del paquete: `/tmp/fase-1-release/probe-paquete.mjs` y `run-probe.sh` (no versionadas).
- Hashes de las dos builds: `/tmp/fase-1-release/hashes-build1.txt` y `hashes-build2.txt`.
- Comandos de inmutabilidad: `git diff --stat 744aaa5 -- ui/src e2e` (vacío), `git diff --name-status 744aaa5..HEAD` (solo documentación), `git rev-parse 'fase-1-congelado^{commit}'` = `744aaa5`.
