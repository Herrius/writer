/**
 * Construcciones Markdown que el esquema del editor no conserva.
 *
 * El corpus golden documenta las normalizaciones seguras (setext → ATX,
 * referencias → enlaces inline, escapado de literales). Aquí solo se detectan
 * las que cambian el significado o pierden contenido. Si un documento las usa,
 * la interfaz lo abre en solo lectura para no destruirlas al autoguardar.
 */

export type HallazgoCompatibilidad = {
  construccion: string;
  detalle: string;
};

type Patron = {
  construccion: string;
  detalle: string;
  probar: (texto: string) => boolean;
};

/** Quita bloques y fragmentos de código: dentro de ellos todo es literal. */
export function sinCodigo(markdown: string): string {
  const sinBloques = markdown.replace(/^(```|~~~)[^\n]*\n[\s\S]*?^\1[^\n]*$/gm, '');
  return sinBloques.replace(/(`+)([^`]|[^`][\s\S]*?[^`])\1(?!`)/g, '');
}

function tieneTabla(texto: string): boolean {
  const lineas = texto.split('\n');
  for (let indice = 0; indice < lineas.length - 1; indice += 1) {
    const actual = lineas[indice].trim();
    const siguiente = lineas[indice + 1].trim();
    if (!actual.includes('|') || !siguiente.includes('|')) {
      continue;
    }
    const separador = siguiente.replace(/\|/g, '').replace(/[\s:-]/g, '');
    if (separador === '' && siguiente.includes('-')) {
      return true;
    }
  }
  return false;
}

const PATRONES: Patron[] = [
  {
    construccion: 'HTML embebido',
    detalle: 'El editor descarta las etiquetas y conserva solo el texto.',
    probar: (texto) =>
      /<\/?[a-z][a-z0-9-]*(?:\s[^>]*)?\/?>/i.test(texto) || texto.includes('<!--'),
  },
  {
    construccion: 'imágenes',
    detalle: 'El editor conserva el texto alternativo, pero pierde la imagen.',
    probar: (texto) => /!\[[^\]]*\]\([^)]*\)/.test(texto),
  },
  {
    construccion: 'tablas',
    detalle: 'El editor convierte la tabla en un párrafo y pierde las celdas.',
    probar: tieneTabla,
  },
  {
    construccion: 'listas de tareas',
    detalle: 'El editor conserva el texto, pero pierde las casillas.',
    probar: (texto) => /^\s*[-*+]\s+\[[ xX]\]\s+/m.test(texto),
  },
  {
    construccion: 'notas al pie',
    detalle: 'El editor escapa la referencia y la deja como texto literal.',
    probar: (texto) => /\[\^[^\]]+\]/.test(texto),
  },
];

/**
 * Devuelve los hallazgos que impiden editar con seguridad.
 *
 * Es una comprobación deliberadamente conservadora: un falso positivo abre el
 * documento en solo lectura y nunca lo sobrescribe.
 */
export function analizarCompatibilidad(markdown: string): HallazgoCompatibilidad[] {
  const texto = sinCodigo(markdown);
  return PATRONES.filter((patron) => patron.probar(texto)).map(
    ({ construccion, detalle }) => ({ construccion, detalle }),
  );
}
