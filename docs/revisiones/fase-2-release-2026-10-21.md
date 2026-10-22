# Fase 2 — Release engineer

- **Commit revisado:** `b336467726373ea863eb960796f817ae7b8d0b48`
- **HEAD del pipeline:** `4e9304edbf390a8a3b114c9b5aed38beeb953313`
- **Fecha:** 2026-10-21
- **Entorno:** Linux `enrique-ms7d43` 7.2.9-arch1-1 x86_64 (EndeavourOS; glibc 2.44); Rust 1.99.1; Cargo 1.99.1; pnpm 12.14.0; Node 26.11.1; Tauri CLI 2.11.0; `tauri` 2.12.4; WebKitGTK 4.1 2.54.1; GTK 3.24.52; RPM 6.0.1; GNU ar 2.47; Python 3.14.8.
- **Árbol:** limpio al empezar (`git status --porcelain` vacío). Al terminar, el único cambio es este informe sin versionar.
- **Decisión:** **aprobado con observaciones** (P1: 0 · P2: 0 · P3: 3 · P4: 5).

## 1. Alcance y método

Revisé el artefacto congelado, no la justificación del builder. El alcance es el release de Linux x86_64: build, contenido y metadatos de los paquetes, coherencia de versiones, dependencias runtime, instalación/desinstalación lógica, reproducibilidad y carencias de publicación.

No abrí la app de escritorio ni ejecuté `e2e-tauri`, conforme a la restricción de entorno. Para el comportamiento de escritorio uso la evidencia de QA r3 (5/5, sin procesos huérfanos), no una ejecución propia. Tampoco instalé los paquetes en el sistema: la revisión de instalación es estática (contenido, rutas, scripts, permisos y conflictos).

Comprobaciones de estado y comandos principales:

```text
git status --porcelain                                      vacío
git rev-parse HEAD                                          4e9304edbf390a8a3b114c9b5aed38beeb953313
git diff --stat b336467..HEAD                               6 archivos, solo documentación
python3 scripts/ci.py empaquetado                            OK (20,3 s y 20,0 s en dos ejecuciones)
ar t / ar p + tar -tzvf                                      inspección de .deb
rpm -qip / -qplv / -qpR / -qp --scripts / -Kv                inspección de .rpm
sha256sum / md5sum / cmp / readelf / objdump / ldd / strings  artefactos y binario
desktop-file-validate                                        OK
pnpm install --frozen-lockfile --offline                     OK (86 ms, lockfile al día)
cargo metadata --locked (raíz y desktop)                     OK
```

El build se ejecutó con `nice -n 19` y en segundo plano. La primera ejecución partió de un `target/` preexistente: Cargo recompiló `writer-core` y `writer-desktop` y volvió a enlazar; no se reconstruyeron todas las dependencias. Para la prueba de reproducibilidad limpié solo los artefactos de esos dos paquetes (404,8 MiB) y repetí el gate. No se modificó código, tests, configuración, lockfiles ni artefactos versionados.

`git diff --stat b336467..HEAD` **no** queda limitado a `docs/revisiones/`: incluye también `HANDOFF.md` y `README.md` (actualizaciones de estado). No hay cambios de código, tests, configuración ni dependencias entre el commit congelado y HEAD. Lo registro como observación de trazabilidad (REL-07), no como riesgo de release.

## 2. Build y artefactos

### 2.1 Gate de empaquetado

`python3 scripts/ci.py empaquetado` ejecuta `pnpm build` → `pnpm --dir apps/desktop build` → `tauri build` y después `_comprobar_deb()` y `_comprobar_rpm()`. En ambas ejecuciones terminó en verde:

```text
== empaquetado: OK en 20.3 s ==
Empaquetado: writer_0.2.0_amd64.deb verificado (estructura, permisos, md5)
Empaquetado: writer-0.2.0-1.x86_64.rpm verificado (estructura, permisos, digests)
```

La UI se regeneró durante el build (`tsc --noEmit && vite build`, 106 módulos) y el binario referencia los mismos assets con hash que `ui/dist/assets/`: `index-D8x7hUWR.js` y `index-afwI9ch8.css`. No hay UI obsoleta embebida.

### 2.2 Artefactos finales

