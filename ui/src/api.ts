import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { DocumentoDto } from './types/DocumentoDto';
import type { BusquedaDto } from './types/BusquedaDto';
import type { CuarentenaDto } from './types/CuarentenaDto';
import type { ErrorDto } from './types/ErrorDto';
import type { EscaneoDto } from './types/EscaneoDto';
import type { GuardadoDto } from './types/GuardadoDto';
import type { ProyectoDto } from './types/ProyectoDto';
import type { ReversionDto } from './types/ReversionDto';
import type { RevisionDetalleDto } from './types/RevisionDetalleDto';
import type { RevisionDto } from './types/RevisionDto';
import type { TipoError } from './types/TipoError';

export const esTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

function hashDemo(texto: string): string {
  let hash = 5381;
  for (let indice = 0; indice < texto.length; indice += 1) {
    hash = (hash * 33) ^ texto.charCodeAt(indice);
  }
  return `demo:${(hash >>> 0).toString(16)}`;
}

const demoProyecto: ProyectoDto = {
  raiz: '(modo demo)',
  titulo: 'El umbral',
  id: '01KDVDNAZ84TFF59TDWH9EDD1R',
  autor: 'Autora',
  idioma: 'es',
  revision: 3,
  escenas: 2,
  notas: 2,
  errores: [],
  avisos: [],
};

const demoCuarentena: CuarentenaDto[] = [
  {
    archivo: '20260101T000000Z-primera-escena-esquema.md',
    rutaOriginal: 'manuscrito/01-libro/01-capitulo/001-el-umbral.md',
    motivo: 'esquema: falta el cierre del frontmatter',
    ts: '2026-01-01T00:00:00Z',
    tamano: 512,
  },
];

const demoDocumentos: DocumentoDto[] = [
  {
    tipo: 'escena',
    id: '01KDVDNBYG1HEAD8X4A1JH69RE',
    ruta: 'manuscrito/01-libro/01-capitulo/001-el-umbral.md',
    titulo: 'El umbral',
    cuerpo:
      'La casa olía a lluvia vieja. Marta dejó la maleta en el umbral y escuchó el silencio, como si la casa hubiera estado esperándola.\n\n> No era la misma casa.\n\nEl faro seguía allí, al fondo del puerto, con su luz girando cada doce segundos.',
    hashArchivo: 'demo:umbral',
    orden: 10,
    estado: 'revision',
    clase: null,
  },
  {
    tipo: 'escena',
    id: '01KDVDNCXRH4QX4FR84G98PBSK',
    ruta: 'manuscrito/01-libro/01-capitulo/002-la-llave.md',
    titulo: 'La llave',
    cuerpo:
      'En el cajón de la cocina encontró una llave de bronce. No abría ninguna puerta de la casa; lo supo antes de probarla.\n\nEl cuaderno azul estaba debajo, con la primera página arrancada.',
    hashArchivo: 'demo:llave',
    orden: 20,
    estado: 'borrador',
    clase: null,
  },
  {
    tipo: 'nota',
    id: '01KDVDNBYG20KW71NVRK6P2FHG',
    ruta: 'notas/personajes/marta.md',
    titulo: 'Marta',
    cuerpo: 'Protagonista. Vuelve a Puerto Viejo después de diez años. No confía en su memoria.',
    hashArchivo: 'demo:marta',
    orden: null,
    estado: null,
    clase: 'personajes',
  },
  {
    tipo: 'nota',
    id: '01KDVDNFVGFKJ1G1SEHGTVZ69D',
    ruta: 'notas/tramas/el-mapa-incompleto.md',
    titulo: 'El mapa incompleto',
    cuerpo: 'La casa heredada esconde un plano que solo se entiende al final.',
    hashArchivo: 'demo:mapa',
    orden: null,
    estado: null,
    clase: 'tramas',
  },
];

function demoFiltrado(notas: boolean): DocumentoDto[] {
  return demoDocumentos.filter((documento) => notas || documento.tipo === 'escena');
}

