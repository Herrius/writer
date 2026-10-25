import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { DocumentoDto } from './types/DocumentoDto';
import type { BusquedaDto } from './types/BusquedaDto';
import type { CambioDetalleDto } from './types/CambioDetalleDto';
import type { CambioResumenDto } from './types/CambioResumenDto';
import type { ComparacionDto } from './types/ComparacionDto';
import type { ComentarioDto } from './types/ComentarioDto';
import type { CuarentenaDto } from './types/CuarentenaDto';
import type { ErrorDto } from './types/ErrorDto';
import type { EscaneoDto } from './types/EscaneoDto';
import type { GuardadoDto } from './types/GuardadoDto';
import type { MetasDto } from './types/MetasDto';
import type { PapeleraDto } from './types/PapeleraDto';
import type { FormatoPdfDto } from './types/FormatoPdfDto';
import type { PlotGridDto } from './types/PlotGridDto';
import type { PlantillaDto } from './types/PlantillaDto';
import type { ProyectoDto } from './types/ProyectoDto';
import type { SesionDto } from './types/SesionDto';
import type { ReversionDto } from './types/ReversionDto';
import type { RevisionDetalleDto } from './types/RevisionDetalleDto';
import type { RevisionDto } from './types/RevisionDto';
import type { TipoError } from './types/TipoError';
import { compararLineas } from './diff';

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

const demoMetas: MetasDto = {
  version: 1,
  diaria: { objetivo: 500, desde: '2026-10-23' },
  libro: { objetivo: 80000 },
  desafio: null,
  archivadas: [],
};

let metasDemoActuales: MetasDto = demoMetas;
const sesionesDemo: SesionDto[] = [];
const comentariosDemo: ComentarioDto[] = [];
let comentarioDemoSeq = 0;
let plotGridDemo: PlotGridDto = { version: 1, tramas: [] };
let cambiosDemo: CambioDetalleDto[] = [];
let papeleraDemo: PapeleraDto[] = [];
let cambioDemoSeq = 0;

const demoCuarentena: CuarentenaDto[] = [
  {
    archivo: '20260101T000000Z-primera-escena-esquema.md',
    rutaOriginal: 'manuscrito/01-libro/01-capitulo/001-el-umbral.md',
    motivo: 'esquema: falta el cierre del frontmatter',
    ts: '2026-01-01T00:00:00Z',
    tamano: 512,
  },
];

const demoCarpetasExtra: string[] = [];

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
    archivo: null,
    etiquetas: [],
    rol: null,
    cast: [],
    alias: [],
    clase: null,
  },
  {
    tipo: 'escena',
    id: '01KDVDNCXRH4QX4FR84G98PBSK',
    ruta: 'manuscrito/01-libro/02-capitulo/002-la-llave.md',
    titulo: 'La llave',
    cuerpo:
      'En el cajón de la cocina encontró una llave de bronce. No abría ninguna puerta de la casa; lo supo antes de probarla.\n\nEl cuaderno azul estaba debajo, con la primera página arrancada.',
    hashArchivo: 'demo:llave',
    orden: 20,
    estado: 'borrador',
    archivo: null,
    etiquetas: [],
    rol: null,
    cast: [],
    alias: [],
    clase: null,
  },
  {
    tipo: 'escena',
    id: '01KDVDNBYG1HEAD8X4A1JH69RF',
    ruta: 'manuscrito/01-libro/02-capitulo/001-la-carta.md',
    titulo: 'La carta',
    cuerpo: 'La carta llegó sin remitente.\n\nDentro solo había una frase.',
    hashArchivo: 'demo:carta',
    orden: 10,
    estado: 'borrador',
    archivo: null,
    etiquetas: [],
    rol: null,
    cast: [],
    alias: [],
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
    archivo: null,
    etiquetas: [],
    rol: null,
    cast: [],
    alias: [],
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
    archivo: null,
    etiquetas: [],
    rol: null,
    cast: [],
    alias: [],
    clase: 'tramas',
  },
];

const demoHistoricos = new Map<number, Map<string, string>>();

function capturarHistoricoDemo(revision: number): void {
  demoHistoricos.set(
    revision,
    new Map(demoDocumentos.map((documento) => [documento.ruta, documento.cuerpo])),
  );
}

for (let revision = 1; revision <= demoProyecto.revision; revision += 1) {
  const mapa = new Map<string, string>();
  for (const documento of demoDocumentos) {
    let cuerpo = documento.cuerpo;
    if (revision === 1 && documento.ruta.includes('umbral')) {
      cuerpo = cuerpo.replace(
        'La casa olía a lluvia vieja.',
        'La casa olía a humedad y a leña mojada.',
      );
    } else if (revision < demoProyecto.revision) {
      cuerpo = `${cuerpo}\n\n[Versión ${revision} de la demo]`;
    }
    mapa.set(documento.ruta, cuerpo);
  }
  demoHistoricos.set(revision, mapa);
}

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
  return `${inicio > 0 ? '…' : ''}${cuerpo.slice(inicio, indice)}\u0001${cuerpo.slice(
    indice,
    indice + consulta.length,
  )}\u0002${cuerpo.slice(indice + consulta.length, fin)}${fin < cuerpo.length ? '…' : ''}`;
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