| Artefacto | Tamaño | SHA-256 |
|---|---:|---|
| `bundle/deb/writer_0.2.0_amd64.deb` | 5.169.968 bytes | `2fc369947e9310f11b0b87780fc7cf3e99a0599431f4d60aacacf838ff060cc7` |
| `bundle/rpm/writer-0.2.0-1.x86_64.rpm` | 5.169.969 bytes | `c28d21cf1226a8acc4a6d323b4e18af908267d3636766268169dfa3bc856520f` |
| `writer-desktop` (target, estado final `UNK`) | 16.113.936 bytes | `f13a9606e81df5529d717244f0eb14b6f4720b48395ab6883116b34e4a414e92` |

Dentro de los paquetes, el binario tiene 16.113.936 bytes en ambos casos, pero no es el mismo archivo: Tauri parchea tres bytes con el tipo de bundle (`DEB` / `RPM`). Los hashes del binario empaquetado son:

| Binario extraído | SHA-256 | Marca |
|---|---|---|
| `.deb` → `usr/bin/writer-desktop` | `70c43639bf534edb586a74afb1c941905a891dcf3f609e0dfc5180f5af3ff302` | `DEB` |
| `.rpm` → `usr/bin/writer-desktop` | `1c2716c4e9ec6846748a2a65d604f27882f7e5cd1a5c929e7fe870a6cdaad17d` | `RPM` |

`cmp -l` entre ambos binarios da exactamente 3 bytes de diferencia, en el offset 1.593.902; el resto es idéntico. El binario de `target/release` termina con la marca `UNK`. No es una divergencia funcional.

Los artefactos previos del árbol (paquetes de 17:08 y binario de 17:40, anteriores al commit congelado) fueron reemplazados por los de 18:57/18:59.

### 2.3 Lanzador, iconos y metadatos

- `.desktop` instalado en `/usr/share/applications/writer.desktop` (202 bytes), `Exec=writer-desktop`, `Icon=writer-desktop`, `Terminal=false`, `Type=Application`, `StartupWMClass=writer-desktop`, `Categories=Office;`. `desktop-file-validate` termina 0.
- Iconos PNG válidos en `/usr/share/icons/hicolor/{32x32,128x128,256x256@2}/apps/writer-desktop.png`; dimensiones verificadas (32×32, 128×128, 256×256, RGBA).
- `.deb`: `Package: writer`, `Version: 0.2.0`, `Architecture: amd64`, `Installed-Size: 15806`, `Priority: optional`, `Depends: libwebkit2gtk-4.1-0, libgtk-3-0`.
- `.rpm`: `writer-0.2.0-1.x86_64`, `License: MIT`, `Size: 16132434`, `Build Host: (none)`, payload gzip; `Requires: libwebkit2gtk-4.1.so.0()(64bit), libgtk-3.so.0()(64bit)` más los `rpmlib(...)` habituales.
- El `.deb` no declara `Section`, `Homepage` ni `Maintainer` con formato `Nombre <correo>`; el RPM no declara `URL`, `Vendor` ni `Packager`. No hay AppStream/metainfo, copyright, changelog ni man page instalados.

### 2.4 Firma, updater, AppImage y checksums publicados

| Elemento | Estado |
|---|---|
| AppImage | **Ausente**. No hay `*.AppImage`; quedan directorios obsoletos `bundle/appimage/` y `bundle/appimage_deb/` de un intento anterior (13:50). |
| Firma del `.deb` | **Ausente**. El archivo `ar` solo contiene `debian-binary`, `control.tar.gz` y `data.tar.gz`; no hay `_gpgorigin`. |
| Firma del `.rpm` | **Ausente**. `rpm -Kv` devuelve `Header OpenPGP ... NOTFOUND` y `Signature: (none)`; los digests SHA-256 de cabecera y payload son correctos. |
| Notarización | No aplica en Linux; no hay artefactos macOS. |
| Updater | **Ausente**. No hay `plugins.updater`, endpoints ni `createUpdaterArtifacts` en `tauri.conf.json`; el crate/plugin no está en `Cargo.toml`. |
| Checksums publicados | **Ausentes**. No hay `SHA256SUMS`, `.sha256` ni `.sig` en el bundle ni en el repositorio. Solo existen los `md5sums` internos del `.deb` y los digests internos del `.rpm`. |

## 3. Contenido de los paquetes