function fragmentoDemo(cuerpo: string, consulta: string): string {
  const aguja = consulta.toLocaleLowerCase();
  const indice = cuerpo.toLocaleLowerCase().indexOf(aguja);
  if (indice < 0) {
    return cuerpo.slice(0, 140);
  }
  const inicio = Math.max(0, indice - 40);
  const fin = Math.min(cuerpo.length, indice + consulta.length + 60);
  return `${inicio > 0 ? '…' : ''}${cuerpo.slice(inicio, indice)}[${cuerpo.slice(
    indice,
    indice + consulta.length,
  )}]${cuerpo.slice(indice + consulta.length, fin)}${fin < cuerpo.length ? '…' : ''}`;
}

export async function elegirProyecto(): Promise<string | null> {
  if (!esTauri) {
    return window.prompt('Ruta del proyecto (modo demo: deja vacío para usar la demo)');
  }
  const seleccion = await open({
    directory: true,
    multiple: false,
    title: 'Abrir proyecto writer',
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function elegirDestinoDocx(titulo: string): Promise<string | null> {
  const nombre = `${titulo.replace(/[\\/:*?"<>|]/g, '').trim() || 'novela'}.docx`;
  if (!esTauri) {
    return window.prompt('Ruta del DOCX', nombre);
  }
  const seleccion = await save({
    title: 'Exportar manuscrito a DOCX',
    defaultPath: nombre,
    filters: [{ name: 'Documento Word', extensions: ['docx'] }],
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function abrirProyecto(ruta: string): Promise<ProyectoDto> {
  if (!esTauri) {
    return demoProyecto;
  }
  return invoke<ProyectoDto>('abrir_proyecto', { ruta });
}

export async function listar(ruta: string, notas: boolean): Promise<DocumentoDto[]> {
  if (!esTauri) {
    return demoFiltrado(notas);
  }
  return invoke<DocumentoDto[]>('listar', { ruta, notas });
}

export async function leer(ruta: string, objetivo: string): Promise<DocumentoDto> {
  if (!esTauri) {
    const documento = demoDocumentos.find(
      (candidato) => candidato.id === objetivo || candidato.ruta === objetivo,
    );
    if (!documento) {
      throw {
        tipo: 'no-encontrado',
        mensaje: `documento no encontrado en la demo: ${objetivo}`,
      } satisfies ErrorDto;
    }
    return { ...documento };
  }
  return invoke<DocumentoDto>('leer', { ruta, objetivo });
}

export async function verificar(ruta: string): Promise<EscaneoDto> {
  if (!esTauri) {
    return {
      escenas: demoProyecto.escenas,
      notas: demoProyecto.notas,
      errores: [],
      avisos: [],
    };
  }
  return invoke<EscaneoDto>('verificar', { ruta });
}

export async function buscar(
  ruta: string,
  consulta: string,
  limite: number,
): Promise<BusquedaDto[]> {
  if (!esTauri) {
    const aguja = consulta.toLocaleLowerCase();
    return demoDocumentos
      .filter(
        (documento) =>
          documento.titulo.toLocaleLowerCase().includes(aguja) ||
          documento.cuerpo.toLocaleLowerCase().includes(aguja),
      )
      .slice(0, limite)
      .map((documento) => ({
        id: documento.id,
        ruta: documento.ruta,
        titulo: documento.titulo,
        tipo: documento.tipo,
        fragmento: fragmentoDemo(documento.cuerpo, consulta),
      }));
  }
  return invoke<BusquedaDto[]>('buscar', { ruta, consulta, limite });
}

export async function historial(ruta: string): Promise<RevisionDto[]> {
  if (!esTauri) {
    const rutas = demoDocumentos.map((documento) => documento.ruta);
    const revisiones: RevisionDto[] = [];
    for (let revision = demoProyecto.revision; revision >= 1 && revisiones.length < 5; revision -= 1) {
      revisiones.push({
        revision,
        opId: `demo-op-${revision}`,
        ts: new Date(Date.UTC(2026, 9, 20, 17, revision)).toISOString(),
        autor: 'humano',
        scope: revision === demoProyecto.revision ? 'prosa' : 'estructura',
        efectos: revision === demoProyecto.revision ? 1 : 2,
        rutas: revision === demoProyecto.revision ? [rutas[0]] : rutas.slice(0, 2),
      });
    }
    return revisiones;
  }
  return invoke<RevisionDto[]>('historial', { ruta });
}

export async function mostrar(ruta: string, revision: number): Promise<RevisionDetalleDto> {
  if (!esTauri) {
    const entrada = (await historial(ruta)).find((item) => item.revision === revision);
    return {
      revision,
      opId: entrada?.opId ?? `demo-op-${revision}`,
      ts: entrada?.ts ?? '2026-10-20T17:00:00Z',
      autor: 'humano',
      scope: entrada?.scope ?? 'prosa',
      contenidos: demoDocumentos.map((documento) => ({
        ruta: documento.ruta,
        contenido:
          `---\nid: ${documento.id}\ntitulo: "${documento.titulo}"\n---\n\n` + documento.cuerpo,
      })),
    };
  }
  return invoke<RevisionDetalleDto>('mostrar', { ruta, revision });
}

export async function revertir(ruta: string, revision: number): Promise<ReversionDto> {
  if (!esTauri) {
    demoProyecto.revision += 1;
    return { revision: demoProyecto.revision, erroresPendientes: [] };
  }
  return invoke<ReversionDto>('revertir', { ruta, revision });
}

export async function exportarDocx(
  ruta: string,
  destino: string,
  incluirNotas: boolean,
  forzar: boolean,
): Promise<string> {
  if (!esTauri) {
    return destino;
  }
  return invoke<string>('exportar_docx', { ruta, destino, incluirNotas, forzar });
}

export async function cuarentena(ruta: string): Promise<CuarentenaDto[]> {
  if (!esTauri) {
    return demoCuarentena.map((item) => ({ ...item }));
  }
  return invoke<CuarentenaDto[]>('cuarentena', { ruta });
}

export async function descartarCuarentena(ruta: string, archivo: string): Promise<void> {
  if (!esTauri) {
    const indice = demoCuarentena.findIndex((item) => item.archivo === archivo);
    if (indice >= 0) {
      demoCuarentena.splice(indice, 1);
    }
    return;
  }
  return invoke<void>('descartar_cuarentena', { ruta, archivo });
}

export async function proyectoE2e(): Promise<string | null> {
  if (!esTauri) {
    return null;
  }
  return invoke<string | null>('proyecto_e2e');
}

export async function escribir(
  ruta: string,
  objetivo: string,
  cuerpo: string,
  hashBase: string | null,
): Promise<GuardadoDto> {
  if (!esTauri) {
    const documento = demoDocumentos.find((candidato) => candidato.id === objetivo);
    if (!documento) {
      throw {
        tipo: 'no-encontrado',
        mensaje: `documento no encontrado en la demo: ${objetivo}`,
      } satisfies ErrorDto;
    }
    if (hashBase !== null && documento.hashArchivo !== hashBase) {
      throw {
        tipo: 'conflicto',
        mensaje: `el documento cambió fuera del editor: ${objetivo}`,
      } satisfies ErrorDto;
    }
    documento.cuerpo = cuerpo;
    documento.hashArchivo = hashDemo(cuerpo);
    demoProyecto.revision += 1;
    return { revision: demoProyecto.revision, hashArchivo: documento.hashArchivo };
  }
  return invoke<GuardadoDto>('escribir', { ruta, objetivo, cuerpo, hashBase });
}

/** Normaliza el rechazo de Tauri (DTO) o de JavaScript a un error tipado. */
export function errorDe(fallo: unknown): ErrorDto {
  if (typeof fallo === 'object' && fallo !== null && 'tipo' in fallo && 'mensaje' in fallo) {
    const candidato = fallo as { tipo: TipoError; mensaje: unknown };
    return { tipo: candidato.tipo, mensaje: String(candidato.mensaje) };
  }
  return {
    tipo: 'otro',
    mensaje: fallo instanceof Error ? fallo.message : String(fallo),
  };
}
