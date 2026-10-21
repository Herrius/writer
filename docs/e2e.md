# E2E — Playwright sobre la demo

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Ámbito:** `e2e/` y `playwright.config.ts`

## 1. Alcance

Playwright arranca Vite en `http://127.0.0.1:1420` y prueba la UI en **modo
demo** (sin Tauri). Cubre:

1. abrir la demo, editar, esperar el autosave, cambiar de documento y volver
   para comprobar que el texto exacto persiste;
2. búsqueda de `llave` y salto a **otro** documento (`La llave`), comprobando
   título y cuerpo;
3. historial y detalle de una revisión;
4. aplicar personaje, sustituirlo por trama y comprobar que no se anidan;
5. cuarentena visible y descarte en dos pasos;
6. exportación DOCX (diálogo `prompt` simulado).

No prueba comandos Tauri ni escritura real en disco. Esa parte sigue pendiente
con `tauri-driver`/WebdriverIO.

## 2. Comandos

```bash
pnpm e2e:install   # descarga Chromium de Playwright (una vez)
pnpm e2e           # 6 pruebas
python scripts/ci.py e2e
```

Requisitos: pnpm 12 y Chromium de Playwright. No se usa `npm`/`npx`.

## 3. Configuración

- `playwright.config.ts` define `webServer` con `pnpm --dir ui dev`.
- `reuseExistingServer` está activo fuera de CI.
- Un trabajador, Chromium headless, trazas solo en fallo.
- Los artefactos de `test-results/` y `playwright-report/` están ignorados.

## 4. Limitaciones

- La demo mantiene estado en memoria; recargar la página lo reinicia.
- No se prueba el diálogo nativo de Tauri ni el filesystem real.
- No hay E2E de Tauri todavía.
- Chromium de Playwright se descarga fuera del repositorio.

## 5. Pendiente

- E2E de escritorio con `tauri-driver` y WebdriverIO (Linux) o WebView2 (Windows).
- Probar el flujo real de abrir proyecto, guardar, exportar y descartar cuarentena.
