import {
  Mark,
  mergeAttributes,
  type MarkdownParseHelpers,
  type MarkdownToken,
  type MarkdownTokenizer,
} from '@tiptap/core';

/**
 * Marcas propias del proyecto.
 *
 * Sintaxis reservada (ver `docs/marcas-v1.md`):
 *   [[personaje:ULID|texto]]
 *   [[trama:ULID|texto]]
 *   [[comentario:texto|texto]]
 *
 * No son HTML: el Markdown sigue siendo la fuente de verdad y el motor no
 * conoce estas marcas.
 */

type ClaseMarca = 'personaje' | 'trama' | 'comentario';

type TokenMarca = MarkdownToken & {
  clase: ClaseMarca;
  referencia: string;
};

const PATRON = /^\[\[(personaje|trama|comentario):([^|\]]+)\|([\s\S]+?)\]\]/;
const ULID = /^[0-9A-HJKMNP-TV-Z]{26}$/;

const tokenizer: MarkdownTokenizer = {
  name: 'marcaWriter',
  level: 'inline',
  start: (src) => src.indexOf('[['),
  tokenize(src, _tokens, lexer) {
    const coincidencia = PATRON.exec(src);
    if (!coincidencia) {
      return;
    }
    const [raw, clase, referencia, contenido] = coincidencia;
    const tipo = clase as ClaseMarca;
    if (tipo !== 'comentario' && !ULID.test(referencia)) {
      return;
    }
    if (tipo === 'comentario' && referencia.trim() === '') {
      return;
    }
    const token: TokenMarca = {
      type: 'marcaWriter',
      raw,
      clase: tipo,
      referencia,
      tokens: lexer.inlineTokens(contenido),
    };
    return token;
  },
};

const MarcaWriter = Mark.create({
  name: 'marcaWriter',
  markdownTokenName: 'marcaWriter',
  markdownTokenizer: tokenizer,
  parseMarkdown(token: MarkdownToken, helpers: MarkdownParseHelpers) {
    const { clase, referencia } = token as TokenMarca;
    const attrs = clase === 'comentario' ? { texto: referencia } : { id: referencia };
    return helpers.applyMark(clase, helpers.parseInline(token.tokens ?? []), attrs);
  },
});

function crearMarca(clase: ClaseMarca, atributo: string, etiqueta: string) {
  return Mark.create({
    name: clase,
    addAttributes() {
      return { [atributo]: { default: null } };
    },
    parseHTML() {
      return [{ tag: `span[data-marca-${clase}]` }];
    },
    renderHTML({ HTMLAttributes }) {
      return [
        'span',
        mergeAttributes(HTMLAttributes, {
          [`data-marca-${clase}`]: '',
          class: `marca marca-${clase}`,
          title: etiqueta,
        }),
        0,
      ];
    },
    renderMarkdown(node, helpers) {
      const valor = node.attrs?.[atributo] ?? '';
      return `[[${clase}:${valor}|${helpers.renderChildren(node)}]]`;
    },
    ...(clase === 'comentario' ? { clearable: false } : {}),
  });
}

export const Personaje = crearMarca('personaje', 'id', 'Personaje');
export const Trama = crearMarca('trama', 'id', 'Trama');
export const Comentario = crearMarca('comentario', 'texto', 'Comentario');

export const marcasEditor = [MarcaWriter, Personaje, Trama, Comentario];
