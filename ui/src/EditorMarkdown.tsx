import { EditorContent, useEditor, useEditorState, type Editor } from '@tiptap/react';
import { MessageSquarePlus } from 'lucide-react';
import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from 'react';
import { extensionesEditor } from './editor/extensions';
import { comentarioValido } from './editor/marcas';
import { actualizarReferencias } from './editor/referencias';
import { contarPalabras, textoSinMarcas } from './palabras';

export type OpcionMarca = {
  id: string;
  titulo: string;
};

export type EditorMarkdownHandle = {
  /** Cuenta coincidencias sin mover la selección. */
  contar: (texto: string) => number;
  /** Selecciona una coincidencia y devuelve el total y el índice activo. */
  buscar: (texto: string, indice?: number) => { total: number; indice: number };
};

type Coincidencia = { from: number; to: number };

type BurbujaSeleccion = {
  top: number;
  left: number;
  from: number;
  to: number;
  abierta: boolean;
};

function coincidenciasEnEditor(editor: Editor | null, texto: string): Coincidencia[] {
  if (!editor || !texto.trim()) {
    return [];
  }
  const aguja = texto.toLocaleLowerCase();
  const coincidencias: Coincidencia[] = [];
  editor.state.doc.descendants((nodo, posicion) => {
    if (!nodo.isText || !nodo.text) {
      return true;
    }
    const contenido = nodo.text.toLocaleLowerCase();
    let desde = 0;
    while (desde < contenido.length) {
      const indice = contenido.indexOf(aguja, desde);
      if (indice < 0) {
        break;
      }
      coincidencias.push({
        from: posicion + indice,
        to: posicion + indice + aguja.length,
      });
      desde = indice + aguja.length;
    }
    return true;
  });
  return coincidencias;
}

/**
 * Coloca el globo junto al final de la selección y lo mantiene dentro de la
 * ventana. Si el final queda fuera de la pantalla (selecciones grandes o
 * Ctrl+A), se pega al borde visible más cercano para que siempre se pueda usar.
 */
function posicionBurbuja(
  editor: Editor,
  posicion: number,
  alto: number,
): { top: number; left: number } {
  const ancho = 248;
  let coords = { top: 96, bottom: 116, left: 96, right: 96 };
  try {
    coords = editor.view.coordsAtPos(posicion);
  } catch {
    // En entornos sin motor de disposición (jsdom) se usa una posición estable;
    // en el WebView real coordsAtPos devuelve la del cursor.
  }
  const centro = (coords.left + coords.right) / 2;
  const left = Math.min(
    Math.max(12, centro - ancho / 2),
    Math.max(12, window.innerWidth - ancho - 12),
  );
  const sobre = coords.top - alto - 8;
  const debajo = coords.bottom + 10;
  const top = Math.min(
    Math.max(8, sobre >= 8 ? sobre : debajo),
    Math.max(8, window.innerHeight - alto - 8),
  );
  return { top, left };
}



type Props = {
  markdown: string;
  editable: boolean;
  personajes: OpcionMarca[];
  tramas: OpcionMarca[];
  onCambio: (markdown: string) => void;
  onComentario?: (texto: string, textoMarcado: string) => Promise<string | null>;
  onAbrirComentario?: (id: string) => void;
  onQuitarComentario?: (id: string) => void;
  ancla?: string | null;
  corrector?: boolean;
  onPegar?: (palabras: number) => void;
};

