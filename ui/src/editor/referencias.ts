/**
 * Resolución de los ULID de las marcas contra las notas cargadas.
 *
 * Las extensiones de Tiptap se crean una sola vez; este registro mutable les
 * permite leer el título vigente sin mutar el DOM que ProseMirror observa.
 */

export type ReferenciaMarca = {
  id: string;
  titulo: string;
};

const personajes = new Map<string, string>();
const tramas = new Map<string, string>();

function reemplazar(destino: Map<string, string>, opciones: ReferenciaMarca[]): void {
  destino.clear();
  for (const opcion of opciones) {
    destino.set(opcion.id, opcion.titulo);
  }
}

export function actualizarReferencias(
  nuevosPersonajes: ReferenciaMarca[],
  nuevasTramas: ReferenciaMarca[],
): void {
  reemplazar(personajes, nuevosPersonajes);
  reemplazar(tramas, nuevasTramas);
}

export function tituloPersonaje(id: string): string | null {
  return personajes.get(id) ?? null;
}

export function tituloTrama(id: string): string | null {
  return tramas.get(id) ?? null;
}
