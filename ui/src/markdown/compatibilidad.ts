/**
 * Construcciones Markdown que el esquema del editor no conserva.
 *
 * El corpus golden documenta las normalizaciones seguras (setext → ATX,
 * referencias → enlaces inline, escapado de literales). Aquí solo se detectan
 * las que cambian el significado o pierden contenido. Si un documento las usa,
 * la interfaz lo abre en solo lectura para no destruirlas al autoguardar.
 *
 * La detección usa el lexer de CommonMark/GFM (`marked`) en vez de expresiones
 * regulares sobre el texto: así las tablas y las tareas dentro de citas o listas
 * se detectan igual que en el resto del documento, y el contenido de los bloques
 * de código se ignora sin falsos positivos.
 */

import { marked } from 'marked';

export type HallazgoCompatibilidad = {
  construccion: string;
  detalle: string;
};

type Construccion =
  | 'HTML embebido'
  | 'imágenes'
  | 'tablas'
  | 'listas de tareas'
  | 'notas al pie';

const DETALLES: Record<Construccion, string> = {
  'HTML embebido': 'El editor descarta las etiquetas y conserva solo el texto.',
  imágenes: 'El editor conserva el texto alternativo, pero pierde la imagen.',
  tablas: 'El editor convierte la tabla en un párrafo y pierde las celdas.',
  'listas de tareas': 'El editor conserva el texto, pero pierde las casillas.',
  'notas al pie': 'El editor escapa la referencia y la deja como texto literal.',
};

const ORDEN: Construccion[] = [
  'HTML embebido',
  'imágenes',
  'tablas',
  'listas de tareas',
  'notas al pie',
];

type TokenMarcado = {
  type?: string;
  task?: boolean;
  raw?: string;
  text?: string;
  tag?: string;
};

function detectar(markdown: string): Set<Construccion> {
  const encontradas = new Set<Construccion>();
  const tokens = marked.lexer(markdown);
  marked.walkTokens(tokens, (token) => {
    const actual = token as TokenMarcado;
    switch (actual.type) {
      case 'html':
        encontradas.add('HTML embebido');
        break;
      case 'image':
        encontradas.add('imágenes');
        break;
      case 'table':
        encontradas.add('tablas');
        break;
      case 'list_item':
        if (actual.task) {
          encontradas.add('listas de tareas');
        }
        break;
      case 'link':
        if (actual.raw?.startsWith('[^')) {
          encontradas.add('notas al pie');
        }
        break;
      case 'def':
        if (actual.tag?.startsWith('^')) {
          encontradas.add('notas al pie');
        }
        break;
      case 'text':
        // Una imagen de referencia sin definición queda como texto en el lexer;
        // igualmente perdería su sintaxis al editar, así que se bloquea.
        if (actual.text?.includes('![')) {
          encontradas.add('imágenes');
        }
        if (actual.text?.includes('[^')) {
          encontradas.add('notas al pie');
        }
        break;
      default:
        break;
    }
  });
  return encontradas;
}

/**
 * Devuelve los hallazgos que impiden editar con seguridad.
 *
 * Es una comprobación deliberadamente conservadora: un falso positivo abre el
 * documento en solo lectura y nunca lo sobrescribe.
 */
export function analizarCompatibilidad(markdown: string): HallazgoCompatibilidad[] {
  const encontradas = detectar(markdown);
  return ORDEN.filter((construccion) => encontradas.has(construccion)).map(
    (construccion) => ({ construccion, detalle: DETALLES[construccion] }),
  );
}