export const EditorMarkdown = forwardRef<EditorMarkdownHandle, Props>(function EditorMarkdown(
  {
    markdown,
    editable,
    personajes,
    tramas,
    onCambio,
    onComentario,
    onAbrirComentario,
    onQuitarComentario,
    ancla,
    corrector = true,
    onPegar,
  },
  ref,
) {
  const alCambiar = useRef(onCambio);
  alCambiar.current = onCambio;
  const [comentarioAbierto, setComentarioAbierto] = useState(false);
  const [comentarioTexto, setComentarioTexto] = useState('');
  const [errorComentario, setErrorComentario] = useState<string | null>(null);
  const [burbuja, setBurbuja] = useState<BurbujaSeleccion | null>(null);
  const [notaBurbuja, setNotaBurbuja] = useState('');
  const [errorBurbuja, setErrorBurbuja] = useState<string | null>(null);
  const burbujaRef = useRef<HTMLDivElement | null>(null);
  const [notaVisible, setNotaVisible] = useState<{
    top: number;
    left: number;
    id: string | null;
    texto: string;
    from: number;
    to: number;
  } | null>(null);
  const notaVisibleRef = useRef<HTMLDivElement | null>(null);
  const [palabras, setPalabras] = useState(() =>
    contarPalabras(textoSinMarcas(markdown)),
  );
  const ultimaAncla = useRef<string | null>(null);

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
        'aria-label': 'Texto del documento',
      },
    },
    onUpdate: ({ editor: instancia }) => {
      const serializado = instancia.getMarkdown();
      setPalabras(contarPalabras(instancia.getText()));
      alCambiar.current(serializado);
    },
  });

  useEffect(() => {
    if (editor && editor.isEditable !== editable) {
      editor.setEditable(editable);
    }
  }, [editor, editable]);

  useEffect(() => {
    editor?.view.dom.setAttribute('spellcheck', corrector ? 'true' : 'false');
  }, [editor, corrector]);

  useEffect(() => {
    if (!editor || !onPegar) {
      return;
    }
    const dom = editor.view.dom;
    const alPegar = (evento: ClipboardEvent) => {
      const texto = evento.clipboardData?.getData('text/plain') ?? '';
      if (texto.trim()) {
        onPegar(texto.trim().split(/\s+/u).filter(Boolean).length);
      }
    };
    dom.addEventListener('paste', alPegar);
    return () => dom.removeEventListener('paste', alPegar);
  }, [editor, onPegar]);

  useEffect(() => {
    if (!editor || !editable) {
      setBurbuja(null);
      return;
    }
    const calcular = () => {
      const { from, to } = editor.state.selection;
      if (from === to) {
        setBurbuja(null);
        return;
      }
      const { top, left } = posicionBurbuja(
        editor,
        to,
        Math.max(52, burbujaRef.current?.offsetHeight ?? 52),
      );
      setBurbuja((anterior) => {
        if (!anterior || anterior.from !== from || anterior.to !== to) {
          return { top, left, from, to, abierta: false };
        }
        if (anterior.top === top && anterior.left === left) {
          return anterior;
        }
        return { ...anterior, top, left };
      });
    };
    const alDesplazar = () => calcular();
    const alPulsarFuera = (evento: PointerEvent) => {
      const objetivo = evento.target;
      if (!(objetivo instanceof Node)) {
        return;
      }
      if (burbujaRef.current?.contains(objetivo)) {
        return;
      }
      if (editor.view.dom.contains(objetivo)) {
        return;
      }
      setBurbuja(null);
    };
    editor.on('selectionUpdate', calcular);
    editor.on('focus', calcular);
    window.addEventListener('scroll', alDesplazar, true);
    window.addEventListener('resize', alDesplazar);
    document.addEventListener('pointerdown', alPulsarFuera);
    calcular();
    return () => {
      editor.off('selectionUpdate', calcular);
      editor.off('focus', calcular);
      window.removeEventListener('scroll', alDesplazar, true);
      window.removeEventListener('resize', alDesplazar);
      document.removeEventListener('pointerdown', alPulsarFuera);
    };
  }, [editor, editable]);

  useEffect(() => {
    if (!editor) {
      return;
    }
    const dom = editor.view.dom;
    const alHacerClic = (evento: MouseEvent) => {
      const objetivo = evento.target;
      if (!(objetivo instanceof HTMLElement)) {
        return;
      }
      const marca = objetivo.closest<HTMLElement>('[data-marca-comentario]');
      if (!marca) {
        if (!notaVisibleRef.current?.contains(objetivo)) {
          setNotaVisible(null);
        }
        return;
      }
      const rect = marca.getBoundingClientRect();
      const ancho = 264;
      const alto = 180;
      const left = Math.min(
        Math.max(12, rect.left),
        Math.max(12, window.innerWidth - ancho - 12),
      );
      const sobre = rect.top - alto - 8;
      const top = Math.min(
        Math.max(8, sobre >= 8 ? sobre : rect.bottom + 8),
        Math.max(8, window.innerHeight - alto - 8),
      );
      let from = 0;
      let to = 0;
      try {
        from = editor.view.posAtDOM(marca, 0);
        to = editor.view.posAtDOM(marca, marca.childNodes.length);
      } catch {
        // Sin posiciones válidas solo se muestra la nota, sin acción de quitar.
      }
      setNotaVisible({
        top,
        left,
        id: marca.getAttribute('data-id'),
        texto: marca.getAttribute('data-referencia') ?? '',
        from,
        to,
      });
    };
    const alPulsarFuera = (evento: PointerEvent) => {
      if (!(evento.target instanceof Node)) {
        return;
      }
      if (notaVisibleRef.current?.contains(evento.target)) {
        return;
      }
      setNotaVisible(null);
    };
    dom.addEventListener('click', alHacerClic);
    document.addEventListener('pointerdown', alPulsarFuera);
    return () => {
      dom.removeEventListener('click', alHacerClic);
      document.removeEventListener('pointerdown', alPulsarFuera);
    };
  }, [editor]);

  useEffect(() => {
    if (!ancla || ancla === ultimaAncla.current) {
      return;
    }
    let inicio: number | null = null;
    let fin: number | null = null;
    editor?.state.doc.descendants((nodo, posicion) => {
      if (inicio !== null) {
        return false;
      }
      if (nodo.isText && nodo.text && nodo.text.includes(ancla)) {
        inicio = posicion + nodo.text.indexOf(ancla);
        fin = inicio + ancla.length;
        return false;
      }
      return true;
    });
    if (inicio !== null && fin !== null) {
      editor?.chain().focus().setTextSelection({ from: inicio, to: fin }).scrollIntoView().run();
      ultimaAncla.current = ancla;
    }
  }, [editor, ancla]);

  useEffect(() => {
    actualizarReferencias(personajes, tramas);
    // Un cambio de notas no cambia el documento, pero sí cómo se pinta el
    // título de las marcas. Se fuerza un repintado con una transacción sin
    // cambios (sin tocar el DOM a mano ni crear pasos de deshacer).
    editor?.view.dispatch(editor.state.tr.setMeta('referencias-marcas', true));
  }, [editor, personajes, tramas]);

  useImperativeHandle(
    ref,
    () => ({
      contar: (texto) => coincidenciasEnEditor(editor, texto).length,
      buscar: (texto, indice = 0) => {
        const coincidencias = coincidenciasEnEditor(editor, texto);
        if (!editor || coincidencias.length === 0) {
          return { total: 0, indice: 0 };
        }
        const activo = ((indice % coincidencias.length) + coincidencias.length) %
          coincidencias.length;
        const coincidencia = coincidencias[activo];
        editor
          .chain()
          .setTextSelection({ from: coincidencia.from, to: coincidencia.to })
          .scrollIntoView()
          .run();
        return { total: coincidencias.length, indice: activo };
      },
    }),
    [editor],
  );

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

  const aplicarComentarioEn = async (
    texto: string,
    from: number,
    to: number,
  ): Promise<string | null> => {
    const limpio = texto.trim();
    if (!comentarioValido(limpio)) {
      return 'El comentario no puede estar vacío ni contener «|» o «]]».';
    }
    if (from === to) {
      return 'Selecciona texto para crear el comentario.';
    }
    const seleccion = editor.state.doc.textBetween(from, to, '\n');
    if (!seleccion.trim()) {
      return 'Selecciona texto para crear el comentario.';
    }
    if (seleccion.includes(']]') || seleccion.includes('|')) {
      return 'La selección no puede contener «|» ni «]]».';
    }
    const id = onComentario ? await onComentario(limpio, seleccion) : null;
    if (onComentario && !id) {
      return 'No se pudo crear el hilo del comentario.';
    }
    editor
      .chain()
      .focus()
      .setTextSelection({ from, to })
      .unsetMark('personaje')
      .unsetMark('trama')
      .unsetMark('comentario')
      .setMark('comentario', id ? { texto: limpio, id } : { texto: limpio })
      .run();
    return null;
  };

  const aplicarComentario = async () => {
    const { from, to } = editor.state.selection;
    const error = await aplicarComentarioEn(comentarioTexto, from, to);
    if (error) {
      setErrorComentario(error);
      return;
    }
    setComentarioTexto('');
    setComentarioAbierto(false);
    setErrorComentario(null);
  };

  const aplicarComentarioBurbuja = async () => {
    if (!burbuja) {
      return;
    }
    const error = await aplicarComentarioEn(notaBurbuja, burbuja.from, burbuja.to);
    if (error) {
      setErrorBurbuja(error);
      return;
    }
    setNotaBurbuja('');
    setErrorBurbuja(null);
    setBurbuja(null);
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
    <div className="editor-content">
      {editable && (
        <div className="editor-toolbar" role="toolbar" aria-label="Marcas del texto">
          <button
            type="button"
            className={`formato ${estado?.negrita ? 'activo' : ''}`}
            aria-label="Negrita"
            aria-pressed={estado?.negrita ?? false}
            title="Negrita (Ctrl+B)"
            onClick={() => editor.chain().focus().toggleBold().run()}
          >
            B
          </button>
          <button
            type="button"
            className={`formato cursiva ${estado?.cursiva ? 'activo' : ''}`}
            aria-label="Cursiva"
            aria-pressed={estado?.cursiva ?? false}
            title="Cursiva (Ctrl+I)"
            onClick={() => editor.chain().focus().toggleItalic().run()}
          >
            I
          </button>
          <button
            type="button"
            className={`formato tachado ${estado?.tachado ? 'activo' : ''}`}
            aria-label="Tachado"
            aria-pressed={estado?.tachado ?? false}
            title="Tachado"
            onClick={() => editor.chain().focus().toggleStrike().run()}
          >
            S
          </button>
          <button
            type="button"
            className={`formato ${estado?.titulo2 ? 'activo' : ''}`}
            aria-label="Título de sección"
            aria-pressed={estado?.titulo2 ?? false}
            title="Título de sección"
            onClick={() => editor.chain().focus().toggleHeading({ level: 2 }).run()}
          >
            H2
          </button>
          <button
            type="button"
            className={`formato ${estado?.cita ? 'activo' : ''}`}
            aria-label="Cita"
            aria-pressed={estado?.cita ?? false}
            title="Cita"
            onClick={() => editor.chain().focus().toggleBlockquote().run()}
          >
            ❝
          </button>
          <button
            type="button"
            className={`formato ${estado?.lista ? 'activo' : ''}`}
            aria-label="Lista de viñetas"
            aria-pressed={estado?.lista ?? false}
            title="Lista de viñetas"
            onClick={() => editor.chain().focus().toggleBulletList().run()}
          >
            •
          </button>
          <button
            type="button"
            className={`formato ${estado?.ordenada ? 'activo' : ''}`}
            aria-label="Lista ordenada"
            aria-pressed={estado?.ordenada ?? false}
            title="Lista ordenada"
            onClick={() => editor.chain().focus().toggleOrderedList().run()}
          >
            1.
          </button>
          <button
            type="button"
            className={`formato ${estado?.codigo ? 'activo' : ''}`}
            aria-label="Bloque de código"
            aria-pressed={estado?.codigo ?? false}
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
                  if (evento.nativeEvent.isComposing) {
                    return;
                  }
                  if (evento.key === 'Enter') {
                    evento.preventDefault();
                    void aplicarComentario();
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
                onClick={() => void aplicarComentario()}
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
          <span className="editor-contador">{palabras} palabras</span>
        </div>
      )}
      <EditorContent editor={editor} />
      {burbuja && editable && (
        <div
          className="editor-burbuja"
          ref={burbujaRef}
          style={{ top: `${burbuja.top}px`, left: `${burbuja.left}px` }}
          role="dialog"
          aria-label="Nota sobre el texto seleccionado"
          onMouseDown={(evento) => {
            if (!burbuja.abierta) {
              evento.preventDefault();
            }
          }}
        >
          {burbuja.abierta ? (
            <div className="editor-burbuja-form">
              <input
                autoFocus
                aria-label="Texto de la nota"
                placeholder="Escribe tu nota…"
                value={notaBurbuja}
                onChange={(evento) => {
                  setNotaBurbuja(evento.target.value);
                  setErrorBurbuja(null);
                }}
                onKeyDown={(evento) => {
                  if (evento.nativeEvent.isComposing) {
                    return;
                  }
                  if (evento.key === 'Enter') {
                    evento.preventDefault();
                    void aplicarComentarioBurbuja();
                  } else if (evento.key === 'Escape') {
                    setBurbuja(null);
                    setNotaBurbuja('');
                    setErrorBurbuja(null);
                  }
                }}
              />
              <div className="editor-burbuja-acciones">
                <button
                  type="button"
                  className="secundario pequeno"
                  onClick={() => {
                    setBurbuja(null);
                    setNotaBurbuja('');
                    setErrorBurbuja(null);
                  }}
                >
                  Cancelar
                </button>
                <button
                  type="button"
                  className="dabble-boton-primario pequeno"
                  onClick={() => void aplicarComentarioBurbuja()}
                  disabled={notaBurbuja.trim() === ''}
                >
                  Añadir nota
                </button>
              </div>
              {errorBurbuja && (
                <span className="editor-error" role="alert">
                  {errorBurbuja}
                </span>
              )}
            </div>
          ) : (
            <button
              type="button"
              className="editor-burbuja-boton"
              onClick={() => {
                const { top, left } = posicionBurbuja(editor, burbuja.to, 180);
                setBurbuja({ ...burbuja, top, left, abierta: true });
                setNotaBurbuja('');
                setErrorBurbuja(null);
              }}
            >
              <MessageSquarePlus size={15} strokeWidth={1.8} aria-hidden="true" />
              Comentar
            </button>
          )}
        </div>
      )}
      {notaVisible && (
        <div
          className="editor-nota-popover"
          ref={notaVisibleRef}
          style={{ top: `${notaVisible.top}px`, left: `${notaVisible.left}px` }}
          role="dialog"
          aria-label="Comentario del texto"
        >
          <header className="editor-nota-cabecera">
            <span>Nota</span>
            <button
              type="button"
              className="cerrar-modal"
              aria-label="Cerrar nota"
              onClick={() => setNotaVisible(null)}
            >
              ×
            </button>
          </header>
          <p>{notaVisible.texto || 'Esta nota no tiene texto.'}</p>
          {(notaVisible.from < notaVisible.to ||
            (notaVisible.id && onAbrirComentario)) && (
            <div className="editor-nota-acciones">
              {notaVisible.from < notaVisible.to && (
                <button
                  type="button"
                  className="secundario pequeno"
                  onClick={() => {
                    if (notaVisible.from < notaVisible.to) {
                      editor
                        .chain()
                        .focus()
                        .setTextSelection({
                          from: notaVisible.from,
                          to: notaVisible.to,
                        })
                        .unsetMark('comentario')
                        .run();
                    }
                    if (notaVisible.id) {
                      onQuitarComentario?.(notaVisible.id);
                    }
                    setNotaVisible(null);
                  }}
                >
                  Quitar marca
                </button>
              )}
              <button
                type="button"
                className="secundario pequeno"
                onClick={() => {
                  if (notaVisible.id) {
                    onAbrirComentario?.(notaVisible.id);
                  }
                  setNotaVisible(null);
                }}
              >
                Ver en Comentarios
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
});
