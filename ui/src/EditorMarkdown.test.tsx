import { Editor } from '@tiptap/core';
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { EditorMarkdown } from './EditorMarkdown';
import { extensionesEditor } from './editor/extensions';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

const personajes = [{ id: '01KDVDNBYG20KW71NVRK6P2FHG', titulo: 'Marta' }];
const tramas = [{ id: '01KDVDNFVGFKJ1G1SEHGTVZ69D', titulo: 'El faro' }];

async function seleccionarTexto(markdown = 'Marta') {
  // La selección se fija en el editor real; jsdom no necesita ejecutar el
  // desplazamiento de la vista que solicita focus() en la toolbar.
  vi.stubGlobal('requestAnimationFrame', () => 0);
  const onCambio = vi.fn();
  render(
    <EditorMarkdown
      markdown={markdown}
      editable
      personajes={personajes}
      tramas={tramas}
      onCambio={onCambio}
    />,
  );
  await screen.findByRole('toolbar', { name: 'Marcas del texto' });
  const dom = document.querySelector('[contenteditable=true]') as HTMLElement & { editor: Editor };
  act(() => dom.editor.commands.selectAll());
  return { editor: dom.editor, onCambio };
}

function reabrir(markdown: string) {
  return new Editor({ extensions: extensionesEditor, content: markdown, contentType: 'markdown' });
}

describe('EditorMarkdown con marcas', () => {
  it('muestra la toolbar con personajes y tramas', async () => {
    render(
      <EditorMarkdown
        markdown="Marta llegó al faro."
        editable
        personajes={personajes}
        tramas={tramas}
        onCambio={() => {}}
      />,
    );

    expect(await screen.findByRole('toolbar', { name: 'Marcas del texto' })).toBeTruthy();
    expect(screen.getByRole('combobox', { name: 'Aplicar personaje' })).toBeTruthy();
    expect(screen.getByRole('combobox', { name: 'Aplicar trama' })).toBeTruthy();
    expect(screen.getByRole('option', { name: 'Marta' })).toBeTruthy();
    expect(screen.getByRole('option', { name: 'El faro' })).toBeTruthy();
  });

  it('cuenta palabras al abrir con la definición del motor', async () => {
    render(
      <EditorMarkdown
        markdown="uno-dos"
        editable
        personajes={personajes}
        tramas={tramas}
        onCambio={() => {}}
      />,
    );
    await screen.findByRole('toolbar', { name: 'Marcas del texto' });
    expect(screen.getByText('2 palabras')).toBeTruthy();

    const dom = document.querySelector('[contenteditable=true]') as HTMLElement & {
      editor: Editor;
    };
    act(() => {
      dom.editor.commands.selectAll();
      dom.editor.commands.insertContent('tres');
    });
    expect(screen.getByText('1 palabras')).toBeTruthy();
  });

  it('resuelve el título de la marca y avisa de referencias desconocidas', async () => {
    render(
      <EditorMarkdown
        markdown={`[[personaje:${personajes[0].id}|Marta]] y [[trama:01KDVDNFVGFKJ1G1SEHGTVZ69D|El faro]]`}
        editable
        personajes={personajes}
        tramas={[]}
        onCambio={() => {}}
      />,
    );
    await screen.findByRole('toolbar', { name: 'Marcas del texto' });
    const personaje = document.querySelector('.marca-personaje') as HTMLElement;
    const trama = document.querySelector('.marca-trama') as HTMLElement;

    await waitFor(() => expect(personaje.title).toContain('Marta'));
    expect(trama.title).toContain('referencia desconocida');
    expect(trama.getAttribute('data-referencia-desconocida')).toBe('true');
  });

  it('oculta la toolbar en solo lectura', async () => {
    render(
      <EditorMarkdown
        markdown="Marta llegó al faro."
        editable={false}
        personajes={personajes}
        tramas={tramas}
        onCambio={() => {}}
      />,
    );

    await screen.findByLabelText('Texto del documento');
    expect(screen.queryByRole('toolbar')).toBeNull();
  });

  it('sustituye personaje por trama y conserva la prosa al reabrir', async () => {
    const { onCambio } = await seleccionarTexto();
    fireEvent.change(screen.getByRole('combobox', { name: 'Aplicar personaje' }), {
      target: { value: personajes[0].id },
    });
    fireEvent.change(screen.getByRole('combobox', { name: 'Aplicar trama' }), {
      target: { value: tramas[0].id },
    });
    const markdown = onCambio.mock.calls.at(-1)?.[0] as string;
    expect(markdown).toBe(`[[trama:${tramas[0].id}|Marta]]`);
    const editor = reabrir(markdown);
    try {
      expect(editor.getText()).toBe('Marta');
      expect(editor.getMarkdown()).toBe(markdown);
    } finally {
      editor.destroy();
    }
  });

  it.each(['Nota | secreta', 'Nota ]] privada'])(
    'rechaza el comentario %s antes de cambiar el manuscrito',
    async (texto) => {
      const { editor, onCambio } = await seleccionarTexto();
      fireEvent.click(screen.getByRole('button', { name: 'Comentario' }));
      fireEvent.change(screen.getByRole('textbox', { name: 'Texto del comentario' }), {
        target: { value: texto },
      });
      fireEvent.click(screen.getByRole('button', { name: 'Aplicar' }));
      expect(screen.getByRole('alert').textContent).toContain('El comentario');
      expect(editor.getText()).toBe('Marta');
      expect(onCambio).not.toHaveBeenCalled();
    },
  );

  it('conserva un comentario con corchetes simples al aplicar y reabrir', async () => {
    const { onCambio } = await seleccionarTexto();
    fireEvent.click(screen.getByRole('button', { name: 'Comentario' }));
    fireEvent.change(screen.getByRole('textbox', { name: 'Texto del comentario' }), {
      target: { value: 'ver [capítulo 2]' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Aplicar' }));
    const markdown = onCambio.mock.calls.at(-1)?.[0] as string;
    expect(markdown).toBe('[[comentario:ver [capítulo 2]|Marta]]');
    const editor = reabrir(markdown);
    try {
      expect(editor.getText()).toBe('Marta');
      expect(editor.getAttributes('comentario').texto).toBe('ver [capítulo 2]');
      expect(editor.getMarkdown()).toBe(markdown);
    } finally {
      editor.destroy();
    }
  });

  it('rechaza delimitadores de cierre en la selección para personaje y trama', async () => {
    const { editor, onCambio } = await seleccionarTexto('Marta ]] literal');
    for (const [nombre, id] of [
      ['Aplicar personaje', personajes[0].id],
      ['Aplicar trama', tramas[0].id],
    ]) {
      fireEvent.change(screen.getByRole('combobox', { name: nombre }), { target: { value: id } });
      expect(screen.getByRole('alert').textContent).toContain('La selección');
    }
    expect(editor.getText()).toBe('Marta ]] literal');
    expect(onCambio).not.toHaveBeenCalled();
  });
});
