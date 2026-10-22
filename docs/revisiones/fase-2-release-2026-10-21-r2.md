# Fase 2 — Release engineer (r2)

- **Commit revisado:** `c39dcae53fbee53f6340ec4a8ad09a660d41a150`
- **HEAD del pipeline:** `eef1e48a94e06ce7525b9a553d03c82ac34f6d74`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64 (EndeavourOS; glibc 2.44); Rust 1.99.1; Cargo 1.99.1; pnpm 12.14.0; Node 26.11.1; Tauri CLI 2.11.0; `tauri` 2.12.4; WebKitGTK 4.1 2.54.1; GTK 3.24.52; RPM 6.0.1; GNU ar 2.47; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **aprobado con observaciones** (P1: 0 · P2: 0 · P3: 4 · P4: 5).

## 1. Alcance

Revalidé el release de Linux x86_64 sobre el commit posterior a las correcciones de UX y al cambio de recuento del núcleo. Leí el informe de release anterior, el acta r13, la iteración 13, el informe UX r5, `HANDOFF.md`, `README.md`, `docs/empaquetado.md`, `docs/ci.md`, `tauri.conf.json`, `capabilities/default.json`, `Cargo.toml` y `package.json`.

No abrí la app ni ejecuté `e2e-tauri`, conforme a la restricción. Para el comportamiento de escritorio uso la evidencia de UX r5 (9/9 E2E de navegador) y QA r3 (E2E de escritorio), no una ejecución propia. Tampoco instalé los paquetes en el sistema: la revisión de instalación sigue siendo estática.

El build se ejecutó con `nice -n 19` y en segundo plano. La primera ejecución recompiló `writer-desktop`; la segunda, tras limpiar los artefactos de `writer-core` y `writer-desktop`, recompiló ambos y volvió a enlazar. No se modificó código, tests, configuración, lockfiles ni artefactos versionados.

Comprobaciones de estado y comandos principales:

```text
git status --porcelain                                      vacío
git rev-parse HEAD                                          eef1e48a94e06ce7525b9a553d03c82ac34f6d74
git diff --stat c39dcae..HEAD                               6 archivos, solo documentación
python3 scripts/ci.py empaquetado                            OK (24,9 s y 28,0 s)
pnpm install --frozen-lockfile --offline                     OK (21 ms, lockfile al día)
cargo metadata --locked (raíz y desktop)                     OK
ar / tar / rpm / sha256sum / readelf / objdump / ldd          inspección de artefactos
desktop-file-validate                                        OK
```

`git diff --stat c39dcae..HEAD` **no** queda limitado a `docs/revisiones/`: incluye también `HANDOFF.md` y `README.md` (actualizaciones de estado). No hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD. Lo registro como REL-07, igual que en la revalidación anterior.

## 2. Build y artefactos

### 2.1 Gate de empaquetado

`python3 scripts/ci.py empaquetado` ejecuta `pnpm build` → `pnpm --dir apps/desktop build` → `tauri build` y después `_comprobar_deb()` y `_comprobar_rpm()`. Ambas ejecuciones terminaron en verde:

```text
== empaquetado: OK en 24.9 s ==
Empaquetado: writer_0.2.0_amd64.deb verificado (estructura, permisos, md5)
Empaquetado: writer-0.2.0-1.x86_64.rpm verificado (estructura, permisos, digests)
```

La UI se regeneró durante el build (`tsc --noEmit && vite build`, 106 módulos). Los assets producidos son `index-rmqd5UR4.js` (734.260 B; 231,56 kB gzip), `index-CP6axgH7.css` (18.021 B; 4,36 kB gzip) y `window-BOvJm1qw.js` (15.163 B). Coinciden con los que cita UX r5, así que el paquete incluye exactamente la UI revisada.

### 2.2 Artefactos finales

