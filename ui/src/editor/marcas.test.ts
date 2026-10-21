import { Editor } from '@tiptap/core';
import { describe, expect, it } from 'vitest';
import { extensionesEditor } from './extensions';
import { comentarioValido, ulidValido } from './marcas';

const ID = '01KDVDNBYG20KW71NVRK6P2FHG';
const TRAMA = '01KDVDNFVGFKJ1G1SEHGTVZ69D';

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
    const markdown = `Una [[personaje:${ID}|Marta]] y una [[trama:${TRAMA}|trama]].\n`;
    expect(serializar(markdown)).toBe(markdown.trimEnd());
  });

  it('reemplaza marcas al superponer personaje y trama', () => {
    const editor = new Editor({
      extensions: extensionesEditor,
      content: 'Marta',
      contentType: 'markdown',
    });
    try {
      editor.commands.selectAll();
      editor.chain().setMark('personaje', { id: ID }).run();
      editor.chain().setMark('trama', { id: TRAMA }).run();
      const marcas = editor.getJSON().content?.[0]?.content?.[0]?.marks ?? [];
      expect(marcas.map((marca) => marca.type)).toEqual(['trama']);
      expect(editor.getMarkdown()).toBe(`[[trama:${TRAMA}|Marta]]`);
    } finally {
      editor.destroy();
    }
  });

  it('no interpreta ULID inválidos ni sintaxis ajena', () => {
    const markdown = 'Texto [[personaje:invalido|x]] y [[nota]] normal.\n';
    const salida = serializar(markdown);
    expect(salida).toContain('personaje:invalido');
    expect(serializar(salida)).toBe(salida);

    const primer_caracter = `[[personaje:Z1KDVDNBYG20KW71NVRK6P2FHG|Marta]]\n`;
    const con_z = serializar(primer_caracter);
    expect(con_z).toContain('Z1KDVDNBYG20KW71NVRK6P2FHG');
    expect(serializar(con_z)).toBe(con_z);
  });

  it('valida comentarios y admite corchetes simples', () => {
    expect(comentarioValido('revisar el ritmo')).toBe(true);
    expect(comentarioValido('ver [capítulo 2]')).toBe(true);
    expect(comentarioValido('Nota | secreta')).toBe(false);
    expect(comentarioValido('Nota ]] privada')).toBe(false);
    expect(comentarioValido('  ')).toBe(false);

    const markdown = `[[comentario:ver [capítulo 2]|Marta]]\n`;
    expect(serializar(markdown)).toBe(markdown.trimEnd());
  });

  it('conserva como literal un comentario con delimitador de cierre en el atributo', () => {
    const markdown = '[[comentario:Nota ]] privada|Marta]]';
    const editor = new Editor({
      extensions: extensionesEditor,
      content: markdown,
      contentType: 'markdown',
    });
    try {
      expect(editor.getText()).toBe(markdown);
      const salida = editor.getMarkdown();
      expect(serializar(salida)).toBe(salida);
    } finally {
      editor.destroy();
    }
  });

  it('aplica la regla ULID del esquema', () => {
    expect(ulidValido(ID)).toBe(true);
    expect(ulidValido('Z1KDVDNBYG20KW71NVRK6P2FHG')).toBe(false);
    expect(ulidValido('81KDVDNBYG20KW71NVRK6P2FHG')).toBe(false);
    expect(ulidValido('71KDVDNBYG20KW71NVRK6P2FHG')).toBe(true);
  });

  it('admite formato inline dentro de una marca', () => {
    const markdown = `[[personaje:${ID}|**Marta** y *Lucía*]]\n`;
    expect(serializar(markdown)).toBe(markdown.trimEnd());
  });
});