Ambos paquetes contienen exactamente los mismos cinco archivos, todos bajo `/usr`:

| Ruta | Tipo / permisos | Propietario |
|---|---|---|
| `/usr/bin/writer-desktop` | `-rwxr-xr-x` | root:root |
| `/usr/share/applications/writer.desktop` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/32x32/apps/writer-desktop.png` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/128x128/apps/writer-desktop.png` | `-rw-r--r--` | root:root |
| `/usr/share/icons/hicolor/256x256@2/apps/writer-desktop.png` | `-rw-r--r--` | root:root |

Verificaciones:

- `.deb`: `md5sums` con 5 entradas; las 5 coinciden con el contenido extraído de `data.tar.gz`. `debian-binary` contiene `2.0\n`. No hay enlaces simbólicos, enlaces duros, dispositivos, setuid ni archivos escribibles por todos.
- El `control.tar.gz` solo contiene `control` y `md5sums`: no hay `preinst`, `postinst`, `prerm`, `postrm`, `triggers` ni `conffiles`.
- El `.rpm` no tiene scripts (`rpm -qp --scripts` vacío), ni triggers, ni conflictos, ni obsoletos, ni archivos marcados como configuración o documentación. `rpm -qplv` muestra los cinco archivos con propietario root:root y permisos correctos.
- No hay contenido fuera de `/usr`: el `data.tar.gz` solo tiene entradas bajo `usr/`; el RPM solo lista rutas `/usr/...`.
- El lanzador apunta al binario instalado y el icono referenciado existe con el nombre correcto.

## 4. Reproducibilidad y versionado

### 4.1 Binario

Repetí el build desde el mismo commit, misma ruta y mismo toolchain, tras limpiar los artefactos de `writer-core` y `writer-desktop`. El binario extraído del `.deb` de la segunda ejecución es **byte a byte idéntico** al de la primera:

```text
70c43639bf534edb586a74afb1c941905a891dcf3f609e0dfc5180f5af3ff302  build1/extracted-deb/usr/bin/writer-desktop
70c43639bf534edb586a74afb1c941905a891dcf3f609e0dfc5180f5af3ff302  build2/extracted-deb/usr/bin/writer-desktop
cmp -l ... | wc -l  →  0
```

Esto es una reproducibilidad razonable del código compilado. No hice un build desde cero de todas las dependencias de Tauri; la prueba cubre la recompilación de los crates del producto y el enlazado, que es donde se introducen los cambios del commit.

### 4.2 Paquetes

Los paquetes **no** son byte-reproducibles, pero la diferencia es de empaquetado, no de contenido:

| Comparación | Resultado |
|---|---|
| SHA-256 del `.deb` build 1 vs build 2 | `1019b277...` vs `2fc36994...` (distintos) |
| SHA-256 del `.rpm` build 1 vs build 2 | `49ddaa83...` vs `c28d21cf...` (distintos) |
| `data.tar.gz` extraído del `.deb` | `diff -r` sin diferencias |
| Payload extraído del `.rpm` | `diff -r` sin diferencias |
| Binario del `.deb` | idéntico (ver §4.1) |
| Metadatos variables | mtimes de `ar`/tar (18:57 vs 18:59) y `BUILDTIME` del RPM (`1792627068` vs `1792627162`, 18:57:48 vs 18:59:22) |

En esta configuración no se aplica `SOURCE_DATE_EPOCH` ni se normalizan los timestamps de `ar`, `tar` o de la cabecera RPM. Por tanto:

- **No es un fallo de build**: el contenido instalable es idéntico.
- **Sí es una limitación de release**: no se puede verificar un artefacto publicado reconstruyéndolo y comparando el hash del paquete; hace falta publicar checksums o normalizar timestamps.

### 4.3 Coherencia de versión

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

La numeración dual es intencionada: el motor y la CLI de la Fase 1 siguen en 0.1.0 y la app de escritorio de la Fase 2 está en 0.2.0; el desktop depende de `writer-core` 0.1.0. No hay desalineación dentro del producto Tauri ni entre `tauri.conf.json`, `Cargo.toml` y los paquetes. Conviene documentarlo para que no parezca un error en una revisión futura.

### 4.4 Lockfiles, packageManager y scripts