| Artefacto | Tamaño | SHA-256 |
|---|---:|---|
| `bundle/deb/writer_0.2.0_amd64.deb` | 5.171.448 B | `5c6502d987bc73daae848f83ac0993f7514da0ac16464e1182ba2c5774f05e83` |
| `bundle/rpm/writer-0.2.0-1.x86_64.rpm` | 5.171.439 B | `15a907b738dba01e18ac56021786d27ce96b0bd5a312ad4c2292650de05a29f9` |
| `writer-desktop` (target, estado final `UNK`) | 16.115.584 B | `e33cc0fa7ebf9838d7519715caea9970534ebdd8325a18422acb1fb20fea98d0` |

Dentro de los paquetes el binario tiene 16.115.584 B. Como en el release anterior, Tauri parchea tres bytes con el tipo de bundle: `.deb` → `5bfa82b6d773f1633f2b3ef9cf33c8a1b7ad19a5ba5897492e3ba9d3229a9086` (marca `DEB`), `.rpm` → `31a7217cc7039027eafbb15e064c02e1fcb3891e717afaa2c2351c5ae111b390` (marca `RPM`). `cmp -l` entre ambos da exactamente 3 bytes de diferencia, en el offset 1.594.894; el resto es idéntico.

Comparación con el release anterior:

| Artefacto | Release anterior (`b336467`) | Release actual (`c39dcae`) | Diferencia |
|---|---:|---:|---:|
| `.deb` | 5.169.968 B / `2fc36994...` | 5.171.448 B / `5c6502d9...` | +1.480 B |
| `.rpm` | 5.169.969 B / `c28d21cf...` | 5.171.439 B / `15a907b7...` | +1.470 B |
| Binario | 16.113.936 B / `70c43639...` (deb) | 16.115.584 B / `5bfa82b6...` (deb) | +1.648 B |
| UI JS | 732.103 B / `index-D8x7hUWR.js` | 734.260 B / `index-rmqd5UR4.js` | +2.157 B |
| UI CSS | 16.836 B / `index-afwI9ch8.css` | 18.021 B / `index-CP6axgH7.css` | +1.185 B |

El incremento de paquete es coherente con el de los assets y el binario; no hay un salto anómalo de tamaño.

### 2.3 Contenido, lanzador y metadatos

Ambos paquetes contienen los mismos cinco archivos, todos bajo `/usr`:

