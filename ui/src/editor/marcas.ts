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

type ClaseMarca = 'personaje' | 'trama' | 'comentario' | 'nota';

type TokenMarca = MarkdownToken & {
  clase: ClaseMarca;
  referencia: string;
  id?: string;
  texto?: string;
};

const PATRON_PERSONAJE = /^\[\[(personaje|trama):([0-7][0-9A-HJKMNP-TV-Z]{25})\|([\s\S]+?)\]\]/;
const PATRON_NOTA = /^\[\[nota:([0-7][0-9A-HJKMNP-TV-Z]{25})\|([\s\S]+?)\]\]/;
const PATRON_COMENTARIO_V2 =
  /^\[\[comentario:([0-7][0-9A-HJKMNP-TV-Z]{25})\|([^|]+?)\|([\s\S]+?)\]\]/;
const PATRON_COMENTARIO = /^\[\[comentario:([^|]+?)\|([\s\S]+?)\]\]/;
const ULID = /^[0-7][0-9A-HJKMNP-TV-Z]{25}$/;

export function ulidValido(texto: string): boolean {
  return ULID.test(texto);
}

export function comentarioValido(texto: string): boolean {
  return (
    texto.trim() !== '' &&
    texto === texto.trim() &&
    !texto.includes('|') &&
    !texto.includes(']]')
  );
}

const tokenizer: MarkdownTokenizer = {
  name: 'marcaWriter',
  level: 'inline',
  start: (src) => src.indexOf('[['),
  tokenize(src, _tokens, lexer) {
    const dePersonaje = PATRON_PERSONAJE.exec(src);
    if (dePersonaje) {
      const [raw, clase, referencia, contenido] = dePersonaje;
      const token: TokenMarca = {
        type: 'marcaWriter',
        raw,
        clase: clase as ClaseMarca,
        referencia,
        tokens: lexer.inlineTokens(contenido),
      };
      return token;
    }
    const deNota = PATRON_NOTA.exec(src);
    if (deNota) {
      const [raw, referencia, contenido] = deNota;
      return {
        type: 'marcaWriter',
        raw,
        clase: 'nota',
        referencia,
        tokens: lexer.inlineTokens(contenido),
      } satisfies TokenMarca;
    }
    const deComentarioV2 = PATRON_COMENTARIO_V2.exec(src);
    if (deComentarioV2) {
      const [raw, id, texto, contenido] = deComentarioV2;
      if (!comentarioValido(texto)) {
        return;
      }
      return {
        type: 'marcaWriter',
        raw,
        clase: 'comentario',
        referencia: id,
        id,
        texto,
        tokens: lexer.inlineTokens(contenido),
      } satisfies TokenMarca;
    }
    const deComentario = PATRON_COMENTARIO.exec(src);
    if (!deComentario) {
      return;
    }
    const [raw, referencia, contenido] = deComentario;
    if (!comentarioValido(referencia) || raw.split('|').length !== 2) {
      return;
    }
    const token: TokenMarca = {
      type: 'marcaWriter',
      raw,
      clase: 'comentario',
      referencia,
      texto: referencia,
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
    const { clase, referencia, id, texto } = token as TokenMarca;
    const attrs =
      clase === 'comentario'
        ? { texto: texto ?? referencia, id: id ?? null }
        : { id: referencia };
    return helpers.applyMark(clase, helpers.parseInline(token.tokens ?? []), attrs);
  },
});

function crearMarca(clase: ClaseMarca, atributo: string) {
  return Mark.create({
    name: clase,
    excludes: 'personaje trama comentario nota',
    addAttributes() {
      if (clase === 'comentario') {
        return {
          texto: {
            default: null,
            parseHTML: (elemento) => elemento.getAttribute('data-referencia'),
            renderHTML: (atributos) => ({ 'data-referencia': atributos.texto }),
          },
          id: {
            default: null,
            parseHTML: (elemento) => elemento.getAttribute('data-id'),
            renderHTML: (atributos) => ({ 'data-id': atributos.id }),
          },
        };
      }
      return {
        [atributo]: {
          default: null,
          parseHTML: (elemento) => elemento.getAttribute('data-referencia'),
          renderHTML: (atributos) => ({ 'data-referencia': atributos[atributo] }),
        },
      };
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
        }),
        0,
      ];
    },
    renderMarkdown(node, helpers) {
      if (clase === 'comentario') {
        const texto = node.attrs?.texto ?? '';
        const id = node.attrs?.id;
        return id
          ? `[[comentario:${id}|${texto}|${helpers.renderChildren(node)}]]`
          : `[[comentario:${texto}|${helpers.renderChildren(node)}]]`;
      }
      const valor = node.attrs?.[atributo] ?? '';
      return `[[${clase}:${valor}|${helpers.renderChildren(node)}]]`;
    },
    ...(clase === 'comentario' ? { clearable: false } : {}),
  });
}

export const Personaje = crearMarca('personaje', 'id');
export const Trama = crearMarca('trama', 'id');
export const Nota = crearMarca('nota', 'id');
export const Comentario = crearMarca('comentario', 'texto');

export const marcasEditor = [MarcaWriter, Personaje, Trama, Nota, Comentario];
