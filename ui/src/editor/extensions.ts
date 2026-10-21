import { Markdown } from '@tiptap/markdown';
import StarterKit from '@tiptap/starter-kit';
import { BloqueCodigo } from './code-block';
import { marcasEditor } from './marcas';
import { ReferenciasMarcas } from './referencias-plugin';

/**
 * Esquema único del editor.
 *
 * Debe coincidir con el corpus golden y con el ADR 0001: párrafo, título,
 * énfasis, negrita, tachado, código, listas, cita, bloque de código, enlace,
 * separador, salto de línea y las marcas propias de personajes, tramas y
 * comentarios. Cualquier extensión nueva exige ampliar el corpus y documentar la
 * pérdida o la normalización.
 */
export const extensionesEditor = [
  StarterKit.configure({ codeBlock: false }),
  BloqueCodigo,
  Markdown,
  ...marcasEditor,
  ReferenciasMarcas,
];