export async function elegirCarpetaCreacion(): Promise<string | null> {
  if (!esTauri) {
    return window.prompt('Carpeta padre donde crear el proyecto');
  }
  const seleccion = await open({
    directory: true,
    multiple: false,
    title: 'Elegir carpeta para el nuevo proyecto',
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function crearProyecto(
  ruta: string,
  titulo: string,
  autor: string | null,
  idioma: string | null,
  notasIniciales: boolean,
): Promise<ProyectoDto> {
  if (!esTauri) {
    throw {
      tipo: 'proyecto',
      mensaje: 'La creación de proyectos requiere la app de escritorio; en la demo usa Abrir proyecto.',
    } satisfies ErrorDto;
  }
  return invoke<ProyectoDto>('crear_proyecto', {
    ruta,
    titulo,
    autor,
    idioma,
    notasIniciales,
  });
}

export async function elegirDestinoPdf(titulo: string): Promise<string | null> {
  const nombre = `${titulo.replace(/[\\/:*?"<>|]/g, '').trim() || 'novela'}.pdf`;
  if (!esTauri) {
    return window.prompt('Ruta del PDF', nombre);
  }
  const seleccion = await save({
    title: 'Exportar manuscrito a PDF',
    defaultPath: nombre,
    filters: [{ name: 'Documento PDF', extensions: ['pdf'] }],
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function leerFormatoPdf(ruta: string): Promise<FormatoPdfDto> {
  if (!esTauri) {
    return { fuente: 12, interlineado: 18, margen: 72 };
  }
  return invoke<FormatoPdfDto>('leer_formato_pdf', { ruta });
}

export async function guardarFormatoPdf(ruta: string, formato: FormatoPdfDto): Promise<void> {
  if (!esTauri) {
    return;
  }
  await invoke('guardar_formato_pdf', { ruta, formato });
}

export async function exportarPdf(
  ruta: string,
  destino: string,
  forzar: boolean,
): Promise<string> {
  if (!esTauri) {
    return destino;
  }
  return invoke<string>('exportar_pdf', { ruta, destino, forzar });
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

export async function leerMetas(ruta: string): Promise<MetasDto> {
  if (!esTauri) {
    return metasDemoActuales;
  }
  return invoke<MetasDto>('leer_metas', { ruta });
}

export async function guardarMetas(ruta: string, metas: MetasDto): Promise<void> {
  if (!esTauri) {
    metasDemoActuales = metas;
    return;
  }
  await invoke('guardar_metas', { ruta, metas });
}

export async function agregarSesion(ruta: string, sesion: SesionDto): Promise<string> {
  if (!esTauri) {
    sesionesDemo.push(sesion);
    return '(demo)';
  }
  return invoke<string>('agregar_sesion', { ruta, sesion });
}

export async function listarSesiones(ruta: string): Promise<SesionDto[]> {
  if (!esTauri) {
    return sesionesDemo;
  }
  return invoke<SesionDto[]>('listar_sesiones', { ruta });
}

export async function leerSesiones(ruta: string, mes: string): Promise<SesionDto[]> {
  if (!esTauri) {
    return sesionesDemo.filter((sesion) => sesion.inicio.startsWith(mes));
  }
  return invoke<SesionDto[]>('leer_sesiones', { ruta, mes });
}

export async function listarComentarios(
  ruta: string,
  documento?: string,
): Promise<ComentarioDto[]> {
  if (!esTauri) {
    return documento
      ? comentariosDemo.filter((comentario) => comentario.documento === documento)
      : comentariosDemo;
  }
  return invoke<ComentarioDto[]>('listar_comentarios', { ruta, documento });
}

export async function crearComentario(
  ruta: string,
  documento: string,
  texto: string,
  textoMarcado: string,
  autor: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const ahora = new Date().toISOString();
    const comentario: ComentarioDto = {
      version: 2,
      id: `01ARZ3NDEKTSV4RRFFQ69G5FA${(comentarioDemoSeq += 1) % 10}`,
      tipo: 'comentario',
      documento,
      ancla: {
        texto: textoMarcado,
        hash: `demo:${textoMarcado.length}`,
        inicio: 0,
        fin: 0,
      },
      estado: 'abierto',
      creado: ahora,
      actualizado: ahora,
      autor,
      huerfano: false,
      mensajes: [
        {
          id: `demo-mensaje-${Date.now()}`,
          tipo: 'comentario',
          autor,
          ts: ahora,
          texto,
          reemplaza: null,
        },
      ],
    };
    comentariosDemo.push(comentario);
    return comentario;
  }
  return invoke<ComentarioDto>('crear_comentario', {
    ruta,
    documento,
    texto,
    textoMarcado,
    autor,
  });
}

export async function crearSticky(
  ruta: string,
  documento: string,
  texto: string,
  autor: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const ahora = new Date().toISOString();
    const comentario: ComentarioDto = {
      version: 2,
      id: `01ARZ3NDEKTSV4RRFFQ69G5FB${(comentarioDemoSeq += 1) % 10}`,
      tipo: 'sticky',
      documento,
      ancla: null,
      estado: 'abierto',
      creado: ahora,
      actualizado: ahora,
      autor,
      huerfano: false,
      mensajes: [
        {
          id: `demo-mensaje-${Date.now()}`,
          tipo: 'comentario',
          autor,
          ts: ahora,
          texto,
          reemplaza: null,
        },
      ],
    };
    comentariosDemo.push(comentario);
    return comentario;
  }
  return invoke<ComentarioDto>('crear_sticky', { ruta, documento, texto, autor });
}

export async function agregarMensajeComentario(
  ruta: string,
  id: string,
  tipo: string,
  autor: string,
  texto: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const comentario = comentariosDemo.find((candidato) => candidato.id === id);
    if (!comentario) {
      throw { tipo: 'no-encontrado', mensaje: `comentario no encontrado: ${id}` } satisfies ErrorDto;
    }
    comentario.mensajes.push({
      id: `demo-mensaje-${Date.now()}`,
      tipo,
      autor,
      ts: new Date().toISOString(),
      texto,
      reemplaza: null,
    });
    comentario.actualizado = new Date().toISOString();
    return comentario;
  }
  return invoke<ComentarioDto>('agregar_mensaje_comentario', {
    ruta,
    id,
    tipo,
    autor,
    texto,
  });
}

export async function sincronizarComentariosHuerfanos(
  ruta: string,
  documento: string,
  texto: string,
): Promise<ComentarioDto[]> {
  if (!esTauri) {
    return comentariosDemo
      .filter((comentario) => comentario.documento === documento)
      .map((comentario) => ({
        ...comentario,
        huerfano: comentario.ancla ? !texto.includes(comentario.ancla.texto) : false,
      }));
  }
  return invoke<ComentarioDto[]>('sincronizar_comentarios_huerfanos', {
    ruta,
    documento,
    texto,
  });
}

export async function reabrirComentario(
  ruta: string,
  id: string,
  autor: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const comentario = comentariosDemo.find((candidato) => candidato.id === id);
    if (!comentario) {
      throw { tipo: 'no-encontrado', mensaje: `comentario no encontrado: ${id}` } satisfies ErrorDto;
    }
    comentario.estado = 'abierto';
    comentario.actualizado = new Date().toISOString();
    return comentario;
  }
  return invoke<ComentarioDto>('reabrir_comentario', { ruta, id, autor });
}

export async function editarMensajeComentario(
  ruta: string,
  id: string,
  mensajeId: string,
  autor: string,
  texto: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const comentario = comentariosDemo.find((candidato) => candidato.id === id);
    if (!comentario) {
      throw { tipo: 'no-encontrado', mensaje: `comentario no encontrado: ${id}` } satisfies ErrorDto;
    }
    comentario.mensajes.push({
      id: `demo-edicion-${Date.now()}`,
      tipo: 'edicion',
      autor,
      ts: new Date().toISOString(),
      texto,
      reemplaza: mensajeId,
    });
    return comentario;
  }
  return invoke<ComentarioDto>('editar_mensaje_comentario', {
    ruta,
    id,
    mensajeId,
    autor,
    texto,
  });
}

export async function borrarMensajeComentario(
  ruta: string,
  id: string,
  mensajeId: string,
  autor: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const comentario = comentariosDemo.find((candidato) => candidato.id === id);
    if (!comentario) {
      throw { tipo: 'no-encontrado', mensaje: `comentario no encontrado: ${id}` } satisfies ErrorDto;
    }
    comentario.mensajes.push({
      id: `demo-borrado-${Date.now()}`,
      tipo: 'borrado',
      autor,
      ts: new Date().toISOString(),
      texto: 'Mensaje borrado.',
      reemplaza: mensajeId,
    });
    return comentario;
  }
  return invoke<ComentarioDto>('borrar_mensaje_comentario', { ruta, id, mensajeId, autor });
}

export async function archivarComentario(ruta: string, id: string): Promise<void> {
  if (!esTauri) {
    const indice = comentariosDemo.findIndex((candidato) => candidato.id === id);
    if (indice >= 0) {
      comentariosDemo.splice(indice, 1);
    }
    return;
  }
  await invoke('archivar_comentario', { ruta, id });
}

export async function resolverComentario(
  ruta: string,
  id: string,
  autor: string,
): Promise<ComentarioDto> {
  if (!esTauri) {
    const comentario = comentariosDemo.find((candidato) => candidato.id === id);
    if (!comentario) {
      throw { tipo: 'no-encontrado', mensaje: `comentario no encontrado: ${id}` } satisfies ErrorDto;
    }
    comentario.estado = 'resuelto';
    comentario.actualizado = new Date().toISOString();
    return comentario;
  }
  return invoke<ComentarioDto>('resolver_comentario', { ruta, id, autor });
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

export async function mover(
  ruta: string,
  objetivo: string,
  libro: string | null,
  capitulo: string | null,
  tipo: string | null,
): Promise<DocumentoDto> {
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
    if (documento.tipo === 'escena' && libro && capitulo) {
      const nombre = documento.ruta.split('/').at(-1) ?? '001-escena.md';
      documento.ruta = `manuscrito/${libro}/${capitulo}/${nombre}`;
    } else if (documento.tipo === 'nota' && tipo) {
      const nombre = documento.ruta.split('/').at(-1) ?? 'nota.md';
      documento.ruta = `notas/${tipo}/${nombre}`;
      documento.clase = tipo;
    }
    return { ...documento };
  }
  return invoke<DocumentoDto>('mover', { ruta, objetivo, libro, capitulo, tipo });
}

export async function actualizarEscena(
  ruta: string,
  objetivo: string,
  estado: string | null,
  etiquetas: string[],
): Promise<DocumentoDto> {
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
    documento.estado = estado;
    documento.etiquetas = etiquetas;
    return { ...documento };
  }
  return invoke<DocumentoDto>('actualizar_escena', {
    ruta,
    objetivo,
    estado,
    etiquetas,
  });
}