| Ruta | Permisos | Propietario |
|---|---|---|
| `/usr/bin/writer-desktop` | `-rwxr-xr-x` | root:root |
| `/usr/share/applications/writer.desktop` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/32x32/apps/writer-desktop.png` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/128x128/apps/writer-desktop.png` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/256x256@2/apps/writer-desktop.png` | `-rw-r--r--` | root:root |

- `.deb`: 5 entradas en `md5sums`; las 5 coinciden con `data.tar.gz`. `debian-binary` contiene `2.0\n`. `Installed-Size: 15807`.
- `.rpm`: `writer-0.2.0-1.x86_64`, `License: MIT`, `Size: 16134082`, payload gzip; digests SHA-256 de cabecera y payload correctos.
- `.desktop`: `Exec=writer-desktop`, `Icon=writer-desktop`, `Terminal=false`, `StartupWMClass=writer-desktop`, `Categories=Office;`; `desktop-file-validate` termina 0.
- Iconos PNG válidos de 32×32, 128×128 y 256×256 (RGBA).
- No hay `preinst`/`postinst`/`prerm`/`postrm`, `triggers` ni `conffiles` en el `.deb`; el RPM no tiene scripts ni triggers. No hay enlaces simbólicos, enlaces duros, dispositivos, setuid ni archivos fuera de `/usr`.
- La instalación sigue siendo una copia pura de cinco archivos; desinstalar no toca proyectos ni `.escritor/`.

### 2.4 Firma, updater, AppImage y checksums publicados

| Elemento | Estado |
|---|---|
| AppImage | **Ausente**. No hay `*.AppImage`; siguen los directorios obsoletos `bundle/appimage/` y `bundle/appimage_deb/` (13:50). |
| Firma `.deb` | **Ausente**. El `ar` solo tiene `debian-binary`, `control.tar.gz` y `data.tar.gz`. |
| Firma `.rpm` | **Ausente**. `rpm -qip` muestra `Signature: (none)` y `rpm -Kv` devuelve `NOTFOUND`; los digests SHA-256 de cabecera y payload son correctos. |
| Notarización | No aplica en Linux; no hay artefactos macOS. |
| Updater | **Ausente**. No hay `plugins.updater`, endpoints ni `createUpdaterArtifacts`. |
| Checksums publicados | **Ausentes**. No hay `SHA256SUMS`, `.sha256` ni `.sig` en el bundle ni en el repositorio. |

## 3. Contenido y assets de UI

### 3.1 Assets nuevos y embebidos

| Asset | Tamaño | SHA-256 | Referencia |
|---|---:|---|---|
| `ui/dist/assets/index-rmqd5UR4.js` | 734.260 B | `ee478a74998790bfaae21988af22a97646317c31e52c5ff3d2f2094f26466671` | `ui/dist/index.html` |
| `ui/dist/assets/index-CP6axgH7.css` | 18.021 B | `3cb61a244b8d6d8541df24d66afd79e2620f7555ab4818758347889d4325c6ff` | `ui/dist/index.html` |
| `ui/dist/assets/window-BOvJm1qw.js` | 15.163 B | `86587e0daf368dbf367fbf37d6519a6d0be6d601f46947822fba4ba43b5ab1cb` | chunk de ventana |

Verificaciones:

- `ui/dist/index.html` referencia `index-rmqd5UR4.js` y `index-CP6axgH7.css`.
- El binario del `.deb` contiene los nombres `index-rmqd5UR4.js`, `index-CP6axgH7.css` y `window-BOvJm1qw.js`.
- No contiene los nombres antiguos `index-D8x7hUWR.js` ni `index-afwI9ch8.css`.
- `ui/dist/assets/` solo tiene los tres assets nuevos; no quedan restos de la build anterior.
- Los iconos y el `.desktop` del paquete conservan exactamente los mismos MD5 que en el release anterior; el único archivo del paquete que cambia es el binario (`794ec2bc...` → `ce4e9474...`).

El binario se reconstruyó, por tanto, con la UI nueva; no es el binario de `b336467` con otro nombre.

### 3.2 Zoom, permisos de webview y CSS

- `tauri.conf.json` declara `"zoomHotkeysEnabled": true` en la ventana `main`; el diff solo añade esa propiedad.
- `capabilities/default.json` añade `core:webview:allow-set-webview-zoom`, limitado a la ventana `main`; no añade `fs`, `shell`, `http` ni permisos de red.
- El binario contiene las cadenas `zoomHotkeysEnabled`, `plugin:webview|set_webview_zoom` y `allow-set-webview-zoom`, señal de que la configuración y el permiso se compilaron.
- El CSP no cambió: sigue siendo `default-src 'self'` con `connect-src ipc: http://ipc.localhost` (y el `devCsp` de desarrollo).
- El CSS nuevo se emite y se embebe; el `.desktop` sigue validando. No hay regresión de empaquetado por el zoom ni por los permisos.

**Observación de proceso (no bloqueante):** el permiso de zoom es posterior a security r3, que revisó `b336467`. Su alcance es solo zoom de la ventana principal y no amplía a fs/shell/red, pero conviene que seguridad lo confirme si el pipeline exige que el commit final quede cubierto por esa revisión.

**Deuda conocida (P3, no bloqueo):** Vite emite el warning de chunk >500 kB: `index-rmqd5UR4.js` son 734,26 kB (231,56 kB gzip). Es la deuda ya registrada en el release anterior; el build termina en verde y el asset se empaqueta correctamente. No la cuento como hallazgo nuevo.

## 4. Reproducibilidad y versionado

### 4.1 Binario

Repetí el build tras limpiar los artefactos de `writer-core` y `writer-desktop`, de modo que la segunda ejecución recompiló también el núcleo modificado. El binario extraído del `.deb` es byte a byte idéntico en las dos ejecuciones:

