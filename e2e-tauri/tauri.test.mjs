import { readFileSync } from 'node:fs';
import path from 'node:path';
import { remote } from 'webdriverio';

const proyecto = process.env.WRITER_E2E_PROJECT;
const aplicacion = process.env.WRITER_DESKTOP_BIN;

if (!proyecto || !aplicacion) {
  throw new Error('faltan WRITER_E2E_PROJECT o WRITER_DESKTOP_BIN');
}

const escena = path.join(
  proyecto,
  'manuscrito',
  '01-libro',
  '01-capitulo',
  '001-primera-escena.md',
);
const texto = 'Texto E2E Tauri.';

const navegador = await remote({
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',
  logLevel: 'error',
  capabilities: {
    'tauri:options': { application: aplicacion },
  },
});

try {
  const editor = await navegador.$('.ProseMirror');
  await editor.waitForDisplayed({ timeout: 20_000 });
  await editor.click();
  await navegador.keys(['Control', 'a']);
  await navegador.keys(texto);
  await navegador.keys(['Control', 's']);

  let contenido = '';
  for (let intento = 0; intento < 24; intento += 1) {
    contenido = readFileSync(escena, 'utf8');
    if (contenido.includes(texto)) {
      break;
    }
    await navegador.pause(250);
  }
  if (!contenido.includes(texto)) {
    throw new Error(`el archivo no contiene el texto guardado:\n${contenido}`);
  }
  const titulo = await navegador.$('h2');
  if (!(await titulo.getText()).includes('Primera escena')) {
    throw new Error('no se abrió la escena esperada');
  }
  console.log('E2E Tauri: texto guardado en disco y escena visible');
} finally {
  await navegador.deleteSession();
}