- `pnpm-lock.yaml` (`lockfileVersion: 9.0`) está versionado y al día: `pnpm install --frozen-lockfile --offline` termina 0 en 86 ms y responde `Lockfile is up to date`.
- `Cargo.lock` de la raíz y `apps/desktop/src-tauri/Cargo.lock` están versionados; `cargo metadata --locked` funciona en ambos. No hay dependencias Git ni tarballs externos en ninguno de los dos ecosistemas.
- `packageManager` es `pnpm@12.14.0`; la versión instalada es 12.14.0. Tauri CLI está fijado en `@tauri-apps/cli` 2.11.0 y `tauri`/`tauri-build` están fijados con `=` en Cargo.
- Los scripts de build usan `pnpm` exclusivamente. `git grep` no encuentra `npm run`, `npm install`, `npm ci` ni `npx` en scripts, configuración o documentación operativa; las únicas menciones son los mensajes del gate que prohíben su uso.
- El build no depende de `npm`/`npx`. Sí depende de que `node_modules` exista: `scripts/ci.py empaquetado` no ejecuta `pnpm install --frozen-lockfile`; esa comprobación vive en el gate `interfaz`. En este entorno el lockfile estaba al día, pero el gate de release no lo garantiza por sí solo (REL-03/REL-06).

## 5. Instalación y dependencias

### 5.1 Instalación/desinstalación lógica

- La instalación es una copia pura de cinco archivos bajo `/usr`; no se ejecutan scripts ni se crean usuarios, servicios, unidades systemd, reglas udev o archivos en `/etc`, `/var`, `/opt` o `/home`.
- La desinstalación de cualquiera de los dos paquetes elimina esos cinco archivos. No hay datos de usuario ni configuración del proyecto dentro del paquete, así que desinstalar no borra proyectos ni `.escritor/`. Es el comportamiento correcto.
- No hay `Conflicts`, `Replaces`, `Obsoletes` ni `Provides` especiales. El nombre `writer` es genérico; si en el futuro se publica también la CLI como paquete, conviene decidir nombres (`writer` vs `writer-desktop`) y conflictos. Hoy no hay colisión conocida en este entorno.
- Los dos paquetes instalan las mismas rutas; instalarlos simultáneamente produciría colisión de archivos, pero es un escenario impropio (se elige deb o rpm).
- Los miembros del archivo `ar` del `.deb` aparecen con uid/gid 1000/1000, no 0/0. `dpkg` no usa ese propietario para instalar (el payload sí es root:root), pero es una señal de empaquetado no normalizado (REL-05).

### 5.2 Dependencias runtime

`ldd` resuelve todas las bibliotecas en el host de build; no hay `not found` ni `RPATH`/`RUNPATH`. Dependencias directas (`DT_NEEDED`): `libgdk-3`, `libgdk_pixbuf-2.0`, `libcairo`, `libgobject-2.0`, `libglib-2.0`, `libdbus-1`, `libwebkit2gtk-4.1`, `libgtk-3`, `libsoup-3.0`, `libgio-2.0`, `libjavascriptcoregtk-4.1`, `libgcc_s`, `libm`, `libc` y `ld-linux-x86-64`.

Las dependencias declaradas son suficientes en Debian/Ubuntu actuales porque `libwebkit2gtk-4.1-0` arrastra `libjavascriptcoregtk-4.1-0` y `libsoup-3.0-0`, y `libgtk-3-0` arrastra GDK/GLib/Pango/Cairo. El RPM declara los sonames de WebKitGTK y GTK; el resto llega por transitividad. No se declara `libayatana-appindicator3` porque la app no usa tray (`tauri` sin features de tray).

Suelo real de glibc: el binario requiere `GLIBC_2.34` como versión no débil; `GLIBC_2.39` aparece solo como `WEAK` por `pidfd_spawnp`/`pidfd_getpid`, que Rust enlaza de forma opcional. El suelo efectivo es, por tanto, glibc 2.34: debería arrancar en Ubuntu 22.04 (2.35) y Debian 12 (2.36), pero no en sistemas con glibc < 2.34. El `.deb` no declara `libc6 (>= 2.34)` y el RPM no declara una versión de glibc; en una distro antigua la instalación podría completarse y el binario fallar al arrancar (REL-04, P4).

