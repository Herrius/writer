// E2E nativo de Windows sobre WebView2 usando CDP y Playwright.
// Evita tauri-driver/msedgedriver, que en CI falla con DevToolsActivePort.
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { chromium } from '@playwright/test';

const proyecto = process.env.WRITER_E2E_PROJECT;
const puerto = Number(process.env.WRITER_CDP_PORT ?? 9222);
const modo = process.env.WRITER_E2E_MODE ?? 'guardar';
const pid = Number(process.env.WRITER_DESKTOP_PID);

if (!proyecto || !Number.isFinite(pid) || pid <= 0) {
  throw new Error('faltan WRITER_E2E_PROJECT o WRITER_DESKTOP_PID');
}

const escena = path.join(
  proyecto,
  'manuscrito',
  '01-libro',
  '01-capitulo',
  '001-primera-escena.md',
);
const textoGuardado = 'Texto E2E Tauri.';
const textoCierre = 'PENDIENTE VENTANA.';

const esperar = (ms) => new Promise((resolver) => setTimeout(resolver, ms));

async function conectar() {
  let ultimoError;
  for (let intento = 0; intento < 80; intento += 1) {
    try {
      return await chromium.connectOverCDP(`http://127.0.0.1:${puerto}`);
    } catch (error) {
      ultimoError = error;
      await esperar(250);
    }
  }
  throw new Error(`no se pudo conectar por CDP: ${ultimoError?.message ?? ultimoError}`);
}

async function paginaDeLaApp(navegador) {
  for (const contexto of navegador.contexts()) {
    for (const pagina of contexto.pages()) {
      const url = pagina.url();
      if (url.startsWith('tauri://') || url.includes('tauri.localhost')) {
        return pagina;
      }
    }
  }
  const contexto = navegador.contexts()[0];
  const pagina = contexto?.pages().at(-1);
  if (!pagina) {
    throw new Error('CDP no expuso ninguna página de WebView2');
  }
  return pagina;
}

async function esperarProyecto(pagina) {
  await pagina.waitForSelector('.ProseMirror', { timeout: 30_000 });
  await pagina.waitForFunction(
    () => document.querySelector('h1')?.textContent?.includes('Primera escena'),
    undefined,
    { timeout: 30_000 },
  );
}

async function escribirEnEditor(pagina, contenido) {
  for (let intento = 0; intento < 4; intento += 1) {
    await pagina.locator('.ProseMirror').click();
    await pagina.keyboard.press('Control+A');
    await pagina.keyboard.type(contenido);
    try {
      await pagina.waitForFunction(
        (texto) => document.querySelector('.ProseMirror')?.textContent?.includes(texto),
        contenido,
        { timeout: 3_000 },
      );
      return;
    } catch {
      // WebView2 puede perder las primeras pulsaciones al activar la ventana.
    }
  }
  throw new Error('el editor no recibió el texto tras varios intentos');
}

async function esperarTextoEnArchivo(esperado, intentos = 60) {
  let contenido = '';
  for (let intento = 0; intento < intentos; intento += 1) {
    contenido = readFileSync(escena, 'utf8');
    if (contenido.includes(esperado)) {
      return contenido;
    }
    await esperar(250);
  }
  return contenido;
}

function procesoVivo(pidProceso) {
  try {
    process.kill(pidProceso, 0);
    return true;
  } catch {
    return false;
  }
}

async function esperarProcesoCerrado(pidProceso, intentos = 60) {
  for (let intento = 0; intento < intentos; intento += 1) {
    if (!procesoVivo(pidProceso)) {
      return true;
    }
    await esperar(250);
  }
  return false;
}

function cerrarVentana(pidProceso) {
  const script = `
$ErrorActionPreference = 'Stop'
$p = Get-Process -Id ${pidProceso} -ErrorAction Stop
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32Writer {
  [DllImport("user32.dll", SetLastError = true)]
  public static extern bool PostMessage(IntPtr hWnd, uint Msg, IntPtr wParam, IntPtr lParam);
}
"@
[void][Win32Writer]::PostMessage($p.MainWindowHandle, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)
`;
  execFileSync('powershell', ['-NoProfile', '-NonInteractive', '-Command', script], {
    encoding: 'utf8',
  });
}

const navegador = await conectar();
let fallo = null;
try {
  const pagina = await paginaDeLaApp(navegador);
  await esperarProyecto(pagina);

  if (modo === 'guardar') {
    await escribirEnEditor(pagina, textoGuardado);
    await pagina.keyboard.press('Control+S');
    const contenido = await esperarTextoEnArchivo(textoGuardado);
    if (!contenido.includes(textoGuardado)) {
      throw new Error(`el archivo no contiene el texto guardado:\n${contenido}`);
    }
    await pagina.waitForFunction(
      () =>
        document
          .querySelector('.dabble-top-acciones .estado-guardado')
          ?.textContent?.toLowerCase()
          .includes('guardado'),
      undefined,
      { timeout: 10_000 },
    );
    const titulo = await pagina.locator('h1').textContent();
    if (!titulo?.includes('Primera escena')) {
      throw new Error('no se abrió la escena esperada');
    }
    console.log('E2E Windows CDP: Ctrl+S guarda en disco y la escena sigue visible');
  } else if (modo === 'cerrar') {
    await escribirEnEditor(pagina, textoCierre);
    cerrarVentana(pid);
    const contenido = await esperarTextoEnArchivo(textoCierre, 80);
    if (!contenido.includes(textoCierre)) {
      throw new Error(`el cierre de ventana perdió los cambios pendientes:\n${contenido}`);
    }
    if (!(await esperarProcesoCerrado(pid))) {
      throw new Error('el proceso de writer sigue vivo después de cerrar');
    }
    console.log('E2E Windows CDP: cerrar la ventana guarda y termina el proceso');
  } else {
    throw new Error(`modo de E2E desconocido: ${modo}`);
  }
} catch (error) {
  fallo = error;
} finally {
  try {
    await navegador.close();
  } catch {
    // El cierre de ventana ya puede haber desconectado CDP.
  }
}

if (fallo) {
  throw fallo;
}
