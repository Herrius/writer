import { EditorContent, useEditor } from '@tiptap/react';
import { useEffect, useRef } from 'react';
import { extensionesEditor } from './editor/extensions';

type Props = {
  markdown: string;
  editable: boolean;
  onCambio: (markdown: string) => void;
};

export function EditorMarkdown({ markdown, editable, onCambio }: Props) {
  const alCambiar = useRef(onCambio);
  alCambiar.current = onCambio;

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

  if (!editor) {
    return <div className="editor-cargando">Preparando el editor…</div>;
  }

  return (
    <div className="editor-content" aria-label="Texto del documento">
      <EditorContent editor={editor} />
    </div>
  );
}