```text
5bfa82b6d773f1633f2b3ef9cf33c8a1b7ad19a5ba5897492e3ba9d3229a9086  build1/.../writer-desktop
5bfa82b6d773f1633f2b3ef9cf33c8a1b7ad19a5ba5897492e3ba9d3229a9086  build2/.../writer-desktop
cmp -l ... | wc -l  →  0
```

Esto confirma que el primer artefacto ya incluía el cambio de `contar_palabras`; la segunda build, que recompiló `writer-core` desde fuente, produjo el mismo binario. La reproducibilidad del código compilado se mantiene.

### 4.2 Paquetes

Los paquetes siguen sin ser byte-reproducibles, por timestamps, no por contenido:

| Comparación | Resultado |
|---|---|
| `.deb` build 1 vs build 2 | `70f8b5cb...` vs `5c6502d9...` (distintos) |
| `.rpm` build 1 vs build 2 | `53ae5383...` vs `15a907b7...` (distintos) |
| `data.tar.gz` del `.deb` | `diff -r` sin diferencias |
| Payload del `.rpm` | `diff -r` sin diferencias |
| Binario del `.deb` | idéntico (ver §4.1) |
| Metadatos variables | mtimes `ar`/tar (20:36 vs 20:37) y `BUILDTIME` del RPM (`1792632964` vs `1792633035`, 20:36:04 vs 20:37:15) |

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

La versión es coherente entre todos los artefactos Tauri, pero **es la misma que la del release anterior** (`0.2.0` / `0.2.0-1`) pese a que el contenido cambió. Los paquetes se llaman igual y tienen hashes distintos; esto es REL-09.

`pnpm-lock.yaml` y los dos `Cargo.lock` están al día: `pnpm install --frozen-lockfile --offline` termina 0 (21 ms) y `cargo metadata --locked` funciona en raíz y desktop. No hay cambios de lockfiles ni de dependencias entre `b336467` y `c39dcae`; los scripts siguen usando `pnpm` exclusivamente, sin `npm`/`npx`.

## 5. Hallazgos

### REL-01 — [P3] El binario sigue sin strip y conserva rutas absolutas del entorno de build

- **Real:** `file` reporta `not stripped`; el ELF conserva `.symtab`. `strings` encuentra 286 rutas `/home/enrique`: 280 bajo `.cargo/registry/src/...` y 6 bajo `crates/core/src/` (`docx.rs`, `fsutil.rs`, `index.rs`, `model.rs`, `oplog.rs`, `yaml.rs`). El binario creció a 16.115.584 B. No hay claves, tokens ni patrones de secreto.
- **Esperado:** binario de release sin tabla de símbolos y sin rutas del home del builder.
- **Reproducción:** `file`, `readelf -S`, `strings -a writer-desktop | grep -c /home/enrique`, `strip --strip-all` sobre una copia.
- **Impacto:** divulgación del nombre de usuario y de la estructura del proyecto; ~4,7 MB de tamaño evitable; las rutas impiden reproducibilidad entre directorios de build.
- **Corrección:** `[profile.release] strip = "symbols"` y `RUSTFLAGS=--remap-path-prefix=<home>=/build`; añadir la comprobación al gate. Se mantiene respecto al release anterior.

### REL-02 — [P3] Los paquetes siguen sin ser byte-reproducibles

- **Real:** dos builds del mismo commit producen binarios idénticos, pero `.deb` y `.rpm` con hashes distintos (`70f8b5cb...`/`5c6502d9...` y `53ae5383...`/`15a907b7...`). Los payloads extraídos son idénticos; las diferencias son mtimes y `BUILDTIME`. No se aplica `SOURCE_DATE_EPOCH`.
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

### REL-09 — [P3] La versión `0.2.0` no distingue el release nuevo del anterior

