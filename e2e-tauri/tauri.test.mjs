import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { remote } from 'webdriverio';

const proyecto = process.env.WRITER_E2E_PROJECT;
const aplicacion = process.env.WRITER_DESKTOP_BIN;
const puerto = Number(process.env.WRITER_E2E_PORT ?? 4444);
const modo = process.env.WRITER_E2E_MODE ?? 'guardar';

if (!proyecto || !aplicacion) {
  throw new Error('faltan WRITER_E2E_PROJECT o WRITER_DESKTOP_BIN');
}

const carpetaCapitulo = '01-capitulo';
const archivoEscena = '001-primera-escena.md';
const escena = path.join(proyecto, 'manuscrito', '01-libro', carpetaCapitulo, archivoEscena);
const textoGuardado = 'Texto E2E Tauri.';
const textoCierre = 'PENDIENTE VENTANA.';

function abrirSesion() {
  const opciones = { application: aplicacion };
  if (process.platform === 'win32') {
    // WebView2 necesita una carpeta de datos estable y escribible; sin esto,
    // msedgedriver puede fallar con DevToolsActivePort durante el arranque.
    opciones.webviewOptions = {
      userDataFolder: path.join(os.tmpdir(), `writer-e2e-webview2-${process.pid}`),
    };
  }
  return remote({
    hostname: '127.0.0.1',
    port: puerto,
    path: '/',
    logLevel: 'error',
    capabilities: {
      'tauri:options': opciones,
    },
  });
}

async function cerrarSesion(navegador) {
  try {
    await navegador.deleteSession();
  } catch (error) {
    console.error(`aviso: no se pudo cerrar la sesión WebDriver: ${error.message}`);
  }
}

async function esperarProyecto(navegador) {
  const editor = await navegador.$('.ProseMirror');
  await editor.waitForDisplayed({ timeout: 20_000 });
  await navegador.waitUntil(
    async () => {
      const titulo = await navegador.$('h1');
      return (await titulo.getText()).includes('Primera escena');
    },
    {
      timeout: 20_000,
      interval: 100,
      timeoutMsg: 'la app no abrió el proyecto del E2E',
    },
  );
  return editor;
}

async function escribirEnEditor(navegador, contenido) {
  for (let intento = 0; intento < 4; intento += 1) {
    await navegador.execute(() => {
      const editor = document.querySelector('.ProseMirror');
      if (editor instanceof HTMLElement) {
        editor.focus();
      }
    });
    await navegador.pause(200);
    await navegador.keys(['Control', 'a']);
    await navegador.keys(contenido);
    try {
      await navegador.waitUntil(
        async () => {
          const actual = await navegador.$('.ProseMirror');
          return (await actual.getText()).includes(contenido);
        },
        { timeout: 3000, interval: 100 },
      );
      return;
    } catch {
      // WebKit puede perder las primeras pulsaciones mientras la ventana
      // termina de activarse; se reintenta con una referencia fresca.
    }
  }
  const actual = await navegador.$('.ProseMirror');
  throw new Error(
    `el editor no recibió el texto tras varios intentos; texto actual: ${await actual.getText()}`,
  );
}

function cerrarVentanaX11() {
  // WebDriver cierra el WebView sin pasar por WM_DELETE_WINDOW; para probar el
  // handler real de Tauri hay que cerrar la ventana como lo haría el usuario.
  const ids = execFileSync('xdotool', ['search', '--name', '^writer$'], {
    encoding: 'utf8',
  })
    .trim()
    .split('\n')
    .filter(Boolean);
  if (ids.length === 0) {
    throw new Error('no se encontró la ventana X11 de writer');
  }
  const ventana = ids.at(-1);
  const pid = Number(
    execFileSync('xdotool', ['getwindowpid', ventana], { encoding: 'utf8' }).trim(),
  );
  execFileSync('xdotool', ['windowactivate', '--sync', ventana]);
  execFileSync('xdotool', ['key', '--clearmodifiers', 'alt+F4']);
  return pid;
}

function cerrarVentanaWindows() {
  // WM_CLOSE reproduce el cierre real de la ventana; `deleteSession()` no pasa
  // por el handler de Tauri y no probaría el guardado al cerrar.
  const script = `
$ErrorActionPreference = 'Stop'
$p = Get-Process -ErrorAction Stop |
  Where-Object { $_.MainWindowTitle -eq 'writer' -and $_.MainWindowHandle -ne 0 } |
  Select-Object -First 1
if (-not $p) { throw 'no se encontró la ventana de writer' }
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32Writer {
  [DllImport("user32.dll", SetLastError = true)]
  public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
}
"@
[void][Win32Writer]::PostMessage($p.MainWindowHandle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
$p.Id
`;
  const salida = execFileSync(
    'powershell',
    ['-NoProfile', '-NonInteractive', '-Command', script],
    { encoding: 'utf8' },
  ).trim();
  const pid = Number(salida);
  if (!Number.isFinite(pid) || pid <= 0) {
    throw new Error(`no se pudo cerrar la ventana de writer: ${salida}`);
  }
  return pid;
}

