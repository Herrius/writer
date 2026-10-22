# Fase 2.1 — Release engineer

- **Commit revisado:** `e0bb3fc3f8f2cbc7a12a7c8ff5f586adac51a002`
- **HEAD del pipeline:** `9f87800b8348793d294bee05dbd1d9c6b2f6bd42`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64 (EndeavourOS; glibc 2.44); Rust 1.99.1; Cargo 1.99.1; pnpm 12.14.0; Node 26.11.1; Tauri CLI 2.11.0; `tauri` 2.12.4; WebKitGTK 4.1 2.54.1; GTK 3.24.52; RPM 6.0.1; GNU ar 2.47; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **aprobado con observaciones** (P1: 0 · P2: 0 · P3: 4 · P4: 5).

## 1. Alcance

Revalidé el paquete de la Fase 2.1 (panel de inicio y creación de proyectos) sobre el commit congelado, con el mismo alcance que el release r2: Linux x86_64, build local, `.deb`/`.rpm`, contenido, metadatos, dependencias runtime, assets de UI, reproducibilidad y carencias de publicación.

No abrí la app ni ejecuté `e2e-tauri`. Para el flujo de escritorio uso la evidencia del builder y de QA/Security de la Fase 2.1, no una ejecución propia. No instalé los paquetes en el sistema; la revisión de instalación es estática.

Leí el informe de release r2, el acta `fase-2-1-20-congelado-release.md`, la iteración 15, `docs/inicio-v1.md`, el informe de seguridad r3 de la Fase 2.1, `HANDOFF.md`, `docs/empaquetado.md`, `docs/ci.md`, `tauri.conf.json`, `capabilities/default.json`, `Cargo.toml` y `package.json`.

El build se ejecutó con `nice -n 19` y en segundo plano. La primera ejecución compiló `writer-core` y `writer-desktop`; la segunda, tras limpiar los artefactos de ambos, los recompiló y volvió a enlazar. No se modificó código, tests, configuración, lockfiles ni artefactos versionados.

```text
git status --porcelain                                      vacío
git rev-parse HEAD                                          9f87800b8348793d294bee05dbd1d9c6b2f6bd42
git diff --stat e0bb3fc..HEAD                               5 archivos, solo documentación
python3 scripts/ci.py empaquetado                            OK (23,0 s y 25,4 s)
pnpm install --frozen-lockfile --offline                     OK (57 ms, lockfile al día)
cargo metadata --locked (raíz y desktop)                     OK
```

`git diff --stat e0bb3fc..HEAD` **no** queda limitado a `docs/revisiones/`: incluye `HANDOFF.md` además de `docs/revisiones/README.md`, el acta de congelado y el informe de seguridad r3. No hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD. Lo registro como REL-07.

## 2. Build y artefactos

### 2.1 Gate de empaquetado

`python3 scripts/ci.py empaquetado` ejecuta `pnpm build` → `pnpm --dir apps/desktop build` → `tauri build` y después `_comprobar_deb()` y `_comprobar_rpm()`. Ambas ejecuciones terminaron en verde:

```text
== empaquetado: OK en 23.0 s ==  (primera)
== empaquetado: OK en 25.4 s ==  (segunda)
Empaquetado: writer_0.2.0_amd64.deb verificado (estructura, permisos, md5)
Empaquetado: writer-0.2.0-1.x86_64.rpm verificado (estructura, permisos, digests)
```

La UI se regeneró durante el build (`tsc --noEmit && vite build`, 109 módulos) y Vite emitió `index-YQtoJIi0.js` (746,09 kB), `index-BVfPg3Gh.css` (29,36 kB) y `window-CO1JT7GA.js` (15,16 kB). El detalle de assets está en §3.

### 2.2 Artefactos finales

