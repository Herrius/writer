import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import App from './App';
import {
  abrirProyecto,
  agregarSesion,
  buscar,
  crearProyecto,
  cuarentena,
  descartarCuarentena,
  duplicar,
  elegirCarpetaCreacion,
  elegirDestinoDocx,
  escribir,
  exportarDocx,
  guardarMetas,
  historial,
  leer,
  leerMetas,
  leerSesiones,
  listar,
  mostrar,
  renombrar,
  revertir,
} from './api';
import type { DocumentoDto } from './types/DocumentoDto';
import type { ProyectoDto } from './types/ProyectoDto';
import type { RevisionDetalleDto } from './types/RevisionDetalleDto';

vi.mock('./EditorMarkdown', () => ({
  EditorMarkdown: ({
    markdown,
    editable,
    onCambio,
  }: {
    markdown: string;
    editable: boolean;
    onCambio: (markdown: string) => void;
  }) => (
    <textarea
      aria-label="editor de prueba"
      defaultValue={markdown}
      disabled={!editable}
      onChange={(evento) => onCambio(evento.target.value)}
    />
  ),
}));

vi.mock('./api', () => ({
  esTauri: false,
  elegirProyecto: vi.fn(),
  abrirProyecto: vi.fn(),
  crearProyecto: vi.fn(),
  elegirCarpetaCreacion: vi.fn(),
  listar: vi.fn(),
  leer: vi.fn(),
  verificar: vi.fn(),
  escribir: vi.fn(),
  buscar: vi.fn(),
  historial: vi.fn(),
  mostrar: vi.fn(),
  revertir: vi.fn(),
  elegirDestinoDocx: vi.fn(),
  exportarDocx: vi.fn(),
  cuarentena: vi.fn(),
  descartarCuarentena: vi.fn(),
  leerMetas: vi.fn(),
  guardarMetas: vi.fn(),
  agregarSesion: vi.fn(),
  leerSesiones: vi.fn(),
  renombrar: vi.fn(),
  duplicar: vi.fn(),
  proyectoE2e: vi.fn().mockResolvedValue(null),
  errorDe: (fallo: unknown) => {
    if (typeof fallo === 'object' && fallo !== null && 'tipo' in fallo && 'mensaje' in fallo) {
      return fallo;
    }
    return { tipo: 'otro', mensaje: String(fallo) };
  },
}));

const abrirMock = vi.mocked(abrirProyecto);
const crearProyectoMock = vi.mocked(crearProyecto);
const elegirCarpetaCreacionMock = vi.mocked(elegirCarpetaCreacion);
const listarMock = vi.mocked(listar);
const leerMock = vi.mocked(leer);
const escribirMock = vi.mocked(escribir);
const buscarMock = vi.mocked(buscar);
const historialMock = vi.mocked(historial);
const mostrarMock = vi.mocked(mostrar);
const revertirMock = vi.mocked(revertir);
const elegirDestinoDocxMock = vi.mocked(elegirDestinoDocx);
const exportarDocxMock = vi.mocked(exportarDocx);
const cuarentenaMock = vi.mocked(cuarentena);
const descartarCuarentenaMock = vi.mocked(descartarCuarentena);
const leerMetasMock = vi.mocked(leerMetas);
const guardarMetasMock = vi.mocked(guardarMetas);
const agregarSesionMock = vi.mocked(agregarSesion);
const leerSesionesMock = vi.mocked(leerSesiones);
const renombrarMock = vi.mocked(renombrar);
const duplicarMock = vi.mocked(duplicar);

const proyecto: ProyectoDto = {
  raiz: '/tmp/novela',
  titulo: 'Novela',
  id: '01PROYECTO0000000000000000',
  autor: null,
  idioma: 'es',
  revision: 1,
  escenas: 1,
  notas: 0,
  errores: [],
  avisos: [],
};

const documento: DocumentoDto = {
  tipo: 'escena',
  id: '01DOCUMENTO000000000000000',
  ruta: 'manuscrito/01-libro/01-capitulo/001-uno.md',
  titulo: 'Uno',
  cuerpo: 'Texto original.\n',
  hashArchivo: 'blake3:original',
  orden: 10,
  estado: 'borrador',
  archivo: null,
  etiquetas: [],
  clase: null,
};