- **Real:** el release anterior (`b336467`) produjo `writer_0.2.0_amd64.deb` (`2fc36994...`) y `writer-0.2.0-1.x86_64.rpm` (`c28d21cf...`). El release actual (`c39dcae`) produce los mismos nombres y versiones (`0.2.0` / `0.2.0-1`), pero con contenido y hashes distintos (`5c6502d9...` / `15a907b7...`) y un binario 1.648 B mayor. No hay bump de versión, `Release` ni build metadata.
- **Esperado:** cada artefacto que se distribuye tiene una identidad de versión única, o al menos un `Release`/build que permita distinguirlo y actualizarlo.
- **Reproducción:** comparar nombres, `Version`/`Release` y SHA-256 de `bundle/` antes y después del rebuild.
- **Impacto:** si el `0.2.0` anterior se hubiera publicado, un usuario no podría distinguir ambos artefactos por nombre/versión; RPM podría no reemplazar el mismo NEVRA y los checksums publicados serían ambiguos. **No es P2 en este pipeline porque el release anterior no se publicó** (sin remoto, firma ni checksums), pero debe resolverse antes de una distribución pública.
- **Corrección:** subir a `0.2.1` (o `Release: 2`/build metadata) antes de publicar, y asociar los hashes al commit; si se mantiene `0.2.0` como release candidate interna, documentarlo explícitamente.

### Hallazgos P4 heredados, sin cambios

| ID | Hallazgo | Estado en esta revalidación |
|---|---|---|
| REL-04 | Dependencias sin suelo explícito de glibc/baseline de distro | Sin cambios: `GLIBC_2.34` no débil, `GLIBC_2.39` débil; `.deb` solo declara WebKitGTK/GTK. |
| REL-05 | Metadatos de paquete incompletos | Sin cambios: `Maintainer: writer` sin correo, sin `Section`/`Homepage`/copyright/changelog/AppStream, iconos 32/128/256@2, `ar` uid/gid 1000. |
| REL-06 | Cobertura parcial del gate `empaquetado` | Sin cambios: verifica el último `.deb`/`.rpm`, no versión, binario, assets ni instalación congelada. |
| REL-07 | Trazabilidad del congelado | `git diff c39dcae..HEAD` incluye `HANDOFF.md` y `README.md` además de `docs/revisiones/`; el acta r13 registra `HEAD fe4dfa5...` y el HEAD real es `eef1e48...`. |
| REL-08 | Restos de AppImage y warnings de build | Sin cambios: `bundle/appimage*` de 13:50 sin `.AppImage`; warnings de Vite (734,26 kB) y `STATIC_VCRUNTIME` deprecated. |

No hay P1 ni P2. La deuda conocida del bundle de Vite (734,26 kB > 500 kB) y la observación de que el permiso de zoom es posterior a security r3 no se cuentan como hallazgos nuevos; quedan registradas para UX/seguridad y no bloquean el release.

## 6. Conclusión y decisión

**Decisión: aprobado con observaciones.**

El release de Linux x86_64 sobre `c39dcae` construye correctamente, genera `.deb` y `.rpm` con los cinco archivos esperados, versión coherente con la configuración, lanzador e iconos válidos, permisos correctos, `md5sums` del `.deb` correctos y digests SHA-256 del `.rpm` correctos. La UI nueva está realmente embebida: los assets `index-rmqd5UR4.js`, `index-CP6axgH7.css` y `window-BOvJm1qw.js` coinciden con `ui/dist/`, aparecen en el binario y sustituyen a los antiguos. `zoomHotkeysEnabled` y el permiso de zoom se compilaron sin romper el empaquetado; el CSP no cambió y no se añadieron permisos de fs/shell/red. El binario es reproducible entre dos builds, incluida la recompilación del núcleo.

Se mantienen REL-01, REL-02 y REL-03 (P3) y REL-04 a REL-08 (P4). Se añade REL-09 (P3): el artefacto nuevo conserva la versión `0.2.0`/`0.2.0-1` del anterior, aunque el contenido cambia; debe resolverse antes de publicar. No hay P1 ni P2.

Carencias externas que siguen abiertas: AppImage, firma/notarización, updater, Windows/macOS, CI remoto, prueba en máquina limpia y checksums publicados. Con esto, el release puede pasar a la aceptación del product owner; la Fase 2 no se considera aprobada hasta esa firma.
