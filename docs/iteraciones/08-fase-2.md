# Iteración 08 — Fase 2 (E2E de navegador y empaquetado)

- **Fecha:** 2026-10-21
- **Alcance:** Playwright sobre la demo y habilitación del bundle de Tauri con
  instaladores Linux.
- **Estado:** E2E de navegador y empaquetado deb/rpm completados. E2E de Tauri y
  AppImage pendientes.

## 1. E2E con Playwright

- `playwright.config.ts` arranca Vite y prueba Chromium headless.
- `e2e/demo.spec.ts` cubre 6 flujos:
  1. abrir demo, editar y esperar autosave;
  2. búsqueda y salto al resultado;
  3. historial y detalle de revisión;
  4. marcas de personaje → trama sin anidar;
  5. cuarentena visible y descarte;
  6. exportación DOCX con diálogo simulado.
- Scripts: `pnpm e2e`, `pnpm e2e:install`.
- Gate nuevo `python scripts/ci.py e2e`.
- La demo incluye una copia de cuarentena de ejemplo para poder probar el panel.

## 2. Empaquetado

- `bundle.active=true`.
- Iconos completos generados con `tauri icon` (PNG, `.icns`, `.ico`, Windows
  Store).
- Categoría, descripciones y copyright.
- `tauri.linux.conf.json` limita Linux a `deb` y `rpm`.
- Artefactos generados:

```text
target/release/bundle/deb/writer_0.2.0_amd64.deb
target/release/bundle/rpm/writer-0.2.0-1.x86_64.rpm
```

- El `.deb` incluye binario, `.desktop` e iconos; el `.rpm` se verificó con
  `rpm -qpl`.
- Gate nuevo `python scripts/ci.py empaquetado` (`pnpm build`).
- AppImage falla por falta de `linuxdeploy`; queda documentado y fuera del
  target Linux.

## 3. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | Playwright en modo demo | Valida la UI real sin depender de WebDriver ni de un display. |
| 2 | Chromium de Playwright fuera del repo | No versionar binarios de navegador. |
| 3 | Linux solo deb/rpm | `linuxdeploy` no está disponible; deb/rpm sí se generan y verifican. |
| 4 | Iconos desde el icono existente | Evita introducir un asset nuevo sin revisión visual. |
| 5 | E2E y empaquetado como gates separados | Se pueden ejecutar y diagnosticar por separado. |

## 4. Evidencia

```text
$ pnpm e2e
6 passed (6)

$ pnpm build
Finished 2 bundles:
  .../bundle/deb/writer_0.2.0_amd64.deb
  .../bundle/rpm/writer-0.2.0-1.x86_64.rpm

$ rpm -qpl .../writer-0.2.0-1.x86_64.rpm
/usr/bin/writer-desktop
/usr/share/applications/writer.desktop
/usr/share/icons/hicolor/...

$ cargo test --workspace --all-targets --locked
84/84

$ pnpm ui:test
50/50

$ pnpm typecheck / pnpm check                  OK
$ python scripts/ci.py e2e                     OK
$ python scripts/ci.py empaquetado             OK
```

## 5. Limitaciones

- El E2E es de navegador, no de Tauri: no ejercita comandos, lock, escritura
  atómica ni diálogos nativos.
- AppImage pendiente de `linuxdeploy`.
- Sin firma de código, notarización ni updater.
- No se generaron instaladores Windows/macOS en este entorno.
- Chromium de Playwright no se versiona; hay que instalarlo con
  `pnpm e2e:install`.

## 6. Pendiente

- E2E de escritorio con `tauri-driver`/WebdriverIO o WebView2.
- AppImage y firma/notarización.
- CI remoto con jobs de E2E y empaquetado.
- Prueba visual real de la app empaquetada.
