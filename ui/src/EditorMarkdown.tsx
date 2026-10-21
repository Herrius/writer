import { EditorContent, useEditor, useEditorState } from '@tiptap/react';
import { useEffect, useRef, useState } from 'react';
import { extensionesEditor } from './editor/extensions';
import { comentarioValido } from './editor/marcas';

export type OpcionMarca = {
  id: string;
  titulo: string;
};

function contarPalabras(texto: string): number {
  const limpio = texto.trim();
  return limpio === '' ? 0 : limpio.split(/\s+/u).length;
}

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
}: Props) {  const alCambiar = useRef(onCambio);
  alCambiar.current = onCambio;
  const [comentarioAbierto, setComentarioAbierto] = useState(false);
  const [comentarioTexto, setComentarioTexto] = useState('');
  const [errorComentario, setErrorComentario] = useState<string | null>(null);

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
      negrita: instancia?.isActive('bold') ?? false,
      cursiva: instancia?.isActive('italic') ?? false,
      tachado: instancia?.isActive('strike') ?? false,
      titulo2: instancia?.isActive('heading', { level: 2 }) ?? false,
      cita: instancia?.isActive('blockquote') ?? false,
      lista: instancia?.isActive('bulletList') ?? false,
      ordenada: instancia?.isActive('orderedList') ?? false,
      codigo: instancia?.isActive('codeBlock') ?? false,
      palabras: instancia ? contarPalabras(instancia.getText()) : 0,
    }),
  });

  if (!editor) {
    return <div className="editor-cargando">Preparando el editor…</div>;
  }

  const seleccionVacia = estado?.vacia ?? true;
  const hayMarca = estado?.hayMarca ?? false;

  const reemplazarMarca = (marca: string, attrs: Record<string, unknown>) => {
    const { from, to } = editor.state.selection;
    const seleccion = editor.state.doc.textBetween(from, to, '\n');
    if (from === to) {
      setErrorComentario('Selecciona texto para aplicar la marca.');
      return false;
    }
    if (seleccion.includes(']]')) {
      setErrorComentario('La selección no puede contener «]]».');
      return false;
    }
    if (marca === 'comentario' && seleccion.includes('|')) {
      setErrorComentario('La selección de un comentario no puede contener «|».');
      return false;
    }
    editor
      .chain()
      .focus()
      .unsetMark('personaje')
      .unsetMark('trama')
      .unsetMark('comentario')
      .setMark(marca, attrs)
      .run();
    return true;
  };

  const aplicarPersonaje = (id: string) => {
    if (!id) {
      return;
    }
    setErrorComentario(null);
    reemplazarMarca('personaje', { id });
  };

  const aplicarTrama = (id: string) => {
    if (!id) {
      return;
    }
    setErrorComentario(null);
    reemplazarMarca('trama', { id });
  };

  const aplicarComentario = () => {
    const texto = comentarioTexto.trim();
    if (!comentarioValido(texto)) {
      setErrorComentario('El comentario no puede estar vacío ni contener «|» o «]]».');
      return;
    }
    if (!reemplazarMarca('comentario', { texto })) {
      return;
    }
    setComentarioTexto('');
    setComentarioAbierto(false);
    setErrorComentario(null);
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
          <button
            type="button"
            className={`formato ${estado?.negrita ? 'activo' : ''}`}
            aria-label="Negrita"
            title="Negrita (Ctrl+B)"
            onClick={() => editor.chain().focus().toggleBold().run()}
          >
            B
          </button>
          <button
            type="button"
            className={`formato cursiva ${estado?.cursiva ? 'activo' : ''}`}
            aria-label="Cursiva"
            title="Cursiva (Ctrl+I)"
            onClick={() => editor.chain().focus().toggleItalic().run()}
          >
            I
          </button>
          <button
            type="button"
            className={`formato tachado ${estado?.tachado ? 'activo' : ''}`}
            aria-label="Tachado"
            title="Tachado"
            onClick={() => editor.chain().focus().toggleStrike().run()}
          >
            S
          </button>
          <button
            type="button"
            className={`formato ${estado?.titulo2 ? 'activo' : ''}`}
            aria-label="Título de sección"
            title="Título de sección"
            onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()}
          >
            H2
          </button>
          <button
            type="button"
            className={`formato ${estado?.cita ? 'activo' : ''}`}
            aria-label="Cita"
            title="Cita"
            onClick={() => editor.chain().focus().toggleBlockquote().run()}
          >
            ❝
          </button>
          <button
            type="button"
            className={`formato ${estado?.lista ? 'activo' : ''}`}
            aria-label="Lista de viñetas"
            title="Lista de viñetas"
            onClick={() => editor.chain().focus().toggleBulletList().run()}
          >
            •
          </button>
          <button
            type="button"
            className={`formato ${estado?.ordenada ? 'activo' : ''}`}
            aria-label="Lista ordenada"
            title="Lista ordenada"
            onClick={() => editor.chain().focus().toggleOrderedList().run()}
          >
            1.
          </button>
          <button
            type="button"
            className={`formato ${estado?.codigo ? 'activo' : ''}`}
            aria-label="Bloque de código"
            title="Bloque de código"
            onClick={() => editor.chain().focus().toggleCodeBlock().run()}
          >
            {'{}'}
          </button>
          <span className="toolbar-separador" aria-hidden="true" />
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
            onClick={() => {
              setComentarioAbierto((abierto) => !abierto);
              setErrorComentario(null);
            }}
          >
            Comentario
          </button>
          {comentarioAbierto && (
            <span className="comentario-form">
              <input
                aria-label="Texto del comentario"
                value={comentarioTexto}
                onChange={(evento) => {
                  setComentarioTexto(evento.target.value);
                  setErrorComentario(null);
                }}
                onKeyDown={(evento) => {
                  if (evento.key === 'Enter') {
                    evento.preventDefault();
                    aplicarComentario();
                  } else if (evento.key === 'Escape') {
                    setComentarioAbierto(false);
                    setComentarioTexto('');
                    setErrorComentario(null);
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
          {errorComentario && (
            <span className="editor-error" role="alert">{errorComentario}</span>
          )}
          <button
            type="button"
            className="secundario pequeno"
            disabled={!hayMarca}
            onClick={quitarMarcas}
          >
            Quitar marca
          </button>
          <span className="editor-contador">{estado?.palabras ?? 0} palabras</span>
        </div>
      )}
      <EditorContent editor={editor} />
    </div>
  );
}