function diferida<T>() {
  let resolver!: (valor: T) => void;
  let rechazar!: (error: unknown) => void;
  const promesa = new Promise<T>((resolverPromesa, rechazarPromesa) => {
    resolver = resolverPromesa;
    rechazar = rechazarPromesa;
  });
  return { promesa, resolver, rechazar };
}

function detalleDe(revision: number): RevisionDetalleDto {
  return {
    revision,
    opId: `op-${revision}`,
    ts: '2026-10-21T00:00:00Z',
    autor: 'humano',
    scope: 'prosa',
    contenidos: [
      {
        ruta: documento.ruta,
        contenido: `---\nid: ${documento.id}\n---\n\nTexto de la revisión ${revision}.\n`,
      },
    ],
  };
}

function preparar(documentoActivo: DocumentoDto = documento, resumen: ProyectoDto = proyecto) {
  abrirMock.mockResolvedValue(resumen);
  crearProyectoMock.mockResolvedValue(resumen);
  elegirCarpetaCreacionMock.mockResolvedValue('/tmp');
  listarMock.mockResolvedValue([documentoActivo]);
  leerMock.mockResolvedValue({ ...documentoActivo });
  escribirMock.mockImplementation(async (_ruta, _id, cuerpo) => ({
    revision: 2,
    hashArchivo: `blake3:${cuerpo.length}`,
  }));
  buscarMock.mockResolvedValue([
    {
      id: documentoActivo.id,
      ruta: documentoActivo.ruta,
      titulo: documentoActivo.titulo,
      tipo: documentoActivo.tipo,
      fragmento: '…una luz sobre el \u0001puerto\u0002…',
    },
  ]);
  historialMock.mockResolvedValue([
    {
      revision: 1,
      opId: 'op-1',
      ts: '2026-10-20T17:00:00Z',
      autor: 'humano',
      scope: 'estructura',
      efectos: 1,
      rutas: [documentoActivo.ruta],
    },
    {
      revision: 2,
      opId: 'op-2',
      ts: '2026-10-20T17:01:00Z',
      autor: 'humano',
      scope: 'prosa',
      efectos: 1,
      rutas: [documentoActivo.ruta],
    },
  ]);
  mostrarMock.mockImplementation(async (_ruta, revision) => ({
    revision,
    opId: `op-${revision}`,
    ts: '2026-10-20T17:01:00Z',
    autor: 'humano',
    scope: 'prosa',
    contenidos: [
      {
        ruta: documentoActivo.ruta,
        contenido: `---\nid: ${documentoActivo.id}\n---\n\nTexto de la revisión ${revision}.\n`,
      },
    ],
  }));
  revertirMock.mockResolvedValue({ revision: 3, erroresPendientes: [] });
  elegirDestinoDocxMock.mockResolvedValue('/tmp/salida.docx');
  exportarDocxMock.mockResolvedValue('/tmp/salida.docx');
  cuarentenaMock.mockResolvedValue([]);
  descartarCuarentenaMock.mockResolvedValue();
  leerMetasMock.mockResolvedValue({
    version: 1,
    diaria: null,
    libro: null,
    desafio: null,
    archivadas: [],
  });
  guardarMetasMock.mockResolvedValue();
  agregarSesionMock.mockResolvedValue('(demo)');
  leerSesionesMock.mockResolvedValue([]);
  renombrarMock.mockImplementation(async (_ruta, _objetivo, titulo) => ({
    ...documento,
    titulo,
  }));
  duplicarMock.mockResolvedValue({
    ...documento,
    id: '01DUPLICADO000000000000000',
    titulo: `Copia de ${documento.titulo}`,
    ruta: 'manuscrito/01-libro/01-capitulo/002-copia.md',
  });
}

async function abrirDemo(): Promise<HTMLTextAreaElement> {
  render(<App />);
  fireEvent.click(screen.getByRole('button', { name: 'Ver demo' }));
  return screen.findByRole('textbox', { name: 'editor de prueba' });
}

function estadoGuardado(): string {
  return screen.getByRole('status', { name: 'Estado de guardado' }).textContent ?? '';
}

beforeEach(() => {
  window.localStorage.clear();
  vi.clearAllMocks();
  preparar();
});

