import { Markdown } from '@tiptap/markdown';
import StarterKit from '@tiptap/starter-kit';

/**
 * Esquema único del editor.
 *
 * Debe coincidir con el corpus golden y con el ADR 0001: párrafo, título,
 * énfasis, negrita, tachado, código, listas, cita, bloque de código, enlace,
 * separador y salto de línea. Cualquier extensión nueva exige ampliar el
 * corpus y documentar la pérdida o la normalización.
 */
export const extensionesEditor = [StarterKit, Markdown];
