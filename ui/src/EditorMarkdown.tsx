import { EditorContent, useEditor, useEditorState } from '@tiptap/react';
import { useEffect, useRef, useState } from 'react';
import { extensionesEditor } from './editor/extensions';

export type OpcionMarca = {
  id: string;
  titulo: string;
};

type Props = {
  markdown: string;
  editable: boolean;
  personajes: OpcionMarca[];
  tramas: OpcionMarca[];
  onCambio: (markdown: string) => void;
};

export function EditorMarkdown({
  markdown,
  editable,
  personajes,
  tramas,
  onCambio,
}: Props) {
  const alCambiar = useRef(onCambio);
  alCambiar.current = onCambio;
  const [comentarioAbierto, setComentarioAbierto] = useState(false);
  const [comentarioTexto, setComentarioTexto] = useState('');

  const editor = useEditor({
    extensions: extensionesEditor,
    content: markdown,
    contentType: 'markdown',
    editable,
    immediatelyRender: false,
    editorProps: {
      attributes: {
        class: 'editor-prose',
        spellcheck: 'true',
      },
    },
    onUpdate: ({ editor: instancia }) => {
      alCambiar.current(instancia.getMarkdown());
    },
  });

  useEffect(() => {
    if (editor && editor.isEditable !== editable) {
      editor.setEditable(editable);
    }
  }, [editor, editable]);

  const estado = useEditorState({
    editor,
    selector: ({ editor: instancia }) => ({
      vacia: instancia ? instancia.state.selection.empty : true,
      hayMarca: instancia
        ? instancia.isActive('personaje') ||
          instancia.isActive('trama') ||
          instancia.isActive('comentario')
        : false,
    }),
  });

  if (!editor) {
    return <div className="editor-cargando">Preparando el editor…</div>;
  }

  const seleccionVacia = estado?.vacia ?? true;
  const hayMarca = estado?.hayMarca ?? false;

  const aplicarPersonaje = (id: string) => {
    if (!id) {
      return;
    }
    editor.chain().focus().setMark('personaje', { id }).run();
  };

  const aplicarTrama = (id: string) => {
    if (!id) {
      return;
    }
    editor.chain().focus().setMark('trama', { id }).run();
  };

  const aplicarComentario = () => {
    const texto = comentarioTexto.trim();
    if (texto === '') {
      return;
    }
    editor.chain().focus().setMark('comentario', { texto }).run();
    setComentarioTexto('');
    setComentarioAbierto(false);
  };

  const quitarMarcas = () => {
    editor
      .chain()
      .focus()
      .unsetMark('personaje')
      .unsetMark('trama')
      .unsetMark('comentario')
      .run();
  };

  return (
    <div className="editor-content" aria-label="Texto del documento">
      {editable && (
        <div className="editor-toolbar" role="toolbar" aria-label="Marcas del texto">
          <select
            aria-label="Aplicar personaje"
            value=""
            disabled={seleccionVacia}
            onChange={(evento) => aplicarPersonaje(evento.target.value)}
          >
            <option value="">Personaje…</option>
            {personajes.map((personaje) => (
              <option key={personaje.id} value={personaje.id}>
                {personaje.titulo}
              </option>
            ))}
          </select>
          <select
            aria-label="Aplicar trama"
            value=""
            disabled={seleccionVacia}
            onChange={(evento) => aplicarTrama(evento.target.value)}
          >
            <option value="">Trama…</option>
            {tramas.map((trama) => (
              <option key={trama.id} value={trama.id}>
                {trama.titulo}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="secundario pequeno"
            disabled={seleccionVacia}
            onClick={() => setComentarioAbierto((abierto) => !abierto)}
          >
            Comentario
          </button>
          {comentarioAbierto && (
            <span className="comentario-form">
              <input
                aria-label="Texto del comentario"
                value={comentarioTexto}
                onChange={(evento) => setComentarioTexto(evento.target.value)}
                onKeyDown={(evento) => {
                  if (evento.key === 'Enter') {
                    evento.preventDefault();
                    aplicarComentario();
                  } else if (evento.key === 'Escape') {
                    setComentarioAbierto(false);
                    setComentarioTexto('');
                  }
                }}
                placeholder="Nota de revisión"
              />
              <button
                type="button"
                className="secundario pequeno"
                onClick={aplicarComentario}
                disabled={comentarioTexto.trim() === ''}
              >
                Aplicar
              </button>
            </span>
          )}
          <button
            type="button"
            className="secundario pequeno"
            disabled={!hayMarca}
            onClick={quitarMarcas}
          >
            Quitar marca
          </button>
        </div>
      )}
      <EditorContent editor={editor} />
    </div>
  );
}