export async function actualizarNota(
  ruta: string,
  objetivo: string,
  etiquetas: string[],
): Promise<DocumentoDto> {
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
    documento.etiquetas = etiquetas;
    return { ...documento };
  }
  return invoke<DocumentoDto>('actualizar_nota', { ruta, objetivo, etiquetas });
}

export async function actualizarPersonaje(
  ruta: string,
  objetivo: string,
  rol: string | null,
  cast: string[],
  alias: string[],
): Promise<DocumentoDto> {
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
    documento.rol = rol;
    documento.cast = cast;
    documento.alias = alias;
    return { ...documento };
  }
  return invoke<DocumentoDto>('actualizar_personaje', {
    ruta,
    objetivo,
    rol,
    cast,
    alias,
  });
}

export async function cambiarArchivo(
  ruta: string,
  objetivo: string,
  archivo: 'archivado' | 'papelera' | null,
): Promise<DocumentoDto> {
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
    documento.archivo = archivo;
    return { ...documento };
  }
  return invoke<DocumentoDto>('cambiar_archivo', { ruta, objetivo, archivo });
}

export async function reordenarEscena(
  ruta: string,
  objetivo: string,
  orden: number,
): Promise<DocumentoDto> {
  if (!esTauri) {
    const documento = demoDocumentos.find(
      (candidato) => candidato.id === objetivo || candidato.ruta === objetivo,
    );
    if (!documento) {
      throw { tipo: 'no-encontrado', mensaje: `documento no encontrado: ${objetivo}` } satisfies ErrorDto;
    }
    documento.orden = orden;
    return { ...documento };
  }
  return invoke<DocumentoDto>('reordenar_escena', { ruta, objetivo, orden });
}