No se probó la app instalada desde el `.deb`/`.rpm` ni en una máquina limpia; la comprobación de `ldd` es del host de build y no sustituye esa prueba.

## 6. Carencias externas

Se registran como observaciones aceptadas, no como bloqueos de esta revisión:

| Carencia | Estado / impacto |
|---|---|
| AppImage | No generado; `linuxdeploy` no está disponible. Quedan restos de un intento anterior en `bundle/appimage*` (REL-08). |
| Firma de código y notarización | Sin firma GPG/OpenPGP en deb/rpm; sin notarización (no hay artefactos macOS). Los checksums internos sí son válidos, pero no hay cadena de confianza de publicación. |
| Updater | No hay plugin, endpoints ni artefactos de actualización. |
| Windows y macOS | No hay NSIS/MSI/DMG/APP; requieren sus toolchains y, en macOS, firma/notarización. |
| CI remoto | No existe `.github/` ni remoto; todos los gates son locales. |
| Prueba en máquina limpia | No se instaló el paquete en un sistema limpio ni se probaron los diálogos nativos; QA r3 cubrió la app de desarrollo con E2E de escritorio, no la app instalada. |
| Checksums publicados | No hay `SHA256SUMS` ni firma de artefactos; solo los checksums internos de cada formato. |
| Matriz de distros | No hay baseline declarado (Ubuntu 22.04+, Debian 12+, Fedora...); el paquete se generó en EndeavourOS/Arch con glibc 2.44. |

## 7. Hallazgos

### REL-01 — [P3] El binario de release no se strip-ea y conserva rutas absolutas del entorno de build

- **Real:** `file` reporta `not stripped`; el ELF conserva `.symtab`. `strings` encuentra 285 rutas `/home/enrique`: 280 bajo `/home/enrique/.cargo/registry/src/...` y 5 bajo `/home/enrique/Documentos/software de escritura/writer/crates/core/src/` (`docx.rs`, `fsutil.rs`, `index.rs`, `oplog.rs`, `yaml.rs`). No hay claves, tokens ni patrones de secreto. Una copia con `strip --strip-all` baja de 16.113.936 a 11.359.880 bytes, pero las 285 rutas siguen presentes.
- **Esperado:** binario de release sin tabla de símbolos y sin rutas del home del builder.
- **Reproducción:** `file`, `readelf -S`, `strings -a writer-desktop | grep -c /home/enrique`, `strip --strip-all` sobre una copia.
- **Impacto:** divulgación del nombre de usuario y de la estructura del proyecto; +4,75 MB de artefacto; las rutas absolutas también impiden que el binario sea reproducible entre directorios de build distintos.
- **Corrección:** añadir `[profile.release] strip = "symbols"` en `apps/desktop/src-tauri/Cargo.toml` y compilar con `RUSTFLAGS=--remap-path-prefix=<home>=/build` (o en un directorio neutral); añadir al gate una comprobación de `strings` para rutas y secretos.

### REL-02 — [P3] Los paquetes no son byte-reproducibles; solo el binario lo es

- **Real:** dos builds del mismo commit y misma ruta producen binarios idénticos, pero hashes de paquete distintos: `.deb` `1019b277...` → `2fc36994...`; `.rpm` `49ddaa83...` → `c28d21cf...`. Los payloads extraídos son idénticos (`diff -r` vacío); las diferencias son mtimes de `ar`/tar y `BUILDTIME` del RPM. No hay `SOURCE_DATE_EPOCH` ni normalización de timestamps.
- **Esperado:** o paquetes reproducibles byte a byte, o una limitación documentada y checksums publicados que permitan verificar el artefacto distribuido.
- **Reproducción:** ejecutar `python3 scripts/ci.py empaquetado` dos veces y comparar `sha256sum` de los paquetes, `diff -r` de los payloads y `rpm -qp --qf '%{BUILDTIME}'`.
- **Impacto:** un tercero no puede verificar el paquete reconstruyéndolo; sin `SHA256SUMS` publicado, la integridad depende solo de los digests internos.
- **Corrección:** soportar `SOURCE_DATE_EPOCH` o normalizar mtimes en un paso posterior; publicar `SHA256SUMS` junto a los artefactos; documentar que el binario es reproducible y el paquete no.

### REL-03 — [P3] Toolchain flotante y gate de release que no fuerza lockfiles ni instalación congelada

