# Empaquetado — Tauri

- **Fecha:** 2026-10-26
- **Estado:** Linux (deb/rpm) y Windows (MSI/NSIS) en CI; paquetes sin firma
- **Configuración:** `apps/desktop/src-tauri/tauri.conf.json` y
  `tauri.windows.conf.json`

## 1. Configuración

- `bundle.active = true`.
- Iconos: PNG 32/128/256, `.icns` para macOS y `.ico` para Windows.
- Categoría `Productivity`.
- Descripción corta y larga, copyright.
- En Linux, `tauri.linux.conf.json` limita los targets a `deb` y `rpm`.
- En Windows, `tauri.windows.conf.json` limita los targets a `msi` y `nsis`.

```bash
pnpm check     # compila sin bundle
pnpm build     # compila y empaqueta
python scripts/ci.py empaquetado
```

## 2. Artefactos Linux

En este entorno se generan:

```text
target/release/bundle/deb/writer_0.2.0_amd64.deb
target/release/bundle/rpm/writer-0.2.0-1.x86_64.rpm
```

El gate `empaquetado` no solo ejecuta `pnpm build`: inspecciona los paquetes.

- `.deb`: `ar` + `tarfile`, binario ejecutable, `.desktop`, iconos y `md5sums`.
- `.rpm`: `rpm -qplv` para estructura/permisos y `rpm -K --nosignature` para
  digests.
- En Windows, `scripts/ci.py` comprueba que existan MSI/NSIS, que tengan tamaño
  razonable y cabecera PE/OLE válida.

## 3. AppImage

`targets: "all"` intentó generar AppImage y falló porque falta `linuxdeploy`.
Por eso Linux usa solo `deb` y `rpm`. Para habilitar AppImage hay que instalar
`linuxdeploy` y añadir `appimage` a `tauri.linux.conf.json`.

## 4. Windows y Linux

La configuración principal mantiene `targets: "all"`; Tauri elige los formatos
de cada plataforma. El CI compila en Ubuntu 22.04 y Windows Server 2025:

- Linux: `.deb` y `.rpm`.
- Windows: `.msi` (WiX) y `.exe` (NSIS).

En Windows, Tauri descarga WiX/NSIS si no están instalados. El instalador de
NSIS es el recomendado para pruebas manuales; el MSI sirve para despliegues
gestionados.

## 5. Limitaciones

- Sin firma de código ni notarización; Windows mostrará avisos SmartScreen.
- Sin updater.
- AppImage pendiente de `linuxdeploy`.
- El binario incluye la UI embebida; no requiere servidor.
- Los iconos se generaron desde el icono existente con `tauri icon`.
- La instalación en una máquina limpia (sin WebView2/Rust/Node) y la validación
  de Word/LibreOffice siguen pendientes.

## 6. Pruebas

Linux:

```bash
pnpm build
find apps/desktop/src-tauri/target/release/bundle -maxdepth 2 -type f
rpm -qpl apps/desktop/src-tauri/target/release/bundle/rpm/*.rpm
```

Windows (PowerShell):

```powershell
pnpm build
Get-ChildItem apps/desktop/src-tauri/target/release/bundle -Recurse -File
```