export async function intercambiarOrden(ruta: string, a: string, b: string): Promise<void> {
  if (!esTauri) {
    const uno = demoDocumentos.find((documento) => documento.id === a);
    const otro = demoDocumentos.find((documento) => documento.id === b);
    if (uno && otro) {
      [uno.orden, otro.orden] = [otro.orden, uno.orden];
    }
    return;
  }
  await invoke('intercambiar_orden', { ruta, a, b });
}

export async function convertirIdeaEnProyecto(
  ruta: string,
  objetivo: string,
  destino: string,
  titulo: string,
): Promise<ProyectoDto> {
  if (!esTauri) {
    throw { tipo: 'proyecto', mensaje: 'Convertir una idea requiere la app de escritorio.' } satisfies ErrorDto;
  }
  return invoke<ProyectoDto>('convertir_idea_en_proyecto', { ruta, objetivo, destino, titulo });
}

export async function renombrar(
  ruta: string,
  objetivo: string,
  titulo: string,
): Promise<DocumentoDto> {
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
    documento.titulo = titulo;
    return { ...documento };
  }
  return invoke<DocumentoDto>('renombrar', { ruta, objetivo, titulo });
}

export async function duplicar(ruta: string, objetivo: string): Promise<DocumentoDto> {
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
    const copia: DocumentoDto = {
      ...documento,
      id: `demo-copia-${Date.now()}`,
      titulo: `Copia de ${documento.titulo}`,
      ruta: documento.ruta.replace(/\.md$/, '-copia.md'),
      hashArchivo: `demo:copia-${Date.now()}`,
    };
    demoDocumentos.push(copia);
    return { ...copia };
  }
  return invoke<DocumentoDto>('duplicar', { ruta, objetivo });
}