- **Real:** `rust-toolchain.toml` usa `channel = "stable"` (hoy 1.99.1), no una versión fija. `tauri build` no pasa `--locked` y `scripts/ci.py empaquetado` ejecuta `pnpm build` sin `pnpm install --frozen-lockfile`; el gate confía en el `node_modules` y el `target/` preexistentes. Ahora mismo los lockfiles están al día (`cargo metadata --locked` OK; `pnpm install --frozen-lockfile --offline` OK), así que no hay drift actual.
- **Esperado:** toolchain de release fijado y build que instale/resuelva exclusivamente desde los lockfiles congelados.
- **Reproducción:** leer `rust-toolchain.toml`, `scripts/ci.py` (`empaquetado`) y `apps/desktop/package.json`; comparar dos builds en máquinas con `stable` distinto.
- **Impacto:** un build futuro puede usar otro compilador o dependencias distintas de las revisadas sin que el gate lo detecte; la reproducibilidad depende del estado del workspace, no solo del commit.
- **Corrección:** fijar `channel = "1.99.1"` (o el toolchain del release), ejecutar `pnpm install --frozen-lockfile` al inicio del gate y forzar `cargo --locked` (por ejemplo `CARGO_NET_OFFLINE=true` o el flag equivalente de Tauri).

### REL-04 — [P4] La declaración de dependencias no fija el suelo real de glibc ni un baseline de distro

- **Real:** el binario exige `GLIBC_2.34` como versión no débil; `GLIBC_2.39` solo aparece como `WEAK`. El `.deb` declara solo `libwebkit2gtk-4.1-0, libgtk-3-0`; el RPM solo los dos sonames; ninguno declara `libc6 (>= 2.34)` ni una versión mínima de WebKitGTK. En Ubuntu 22.04 (2.35) y Debian 12 (2.36) funciona; en sistemas con glibc < 2.34 la instalación podría completarse y el arranque fallar.
- **Esperado:** dependencias que expresen el suelo real y una matriz de distros soportadas documentada.
- **Reproducción:** `readelf -V writer-desktop | grep GLIBC_2.34`, `ar p ... control.tar.gz | tar -xzO control`, `rpm -qpR`.
- **Impacto:** fallo en tiempo de ejecución en distros antiguas, sin diagnóstico de instalación; falta de baseline para el product owner.
- **Corrección:** añadir `libc6 (>= 2.34)` al control de Debian, declarar la versión mínima de WebKitGTK y documentar la matriz (por ejemplo Ubuntu 22.04+/Debian 12+); construir en el contenedor de la distro más antigua soportada.

### REL-05 — [P4] Metadatos de paquete incompletos para distribución

- **Real:** `.deb`: `Maintainer: writer` sin correo, sin `Section` ni `Homepage`, sin `/usr/share/doc/writer/copyright` ni changelog. `.rpm`: `Group: Unspecified`, sin `URL`, `Vendor` ni `Packager`, sin archivo de licencia. No hay AppStream/metainfo. El `.desktop` no lleva `Version`, `Keywords` ni `MimeType` y usa solo `Categories=Office;`. Solo se instalan iconos 32, 128 y 256@2 (sin 48/64/512 ni scalable). Los miembros `ar` tienen uid/gid 1000/1000.
- **Esperado:** metadatos conformes a la política de Debian/Fedora y metadatos de escritorio completos.
- **Reproducción:** leer `control` y `rpm -qip`; `desktop-file-validate`; listar iconos y miembros `ar`.
- **Impacto:** avisos de lintian/rpmlint, ausencia en catálogos/centros de software y presentación pobre; no impide instalar ni ejecutar.
- **Corrección:** completar `Maintainer`/`Section`/`Homepage`, instalar `copyright` y changelog, añadir `URL`/`Vendor` y AppStream, ampliar iconos y normalizar propietarios del `ar`.

### REL-06 — [P4] Cobertura parcial del gate `empaquetado`

