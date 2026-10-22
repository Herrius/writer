# E2E — navegador y escritorio

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Ámbito:** `e2e/`, `e2e-tauri/`, `playwright.config.ts` y
  `scripts/e2e-tauri.sh`

## 1. E2E de navegador (Playwright)

Playwright arranca Vite en `http://127.0.0.1:1420` y prueba la UI en **modo
demo** (sin Tauri). Cubre 12 flujos:

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
   de destruirla y que la ventana y el proceso desaparecen;
6. **opcional (fuera del gate):** en una tercera ejecución sin proyecto
   precargado, crea un proyecto desde el panel, navega con el explorador de
   carpetas hasta una carpeta temporal, elige “Usar esta carpeta” y comprueba
   `proyecto.yml`, la estructura inicial y la apertura de la primera escena.

```bash
cargo install tauri-driver --version 2.0.6 --locked
pnpm e2e:tauri
python scripts/ci.py e2e-tauri
```

El tercer escenario de creación desde el explorador está definido, pero no se
ejecuta en el gate por la inestabilidad de WebKitWebDriver en este entorno. Se
puede lanzar con `WRITER_E2E_INCLUIR_CREACION=1 pnpm e2e:tauri`; no se considera
verde hasta estabilizarlo.

Requisitos: Linux, `WebKitWebDriver`, `tauri-driver`, X11
(`GDK_BACKEND=x11`), `xdotool` y las dependencias de Tauri ya necesarias para
compilar. El script usa un puerto libre, un grupo de procesos propio y limpia la
app al terminar.

La feature `e2e` solo existe en la compilación de prueba: la app de producción
no autoabre proyectos ni conserva el hook activo.

## 3. Limitaciones

- El E2E de navegador usa la demo en memoria; recargar reinicia el estado.
- El E2E de Tauri es Linux-only y no cubre diálogos nativos ni Windows/macOS.
- Chromium de Playwright y `tauri-driver` se instalan fuera del repositorio.
- El E2E de escritorio no prueba DOCX, búsqueda, cuarentena ni el diálogo
  nativo de creación; la creación se cubre con el explorador in-app del tercer
  escenario, el modal de navegador y las pruebas Tauri.
- El cierre real depende de X11 y `xdotool`; en Wayland puro o en Windows/macOS
  queda pendiente.

## 4. Pendiente

- E2E de escritorio en Windows (WebView2) y macOS.
- Un flujo de escritorio que use el diálogo nativo de carpeta como alternativa
  al explorador in-app.
- Empaquetar y probar la app instalada, no solo el binario de `target/`.
