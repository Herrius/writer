import { Editor } from '@tiptap/core';
import { describe, expect, it } from 'vitest';
import { extensionesEditor } from '../editor/extensions';

/**
 * Corpus golden de ida y vuelta Markdown.
 *
 * Cada pareja `NN-caso.md` / `NN-caso.esperado.md` fija lo que el editor acepta
 * y cómo lo normaliza. Si una construcción no vuelve idéntica, se documenta en
 * `docs/markdown-v1.md` y, si pierde contenido, se bloquea en
 * `src/markdown/compatibilidad.ts`.
 */

const archivos = import.meta.glob('./corpus/*.md', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function canonico(markdown: string): string {
  const normalizado = markdown.replace(/\r\n/g, '\n').replace(/\r/g, '\n').trimEnd();
  return normalizado === '' ? '' : `${normalizado}\n`;
}

function serializar(markdown: string): string {
  const editor = new Editor({
    extensions: extensionesEditor,
    content: markdown,
    contentType: 'markdown',
  });
  try {
    return canonico(editor.getMarkdown());
  } finally {
    editor.destroy();
  }
}

const casos = Object.keys(archivos)
  .filter((ruta) => !ruta.endsWith('.esperado.md'))
  .sort();

describe('corpus golden de Markdown', () => {
  it('encuentra todas las parejas del corpus', () => {
    expect(casos.length).toBeGreaterThanOrEqual(16);
    for (const ruta of casos) {
      const esperado = ruta.replace(/\.md$/, '.esperado.md');
      expect(archivos[esperado], `falta ${esperado}`).toBeTypeOf('string');
    }
  });

  for (const ruta of casos) {
    const nombre = ruta.split('/').pop() ?? ruta;

    it(`normaliza y estabiliza ${nombre}`, () => {
      const entrada = archivos[ruta];
      const esperado = canonico(archivos[ruta.replace(/\.md$/, '.esperado.md')]);
      const salida = serializar(entrada);
      expect(salida).toBe(esperado);
      expect(serializar(esperado)).toBe(esperado);
    });
  }
});
