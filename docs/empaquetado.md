# Empaquetado — Tauri

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Configuración:** `apps/desktop/src-tauri/tauri.conf.json`

## 1. Configuración

- `bundle.active = true`.
- Iconos: PNG 32/128/256, `.icns` para macOS y `.ico` para Windows.
- Categoría `Productivity`.
- Descripción corta y larga, copyright.
- En Linux, `tauri.linux.conf.json` limita los targets a `deb` y `rpm`.

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

El `.deb` incluye el binario, el `.desktop` y los iconos en
`/usr/share/icons/hicolor/`. El `.rpm` se verificó con `rpm -qpl`.

## 3. AppImage

`targets: "all"` intentó generar AppImage y falló porque falta `linuxdeploy`.
Por eso Linux usa solo `deb` y `rpm`. Para habilitar AppImage hay que instalar
`linuxdeploy` y añadir `appimage` a `tauri.linux.conf.json`.

## 4. Windows y macOS

La configuración principal mantiene `targets: "all"`; Tauri elige los formatos
de cada plataforma. No se generaron instaladores en este entorno:

- Windows: NSIS/MSI requieren Windows y las herramientas de Visual Studio.
- macOS: DMG/APP requieren macOS y, para distribución, firma y notarización.

## 5. Limitaciones

- Sin firma de código ni notarización.
- Sin updater.
- AppImage pendiente de `linuxdeploy`.
- El binario incluye la UI embebida; no requiere servidor.
- Los iconos se generaron desde el icono existente con `tauri icon`.

## 6. Pruebas

```bash
pnpm build
find apps/desktop/src-tauri/target/release/bundle -maxdepth 2 -type f
rpm -qpl apps/desktop/src-tauri/target/release/bundle/rpm/*.rpm
```
