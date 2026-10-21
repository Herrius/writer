import { Editor } from '@tiptap/core';
import { describe, expect, it } from 'vitest';
import { extensionesEditor } from './extensions';

function serializar(markdown: string): string {
  const editor = new Editor({
    extensions: extensionesEditor,
    content: markdown,
    contentType: 'markdown',
  });
  try {
    return editor.getMarkdown();
  } finally {
    editor.destroy();
  }
}

describe('marcas propias', () => {
  it('conserva personajes, tramas y comentarios', () => {
    const markdown =
      'Una [[personaje:01KDVDNBYG20KW71NVRK6P2FHG|Marta]] y una [[trama:01KDVDNFVGFKJ1G1SEHGTVZ69D|trama]].\n';
    expect(serializar(markdown)).toBe(markdown.trimEnd());
  });

  it('no interpreta ULID inválidos ni sintaxis ajena', () => {
    const markdown = 'Texto [[personaje:invalido|x]] y [[nota]] normal.\n';
    const salida = serializar(markdown);
    expect(salida).toContain('personaje:invalido');
    expect(serializar(salida)).toBe(salida);
  });

  it('admite formato inline dentro de una marca', () => {
    const markdown =
      '[[personaje:01KDVDNBYG20KW71NVRK6P2FHG|**Marta** y *Lucía*]]\n';
    expect(serializar(markdown)).toBe(markdown.trimEnd());
  });
});
