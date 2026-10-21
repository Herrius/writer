import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { EditorMarkdown } from './EditorMarkdown';

afterEach(cleanup);

const personajes = [{ id: '01KDVDNBYG20KW71NVRK6P2FHG', titulo: 'Marta' }];
const tramas = [{ id: '01KDVDNFVGFKJ1G1SEHGTVZ69D', titulo: 'El faro' }];

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
});
