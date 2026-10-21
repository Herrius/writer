import CodeBlock from '@tiptap/extension-code-block';

/**
 * Bloque de código con vallas seguras.
 *
 * El serializador de Tiptap usa siempre tres backticks. Si el contenido ya
 * incluye una secuencia de tres, la valla se cierra antes de tiempo y el bloque
 * se corrompe. Elegimos una valla más larga que cualquier secuencia de backticks
 * del contenido, como exige CommonMark.
 */
export const BloqueCodigo = CodeBlock.extend({
  renderMarkdown(node, helpers) {
    const lenguaje = (node.attrs?.language as string | null | undefined) ?? '';
    const contenido = node.content ? helpers.renderChildren(node.content) : '';
    const secuencias = contenido.match(/`+/g) ?? [];
    const mayor = secuencias.reduce((maximo, secuencia) => Math.max(maximo, secuencia.length), 2);
    const valla = '`'.repeat(Math.max(3, mayor + 1));
    return `${valla}${lenguaje}\n${contenido}\n${valla}`;
  },
});