export async function crearNota(
  ruta: string,
  tipo: string,
  titulo: string,
): Promise<DocumentoDto> {
  if (!esTauri) {
    const nota: DocumentoDto = {
      tipo: 'nota',
      id: `demo-nota-${Date.now()}`,
      ruta: `notas/${tipo}/${titulo.toLowerCase().replaceAll(' ', '-')}.md`,
      titulo,
      cuerpo: '',
      hashArchivo: `demo:${Date.now()}`,
      orden: null,
      estado: null,
      archivo: null,
      etiquetas: [],
      rol: null,
      cast: [],
      alias: [],
      clase: tipo,
    };
    demoDocumentos.push(nota);
    return { ...nota };
  }
  return invoke<DocumentoDto>('crear_nota', { ruta, tipo, titulo });
}

export async function crearEscena(
  ruta: string,
  titulo: string,
  libro: string | null,
  capitulo: string | null,
): Promise<DocumentoDto> {
  if (!esTauri) {
    const escena: DocumentoDto = {
      tipo: 'escena',
      id: `demo-escena-${Date.now()}`,
      ruta: `manuscrito/${libro ?? '01-libro'}/${capitulo ?? '01-capitulo'}/999-${Date.now()}.md`,
      titulo,
      cuerpo: '',
      hashArchivo: `demo:${Date.now()}`,
      orden: 30,
      estado: 'borrador',
      archivo: null,
      etiquetas: [],
      rol: null,
      cast: [],
      alias: [],
      clase: null,
    };
    demoDocumentos.push(escena);
    return { ...escena };
  }
  return invoke<DocumentoDto>('crear_escena', { ruta, titulo, libro, capitulo });
}

export async function elegirArchivosImportacion(): Promise<string[]> {
  if (!esTauri) {
    const ruta = window.prompt('Ruta del archivo a importar (.md, .docx, .html o .rtf)');
    return ruta ? [ruta] : [];
  }
  const seleccion = await open({
    multiple: true,
    title: 'Importar documento',
    filters: [
      { name: 'Documentos', extensions: ['md', 'markdown', 'docx', 'html', 'htm', 'rtf'] },
      { name: 'Markdown', extensions: ['md', 'markdown'] },
      { name: 'Word', extensions: ['docx'] },
      { name: 'HTML', extensions: ['html', 'htm'] },
      { name: 'RTF', extensions: ['rtf'] },
    ],
  });
  if (typeof seleccion === 'string') {
    return [seleccion];
  }
  return Array.isArray(seleccion) ? seleccion : [];
}

