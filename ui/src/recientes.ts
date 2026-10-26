export type Reciente = {
  ruta: string;
  titulo: string;
  autor?: string;
  palabras?: number;
  color?: string;
  modificado?: string;
};

const CLAVE_RECIENTES = 'writer.recientes.v1';

const COLORES = [
  '#4d6b8a',
  '#5a8fa3',
  '#4f9b8a',
  '#7ba05b',
  '#c2a24a',
  '#d18b47',
  '#c97a63',
  '#b65f62',
  '#8f5d7a',
  '#9b6aa8',
  '#6f6fa8',
  '#6b6f76',
];

export function colorDesdeTitulo(titulo: string): string {
  let suma = 0;
  for (const caracter of titulo) {
    suma = (suma * 31 + caracter.codePointAt(0)!) >>> 0;
  }
  return COLORES[suma % COLORES.length];
}

export function leerRecientes(): Reciente[] {
  try {
    const datos = JSON.parse(window.localStorage.getItem(CLAVE_RECIENTES) ?? '[]');
    if (!Array.isArray(datos)) {
      return [];
    }
    const vistos = new Set<string>();
    return datos
      .filter(
        (item): item is Reciente =>
          typeof item === 'object' &&
          item !== null &&
          typeof item.ruta === 'string' &&
          item.ruta.length > 0 &&
          item.ruta.length <= 4096 &&
          typeof item.titulo === 'string' &&
          item.titulo.length > 0 &&
          item.titulo.length <= 200 &&
          (item.autor === undefined ||
            (typeof item.autor === 'string' && item.autor.length <= 200)) &&
          (item.palabras === undefined ||
            (typeof item.palabras === 'number' &&
              Number.isFinite(item.palabras) &&
              item.palabras >= 0)) &&
          (item.color === undefined || /^#[0-9a-f]{6}$/i.test(item.color)) &&
          (item.modificado === undefined ||
            (typeof item.modificado === 'string' &&
              item.modificado.length <= 40 &&
              !Number.isNaN(Date.parse(item.modificado)))),
      )
      .filter((item) => {
        if (vistos.has(item.ruta)) {
          return false;
        }
        vistos.add(item.ruta);
        return true;
      })
      .slice(0, 8);
  } catch {
    return [];
  }
}

export function guardarRecientes(recientes: Reciente[]): void {
  try {
    window.localStorage.setItem(CLAVE_RECIENTES, JSON.stringify(recientes));
  } catch {
    // El almacenamiento local puede estar deshabilitado; no bloquea la app.
  }
}

const RESERVADOS_WINDOWS =
  /^(con|prn|aux|nul|com[1-9]|lpt[1-9]|conin\$|conout\$|clock\$)$/i;

export function nombreArchivoSeguro(titulo: string, extension: string): string {
  const limpio = titulo
    .replace(/[\\/:*?"<>|]+/g, '-')
    .replace(/\s+/g, ' ')
    .trim()
    .replace(/[. ]+$/g, '');
  const base = limpio || 'documento';
  const seguro = RESERVADOS_WINDOWS.test(base) ? `documento-${base}` : base;
  return `${seguro.slice(0, 200)}.${extension}`;
}

export function slugProyecto(titulo: string): string {
  const base = titulo
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
  if (base) {
    const recortado = base.slice(0, 120).replace(/-+$/g, '');
    // Un proyecto llamado «CON» no se puede crear en Windows; se prefija para
    // que el mismo título funcione igual en Linux y Windows.
    return RESERVADOS_WINDOWS.test(recortado)
      ? `proyecto-${recortado.slice(0, 110)}`
      : recortado;
  }
  let suma = 0;
  for (const caracter of titulo) {
    suma = (suma * 31 + caracter.codePointAt(0)!) >>> 0;
  }
  return `proyecto-${suma.toString(36)}`;
}
