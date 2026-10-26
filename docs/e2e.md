# E2E — navegador y escritorio

- **Fecha:** 2026-10-26
- **Estado:** 61 flujos de navegador en Linux/Windows y 2 escenarios de
  escritorio en Linux y Windows
- **Ámbito:** `e2e/`, `e2e-tauri/`, `playwright.config.ts`,
  `scripts/e2e-tauri.sh` y `scripts/e2e-tauri.ps1`

## 1. E2E de navegador (Playwright)

Playwright arranca Vite en `http://127.0.0.1:1420` y prueba la UI en **modo
demo** (sin Tauri). Cubre los flujos de las fases 1–3, incluidos autosave, búsqueda, historial, cambios, preferencias, paleta, TTS simulado, filtros, colecciones, plantillas, papelera y zoom. Los 12 flujos originales se conservan y la suite creció a 61 casos con las fases 2 y 3. Los primeros son:

1. abrir el asistente de creación, validar el título y explicar el límite de la
   demo;
2. abrir la demo, editar, esperar el autosave, cambiar de documento y volver
   para comprobar el texto exacto;
3. búsqueda de `llave` y salto a **otro** documento, comprobando título y
   cuerpo;
4. aplicar formato y comprobar el recuento de palabras;
5. historial y detalle de una revisión;
6. aplicar personaje, sustituirlo por trama y comprobar que no se anidan;
7. cuarentena visible y descarte en dos pasos;
8. enlace «Saltar al contenido» y navegación del árbol con flechas y
   `tabindex` itinerante;
9. anillo de foco visible en el editor;
10. exportación DOCX con diálogo simulado;
11. vuelta al panel de Proyectos desde el editor;
12. vista de lista compacta, modal a 960×640 y 200 % con trampa de foco,
    cierre con Escape y retorno del foco.

```bash
pnpm e2e:install   # Chromium de Playwright (una vez)
pnpm e2e
python scripts/ci.py e2e
```

## 2. E2E de escritorio (Tauri/WebDriver, Linux)

`e2e-tauri/tauri.test.mjs` usa `tauri-driver` + `WebKitWebDriver` y la app real:

1. crea un proyecto temporal con la CLI;
2. compila la app con la feature `e2e` y la arranca con `WRITER_E2E_PROJECT`;
3. abre la primera escena;
4. escribe `Texto E2E Tauri.`, guarda con `Ctrl+S` y verifica el archivo, el
   estado y el título;
5. en una segunda ejecución limpia, escribe `PENDIENTE VENTANA.` y cierra la
   ventana con `Alt+F4` (X11), comprobando que el handler de Tauri guarda antes
   de destruirla y que la ventana y el proceso desaparecen.

```bash
cargo install tauri-driver --version 2.0.6 --locked
pnpm e2e:tauri
python scripts/ci.py e2e-tauri
```

Requisitos: Linux, `WebKitWebDriver`, `tauri-driver`, X11
(`GDK_BACKEND=x11`), `xdotool` y las dependencias de Tauri ya necesarias para
compilar. El script usa un puerto libre, un grupo de procesos propio y limpia la
app al terminar.

La feature `e2e` solo existe en la compilación de prueba: la app de producción
no autoabre proyectos ni conserva el hook activo.

## 3. E2E de escritorio (WebView2/CDP, Windows)

`scripts/e2e-tauri.ps1` usa **CDP sobre WebView2** y Playwright, sin
`tauri-driver`/`msedgedriver` (que en CI fallan con `DevToolsActivePort`):

1. crea el proyecto temporal con `target/debug/writer.exe`;
2. compila la app debug con la feature `e2e`, `--no-bundle` y
   `tauri.e2e.conf.json`, que habilita `--remote-debugging-port=9222` solo en
   esa build;
3. arranca la app con una carpeta de datos WebView2 aislada;
4. Playwright se conecta por CDP a la página real y ejecuta el escenario
   `Ctrl+S` con verificación del archivo en disco;
5. ejecuta el escenario de cierre: escribe texto pendiente, envía `WM_CLOSE`,
   comprueba que se guarda y que el proceso termina.

```powershell
pwsh -File scripts/e2e-tauri.ps1
```

El workflow `e2e-tauri-windows` solo necesita Rust, pnpm y WebView2; ya no
instala drivers externos. Requiere Windows 10/11 y Visual Studio Build Tools.

## 4. Limitaciones

- El E2E de navegador usa la demo en memoria; recargar reinicia el estado.
- Los E2E de Tauri no cubren todos los diálogos nativos.
- Chromium de Playwright y, en Linux, `tauri-driver` se instalan fuera del
  repositorio; el E2E de Windows usa Playwright + WebView2, sin driver externo.
- El E2E de escritorio no prueba DOCX, búsqueda, cuarentena ni el diálogo
  nativo de creación; esos flujos se cubren en navegador o con pruebas Tauri.
- El cierre real en Linux depende de X11 y `xdotool`; Wayland puro queda
  pendiente. En Windows se usa `WM_CLOSE` sobre WebView2.

## 5. Pendiente

- Un flujo de escritorio que abra un proyecto con el diálogo nativo.
- Empaquetar y probar la app instalada, no solo el binario de `target/`.