export async function elegirDestinoBackup(titulo: string): Promise<string | null> {
  const nombre = `${titulo.replace(/[\\/:*?"<>|]/g, '').trim() || 'proyecto'}.zip`;
  if (!esTauri) {
    return window.prompt('Ruta del backup ZIP', nombre);
  }
  const seleccion = await save({
    title: 'Guardar backup del proyecto',
    defaultPath: nombre,
    filters: [{ name: 'Backup writer', extensions: ['zip'] }],
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function elegirArchivoBackup(): Promise<string | null> {
  if (!esTauri) {
    return window.prompt('Ruta del backup ZIP');
  }
  const seleccion = await open({
    multiple: false,
    title: 'Restaurar backup',
    filters: [{ name: 'Backup writer', extensions: ['zip'] }],
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function elegirCarpetaRestauracion(): Promise<string | null> {
  if (!esTauri) {
    return window.prompt('Carpeta de destino para restaurar');
  }
  const seleccion = await open({
    directory: true,
    multiple: false,
    title: 'Carpeta de destino de la restauración',
  });
  return typeof seleccion === 'string' ? seleccion : null;
}

export async function previsualizarImportacion(
  archivo: string,
): Promise<{ formato: string; texto: string }> {
  if (!esTauri) {
    throw { tipo: 'formato', mensaje: 'La previsualización requiere la app de escritorio.' } satisfies ErrorDto;
  }
  const [formato, texto] = await invoke<[string, string]>('previsualizar_importacion', {
    archivo,
  });
  return { formato, texto };
}

export async function importarMarkdown(
  ruta: string,
  archivo: string,
  titulo: string,
  tipoNota: string | null,
): Promise<DocumentoDto> {
  if (!esTauri) {
    throw { tipo: 'formato', mensaje: 'Importar requiere la app de escritorio.' } satisfies ErrorDto;
  }
  return invoke<DocumentoDto>('importar_markdown', { ruta, archivo, titulo, tipoNota });
}

export async function importarDocx(
  ruta: string,
  archivo: string,
  titulo: string,
  tipoNota: string | null,
): Promise<DocumentoDto> {
  if (!esTauri) {
    throw { tipo: 'formato', mensaje: 'Importar requiere la app de escritorio.' } satisfies ErrorDto;
  }
  return invoke<DocumentoDto>('importar_docx', { ruta, archivo, titulo, tipoNota });
}

export async function importarHtml(
  ruta: string,
  archivo: string,
  titulo: string,
  tipoNota: string | null,
): Promise<DocumentoDto> {
  if (!esTauri) {
    throw { tipo: 'formato', mensaje: 'Importar requiere la app de escritorio.' } satisfies ErrorDto;
  }
  return invoke<DocumentoDto>('importar_html', { ruta, archivo, titulo, tipoNota });
}

export async function importarRtf(
  ruta: string,
  archivo: string,
  titulo: string,
  tipoNota: string | null,
): Promise<DocumentoDto> {
  if (!esTauri) {
    throw { tipo: 'formato', mensaje: 'Importar requiere la app de escritorio.' } satisfies ErrorDto;
  }
  return invoke<DocumentoDto>('importar_rtf', { ruta, archivo, titulo, tipoNota });
}

export async function crearBackup(ruta: string, destino: string): Promise<string> {
  if (!esTauri) {
    throw { tipo: 'io', mensaje: 'El backup requiere la app de escritorio.' } satisfies ErrorDto;
  }
  return invoke<string>('crear_backup', { ruta, destino });
}

export async function restaurarBackup(archivo: string, destino: string): Promise<void> {
  if (!esTauri) {
    throw { tipo: 'io', mensaje: 'Restaurar requiere la app de escritorio.' } satisfies ErrorDto;
  }
  await invoke('restaurar_backup', { archivo, destino });
}

let plantillasDemo: PlantillaDto[] = [
  {
    slug: 'tres-actos',
    version: 1,
    nombre: 'Novela en tres actos',
    descripcion: 'Estructura base con tres capítulos y fichas iniciales.',
    idioma: 'es',
    estructura: { libros: 1, capitulosPorLibro: 3, escenasPorCapitulo: 1 },
    notasIniciales: ['personajes', 'tramas', 'lugares'],
  },
];

export async function listarPlantillas(): Promise<PlantillaDto[]> {
  if (!esTauri) {
    return plantillasDemo.map((plantilla) => ({ ...plantilla, estructura: { ...plantilla.estructura } }));
  }
  return invoke<PlantillaDto[]>('listar_plantillas');
}

export async function guardarPlantilla(plantilla: PlantillaDto): Promise<PlantillaDto> {
  if (!esTauri) {
    if (plantillasDemo.some((item) => item.slug === plantilla.slug)) {
      throw {
        tipo: 'conflicto',
        mensaje: `Ya existe una plantilla con el slug ${plantilla.slug}.`,
      } satisfies ErrorDto;
    }
    plantillasDemo = [...plantillasDemo, plantilla];
    return plantilla;
  }
  return invoke<PlantillaDto>('guardar_plantilla', { plantilla });
}

export async function eliminarPlantilla(slug: string): Promise<void> {
  if (!esTauri) {
    plantillasDemo = plantillasDemo.filter((item) => item.slug !== slug);
    return;
  }
  await invoke('eliminar_plantilla', { slug });
}

export async function moverProyectoAPapelera(
  ruta: string,
  titulo: string,
  autor: string | null,
  color: string | null,
  palabras: number,
): Promise<PapeleraDto> {
  if (!esTauri) {
    const entrada: PapeleraDto = {
      id: `demo-papelera-${Date.now()}`,
      titulo,
      autor,
      color,
      palabras,
      rutaOriginal: ruta,
      rutaPapelera: `/tmp/.writer-papelera/demo-${Date.now()}`,
      eliminado: new Date().toISOString(),
    };
    papeleraDemo = [entrada, ...papeleraDemo];
    return entrada;
  }
  return invoke<PapeleraDto>('mover_proyecto_a_papelera', {
    ruta,
    titulo,
    autor,
    color,
    palabras,
  });
}

export async function restaurarProyectoDePapelera(rutaPapelera: string): Promise<string> {
  if (!esTauri) {
    let entrada = papeleraDemo.find((item) => item.rutaPapelera === rutaPapelera);
    if (!entrada) {
      try {
        const guardadas = JSON.parse(
          window.localStorage.getItem('writer.papelera.v1') ?? '[]',
        ) as PapeleraDto[];
        entrada = guardadas.find((item) => item.rutaPapelera === rutaPapelera);
      } catch {
        entrada = undefined;
      }
    }
    if (!entrada) {
      throw { tipo: 'no-encontrado', mensaje: 'Proyecto no encontrado en la papelera.' } satisfies ErrorDto;
    }
    papeleraDemo = papeleraDemo.filter((item) => item.rutaPapelera !== rutaPapelera);
    return entrada.rutaOriginal;
  }
  return invoke<string>('restaurar_proyecto_de_papelera', { rutaPapelera });
}

export async function purgarProyectoDePapelera(rutaPapelera: string): Promise<void> {
  if (!esTauri) {
    papeleraDemo = papeleraDemo.filter((item) => item.rutaPapelera !== rutaPapelera);
    return;
  }
  await invoke('purgar_proyecto_de_papelera', { rutaPapelera });
}

export async function crearDesdePlantilla(
  slug: string,
  ruta: string,
  titulo: string,
  autor: string | null,
  idioma: string | null,
): Promise<ProyectoDto> {
  if (!esTauri) {
    throw {
      tipo: 'proyecto',
      mensaje: 'Crear desde plantilla requiere la app de escritorio.',
    } satisfies ErrorDto;
  }
  return invoke<ProyectoDto>('crear_desde_plantilla', {
    slug,
    ruta,
    titulo,
    autor,
    idioma,
  });
}

export async function leerPlotGrid(ruta: string): Promise<PlotGridDto> {
  if (!esTauri) {
    return plotGridDemo;
  }
  return invoke<PlotGridDto>('leer_plot_grid', { ruta });
}

export async function guardarPlotGrid(ruta: string, grid: PlotGridDto): Promise<void> {
  if (!esTauri) {
    plotGridDemo = grid;
    return;
  }
  await invoke('guardar_plot_grid', { ruta, grid });
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
    for (let revision = 1; revision <= demoProyecto.revision && revisiones.length < 5; revision += 1) {
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
      contenidos: demoDocumentos.map((documento) => {
        const cuerpo =
          demoHistoricos.get(revision)?.get(documento.ruta) ?? documento.cuerpo;
        return {
          ruta: documento.ruta,
          contenido:
            `---\nid: ${documento.id}\ntitulo: "${documento.titulo}"\n---\n\n` + cuerpo,
        };
      }),
    };
  }
  return invoke<RevisionDetalleDto>('mostrar', { ruta, revision });
}

export async function revertir(ruta: string, revision: number): Promise<ReversionDto> {
  if (!esTauri) {
    demoProyecto.revision += 1;
    capturarHistoricoDemo(demoProyecto.revision);
    return { revision: demoProyecto.revision, erroresPendientes: [] };
  }
  return invoke<ReversionDto>('revertir', { ruta, revision });
}

function normalizarDemo(texto: string): string {
  const normalizado = texto.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
  return normalizado && !normalizado.endsWith('\n') ? `${normalizado}\n` : normalizado;
}

function resumenCambioDemo(cambio: CambioDetalleDto): CambioResumenDto {
  return {
    id: cambio.id,
    documento: cambio.documento,
    ruta: cambio.ruta,
    autor: cambio.autor,
    ts: cambio.ts,
    baseRevision: cambio.baseRevision,
    insertadas: cambio.insertadas,
    borradas: cambio.borradas,
  };
}

export async function listarCambios(ruta: string): Promise<CambioResumenDto[]> {
  if (!esTauri) {
    return cambiosDemo.map(resumenCambioDemo);
  }
  return invoke<CambioResumenDto[]>('listar_cambios', { ruta });
}

export async function leerCambio(ruta: string, id: string): Promise<CambioDetalleDto> {
  if (!esTauri) {
    const cambio = cambiosDemo.find((item) => item.id === id);
    if (!cambio) {
      throw new Error('Cambio de demostración no encontrado.');
    }
    return { ...cambio, lineas: [...cambio.lineas] };
  }
  return invoke<CambioDetalleDto>('leer_cambio', { ruta, id });
}

export async function registrarCambio(
  ruta: string,
  objetivo: string,
  cuerpo: string,
  autor: string,
): Promise<CambioDetalleDto | null> {
  if (!esTauri) {
    const documento = demoDocumentos.find(
      (item) => item.id === objetivo || item.ruta === objetivo,
    );
    if (!documento) {
      throw new Error('Documento de demostración no encontrado.');
    }
    const propuesta = normalizarDemo(cuerpo);
    const existente = cambiosDemo.find((item) => item.documento === documento.id);
    if (propuesta === normalizarDemo(documento.cuerpo)) {
      cambiosDemo = cambiosDemo.filter((item) => item.documento !== documento.id);
      return null;
    }
    const base = normalizarDemo(existente?.base ?? documento.cuerpo);
    const diff = compararLineas(base, propuesta);
    const cambio: CambioDetalleDto = {
      id: existente?.id ?? `demo-cambio-${++cambioDemoSeq}`,
      documento: documento.id,
      ruta: documento.ruta,
      autor,
      ts: new Date().toISOString(),
      baseRevision: existente?.baseRevision ?? demoProyecto.revision,
      baseHash: existente?.baseHash ?? documento.hashArchivo,
      base,
      propuesta,
      insertadas: diff.insertadas,
      borradas: diff.borradas,
      lineas: diff.lineas,
    };
    cambiosDemo = [...cambiosDemo.filter((item) => item.documento !== documento.id), cambio];
    return cambio;
  }
  return invoke<CambioDetalleDto | null>('registrar_cambio', {
    ruta,
    objetivo,
    cuerpo,
    autor,
  });
}

export async function aplicarCambio(
  ruta: string,
  id: string,
): Promise<GuardadoDto> {
  if (!esTauri) {
    const cambio = cambiosDemo.find((item) => item.id === id);
    if (!cambio) {
      throw new Error('Cambio de demostración no encontrado.');
    }
    const documento = demoDocumentos.find((item) => item.id === cambio.documento);
    if (documento) {
      documento.cuerpo = cambio.propuesta.replace(/\n$/, '');
      documento.hashArchivo = hashDemo(documento.cuerpo);
    }
    demoProyecto.revision += 1;
    capturarHistoricoDemo(demoProyecto.revision);
    cambiosDemo = cambiosDemo.filter((item) => item.id !== id);
    return {
      revision: demoProyecto.revision,
      hashArchivo: documento?.hashArchivo ?? hashDemo(cambio.propuesta),
    };
  }
  return invoke<GuardadoDto>('aplicar_cambio', { ruta, id });
}

export async function descartarCambio(ruta: string, id: string): Promise<void> {
  if (!esTauri) {
    cambiosDemo = cambiosDemo.filter((item) => item.id !== id);
    return;
  }
  await invoke('descartar_cambio', { ruta, id });
}

export async function compararDocumento(
  ruta: string,
  objetivo: string,
  revision: number,
): Promise<ComparacionDto> {
  if (!esTauri) {
    const documento = demoDocumentos.find(
      (item) => item.id === objetivo || item.ruta === objetivo,
    );
    if (!documento) {
      throw new Error('Documento de demostración no encontrado.');
    }
    const base =
      demoHistoricos.get(revision)?.get(documento.ruta) ?? '';
    const diff = compararLineas(base, documento.cuerpo);
    return {
      revision,
      ruta: documento.ruta,
      baseExiste: revision < demoProyecto.revision,
      insertadas: diff.insertadas,
      borradas: diff.borradas,
      lineas: diff.lineas,
    };
  }
  return invoke<ComparacionDto>('comparar_documento', {
    ruta,
    objetivo,
    revision,
  });
}

export async function listarCarpetas(ruta: string): Promise<string[]> {
  if (!esTauri) {
    const carpetas = new Set(demoCarpetasExtra);
    for (const documento of demoDocumentos) {
      const partes = documento.ruta.split('/');
      for (let indice = 2; indice < partes.length; indice += 1) {
        carpetas.add(partes.slice(0, indice).join('/'));
      }
    }
    return [...carpetas].sort();
  }
  return invoke<string[]>('listar_carpetas', { ruta });
}

export async function crearCarpeta(ruta: string, relativa: string): Promise<string> {
  if (!esTauri) {
    if (demoCarpetasExtra.includes(relativa)) {
      throw new Error(`ya existe ${relativa}`);
    }
    demoCarpetasExtra.push(relativa);
    return relativa;
  }
  return invoke<string>('crear_carpeta', { ruta, relativa });
}

export async function renombrarCarpeta(
  ruta: string,
  relativa: string,
  nombre: string,
): Promise<string> {
  if (!esTauri) {
    const padre = relativa.split('/').slice(0, -1).join('/');
    const nueva = `${padre}/${nombre}`;
    demoCarpetasExtra.forEach((carpeta, indice) => {
      if (carpeta === relativa || carpeta.startsWith(`${relativa}/`)) {
        demoCarpetasExtra[indice] = nueva + carpeta.slice(relativa.length);
      }
    });
    for (const documento of demoDocumentos) {
      if (documento.ruta.startsWith(`${relativa}/`)) {
        documento.ruta = nueva + documento.ruta.slice(relativa.length);
      }
    }
    return nueva;
  }
  return invoke<string>('renombrar_carpeta', { ruta, relativa, nombre });
}

export async function eliminarCarpetaVacia(ruta: string, relativa: string): Promise<void> {
  if (!esTauri) {
    const indice = demoCarpetasExtra.indexOf(relativa);
    if (indice >= 0) {
      demoCarpetasExtra.splice(indice, 1);
    }
    return;
  }
  await invoke('eliminar_carpeta_vacia', { ruta, relativa });
}

export async function crearEscenaEn(
  ruta: string,
  carpeta: string,
  titulo: string,
): Promise<DocumentoDto> {
  if (!esTauri) {
    const escena: DocumentoDto = {
      tipo: 'escena',
      id: `demo-escena-${Date.now()}`,
      ruta: `${carpeta}/${String(demoDocumentos.length + 1).padStart(3, '0')}-${titulo
        .toLowerCase()
        .replaceAll(' ', '-')}.md`,
      titulo,
      cuerpo: '',
      hashArchivo: `demo:${Date.now()}`,
      orden: demoDocumentos.length * 10 + 10,
      estado: 'borrador',
      archivo: null,
      etiquetas: [],
      rol: null,
      cast: [],
      alias: [],
      clase: null,
    };
    demoDocumentos.push(escena);
    return { ...escena };
  }
  return invoke<DocumentoDto>('crear_escena_en', { ruta, carpeta, titulo });
}

export async function exportarTexto(
  nombre: string,
  contenido: string,
): Promise<string | null> {
  if (!esTauri) {
    const blob = new Blob([contenido], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const enlace = document.createElement('a');
    enlace.href = url;
    enlace.download = nombre;
    enlace.click();
    URL.revokeObjectURL(url);
    return nombre;
  }
  const destino = await save({
    defaultPath: nombre,
    filters: [{ name: 'Markdown', extensions: ['md'] }],
  });
  if (!destino) {
    return null;
  }
  return invoke<string>('guardar_texto_exportado', {
    destino,
    contenido,
    forzar: false,
  });
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
    capturarHistoricoDemo(demoProyecto.revision);
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