- **Real:** el gate compila y verifica el último `.deb` y el último `.rpm` (`paquetes[-1]`), pero no comprueba la versión contra `package.json`/`tauri.conf.json`, no inspecciona el binario (strip, rutas, secretos, GLIBC), no verifica que se haya ejecutado `pnpm install --frozen-lockfile`, asume `data.tar.gz` y solo exige que haya ≥3 PNG sin validar dimensiones. No tiene caso negativo.
- **Esperado:** un gate que cubra todos los artefactos y los invariantes de release, con al menos una prueba de fallo.
- **Reproducción:** leer `scripts/ci.py` (`_comprobar_deb`, `_comprobar_rpm`, `empaquetado`) y ejecutar el gate con un paquete manipulado.
- **Impacto:** una regresión de versión, un binario con rutas o un paquete incompleto podría pasar el gate.
- **Corrección:** validar versión contra la configuración, recorrer todos los paquetes, comprobar `strings`/`readelf`, validar dimensiones de iconos y añadir un fixture negativo.

### REL-07 — [P4] Trazabilidad del congelado: el diff posterior no se limita a `docs/revisiones/`

- **Real:** `git diff --stat b336467..HEAD` muestra `HANDOFF.md`, `README.md`, `docs/revisiones/README.md`, el acta r6, el acta r7 y el informe de seguridad r3; no hay código, tests, configuración ni dependencias. El acta r7 registra `HEAD 2297d4a...`, mientras que el HEAD real del pipeline es `4e9304e...` (el acta se commiteó después). `git status` está limpio.
- **Esperado:** que el diff posterior al commit congelado se limite a los documentos de revisión, o que el acta registre el HEAD final y explique las actualizaciones de estado.
- **Reproducción:** `git diff --stat b336467..HEAD`; comparar con el encabezado del acta r7.
- **Impacto:** trazabilidad ligeramente ambigua; no hay riesgo de código porque los cambios son de estado.
- **Corrección:** registrar el HEAD final en el acta o en un manifiesto de congelado; mantener `HANDOFF.md`/`README.md` fuera del commit de congelado o documentar su actualización.

### REL-08 — [P4] Restos de AppImage y warnings de build en el árbol de artefactos

- **Real:** `bundle/appimage/` y `bundle/appimage_deb/` existen desde las 13:50, pero no hay ningún `*.AppImage`. El build emite un warning de Vite por el chunk `index-D8x7hUWR.js` (732,10 kB > 500 kB) y el warning `STATIC_VCRUNTIME is deprecated` de `tauri-build`.
- **Esperado:** árbol de bundle limpio o restos documentados; warnings triados.
- **Reproducción:** `find bundle -maxdepth 2`; revisar `/tmp/opencode/release/build2.log`.
- **Impacto:** inventario de artefactos confuso (parece que hay AppImage cuando no lo hay); los warnings no afectan al binario Linux.
- **Corrección:** limpiar `bundle/` antes del gate o borrar los restos de AppImage; revisar el warning de Vite y el de `staticVCRuntime` (Windows) en la configuración de Tauri.

## 8. Conclusión y decisión

**Decisión: aprobado con observaciones.**

El release de Linux x86_64 construye desde el commit congelado, genera `.deb` y `.rpm` con el contenido esperado, versión 0.2.0 coherente, lanzador e iconos válidos, permisos correctos, `md5sums` del `.deb` correctos y digests SHA-256 del `.rpm` correctos. En la prueba realizada —recompilando los crates del producto, no todas las dependencias de Tauri— el binario es reproducible entre dos builds del mismo commit y resuelve todas sus dependencias en el host. No hay P1 ni P2.

Quedan tres P3 que deben cerrarse antes de una distribución pública, no para avanzar en el pipeline:

1. **REL-01:** strip y `--remap-path-prefix` para eliminar símbolos y rutas del builder.
2. **REL-02:** normalizar timestamps o publicar `SHA256SUMS`; documentar que el paquete no es byte-reproducible aunque el payload sí.
3. **REL-03:** fijar el toolchain y hacer que el gate instale desde el lockfile y compile con `--locked`.

Los P4 (REL-04 a REL-08) son de metadatos, cobertura de gate, trazabilidad y limpieza; no bloquean la fase. Las carencias externas de §6 (AppImage, firma/notarización, updater, Windows/macOS, CI remoto y prueba en máquina limpia) siguen abiertas y deben registrarse en la aceptación del product owner.

Con esto, el pipeline puede pasar a **UX/accesibilidad/rendimiento** y después a la aceptación del product owner. La Fase 2 no se considera aprobada hasta esa firma.