function cerrarVentana() {
  return process.platform === 'win32' ? cerrarVentanaWindows() : cerrarVentanaX11();
}

function procesoVivo(pid) {
  if (process.platform === 'win32') {
    try {
      process.kill(pid, 0);
      return true;
    } catch {
      return false;
    }
  }
  if (!existsSync(`/proc/${pid}`)) {
    return false;
  }
  try {
    const estado = readFileSync(`/proc/${pid}/stat`, 'utf8').split(' ')[2];
    return estado !== 'Z';
  } catch {
    return false;
  }
}

async function esperarProcesoCerrado(pid, intentos = 60) {
  for (let intento = 0; intento < intentos; intento += 1) {
    if (!procesoVivo(pid)) {
      return true;
    }
    await new Promise((resolver) => setTimeout(resolver, 250));
  }
  return false;
}

async function esperarVentanaCerrada(pid, intentos = 60) {
  if (process.platform === 'win32') {
    for (let intento = 0; intento < intentos; intento += 1) {
      const salida = execFileSync(
        'powershell',
        [
          '-NoProfile',
          '-NonInteractive',
          '-Command',
          `$p = Get-Process -Id ${pid} -ErrorAction SilentlyContinue; if ($null -eq $p) { '' } else { $p.MainWindowHandle }`,
        ],
        { encoding: 'utf8' },
      ).trim();
      if (salida === '' || salida === '0') {
        return true;
      }
      await new Promise((resolver) => setTimeout(resolver, 250));
    }
    return false;
  }
  for (let intento = 0; intento < intentos; intento += 1) {
    try {
      const salida = execFileSync(
        'xdotool',
        ['search', '--onlyvisible', '--pid', String(pid)],
        { encoding: 'utf8' },
      ).trim();
      if (salida === '') {
        return true;
      }
    } catch {
      // `xdotool search` sale con código 1 cuando ya no queda ventana.
      return true;
    }
    await new Promise((resolver) => setTimeout(resolver, 250));
  }
  return false;
}

async function esperarTextoEnArchivo(esperado, intentos = 40) {
  let contenido = '';
  for (let intento = 0; intento < intentos; intento += 1) {
    contenido = readFileSync(escena, 'utf8');
    if (contenido.includes(esperado)) {
      return contenido;
    }
    await new Promise((resolver) => setTimeout(resolver, 250));
  }
  return contenido;
}

const navegador = await abrirSesion();
let fallo = null;
try {
  await esperarProyecto(navegador);

  if (modo === 'guardar') {
    await escribirEnEditor(navegador, textoGuardado);
    await navegador.keys(['Control', 's']);

    const contenido = await esperarTextoEnArchivo(textoGuardado);
    if (!contenido.includes(textoGuardado)) {
      throw new Error(`el archivo no contiene el texto guardado:\n${contenido}`);
    }

    await navegador.waitUntil(
      async () =>
        (await navegador.$('.dabble-top-acciones .estado-guardado').getText())
          .toLowerCase()
          .includes('guardado'),
      {
        timeout: 10_000,
        interval: 100,
        timeoutMsg: 'la interfaz no confirmó el estado guardado',
      },
    );

    const titulo = await navegador.$('h1');
    if (!(await titulo.getText()).includes('Primera escena')) {
      throw new Error('no se abrió la escena esperada');
    }
    console.log('E2E Tauri: Ctrl+S guarda en disco y la escena sigue visible');
  } else if (modo === 'cerrar') {
    await escribirEnEditor(navegador, textoCierre);
    const pid = cerrarVentana();
    const contenido = await esperarTextoEnArchivo(textoCierre, 60);
    if (!contenido.includes(textoCierre)) {
      throw new Error(
        `el cierre de ventana perdió los cambios pendientes:\n${contenido}`,
      );
    }
    if (!(await esperarVentanaCerrada(pid))) {
      throw new Error('la ventana sigue visible después de guardar y cerrar');
    }
    if (!(await esperarProcesoCerrado(pid))) {
      throw new Error('el proceso de la app sigue vivo después de cerrar');
    }
    console.log(
      'E2E Tauri: cerrar la ventana guarda, destruye la ventana y termina el proceso',
    );
  } else {
    throw new Error(`modo de E2E desconocido: ${modo}`);
  }
} catch (error) {
  fallo = error;
} finally {
  await cerrarSesion(navegador);
}

if (fallo) {
  throw fallo;
}
