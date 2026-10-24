import { afterEach, describe, expect, it } from 'vitest';
import {
  FILTROS_POR_DEFECTO,
  cargarFiltros,
  filtrarRecientes,
  guardarColecciones,
  guardarFiltros,
  leerColecciones,
  leerPapelera,
  guardarPapelera,
  nuevaColeccion,
} from './organizacion';
import type { Reciente } from './recientes';

const ahora = Date.now();
const recientes: Reciente[] = [
  {
    ruta: '/tmp/a',
    titulo: 'El umbral',
    autor: 'Marta',
    palabras: 100,
    modificado: new Date(ahora - 2 * 24 * 60 * 60 * 1000).toISOString(),
  },
  {
    ruta: '/tmp/b',
    titulo: 'La casa',
    autor: 'Lucía',
    palabras: 500,
    modificado: new Date(ahora - 40 * 24 * 60 * 60 * 1000).toISOString(),
  },
  {
    ruta: '/tmp/c',
    titulo: 'Otra novela',
    autor: 'Marta',
    palabras: 200,
    modificado: new Date(ahora - 1 * 24 * 60 * 60 * 1000).toISOString(),
  },
];

afterEach(() => {
  window.localStorage.clear();
});

describe('filtros de proyectos', () => {
  it('combina autor, búsqueda, palabras y periodo', () => {
    const filtrados = filtrarRecientes(
      recientes,
      { ...FILTROS_POR_DEFECTO, autor: 'marta', minimoPalabras: 150, periodo: '7d' },
      '',
    );
    expect(filtrados.map((item) => item.ruta)).toEqual(['/tmp/c']);
  });

  it('ordena por título y palabras', () => {
    expect(
      filtrarRecientes(recientes, { ...FILTROS_POR_DEFECTO, orden: 'titulo' }, '').map(
        (item) => item.titulo,
      ),
    ).toEqual(['El umbral', 'La casa', 'Otra novela']);
    expect(
      filtrarRecientes(recientes, { ...FILTROS_POR_DEFECTO, orden: 'palabras' }, '').map(
        (item) => item.palabras,
      ),
    ).toEqual([500, 200, 100]);
  });

  it('persiste filtros validados', () => {
    guardarFiltros({
      ...FILTROS_POR_DEFECTO,
      autor: 'Marta',
      periodo: '30d',
      minimoPalabras: 100,
    });
    expect(cargarFiltros()).toMatchObject({
      autor: 'Marta',
      periodo: '30d',
      minimoPalabras: 100,
    });
  });
});

describe('colecciones locales', () => {
  it('crea, valida y persiste una colección', () => {
    const coleccion = nuevaColeccion(' Pendientes ', '#4d6b8a');
    expect(coleccion.nombre).toBe('Pendientes');
    guardarColecciones([coleccion]);
    expect(leerColecciones()).toHaveLength(1);
    expect(leerColecciones()[0].rutas).toEqual([]);
  });

  it('descarta entradas inválidas', () => {
    window.localStorage.setItem(
      'writer.colecciones.v1',
      JSON.stringify([
        { id: 'x', nombre: 'Válida', color: '#ffffff', rutas: ['/tmp/a'], creada: 'x' },
        { id: 'y', nombre: '', color: 'rojo', rutas: [], creada: 'x' },
      ]),
    );
    const lista = leerColecciones();
    expect(lista).toHaveLength(1);
    expect(lista[0].nombre).toBe('Válida');
  });
});

describe('papelera local', () => {
  it('valida y persiste entradas', () => {
    const entrada = {
      id: 'demo',
      titulo: 'Proyecto',
      rutaOriginal: '/tmp/proyecto',
      rutaPapelera: '/tmp/.writer-papelera/demo',
      eliminado: new Date().toISOString(),
    };
    guardarPapelera([entrada]);
    expect(leerPapelera()).toEqual([entrada]);
    window.localStorage.setItem('writer.papelera.v1', '[{"id": 1}]');
    expect(leerPapelera()).toEqual([]);
  });
});
