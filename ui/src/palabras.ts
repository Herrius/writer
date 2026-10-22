export function textoSinMarcas(markdown: string): string {
  return markdown.replace(
    /\[\[(?:personaje|trama|comentario):[^|\]]*\|([\s\S]*?)\]\]/g,
    '$1',
  );
}

/**
 * Misma definición que `writer_core::model::contar_palabras`: secuencias
 * maximales de caracteres alfanuméricos Unicode sobre el texto visible. La
 * sintaxis de las marcas no cuenta; sí cuenta el texto marcado.
 */
export function contarPalabras(texto: string): number {
  let total = 0;
  let dentro = false;
  for (const caracter of texto) {
    if (/[\p{L}\p{N}]/u.test(caracter)) {
      if (!dentro) {
        total += 1;
        dentro = true;
      }
    } else {
      dentro = false;
    }
  }
  return total;
}
