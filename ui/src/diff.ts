import type { LineaDiffDto } from './types/LineaDiffDto';

export type ResultadoDiff = {
  lineas: LineaDiffDto[];
  insertadas: number;
  borradas: number;
};

function dividir(texto: string): string[] {
  const normalizado = texto.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  const sinFinal = normalizado.endsWith('\n') ? normalizado.slice(0, -1) : normalizado;
  return sinFinal ? sinFinal.split('\n') : [];
}

/**
 * Diferencias por líneas para el modo demo y las pruebas de UI. El motor real
 * devuelve la misma forma desde `writer-core`.
 */
export function compararLineas(base: string, propuesta: string): ResultadoDiff {
  const a = dividir(base);
  const b = dividir(propuesta);
  let prefijo = 0;
  while (prefijo < a.length && prefijo < b.length && a[prefijo] === b[prefijo]) {
    prefijo += 1;
  }
  let sufijo = 0;
  while (
    sufijo < a.length - prefijo &&
    sufijo < b.length - prefijo &&
    a[a.length - 1 - sufijo] === b[b.length - 1 - sufijo]
  ) {
    sufijo += 1;
  }

  const lineas: LineaDiffDto[] = [];
  for (let indice = 0; indice < prefijo; indice += 1) {
    lineas.push({
      tipo: 'igual',
      numeroBase: indice + 1,
      numeroPropuesta: indice + 1,
      texto: a[indice],
    });
  }

  const medioA = a.slice(prefijo, a.length - sufijo);
  const medioB = b.slice(prefijo, b.length - sufijo);
  const tabla = Array.from({ length: medioA.length + 1 }, () =>
    Array<number>(medioB.length + 1).fill(0),
  );
  for (let fila = medioA.length - 1; fila >= 0; fila -= 1) {
    for (let columna = medioB.length - 1; columna >= 0; columna -= 1) {
      tabla[fila][columna] =
        medioA[fila] === medioB[columna]
          ? tabla[fila + 1][columna + 1] + 1
          : Math.max(tabla[fila + 1][columna], tabla[fila][columna + 1]);
    }
  }

  let fila = 0;
  let columna = 0;
  let indiceBase = prefijo;
  let indicePropuesta = prefijo;
  while (fila < medioA.length && columna < medioB.length) {
    if (medioA[fila] === medioB[columna]) {
      lineas.push({
        tipo: 'igual',
        numeroBase: indiceBase + 1,
        numeroPropuesta: indicePropuesta + 1,
        texto: medioA[fila],
      });
      fila += 1;
      columna += 1;
      indiceBase += 1;
      indicePropuesta += 1;
    } else if (tabla[fila + 1][columna] >= tabla[fila][columna + 1]) {
      lineas.push({
        tipo: 'borrada',
        numeroBase: indiceBase + 1,
        numeroPropuesta: null,
        texto: medioA[fila],
      });
      fila += 1;
      indiceBase += 1;
    } else {
      lineas.push({
        tipo: 'insertada',
        numeroBase: null,
        numeroPropuesta: indicePropuesta + 1,
        texto: medioB[columna],
      });
      columna += 1;
      indicePropuesta += 1;
    }
  }
  while (fila < medioA.length) {
    lineas.push({
      tipo: 'borrada',
      numeroBase: indiceBase + 1,
      numeroPropuesta: null,
      texto: medioA[fila],
    });
    fila += 1;
    indiceBase += 1;
  }
  while (columna < medioB.length) {
    lineas.push({
      tipo: 'insertada',
      numeroBase: null,
      numeroPropuesta: indicePropuesta + 1,
      texto: medioB[columna],
    });
    columna += 1;
    indicePropuesta += 1;
  }

  for (let indice = 0; indice < sufijo; indice += 1) {
    const baseIndice = a.length - sufijo + indice;
    const propuestaIndice = b.length - sufijo + indice;
    lineas.push({
      tipo: 'igual',
      numeroBase: baseIndice + 1,
      numeroPropuesta: propuestaIndice + 1,
      texto: a[baseIndice],
    });
  }

  return {
    lineas,
    insertadas: lineas.filter((linea) => linea.tipo === 'insertada').length,
    borradas: lineas.filter((linea) => linea.tipo === 'borrada').length,
  };
}
