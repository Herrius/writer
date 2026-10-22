import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Inicio } from './Inicio';
import { listarDirectorios } from './api';

vi.mock('./api', () => ({
  elegirCarpetaCreacion: vi.fn(),
  listarDirectorios: vi.fn(),
  errorDe: (fallo: unknown) => ({
    tipo: 'otro',
    mensaje: typeof fallo === 'object' && fallo !== null && 'mensaje' in fallo
      ? String(fallo.mensaje)
      : String(fallo),
  }),
}));

const listarDirectoriosMock = vi.mocked(listarDirectorios);

function renderInicio() {
  render(
    <Inicio
      recientes={[]}
      esTauri
      errorInicial={null}
      onAbrir={vi.fn()}
      onDemo={vi.fn()}
      onAbrirReciente={vi.fn()}
      onCrear={vi.fn()}
    />,
  );
}

describe('explorador de carpetas', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('navega por carpetas y usa la carpeta actual', async () => {
    listarDirectoriosMock
      .mockResolvedValueOnce({
        ruta: '/home/enrique',
        padre: '/home',
        directorios: [
          { nombre: 'Documentos', ruta: '/home/enrique/Documentos' },
          { nombre: 'Imágenes', ruta: '/home/enrique/Imágenes' },
        ],
      })
      .mockResolvedValueOnce({
        ruta: '/home/enrique/Documentos',
        padre: '/home/enrique',
        directorios: [],
      });

    renderInicio();
    fireEvent.click(screen.getByRole('button', { name: /Crear nuevo proyecto/ }));
    fireEvent.click(screen.getByRole('button', { name: 'Examinar…' }));

    expect(await screen.findByRole('dialog', { name: 'Elegir carpeta' })).toBeTruthy();
    expect(await screen.findByText('/home/enrique')).toBeTruthy();
    expect(listarDirectoriosMock).toHaveBeenCalledWith(null);

    fireEvent.click(screen.getByRole('button', { name: /Documentos/ }));
    expect(await screen.findByText('/home/enrique/Documentos')).toBeTruthy();
    expect(listarDirectoriosMock).toHaveBeenLastCalledWith('/home/enrique/Documentos');

    fireEvent.click(screen.getByRole('button', { name: 'Usar esta carpeta' }));
    await waitFor(() =>
      expect((screen.getByLabelText('Carpeta padre') as HTMLInputElement).value).toBe(
        '/home/enrique/Documentos',
      ),
    );
    expect(screen.getByRole('dialog', { name: 'Crear nuevo proyecto' })).toBeTruthy();
  });
});
