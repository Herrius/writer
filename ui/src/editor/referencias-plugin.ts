import { Extension } from '@tiptap/core';
import { Plugin, PluginKey, type EditorState } from '@tiptap/pm/state';
import { Decoration, DecorationSet } from '@tiptap/pm/view';
import { tituloPersonaje, tituloTrama } from './referencias';

/**
 * Decoraciones de las marcas de personaje y trama.
 *
 * El título se calcula en una decoración de ProseMirror, no mutando el DOM
 * desde fuera: así el DOMObserver no entra en conflicto con el repintado de las
 * marcas y no se pierde el foco ni se dispara un bucle.
 */
const clave = new PluginKey('referencias-marcas');

function decoraciones(state: EditorState): DecorationSet {
  const salida: Decoration[] = [];
  state.doc.descendants((nodo, posicion) => {
    if (!nodo.isText) {
      return;
    }
    for (const marca of nodo.marks) {
      if (marca.type.name !== 'personaje' && marca.type.name !== 'trama') {
        continue;
      }
      const referencia = String(marca.attrs.id ?? '');
      const titulo =
        marca.type.name === 'personaje'
          ? tituloPersonaje(referencia)
          : tituloTrama(referencia);
      const etiqueta = marca.type.name === 'personaje' ? 'Personaje' : 'Trama';
      const atributos: Record<string, string> = {
        title: titulo
          ? `${etiqueta}: ${titulo}`
          : `${etiqueta}: referencia desconocida`,
      };
      if (!titulo) {
        atributos['data-referencia-desconocida'] = 'true';
      }
      salida.push(
        Decoration.inline(posicion, posicion + nodo.nodeSize, atributos),
      );
    }
  });
  return DecorationSet.create(state.doc, salida);
}

export const ReferenciasMarcas = Extension.create({
  name: 'referenciasMarcas',
  addProseMirrorPlugins() {
    return [
      new Plugin({
        key: clave,
        props: {
          decorations: (state) => decoraciones(state),
        },
      }),
    ];
  },
});
