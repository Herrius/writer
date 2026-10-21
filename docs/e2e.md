# E2E — navegador y escritorio

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Ámbito:** `e2e/`, `e2e-tauri/`, `playwright.config.ts` y
  `scripts/e2e-tauri.sh`

## 1. E2E de navegador (Playwright)

Playwright arranca Vite en `http://127.0.0.1:1420` y prueba la UI en **modo
demo** (sin Tauri). Cubre 7 flujos:

1. abrir la demo, editar, esperar el autosave, cambiar de documento y volver
   para comprobar el texto exacto;
2. búsqueda de `llave` y salto a **otro** documento, comprobando título y
   cuerpo;
3. aplicar formato y comprobar el recuento de palabras;
4. historial y detalle de una revisión;
5. aplicar personaje, sustituirlo por trama y comprobar que no se anidan;
6. cuarentena visible y descarte en dos pasos;
7. exportación DOCX con diálogo simulado.

```bash
pnpm e2e:install   # Chromium de Playwright (una vez)
pnpm e2e
python scripts/ci.py e2e
```

## 2. E2E de escritorio (Tauri/WebDriver, Linux)

`e2e-tauri/tauri.test.mjs` usa `tauri-driver` + `WebKitWebDriver` y la app real:

1. crea un proyecto temporal con la CLI;
2. arranca la app con `WRITER_E2E_PROJECT` (hook de prueba);
3. abre la primera escena;
4. escribe `Texto E2E Tauri.`;
5. guarda con `Ctrl+S`;
6. verifica el archivo Markdown en disco y el título de la escena.

```bash
cargo install tauri-driver --version 2.0.6 --locked
pnpm e2e:tauri
python scripts/ci.py e2e-tauri
```

Requisitos: Linux, `WebKitWebDriver`, `tauri-driver`, X11
(`GDK_BACKEND=x11`) y las dependencias de Tauri ya necesarias para compilar.

El hook `WRITER_E2E_PROJECT` solo se activa si la variable de entorno existe; en
uso normal la app no abre ningún proyecto automáticamente.

## 3. Limitaciones

- El E2E de navegador usa la demo en memoria; recargar reinicia el estado.
- El E2E de Tauri es Linux-only y no cubre diálogos nativos ni Windows/macOS.
- Chromium de Playwright y `tauri-driver` se instalan fuera del repositorio.
- El E2E de escritorio no prueba DOCX, búsqueda ni cuarentena; esos flujos se
  cubren en navegador.

## 4. Pendiente

- E2E de escritorio en Windows (WebView2) y macOS.
- Un flujo de escritorio que abra un proyecto con el diálogo nativo.
- Empaquetar y probar la app instalada, no solo el binario de `target/`.