afterEach(() => {
  cleanup();
});

describe('autosave de la interfaz', () => {
  it('crea un proyecto desde el asistente y lo abre', async () => {
    crearProyectoMock.mockResolvedValueOnce({
      ...proyecto,
      raiz: '/tmp/mi-novela',
      titulo: 'Mi novela',
    });
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: /Crear nuevo proyecto/ }));

    fireEvent.change(screen.getByLabelText('Título'), {
      target: { value: 'Mi novela' },
    });
    fireEvent.change(screen.getByLabelText('Autor'), {
      target: { value: 'Autora' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Examinar…' }));
    await screen.findByDisplayValue('/tmp');
    fireEvent.click(screen.getByRole('button', { name: 'Guardar y abrir' }));

    await waitFor(() =>
      expect(crearProyectoMock).toHaveBeenCalledWith(
        '/tmp/mi-novela',
        'Mi novela',
        'Autora',
        'es',
        true,
      ),
    );
    expect(await screen.findByRole('textbox', { name: 'editor de prueba' })).toBeTruthy();
  });

  it('confirma Guardar, limpia el filtro y muestra el proyecto nuevo', async () => {
    crearProyectoMock.mockResolvedValueOnce({
      ...proyecto,
      raiz: '/tmp/novela-guardada',
      titulo: 'Novela guardada',
    });
    render(<App />);
    fireEvent.change(screen.getByRole('textbox', { name: 'Buscar proyectos' }), {
      target: { value: 'zzz' },
    });
    fireEvent.click(screen.getByRole('button', { name: /Crear nuevo proyecto/ }));
    fireEvent.change(screen.getByLabelText('Título'), {
      target: { value: 'Novela guardada' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Examinar…' }));
    await screen.findByDisplayValue('/tmp');
    fireEvent.click(screen.getByRole('button', { name: 'Guardar' }));

    await screen.findByText(/Proyecto «Novela guardada» creado y guardado en recientes/);
    expect(
      (screen.getByRole('textbox', { name: 'Buscar proyectos' }) as HTMLInputElement).value,
    ).toBe('');
    expect(screen.getAllByText('Novela guardada').length).toBeGreaterThan(0);
    expect(crearProyectoMock).toHaveBeenCalledWith(
      '/tmp/novela-guardada',
      'Novela guardada',
      null,
      'es',
      true,
    );
  });

  it('guarda tras el debounce y actualiza la revisión', async () => {
    const editor = await abrirDemo();
    fireEvent.change(editor, { target: { value: 'Texto cambiado.\n' } });
    expect(estadoGuardado()).toBe('cambios sin guardar');

    await waitFor(
      () => {
        expect(escribirMock).toHaveBeenCalledTimes(1);
      },
      { timeout: 3000 },
    );
    expect(escribirMock).toHaveBeenCalledWith(
      '/tmp/novela',
      documento.id,
      'Texto cambiado.\n',
      'blake3:original',
    );
    await waitFor(() => expect(estadoGuardado()).toBe('guardado'));
    expect(screen.getAllByText('revisión 2').length).toBeGreaterThan(0);
  });

  it('guarda de inmediato con Ctrl+S', async () => {
    const editor = await abrirDemo();
    fireEvent.change(editor, { target: { value: 'Guardado inmediato.\n' } });
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });

    await waitFor(() => expect(escribirMock).toHaveBeenCalledTimes(1));
    expect(estadoGuardado()).toBe('guardado');
  });

  it('no declara guardado si el editor cambia durante un guardado en vuelo', async () => {
    const enVuelo = diferida<{ revision: number; hashArchivo: string }>();
    escribirMock
      .mockReturnValueOnce(enVuelo.promesa)
      .mockResolvedValueOnce({ revision: 4, hashArchivo: 'blake3:final' });

    const editor = await abrirDemo();
    fireEvent.change(editor, { target: { value: 'Versión intermedia.\n' } });
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });
    await waitFor(() => expect(escribirMock).toHaveBeenCalledTimes(1));

    // El usuario deshace y vuelve al cuerpo original mientras la escritura
    // intermedia sigue en vuelo.
    fireEvent.change(editor, { target: { value: documento.cuerpo } });
    expect(estadoGuardado()).toBe('cambios sin guardar');

    await act(async () => {
      enVuelo.resolver({ revision: 3, hashArchivo: 'blake3:intermedio' });
    });

    await waitFor(() => expect(escribirMock).toHaveBeenCalledTimes(2));
    expect(escribirMock.mock.calls[1][2]).toBe(documento.cuerpo);
    await waitFor(() => expect(estadoGuardado()).toBe('guardado'));
  });

  it('entra en conflicto, bloquea la edición y permite recargar', async () => {
    escribirMock.mockRejectedValueOnce({
      tipo: 'conflicto',
      mensaje: 'el documento cambió fuera del editor',
    });
    const editor = await abrirDemo();
    fireEvent.change(editor, { target: { value: 'Texto en conflicto.\n' } });
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });

    await screen.findByText('No se pudo guardar.');
    await waitFor(() => expect(estadoGuardado()).toBe('conflicto'));
    expect(editor.disabled).toBe(true);
    await waitFor(() =>
      expect(document.activeElement?.getAttribute('role')).toBe('alert'),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Recargar desde disco' }));
    expect(estadoGuardado()).toBe('conflicto');
    fireEvent.click(screen.getByRole('button', { name: 'Confirmar y descartar' }));
    await waitFor(() => expect(estadoGuardado()).toBe('guardado'));
    expect(screen.getByText(/Documento recargado desde disco/)).toBeTruthy();
  });

  it('abre en solo lectura si el proyecto tiene errores', async () => {
    preparar(documento, { ...proyecto, errores: ['fallo de prueba'] });
    const editor = await abrirDemo();
    expect(estadoGuardado()).toBe('solo lectura');
    expect(editor.disabled).toBe(true);
  });

  it('bloquea documentos con Markdown que el editor no conserva', async () => {
    preparar({ ...documento, cuerpo: '# Título\n\n![foto](foto.png)\n' });
    const editor = await abrirDemo();
    expect(estadoGuardado()).toBe('solo lectura');
    expect(editor.disabled).toBe(true);
    expect(screen.getByText(/Markdown que el editor no conserva/)).toBeTruthy();
    expect(screen.getByText(/imágenes/)).toBeTruthy();
  });

  it('agrupa las escenas por libro y capítulo sin fundir las categorías', async () => {
    const segunda: DocumentoDto = {
      ...documento,
      id: '01DOCUMENTO000000000000001',
      ruta: 'manuscrito/02-libro/01-capitulo/001-dos.md',
      titulo: 'Dos',
    };
    abrirMock.mockResolvedValue({ ...proyecto, escenas: 2 });
    listarMock.mockResolvedValue([documento, segunda]);
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Ver demo' }));

    expect(
      await screen.findByText('Libro 1'),
    ).toBeTruthy();
    expect(
      screen.getByText('Libro 2'),
    ).toBeTruthy();
  });

  it('busca en el proyecto y muestra el fragmento resaltado', async () => {
    await abrirDemo();
    const buscador = screen.getByRole('textbox', { name: 'Buscar en el proyecto' });
    fireEvent.change(buscador, { target: { value: 'puerto' } });

    await waitFor(
      () => expect(buscarMock).toHaveBeenCalledWith('/tmp/novela', 'puerto', 30),
      { timeout: 3000 },
    );
    const resaltado = await screen.findByText('puerto');
    expect(resaltado.tagName).toBe('MARK');
    expect(screen.getByText('…una luz sobre el')).toBeTruthy();
  });

  it('muestra el historial y revierte una revisión con confirmación', async () => {
    await abrirDemo();
    fireEvent.click(screen.getByRole('button', { name: 'Historial' }));

    await screen.findByText('Historial de versiones');
    await screen.findByRole('button', { name: /Revisión 2/ });
    fireEvent.click(screen.getByRole('button', { name: /Revisión 2/ }));

    await screen.findByRole('heading', { name: 'Revisión 2' });
    await screen.findByText(/Texto de la revisión 2/);
    fireEvent.click(screen.getByRole('button', { name: 'Recuperar' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirmar recuperación' }));

    await waitFor(() => expect(revertirMock).toHaveBeenCalledWith('/tmp/novela', 2));
    await screen.findByText(/Demo: reversión simulada de la revisión 2/);
  });

  it('conserva el borrador al cambiar de pestaña antes del autosave', async () => {
    const editor = await abrirDemo();
    fireEvent.change(editor, { target: { value: 'Borrador uno.\n' } });

    fireEvent.click(screen.getByRole('button', { name: 'Historial' }));
    fireEvent.click(screen.getByRole('button', { name: /Editor:/ }));

    const remontado = await screen.findByRole('textbox', { name: 'editor de prueba' });
    expect((remontado as HTMLTextAreaElement).value).toBe('Borrador uno.\n');

    fireEvent.change(remontado, { target: { value: 'Borrador uno. Dos.\n' } });
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });
    await waitFor(() =>
      expect(escribirMock).toHaveBeenCalledWith(
        '/tmp/novela',
        documento.id,
        'Borrador uno. Dos.\n',
        'blake3:original',
      ),
    );
  });

  it('bloquea la edición durante una reversión', async () => {
    const pendiente = diferida<{ revision: number; erroresPendientes: string[] }>();
    revertirMock.mockReturnValueOnce(pendiente.promesa);
    await abrirDemo();
    fireEvent.click(screen.getByRole('button', { name: 'Historial' }));
    await screen.findByRole('button', { name: /Revisión 2/ });
    fireEvent.click(screen.getByRole('button', { name: /Revisión 2/ }));
    await screen.findByRole('heading', { name: 'Revisión 2' });
    fireEvent.click(screen.getByRole('button', { name: 'Recuperar' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirmar recuperación' }));
    await waitFor(() => expect(revertirMock).toHaveBeenCalledTimes(1));
    expect(screen.getByText(/Revirtiendo la revisión/)).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: /Editor:/ }));
    const editor = screen.getByRole('textbox', {
      name: 'editor de prueba',
    }) as HTMLTextAreaElement;
    expect(editor.disabled).toBe(true);
    fireEvent.change(editor, { target: { value: 'Texto durante la reversión.\n' } });

    await act(async () => {
      pendiente.resolver({ revision: 3, erroresPendientes: [] });
    });
    await screen.findByText(/Demo: reversión simulada de la revisión 2/);
    expect(escribirMock).not.toHaveBeenCalled();
  });

  it('descarta respuestas antiguas del historial antes de confirmar', async () => {
    const respuesta1 = diferida<RevisionDetalleDto>();
    const respuesta2 = diferida<RevisionDetalleDto>();
    mostrarMock.mockImplementation((_ruta, revision) =>
      revision === 1 ? respuesta1.promesa : respuesta2.promesa,
    );
    await abrirDemo();
    fireEvent.click(screen.getByRole('button', { name: 'Historial' }));
    await screen.findByRole('button', { name: /Revisión 2/ });

    fireEvent.click(screen.getByRole('button', { name: /Versión actual/ }));
    fireEvent.click(screen.getByRole('button', { name: /Revisión 2/ }));
    await act(async () => {
      respuesta2.resolver(detalleDe(2));
    });
    await screen.findByRole('heading', { name: 'Revisión 2' });
    fireEvent.click(screen.getByRole('button', { name: 'Recuperar' }));

    await act(async () => {
      respuesta1.resolver(detalleDe(1));
    });
    expect(screen.getByRole('heading', { name: 'Revisión 2' })).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Confirmar recuperación' }));
    await waitFor(() => expect(revertirMock).toHaveBeenCalledWith('/tmp/novela', 2));
  });

  it('mantiene el historial cuando el proyecto queda sin documentos', async () => {
    abrirMock.mockResolvedValue({ ...proyecto, escenas: 0, notas: 0 });
    listarMock.mockResolvedValue([]);
    render(<App />);
    fireEvent.click(screen.getByRole('button', { name: 'Ver demo' }));

    await screen.findByText('El proyecto no tiene documentos válidos.');
    const botonHistorial = screen.getByRole('button', { name: 'Historial' });
    expect(botonHistorial.hasAttribute('disabled')).toBe(false);
    fireEvent.click(botonHistorial);
    await screen.findByText('Historial de versiones');
  });

  it('repite la búsqueda cuando cambia la revisión', async () => {
    await abrirDemo();
    const buscador = screen.getByRole('textbox', { name: 'Buscar en el proyecto' });
    fireEvent.change(buscador, { target: { value: 'puerto' } });
    await waitFor(
      () => expect(buscarMock).toHaveBeenCalledTimes(1),
      { timeout: 3000 },
    );

    buscarMock.mockResolvedValueOnce([
      {
        id: documento.id,
        ruta: documento.ruta,
        titulo: documento.titulo,
        tipo: documento.tipo,
        fragmento: '…el \u0001faro\u0002 del puerto…',
      },
    ]);
    const editor = screen.getByRole('textbox', { name: 'editor de prueba' });
    fireEvent.change(editor, { target: { value: 'Puerto nuevo.\n' } });
    fireEvent.keyDown(window, { key: 's', ctrlKey: true });

    await waitFor(() => expect(escribirMock).toHaveBeenCalled());
    await waitFor(
      () => expect(buscarMock).toHaveBeenCalledTimes(2),
      { timeout: 3000 },
    );
    expect(await screen.findByText('faro')).toBeTruthy();
  });

  it('renombra y duplica el documento activo', async () => {
    await abrirDemo();
    fireEvent.click(screen.getByRole('tab', { name: 'Notas' }));

    const titulo = screen.getByRole('textbox', { name: 'Título del documento' });
    fireEvent.change(titulo, { target: { value: 'Nuevo título' } });
    fireEvent.click(screen.getByRole('button', { name: 'Renombrar' }));
    await waitFor(() =>
      expect(renombrarMock).toHaveBeenCalledWith('/tmp/novela', documento.id, 'Nuevo título'),
    );

    fireEvent.click(screen.getByRole('button', { name: 'Duplicar' }));
    await waitFor(() => expect(duplicarMock).toHaveBeenCalledWith('/tmp/novela', documento.id));
  });

  it('exporta el manuscrito a DOCX con las notas elegidas', async () => {
    await abrirDemo();
    fireEvent.click(screen.getByRole('checkbox', { name: 'notas' }));
    fireEvent.click(screen.getByRole('button', { name: 'Exportar DOCX' }));

    await waitFor(() =>
      expect(exportarDocxMock).toHaveBeenCalledWith(
        '/tmp/novela',
        '/tmp/salida.docx',
        true,
        true,
      ),
    );
    expect(await screen.findByText(/Demo: exportación simulada a \/tmp\/salida.docx/)).toBeTruthy();
  });

  it('muestra y descarta copias de cuarentena', async () => {
    cuarentenaMock.mockResolvedValue([
      {
        archivo: '20260101T000000Z-roto-esquema.md',
        rutaOriginal: 'manuscrito/01-libro/01-capitulo/999-roto.md',
        motivo: 'esquema',
        ts: '2026-01-01T00:00:00Z',
        tamano: 24,
      },
    ]);
    await abrirDemo();
    expect(await screen.findByText('Cuarentena (1)')).toBeTruthy();
    expect(screen.getByText(/manuscrito\/01-libro/)).toBeTruthy();

    fireEvent.click(screen.getByRole('button', { name: 'Descartar' }));
    fireEvent.click(screen.getByRole('button', { name: 'Confirmar descarte' }));
    await waitFor(() =>
      expect(descartarCuarentenaMock).toHaveBeenCalledWith(
        '/tmp/novela',
        '20260101T000000Z-roto-esquema.md',
      ),
    );
    expect(await screen.findByText(/Copia de cuarentena descartada/)).toBeTruthy();
  });
});

describe('panel de proyectos', () => {
  it('retira meta diaria, proyectos personales, Ideas y Ayuda', () => {
    render(<App />);

    expect(screen.queryByText('Proyectos personales')).toBeNull();
    expect(screen.queryByText('Ideas')).toBeNull();
    expect(screen.queryByRole('button', { name: 'Configurar meta diaria' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Ayuda' })).toBeNull();

    expect(screen.getByRole('button', { name: /Crear nuevo proyecto/ })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Notificaciones' })).toBeTruthy();
    expect(screen.getByRole('button', { name: 'Filtros' })).toBeTruthy();
    expect(screen.getByRole('button', { name: /Agregar una colección/ })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Plantillas' })).toBeTruthy();
    expect(screen.getByRole('tab', { name: 'Papelera' })).toBeTruthy();
  });
});