| Artefacto | Tamaño | SHA-256 |
|---|---:|---|
| `bundle/deb/writer_0.2.0_amd64.deb` | 5.252.084 B | `a15bee6ba13637b0b2463a5a71db7c1ef6abe8c126700fda633ad2dac8fcce94` |
| `bundle/rpm/writer-0.2.0-1.x86_64.rpm` | 5.252.306 B | `fabca278266cc3d0229649060b3071ebab10a636d142505e077a2eb2bb4fb411` |
| `writer-desktop` (target, estado final `UNK`) | 16.295.720 B | `33ee877cdb7864c2f83e80788083e6e5c0474a5f2a85d54b78bf23b5dec7e805` |

Dentro de los paquetes el binario tiene 16.295.720 B. Como siempre, Tauri parchea tres bytes con el tipo de bundle: `.deb` → `3a645eaa0ac7bc0876a843a346ca086e3a7d25bf8cad2636ab14d1b435aa433f` (marca `DEB`), `.rpm` → `ba4018221a23ccbfe734e85a4548d0110bce23e4dc935c63240fbc65ecfb2f41` (marca `RPM`). `cmp -l` da exactamente 3 bytes de diferencia, en el offset 1.603.054; el resto es idéntico.

Comparación con el release de la Fase 2:

| Artefacto | Fase 2 (`c39dcae`) | Fase 2.1 (`e0bb3fc`) | Diferencia |
|---|---:|---:|---:|
| `.deb` | 5.171.448 B / `5c6502d9...` | 5.252.084 B / `a15bee6b...` | +80.636 B |
| `.rpm` | 5.171.439 B / `15a907b7...` | 5.252.306 B / `fabca278...` | +80.867 B |
| Binario | 16.115.584 B / `5bfa82b6...` (deb) | 16.295.720 B / `3a645eaa...` (deb) | +180.136 B |
| UI JS | 734.260 B / `index-rmqd5UR4.js` | 746.086 B / `index-YQtoJIi0.js` | +11.826 B |
| UI CSS | 18.021 B / `index-CP6axgH7.css` | 29.361 B / `index-BVfPg3Gh.css` | +11.340 B |

El crecimiento es coherente con el panel de inicio, el asistente y los estilos nuevos; no hay un salto anómalo ni archivos extraños.

### 2.3 Contenido, lanzador y metadatos

Ambos paquetes contienen exactamente los mismos cinco archivos, todos bajo `/usr`:

| Ruta | Permisos | Propietario |
|---|---|---|
| `/usr/bin/writer-desktop` | `-rwxr-xr-x` | root:root |
| `/usr/share/applications/writer.desktop` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/32x32/apps/writer-desktop.png` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/128x128/apps/writer-desktop.png` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/256x256@2/apps/writer-desktop.png` | `-rw-r--r--` | root:root |

- `.deb`: 5 entradas en `md5sums`; las 5 coinciden con `data.tar.gz`. `debian-binary` contiene `2.0\n`. `Installed-Size: 15983`.
- `.rpm`: `writer-0.2.0-1.x86_64`, `License: MIT`, `Size: 16314218`, payload gzip; digests SHA-256 de cabecera y payload correctos.
- `.desktop`: `Exec=writer-desktop`, `Icon=writer-desktop`, `Terminal=false`, `StartupWMClass=writer-desktop`, `Categories=Office;`; `desktop-file-validate` termina 0.
- Iconos PNG válidos de 32×32, 128×128 y 256×256 (RGBA). Los MD5 de iconos y `.desktop` siguen siendo los mismos que en la Fase 2; solo cambia el binario.
- No hay `preinst`/`postinst`/`prerm`/`postrm`, `triggers` ni `conffiles` en el `.deb`; el RPM no tiene scripts ni triggers. No hay enlaces simbólicos, enlaces duros, dispositivos, setuid ni archivos fuera de `/usr`.
- La instalación sigue siendo una copia pura de cinco archivos; desinstalar no toca proyectos ni `.escritor/`.

### 2.4 Firma, updater, AppImage y checksums publicados

| Elemento | Estado |
|---|---|
| AppImage | **Ausente**. No hay `*.AppImage`; siguen los directorios obsoletos `bundle/appimage/` y `bundle/appimage_deb/` (13:50). |
| Firma `.deb` | **Ausente**. El `ar` solo tiene `debian-binary`, `control.tar.gz` y `data.tar.gz`. |
| Firma `.rpm` | **Ausente**. `rpm -Kv` da `NOTFOUND` y `rpm -qip` muestra `Signature: (none)`; los digests SHA-256 son correctos. |
| Notarización | No aplica en Linux; no hay artefactos macOS. |
| Updater | **Ausente**. No hay `plugins.updater`, endpoints ni `createUpdaterArtifacts`. |
| Checksums publicados | **Ausentes**. No hay `SHA256SUMS`, `.sha256` ni `.sig` en el bundle ni en el repositorio. |

## 3. Assets de UI y `crear_proyecto`

### 3.1 Assets del panel de inicio y modal

| Asset | Tamaño | SHA-256 | Referencia |
|---|---:|---|---|
| `ui/dist/assets/index-YQtoJIi0.js` | 746.086 B | `1a8a0846223628af8ba10b41f9af09c489998c7a3382cd6675fef62b37e26135` | `ui/dist/index.html` |
| `ui/dist/assets/index-BVfPg3Gh.css` | 29.361 B | `34026cc724df35f6e209b4b32c9cc41fe53161d33e3bcef6a7ee06a78123b1a0` | `ui/dist/index.html` |
| `ui/dist/assets/window-CO1JT7GA.js` | 15.163 B | `e0996ed4943b723a839283c4e550bb43e8edb57f6ef5bd887c1e5dfebed6a7b0` | chunk de ventana |

Verificaciones:

- `ui/dist/index.html` referencia `index-YQtoJIi0.js` e `index-BVfPg3Gh.css`.
- El binario del `.deb` contiene los nombres `index-YQtoJIi0.js`, `index-BVfPg3Gh.css` y `window-CO1JT7GA.js`.
- No contiene los assets de la Fase 2 (`index-rmqd5UR4.js`, `index-CP6axgH7.css`, `window-BOvJm1qw.js`) ni los de la build intermedia (`index-Bw4YjE7r.js`, `window-C5Pjun9Q.js`).
- El JS compilado contiene los textos del panel y del asistente: `Mis Proyectos`, `Crear nuevo proyecto`, `Nombre del proyecto`, `Guardar y abrir` y `No proyectos`.
- El JS compilado contiene la llamada `crear_proyecto`; `ui/src/api.ts` la invoca y `apps/desktop/src-tauri/src/lib.rs` la registra en `generate_handler!`.
- `ui/dist/assets/` solo tiene los tres assets nuevos; Vite regeneró el directorio durante el build.

**Observación de build:** antes de ejecutar el gate, `ui/dist` tenía una build intermedia (`index-Bw4YjE7r.js`, 745.707 B; `window-C5Pjun9Q.js`) anterior al commit congelado de las 22:57. El gate regeneró los assets y el paquete final contiene `index-YQtoJIi0.js` y `window-CO1JT7GA.js`. Es decir, el paquete se construyó desde el código congelado, no desde `ui/dist` obsoleto.

### 3.2 `crear_proyecto`, permisos y dependencias

- El comando `crear_proyecto` está registrado en `tauri::generate_handler!` y usa `Project::init` y `Project::crear_nota` del núcleo existente; no añade crates, plugins ni bibliotecas dinámicas.
- `git diff --stat c39dcae..e0bb3fc` para `Cargo.toml`, `Cargo.lock`, `package.json`, `pnpm-lock.yaml`, `tauri.conf.json` y `capabilities/default.json` está **vacío**: no hay cambios de dependencias, configuración ni capacidades.
- `capabilities/default.json` sigue limitado a `core:default`, `core:window:allow-destroy`, `core:webview:allow-set-webview-zoom` y `dialog:default`, con `windows: ["main"]`. El nuevo comando es un comando de la aplicación, no un plugin que requiera un permiso adicional.
- `ldd` y `DT_NEEDED` son idénticos a la Fase 2: las mismas 15 bibliotecas (`gtk3`, `webkit2gtk-4.1`, `soup-3.0`, `javascriptcoregtk-4.1`, etc.), todas resueltas, sin `RPATH`/`RUNPATH`. No hay nueva dependencia runtime.
- `tauri.linux.conf.json` mantiene `targets: ["deb", "rpm"]`; el build solo produce `.deb` y `.rpm`, sin AppImage ni instaladores de otras plataformas.

No hay regresión de empaquetado por el comando, el zoom, los permisos ni el CSS. El bundle de Vite sigue superando el umbral de 500 kB (`746,09 kB`, `234,83 kB` gzip); es la deuda P3 conocida, no un fallo de empaquetado.

## 4. Reproducibilidad y versionado

### 4.1 Binario

Repetí el build tras limpiar los artefactos de `writer-core` y `writer-desktop`; la segunda ejecución recompiló ambos. El binario extraído del `.deb` es byte a byte idéntico:

```text
3a645eaa0ac7bc0876a843a346ca086e3a7d25bf8cad2636ab14d1b435aa433f  build1/.../writer-desktop
3a645eaa0ac7bc0876a843a346ca086e3a7d25bf8cad2636ab14d1b435aa433f  build2/.../writer-desktop
cmp -l ... | wc -l  →  0
```

La reproducibilidad del código compilado se mantiene con el nuevo comando y el panel de inicio.

### 4.2 Paquetes

Los paquetes siguen sin ser byte-reproducibles, por timestamps, no por contenido:

| Comparación | Resultado |
|---|---|
| `.deb` build 1 vs build 2 | `d32e14d4...` vs `a15bee6b...` (distintos) |
| `.rpm` build 1 vs build 2 | `73468371...` vs `fabca278...` (distintos) |
| `data.tar.gz` del `.deb` | `diff -r` sin diferencias |
| Payload del `.rpm` | `diff -r` sin diferencias |
| Binario del `.deb` | idéntico (ver §4.1) |
| Metadatos variables | mtimes `ar`/tar y `BUILDTIME` del RPM (`1792641704` vs `1792641766`) |

Se mantiene REL-02: el binario es reproducible, el paquete no; no hay `SOURCE_DATE_EPOCH` ni normalización de timestamps.

### 4.3 Versión y lockfiles

| Fuente | Versión |
|---|---|
| `package.json` raíz | 0.2.0 |
| `ui/package.json` | 0.2.0 |
| `apps/desktop/package.json` | 0.2.0 |
| `apps/desktop/src-tauri/tauri.conf.json` | 0.2.0 |
| `apps/desktop/src-tauri/Cargo.toml` | 0.2.0 |
| `.deb` (`Version`) | 0.2.0 |
| `.rpm` (`Version`/`Release`) | 0.2.0-1 |
| `Cargo.toml` del workspace raíz (core/CLI) | 0.1.0 |

La versión es coherente entre todos los artefactos Tauri, pero **sigue siendo `0.2.0`/`0.2.0-1`**, la misma que la Fase 2 aceptada por el product owner, pese a que el contenido cambia (nuevo panel, asistente, comando y assets). Los paquetes se llaman igual y tienen hashes distintos; es REL-09.

`pnpm-lock.yaml` y los dos `Cargo.lock` están al día: `pnpm install --frozen-lockfile --offline` termina 0 (57 ms) y `cargo metadata --locked` funciona en raíz y desktop. No hay cambios de lockfiles ni de dependencias entre la Fase 2 y este release. Los scripts siguen usando `pnpm` exclusivamente, sin `npm`/`npx`.

## 5. Hallazgos

### REL-01 — [P3] El binario sigue sin strip y conserva rutas absolutas del entorno de build

- **Real:** `file` reporta `not stripped` y el ELF conserva `.symtab`. `strings` encuentra 289 rutas `/home/enrique`: 283 bajo `.cargo/registry/src/...` y 6 bajo `crates/core/src/` (`docx.rs`, `fsutil.rs`, `index.rs`, `model.rs`, `oplog.rs`, `yaml.rs`). El binario mide 16.295.720 B. No hay claves, tokens ni patrones de secreto.
- **Esperado:** binario de release sin tabla de símbolos y sin rutas del home del builder.
- **Reproducción:** `file`, `readelf -S`, `strings -a writer-desktop | grep -c /home/enrique`, `strip --strip-all` sobre una copia.
- **Impacto:** divulgación del nombre de usuario y de la estructura del proyecto; ~4,7 MB evitables; las rutas impiden reproducibilidad entre directorios de build.
- **Corrección:** `[profile.release] strip = "symbols"` y `RUSTFLAGS=--remap-path-prefix=<home>=/build`; añadir la comprobación al gate. Se mantiene respecto al release r2.

### REL-02 — [P3] Los paquetes siguen sin ser byte-reproducibles

- **Real:** dos builds del mismo commit producen binarios idénticos, pero `.deb` y `.rpm` con hashes distintos (`d32e14d4...`/`a15bee6b...` y `73468371...`/`fabca278...`). Los payloads extraídos son idénticos; las diferencias son mtimes y `BUILDTIME`. No se aplica `SOURCE_DATE_EPOCH`.
- **Esperado:** paquetes reproducibles o limitación documentada con `SHA256SUMS` publicados.
- **Reproducción:** ejecutar `python3 scripts/ci.py empaquetado` dos veces y comparar hashes, `diff -r` de payloads y `rpm -qp --qf '%{BUILDTIME}'`.
- **Impacto:** no se puede verificar un paquete publicado reconstruyéndolo; sin checksums publicados, la integridad depende de los digests internos.
- **Corrección:** normalizar timestamps o soportar `SOURCE_DATE_EPOCH`; publicar checksums. Se mantiene.

### REL-03 — [P3] Toolchain flotante y gate de release sin lockfiles forzados

- **Real:** `rust-toolchain.toml` usa `channel = "stable"` (hoy 1.99.1), no una versión fija. `tauri build` no pasa `--locked` y `scripts/ci.py empaquetado` ejecuta `pnpm build` sin `pnpm install --frozen-lockfile`; el gate confía en `node_modules` y `target/` preexistentes. Ahora los lockfiles están al día, así que no hay drift actual.
- **Esperado:** toolchain de release fijado y build que resuelva exclusivamente desde lockfiles congelados.
- **Reproducción:** leer `rust-toolchain.toml` y `scripts/ci.py`; comparar builds con `stable` distinto.
- **Impacto:** un build futuro puede usar otro compilador o dependencias distintas sin que el gate lo detecte.
- **Corrección:** fijar `channel = "1.99.1"` (o el toolchain del release), ejecutar `pnpm install --frozen-lockfile` al inicio y forzar `cargo --locked`. Se mantiene.

### REL-09 — [P3] La versión `0.2.0` no distingue la Fase 2.1 de la Fase 2 aceptada

- **Real:** la Fase 2 aceptada por el product owner produjo `writer_0.2.0_amd64.deb` (`5c6502d9...`) y `writer-0.2.0-1.x86_64.rpm` (`15a907b7...`). Este release de Fase 2.1 produce los mismos nombres y versiones (`0.2.0` / `0.2.0-1`), pero hashes distintos (`a15bee6b...` / `fabca278...`) y un binario 180.136 B mayor. No hay bump de versión, `Release` ni build metadata.
- **Esperado:** cada artefacto que se distribuye tiene una identidad de versión única, o al menos un `Release`/build que permita distinguirlo y actualizarlo.
- **Reproducción:** comparar nombres, `Version`/`Release` y SHA-256 de `bundle/` antes y después del release.
- **Impacto:** si el `0.2.0` de la Fase 2 se hubiera publicado, un usuario no podría distinguir ambos artefactos por nombre/versión; RPM podría no reemplazar el mismo NEVRA y los checksums publicados serían ambiguos. **No es P2 en este pipeline porque el release público sigue sin autorizar** (sin remoto, firma ni checksums), pero debe resolverse antes de publicar.
- **Corrección:** subir a `0.2.1` (o `Release: 2`/build metadata) antes de publicar, y asociar los hashes al commit; si se mantiene `0.2.0` como release candidate interna, documentarlo explícitamente.

### Hallazgos P4 heredados, sin cambios

| ID | Hallazgo | Estado en esta revalidación |
|---|---|---|
| REL-04 | Dependencias sin suelo explícito de glibc/baseline de distro | Sin cambios: `GLIBC_2.34` no débil, `GLIBC_2.39` débil; `.deb` solo declara WebKitGTK/GTK. |
| REL-05 | Metadatos de paquete incompletos | Sin cambios: `Maintainer: writer` sin correo, sin `Section`/`Homepage`/copyright/changelog/AppStream, iconos 32/128/256@2, `ar` uid/gid 1000. |
| REL-06 | Cobertura parcial del gate `empaquetado` | Sin cambios: verifica el último `.deb`/`.rpm`, no versión, binario, assets ni instalación congelada. |
| REL-07 | Trazabilidad del congelado | `git diff e0bb3fc..HEAD` incluye `HANDOFF.md` además de `docs/revisiones/`; el acta de release registra `HEAD 9ba57e2...` y el HEAD real es `9f87800...`. |
| REL-08 | Restos de AppImage y warnings de build | Sin cambios: `bundle/appimage*` de 13:50 sin `.AppImage`; warnings de Vite (746,09 kB) y `STATIC_VCRUNTIME` deprecated. |

No hay P1 ni P2. La deuda conocida del bundle de Vite (746,09 kB > 500 kB) no se cuenta como hallazgo nuevo; queda registrada para UX/rendimiento y no bloquea. Los residuales de seguridad SEC21-02 (TOCTOU) y SEC21-03 (creación parcial) siguen en el informe de seguridad r3; no son regresiones de empaquetado ni P1/P2 de release.

## 6. Conclusión y decisión

**Decisión: aprobado con observaciones.**

El release de Linux x86_64 sobre `e0bb3fc` construye correctamente, genera `.deb` y `.rpm` con los cinco archivos esperados, versión coherente con la configuración, lanzador e iconos válidos, permisos correctos, `md5sums` del `.deb` correctos y digests SHA-256 del `.rpm` correctos. El panel de inicio y el asistente están realmente embebidos: los assets `index-YQtoJIi0.js`, `index-BVfPg3Gh.css` y `window-CO1JT7GA.js` coinciden con `ui/dist/`, aparecen en el binario, el JS contiene los textos del panel y `crear_proyecto`, y no quedan assets antiguos. El comando `crear_proyecto` no añade dependencias runtime, plugins ni permisos/capacidades Tauri; `ldd`, `DT_NEEDED` y las dependencias declaradas son idénticas a la Fase 2. El paquete sigue siendo deb/rpm Linux, y el binario es reproducible entre dos builds.

Se mantienen REL-01, REL-02 y REL-03 (P3), REL-04 a REL-08 (P4) y REL-09 (P3). No hay P1 ni P2.

Carencias externas que siguen abiertas: AppImage, firma/notarización, updater, Windows/macOS, CI remoto, prueba en máquina limpia y checksums publicados. Con esto, el paquete puede pasar a UX y a la aceptación del product owner de la Fase 2.1; el release público sigue sin autorizar.
