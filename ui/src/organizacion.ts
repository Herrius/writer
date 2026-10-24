import type { Reciente } from './recientes';

export type PeriodoFiltro = 'todos' | '7d' | '30d' | 'ano';
export type OrdenProyectos = 'recientes' | 'titulo' | 'palabras';

export type FiltrosProyectos = {
  version: 1;
  autor: string;
  periodo: PeriodoFiltro;
  orden: OrdenProyectos;
  minimoPalabras: number;
};

export const FILTROS_POR_DEFECTO: FiltrosProyectos = {
  version: 1,
  autor: '',
  periodo: 'todos',
  orden: 'recientes',
  minimoPalabras: 0,
};

const CLAVE_FILTROS = 'writer.filtros.v1';
const CLAVE_COLECCIONES = 'writer.colecciones.v1';
const CLAVE_PAPELERA = 'writer.papelera.v1';

export type Coleccion = {
  id: string;
  nombre: string;
  color: string;
  rutas: string[];
  creada: string;
};

export type EntradaPapelera = {
  id: string;
  titulo: string;
  autor?: string | null;
  color?: string | null;
  palabras?: number;
  rutaOriginal: string;
  rutaPapelera: string;
  eliminado: string;
};

function fechaValida(valor: string | undefined): number | null {
  if (!valor) {
    return null;
  }
  const tiempo = Date.parse(valor);
  return Number.isNaN(tiempo) ? null : tiempo;
}

export function cargarFiltros(): FiltrosProyectos {
  try {
    const datos = JSON.parse(window.localStorage.getItem(CLAVE_FILTROS) ?? 'null');
    if (
      !datos ||
      datos.version !== 1 ||
      typeof datos.autor !== 'string' ||
      !['todos', '7d', '30d', 'ano'].includes(datos.periodo) ||
      !['recientes', 'titulo', 'palabras'].includes(datos.orden)
    ) {
      return { ...FILTROS_POR_DEFECTO };
    }
    return {
      version: 1,
      autor: datos.autor.slice(0, 200),
      periodo: datos.periodo,
      orden: datos.orden,
      minimoPalabras: Math.max(0, Math.min(1_000_000, Number(datos.minimoPalabras) || 0)),
    };
  } catch {
    return { ...FILTROS_POR_DEFECTO };
  }
}

export function guardarFiltros(filtros: FiltrosProyectos): void {
  try {
    window.localStorage.setItem(CLAVE_FILTROS, JSON.stringify(filtros));
  } catch {
    // El almacenamiento puede estar deshabilitado.
  }
}

export function filtrosActivos(filtros: FiltrosProyectos): number {
  return (
    (filtros.autor.trim() ? 1 : 0) +
    (filtros.periodo !== 'todos' ? 1 : 0) +
    (filtros.minimoPalabras > 0 ? 1 : 0)
  );
}

export function filtrarRecientes(
  recientes: Reciente[],
  filtros: FiltrosProyectos,
  consulta: string,
  rutasColeccion: string[] | null = null,
): Reciente[] {
  const termino = consulta.trim().toLocaleLowerCase();
  const autor = filtros.autor.trim().toLocaleLowerCase();
  const ahora = Date.now();
  const limite =
    filtros.periodo === '7d'
      ? ahora - 7 * 24 * 60 * 60 * 1000
      : filtros.periodo === '30d'
        ? ahora - 30 * 24 * 60 * 60 * 1000
        : filtros.periodo === 'ano'
          ? ahora - 365 * 24 * 60 * 60 * 1000
          : null;
  const rutas = rutasColeccion ? new Set(rutasColeccion) : null;
  return recientes
    .filter((reciente) => {
      if (rutas && !rutas.has(reciente.ruta)) {
        return false;
      }
      if (
        termino &&
        !reciente.titulo.toLocaleLowerCase().includes(termino) &&
        !reciente.ruta.toLocaleLowerCase().includes(termino)
      ) {
        return false;
      }
      if (autor && !(reciente.autor ?? '').toLocaleLowerCase().includes(autor)) {
        return false;
      }
      if ((reciente.palabras ?? 0) < filtros.minimoPalabras) {
        return false;
      }
      if (limite !== null) {
        const fecha = fechaValida(reciente.modificado);
        if (fecha === null || fecha < limite) {
          return false;
        }
      }
      return true;
    })
    .sort((a, b) => {
      if (filtros.orden === 'titulo') {
        return a.titulo.localeCompare(b.titulo, 'es', { sensitivity: 'base' });
      }
      if (filtros.orden === 'palabras') {
        return (b.palabras ?? 0) - (a.palabras ?? 0);
      }
      // `recientes` respeta el orden del almacén local, que ya coloca el
      // proyecto abierto o creado más recientemente al principio.
      return 0;
    });
}

export function autoresRecientes(recientes: Reciente[]): string[] {
  return Array.from(
    new Set(
      recientes
        .map((reciente) => reciente.autor?.trim() ?? '')
        .filter((autor) => autor.length > 0),
    ),
  ).sort((a, b) => a.localeCompare(b, 'es', { sensitivity: 'base' }));
}

function idLocal(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }
  return `col-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export function leerColecciones(): Coleccion[] {
  try {
    const datos = JSON.parse(window.localStorage.getItem(CLAVE_COLECCIONES) ?? '[]');
    if (!Array.isArray(datos)) {
      return [];
    }
    return datos
      .filter(
        (item): item is Coleccion =>
          typeof item?.id === 'string' &&
          item.id.length <= 80 &&
          typeof item.nombre === 'string' &&
          item.nombre.trim().length > 0 &&
          item.nombre.length <= 120 &&
          typeof item.color === 'string' &&
          /^#[0-9a-f]{6}$/i.test(item.color) &&
          Array.isArray(item.rutas) &&
          item.rutas.every((ruta: unknown) => typeof ruta === 'string' && ruta.length <= 4096) &&
          typeof item.creada === 'string',
      )
      .map((item) => ({
        ...item,
        nombre: item.nombre.trim(),
        rutas: Array.from(new Set(item.rutas)).slice(0, 1000),
      }))
      .slice(0, 50);
  } catch {
    return [];
  }
}

export function guardarColecciones(colecciones: Coleccion[]): void {
  try {
    window.localStorage.setItem(CLAVE_COLECCIONES, JSON.stringify(colecciones));
  } catch {
    // Sin almacenamiento, la sesión sigue funcionando en memoria.
  }
}

export function nuevaColeccion(nombre: string, color: string): Coleccion {
  return {
    id: idLocal(),
    nombre: nombre.trim().slice(0, 120),
    color: /^#[0-9a-f]{6}$/i.test(color) ? color : '#6b5747',
    rutas: [],
    creada: new Date().toISOString(),
  };
}

export function leerPapelera(): EntradaPapelera[] {
  try {
    const datos = JSON.parse(window.localStorage.getItem(CLAVE_PAPELERA) ?? '[]');
    if (!Array.isArray(datos)) {
      return [];
    }
    return datos
      .filter(
        (item): item is EntradaPapelera =>
          typeof item?.id === 'string' &&
          typeof item.titulo === 'string' &&
          typeof item.rutaOriginal === 'string' &&
          typeof item.rutaPapelera === 'string' &&
          typeof item.eliminado === 'string',
      )
      .slice(0, 100);
  } catch {
    return [];
  }
}

export function guardarPapelera(entradas: EntradaPapelera[]): void {
  try {
    window.localStorage.setItem(CLAVE_PAPELERA, JSON.stringify(entradas));
  } catch {
    // Sin almacenamiento, la papelera visible no persiste.
  }
}
