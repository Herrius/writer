import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
} from 'react';
import {
  abrirProyecto,
  actualizarEscena,
  actualizarNota,
  actualizarPersonaje,
  agregarMensajeComentario,
  agregarSesion,
  aplicarCambio,
  archivarComentario,
  borrarMensajeComentario,
  compararDocumento,
  descartarCambio,
  editarMensajeComentario,
  leerCambio,
  listarCambios,
  registrarCambio,
  buscar as buscarEnProyecto,
  cambiarArchivo as cambiarArchivoApi,
  crearBackup as crearBackupApi,
  crearComentario,
  crearDesdePlantilla as crearDesdePlantillaApi,
  crearEscena as crearEscenaApi,
  crearNota as crearNotaApi,
  crearProyecto as crearProyectoApi,
  convertirIdeaEnProyecto,
  crearSticky,
  cuarentena as pedirCuarentena,
  descartarCuarentena,
  duplicar as duplicarDocumento,
  elegirArchivosImportacion,
  elegirArchivoBackup,
  elegirCarpetaCreacion,
  elegirCarpetaRestauracion,
  elegirDestinoBackup,
  elegirDestinoDocx,
  elegirDestinoPdf,
  elegirProyecto,
  errorDe,
  esTauri,
  escribir,
  exportarDocx,
  exportarPdf,
  guardarFormatoPdf,
  guardarMetas,
  guardarPlotGrid,
  historial as pedirHistorial,
  importarDocx,
  intercambiarOrden,
  importarHtml,
  importarMarkdown,
  importarRtf,
  leer,
  leerFormatoPdf,
  leerMetas,
  leerPlotGrid,
  listarSesiones,
  listar,
  listarComentarios,
  mostrar as pedirRevision,
  previsualizarImportacion,
  mover as moverDocumentoApi,
  proyectoE2e,
  renombrar as renombrarDocumento,
  reabrirComentario,
  resolverComentario,
  restaurarBackup as restaurarBackupApi,
  revertir as revertirRevision,
  sincronizarComentariosHuerfanos,
} from './api';
import type { BusquedaDto } from './types/BusquedaDto';
import type { CambioDetalleDto } from './types/CambioDetalleDto';
import type { CambioResumenDto } from './types/CambioResumenDto';
import type { ComparacionDto } from './types/ComparacionDto';
import type { ComentarioDto } from './types/ComentarioDto';
import type { CuarentenaDto } from './types/CuarentenaDto';
import type { DocumentoDto } from './types/DocumentoDto';
import type { MetasDto } from './types/MetasDto';
import type { PausaDto } from './types/PausaDto';
import type { FormatoPdfDto } from './types/FormatoPdfDto';
import type { PlotGridDto } from './types/PlotGridDto';
import type { ProyectoDto } from './types/ProyectoDto';
import type { RevisionDetalleDto } from './types/RevisionDetalleDto';
import type { RevisionDto } from './types/RevisionDto';
import type { SesionDto } from './types/SesionDto';
import type { TipoError } from './types/TipoError';
import { EditorMarkdown } from './EditorMarkdown';
import { PanelCambios } from './features/cambios/PanelCambios';
import { ModalDiff } from './features/cambios/ModalDiff';
import { DiffLineas } from './features/cambios/DiffLineas';
import { ModalPreferencias } from './features/preferencias/ModalPreferencias';
import {
  aplicarPreferencias,
  cargarPreferencias,
  guardarPreferencias,
  type Preferencias,
} from './features/preferencias/preferencias';
import { Paleta, type ComandoPaleta } from './features/paleta/Paleta';
import { PanelHerramientas } from './features/lectura/PanelHerramientas';
import {
  detenerLectura,
  hablar,
  listarVoces,
  pausarLectura,
  reanudarLectura,
  soportaLectura,
  textoParaLectura,
  type VozSistema,
} from './features/lectura/lectura';
import { Inicio, type DatosNuevoProyecto } from './Inicio';
import { analizarCompatibilidad } from './markdown/compatibilidad';
import { contarPalabras, textoSinMarcas } from './palabras';
import {
  colorDesdeTitulo,
  guardarRecientes,
  leerRecientes,
  slugProyecto,
  type Reciente,
} from './recientes';

type Estado = 'inicio' | 'cargando' | 'listo' | 'error';
type Vista = 'editor' | 'fuente' | 'historial';
type EstadoGuardado =
  | 'guardado'
  | 'pendiente'
  | 'guardando'
  | 'sugerencia'
  | 'conflicto'
  | 'recuperacion'
  | 'ocupado'
  | 'error'
  | 'solo-lectura';

type Pendiente = {
  id: string;
  cuerpo: string;
};

type ConfirmacionReversion = {
  raiz: string;
  revision: number;
  opId: string;
};

const RETARDO_AUTOGUARDADO = 900;

function fechaHoy(): string {
  return new Date().toISOString().slice(0, 10);
}

function esVistaEstrecha(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.matchMedia === 'function' &&
    window.matchMedia('(max-width: 900px)').matches
  );
}

function hayDialogoAbierto(): boolean {
  return (
    typeof document !== 'undefined' &&
    document.querySelector('[role="dialog"][aria-modal="true"]') !== null
  );
}

function formatearTiempo(segundos: number): string {
  const minutos = Math.floor(segundos / 60);
  const resto = segundos % 60;
  return `${minutos}:${resto.toString().padStart(2, '0')}`;
}

type GrupoDocumentos = {
  clave: string;
  etiqueta: string;
  documentos: DocumentoDto[];
};

function etiquetaCarpeta(carpeta: string): string {
  const coincidencia = /^(\d+)[-_](.+)$/.exec(carpeta);
  const numero = coincidencia?.[1] ? Number(coincidencia[1]) : null;
  const nombre = (coincidencia?.[2] ?? carpeta)
    .replace(/[-_]/g, ' ')
    .replace(/\bcapitulo\b/gi, 'capítulo')
    .replace(/\blibro\b/gi, 'libro');
  const capitalizada = nombre
    .split(' ')
    .filter(Boolean)
    .map((palabra) => palabra.charAt(0).toUpperCase() + palabra.slice(1))
    .join(' ');
  return numero === null ? capitalizada : `${capitalizada} ${numero}`;
}

function grupoEscena(documento: DocumentoDto): { clave: string; etiqueta: string } {
  const partes = documento.ruta.split('/');
  if (partes.length < 4) {
    return { clave: 'sin-clasificar', etiqueta: 'Sin clasificar' };
  }
  return {
    clave: `${partes[1]}/${partes[2]}`,
    etiqueta: `${etiquetaCarpeta(partes[1])} / ${etiquetaCarpeta(partes[2])}`,
  };
}

function agrupar(
  documentos: DocumentoDto[],
  clave: (documento: DocumentoDto) => string,
  etiqueta: (documento: DocumentoDto) => string,
): GrupoDocumentos[] {
  const mapa = new Map<string, GrupoDocumentos>();
  for (const documento of documentos) {
    const grupo = clave(documento);
    const existente = mapa.get(grupo);
    if (existente) {
      existente.documentos.push(documento);
    } else {
      mapa.set(grupo, {
        clave: grupo,
        etiqueta: etiqueta(documento),
        documentos: [documento],
      });
    }
  }
  return Array.from(mapa.values());
}

function etiquetaTipo(documento: DocumentoDto): string {
  if (documento.tipo === 'escena') {
    return documento.estado ?? 'borrador';
  }
  return documento.clase ?? 'nota';
}

function etiquetaGuardado(estado: EstadoGuardado): string {
  switch (estado) {
    case 'guardado':
      return 'guardado';
    case 'pendiente':
      return 'cambios sin guardar';
    case 'guardando':
      return 'guardando…';
    case 'sugerencia':
      return 'cambio propuesto';
    case 'conflicto':
      return 'conflicto';
    case 'recuperacion':
      return 'modo recuperación';
    case 'ocupado':
      return 'proyecto ocupado';
    case 'error':
      return 'error al guardar';
    case 'solo-lectura':
      return 'solo lectura';
  }
}

function estadoDeError(tipo: TipoError): EstadoGuardado {
  switch (tipo) {
    case 'conflicto':
      return 'conflicto';
    case 'recuperacion':
      return 'recuperacion';
    case 'ocupado':
      return 'ocupado';
    case 'proyecto':
    case 'esquema':
      return 'solo-lectura';
    default:
      return 'error';
  }
}

function formatearFecha(ts: string | null): string {
  if (!ts) {
    return 'fecha desconocida';
  }
  const fecha = new Date(ts);
  if (Number.isNaN(fecha.getTime())) {
    return ts;
  }
  return fecha.toLocaleString('es', {
    dateStyle: 'medium',
    timeStyle: 'short',
  });
}

function Fragmento({ texto }: { texto: string }) {
  // FTS5 marca la coincidencia con caracteres de control; así los corchetes
  // literales del documento no se confunden con el resaltado.
  const partes = texto.split(/(\u0001[^\u0002]*\u0002)/g);
  return (
    <>
      {partes.map((parte, indice) =>
        parte.startsWith('\u0001') && parte.endsWith('\u0002') ? (
          <mark key={indice}>{parte.slice(1, -1)}</mark>
        ) : (
          <span key={indice}>{parte}</span>
        ),
      )}
    </>
  );
}

export default function App() {
  const [proyecto, setProyecto] = useState<ProyectoDto | null>(null);
  const [documentos, setDocumentos] = useState<DocumentoDto[]>([]);
  const [activo, setActivo] = useState<DocumentoDto | null>(null);
  const [vista, setVista] = useState<Vista>('editor');
  const [estado, setEstado] = useState<Estado>('inicio');
  const [error, setError] = useState<string | null>(null);
  const [mensaje, setMensaje] = useState<string | null>(null);
  const [errorGuardado, setErrorGuardado] = useState<string | null>(null);
  const [estadoGuardado, setEstadoGuardado] = useState<EstadoGuardado>('guardado');
  const [revisionEditor, setRevisionEditor] = useState(0);
  const [confirmarRecarga, setConfirmarRecarga] = useState(false);

  const [consulta, setConsulta] = useState('');
  const [focoArbol, setFocoArbol] = useState<string | null>(null);
  const [panelDerecho, setPanelDerecho] = useState<
    'metas' | 'notas' | 'comentarios' | 'plan' | 'cambios' | 'herramientas'
  >('metas');
  const [comentarios, setComentarios] = useState<ComentarioDto[]>([]);
  const [errorComentarios, setErrorComentarios] = useState<string | null>(null);
  const [textoSticky, setTextoSticky] = useState('');
  const [respuestas, setRespuestas] = useState<Record<string, string>>({});
  const [filtroComentarios, setFiltroComentarios] = useState<'todos' | 'abiertos' | 'resueltos' | 'huerfanos'>(
    'todos',
  );
  const [modoSugerencias, setModoSugerencias] = useState(false);
  const [cambiosProyecto, setCambiosProyecto] = useState<CambioResumenDto[]>([]);
  const [cambioActivo, setCambioActivo] = useState<CambioDetalleDto | null>(null);
  const [cargandoCambios, setCargandoCambios] = useState(false);
  const [errorCambios, setErrorCambios] = useState<string | null>(null);
  const [procesandoCambio, setProcesandoCambio] = useState(false);
  const [comparacion, setComparacion] = useState<ComparacionDto | null>(null);
  const [comparando, setComparando] = useState(false);
  const [errorComparacion, setErrorComparacion] = useState<string | null>(null);
  const [comentariosProyecto, setComentariosProyecto] = useState<ComentarioDto[]>([]);
  const [mensajeEditando, setMensajeEditando] = useState<string | null>(null);
  const [textoEdicionMensaje, setTextoEdicionMensaje] = useState('');
  const [anclaComentario, setAnclaComentario] = useState<string | null>(null);
  const [vistaPreviaImportacion, setVistaPreviaImportacion] = useState<{
    archivos: string[];
    formato: string;
    texto: string;
  } | null>(null);
  const [metas, setMetas] = useState<MetasDto | null>(null);
  const [sesiones, setSesiones] = useState<SesionDto[]>([]);
  const [errorMetas, setErrorMetas] = useState<string | null>(null);
  const [tituloEdicion, setTituloEdicion] = useState('');
  const [destinoArrastre, setDestinoArrastre] = useState<string | null>(null);
  const [filtroArchivo, setFiltroArchivo] = useState<'activo' | 'archivado' | 'papelera'>(
    'activo',
  );
  const [filtroEtiqueta, setFiltroEtiqueta] = useState('');
  const [estadoEdicion, setEstadoEdicion] = useState('borrador');
  const [etiquetasEdicion, setEtiquetasEdicion] = useState('');
  const [documentoCopiado, setDocumentoCopiado] = useState<string | null>(null);
  const [plotGrid, setPlotGrid] = useState<PlotGridDto>({ version: 1, tramas: [] });
  const [nuevaTrama, setNuevaTrama] = useState('');
  const [colorTrama, setColorTrama] = useState('#4d6b8a');
  const [tramaSeleccionada, setTramaSeleccionada] = useState('');
  const [nombreTramaEdit, setNombreTramaEdit] = useState('');
  const [colorTramaEdit, setColorTramaEdit] = useState('#4d6b8a');
  const [nuevaNotaTitulo, setNuevaNotaTitulo] = useState('');
  const [nuevaNotaTipo, setNuevaNotaTipo] = useState('cuaderno');
  const [nuevaEscenaTitulo, setNuevaEscenaTitulo] = useState('');
  const [tipoImportacion, setTipoImportacion] = useState('');
  const [formatoPdf, setFormatoPdf] = useState<FormatoPdfDto>({
    fuente: 12,
    interlineado: 18,
    margen: 72,
  });
  const [rolPersonaje, setRolPersonaje] = useState('');
  const [aliasPersonaje, setAliasPersonaje] = useState('');
  const [metaDiaria, setMetaDiaria] = useState('');
  const [metaLibro, setMetaLibro] = useState('');
  const [desafioObjetivo, setDesafioObjetivo] = useState('');
  const [desafioLimite, setDesafioLimite] = useState('');
  const [preset, setPreset] = useState(5);
  const [restante, setRestante] = useState(5 * 60);
  const [temporizadorActivo, setTemporizadorActivo] = useState(false);
  const [temporizadorPausado, setTemporizadorPausado] = useState(false);
  const [resultados, setResultados] = useState<BusquedaDto[]>([]);
  const [buscando, setBuscando] = useState(false);
  const [errorBusqueda, setErrorBusqueda] = useState<string | null>(null);

  const [revisiones, setRevisiones] = useState<RevisionDto[]>([]);
  const [cargandoHistorial, setCargandoHistorial] = useState(false);
  const [errorHistorial, setErrorHistorial] = useState<string | null>(null);
  const [historialVersion, setHistorialVersion] = useState(0);
  const [revisionSeleccionada, setRevisionSeleccionada] = useState<number | null>(null);
  const [detalleRevision, setDetalleRevision] = useState<RevisionDetalleDto | null>(null);
  const [cargandoDetalle, setCargandoDetalle] = useState(false);
  const [errorDetalle, setErrorDetalle] = useState<string | null>(null);
  const [confirmarReversion, setConfirmarReversion] =
    useState<ConfirmacionReversion | null>(null);
  const [revirtiendo, setRevirtiendo] = useState(false);
  const [exportando, setExportando] = useState(false);
  const [incluirNotas, setIncluirNotas] = useState(false);
  const [cuarentena, setCuarentena] = useState<CuarentenaDto[]>([]);
  const [confirmarDescarte, setConfirmarDescarte] = useState<string | null>(null);
  const [descartando, setDescartando] = useState<string | null>(null);
  const [recientes, setRecientes] = useState<Reciente[]>(() => leerRecientes());
  const [preferencias, setPreferencias] = useState<Preferencias>(() => cargarPreferencias());
  const [preferenciasAbiertas, setPreferenciasAbiertas] = useState(false);
  const [paleta, setPaleta] = useState<'documentos' | 'comandos' | null>(null);
  const [diffAbierto, setDiffAbierto] = useState(false);
  const [panelAbierto, setPanelAbierto] = useState(false);
  const [voces, setVoces] = useState<VozSistema[]>([]);
  const [vozLectura, setVozLectura] = useState('');
  const [velocidadLectura, setVelocidadLectura] = useState(1);
  const [leyendo, setLeyendo] = useState(false);
  const [pausadoLectura, setPausadoLectura] = useState(false);
  const [errorLectura, setErrorLectura] = useState<string | null>(null);
  const [seleccionTexto, setSeleccionTexto] = useState('');

  const cargando = useRef(false);
  const navegando = useRef(false);
  const revirtiendoRef = useRef(false);
  const detalleSeq = useRef(0);
  const busquedaSeq = useRef(0);
  const buscadorRef = useRef<HTMLInputElement | null>(null);
  const bannerErrorRef = useRef<HTMLDivElement | null>(null);
  const panelRef = useRef<HTMLElement | null>(null);
  const panelToggleRef = useRef<HTMLButtonElement | null>(null);
  const proyectoRef = useRef<ProyectoDto | null>(null);
  const activoRef = useRef<DocumentoDto | null>(null);
  const puedeEditarRef = useRef(false);
  const modoSugerenciasRef = useRef(false);
  const cambioActivoRef = useRef<CambioDetalleDto | null>(null);
  const lecturaSeq = useRef(0);
  const pendienteRef = useRef<Pendiente | null>(null);
  const cuerpoEditorRef = useRef('');
  const drenajeRef = useRef<Promise<boolean> | null>(null);
  const temporizadorRef = useRef<number | null>(null);
  const inicioSesionRef = useRef<string | null>(null);
  const documentoInicioRef = useRef<{ id: string; ruta: string; palabras: number } | null>(null);
  const palabrasInicioRef = useRef(0);
  const pausasRef = useRef<PausaDto[]>([]);
  const pausaInicioRef = useRef<string | null>(null);
  const accionesRef = useRef<{ guardar: () => Promise<boolean>; abrir: () => Promise<void> } | null>(
    null,
  );

  const fijarProyecto = (nuevo: ProyectoDto | null) => {
    proyectoRef.current = nuevo;
    setProyecto(nuevo);
  };

  const fijarActivo = (nuevo: DocumentoDto | null) => {
    activoRef.current = nuevo;
    setActivo(nuevo);
  };

  const cerrarPanel = () => {
    setPanelAbierto(false);
    window.setTimeout(() => {
      if (esVistaEstrecha()) {
        panelToggleRef.current?.focus();
      }
    }, 0);
  };

  const enfocarTrasCambio = () => {
    window.setTimeout(() => {
      if (panelAbierto && esVistaEstrecha()) {
        (document.querySelector('.dabble-panel-cerrar') as HTMLElement | null)?.focus();
      } else {
        (document.querySelector('.ProseMirror') as HTMLElement | null)?.focus();
      }
    }, 0);
  };

  useEffect(() => {
    aplicarPreferencias(preferencias);
    guardarPreferencias(preferencias);
  }, [preferencias]);

  useEffect(() => {
    if (!panelAbierto || !esVistaEstrecha()) {
      return;
    }
    const temporizador = window.setTimeout(() => {
      (document.querySelector('.dabble-panel-cerrar') as HTMLElement | null)?.focus();
    }, 0);
    return () => window.clearTimeout(temporizador);
  }, [panelAbierto]);

  useEffect(() => {
    let cancelado = false;
    void listarVoces().then((lista) => {
      if (!cancelado) {
        setVoces(lista);
        setVozLectura((actual) => actual || lista[0]?.nombre || '');
      }
    });
    return () => {
      cancelado = true;
    };
  }, []);

  useEffect(() => {
    const alCambiarSeleccion = () => {
      setSeleccionTexto(window.getSelection()?.toString() ?? '');
    };
    document.addEventListener('selectionchange', alCambiarSeleccion);
    return () => document.removeEventListener('selectionchange', alCambiarSeleccion);
  }, []);

  useEffect(() => {
    lecturaSeq.current += 1;
    detenerLectura();
    setLeyendo(false);
    setPausadoLectura(false);
    setErrorLectura(null);
  }, [activo?.id]);

  const cancelarTemporizador = () => {
    if (temporizadorRef.current !== null) {
      window.clearTimeout(temporizadorRef.current);
      temporizadorRef.current = null;
    }
  };

  const limpiarPendiente = () => {
    cancelarTemporizador();
    pendienteRef.current = null;
  };

  const limpiarPendienteDe = (documentoId: string) => {
    if (pendienteRef.current?.id === documentoId) {
      limpiarPendiente();
    }
  };

  const escenas = useMemo(
    () => documentos.filter((documento) => documento.tipo === 'escena'),
    [documentos],
  );
  const notas = useMemo(
    () => documentos.filter((documento) => documento.tipo === 'nota'),
    [documentos],
  );
  const escenasVisibles = useMemo(() => {
    const termino = filtroEtiqueta.trim().toLocaleLowerCase();
    return escenas.filter(
      (documento) =>
        (documento.archivo ?? 'activo') === filtroArchivo &&
        (!termino ||
          documento.etiquetas.some((etiqueta) =>
            etiqueta.toLocaleLowerCase().includes(termino),
          )),
    );
  }, [escenas, filtroArchivo, filtroEtiqueta]);
  const notasVisibles = useMemo(() => {
    const termino = filtroEtiqueta.trim().toLocaleLowerCase();
    return notas.filter(
      (documento) =>
        (documento.archivo ?? 'activo') === filtroArchivo &&
        (!termino ||
          documento.etiquetas.some((etiqueta) =>
            etiqueta.toLocaleLowerCase().includes(termino),
          )),
    );
  }, [notas, filtroArchivo, filtroEtiqueta]);
  const idArbolTabbable = focoArbol ?? activo?.id ?? documentos[0]?.id ?? null;

  const moverFocoArbol = (documento: DocumentoDto, destino: 'siguiente' | 'anterior' | 'inicio' | 'fin') => {
    const indice = documentos.findIndex((candidato) => candidato.id === documento.id);
    if (indice < 0 || documentos.length === 0) {
      return;
    }
    const siguienteIndice =
      destino === 'inicio'
        ? 0
        : destino === 'fin'
          ? documentos.length - 1
          : destino === 'siguiente'
            ? Math.min(documentos.length - 1, indice + 1)
            : Math.max(0, indice - 1);
    const siguiente = documentos[siguienteIndice];
    setFocoArbol(siguiente.id);
    window.setTimeout(() => document.getElementById(`doc-${siguiente.id}`)?.focus(), 0);
  };

  const manejarTeclaArbol = (evento: ReactKeyboardEvent, documento: DocumentoDto) => {
    if (evento.key === 'ArrowDown' || evento.key === 'ArrowUp') {
      evento.preventDefault();
      moverFocoArbol(documento, evento.key === 'ArrowDown' ? 'siguiente' : 'anterior');
    } else if (evento.key === 'Home' || evento.key === 'End') {
      evento.preventDefault();
      moverFocoArbol(documento, evento.key === 'Home' ? 'inicio' : 'fin');
    }
  };

  const gruposEscenas = useMemo(
    () =>
      agrupar(
        escenasVisibles,
        (documento) => grupoEscena(documento).clave,
        (documento) => grupoEscena(documento).etiqueta,
      ),
    [escenasVisibles],
  );
  const libros = useMemo(() => {
    const porLibro = new Map<
      string,
      { etiqueta: string; capitulos: Array<{ clave: string; etiqueta: string; documentos: DocumentoDto[] }> }
    >();
    for (const grupo of gruposEscenas) {
      const [claveLibro, claveCapitulo] = grupo.clave.split('/');
      const [etiquetaLibro, etiquetaCapitulo] = grupo.etiqueta.split(' / ');
      const libro = porLibro.get(claveLibro) ?? {
        etiqueta: etiquetaLibro ?? claveLibro,
        capitulos: [],
      };
      libro.capitulos.push({
        clave: grupo.clave,
        etiqueta: etiquetaCapitulo ?? claveCapitulo,
        documentos: grupo.documentos,
      });
      porLibro.set(claveLibro, libro);
    }
    return Array.from(porLibro.values());
  }, [gruposEscenas]);
  const personajes = useMemo(
    () =>
      notas
        .filter((documento) => documento.clase === 'personajes')
        .map((documento) => ({ id: documento.id, titulo: documento.titulo })),
    [notas],
  );
  const tramas = useMemo(
    () =>
      notas
        .filter((documento) => documento.clase === 'tramas')
        .map((documento) => ({ id: documento.id, titulo: documento.titulo })),
    [notas],
  );

  const hallazgos = useMemo(
    () => (activo ? analizarCompatibilidad(activo.cuerpo) : []),
    [activo],
  );
  const soloLecturaProyecto = (proyecto?.errores.length ?? 0) > 0;
  const bloqueoDocumento = hallazgos.length > 0;
  const bloqueoGuardado =
    estadoGuardado === 'conflicto' ||
    estadoGuardado === 'recuperacion' ||
    estadoGuardado === 'solo-lectura';
  const puedeEditar =
    activo !== null &&
    !soloLecturaProyecto &&
    !bloqueoDocumento &&
    !bloqueoGuardado &&
    !revirtiendo &&
    !procesandoCambio;
  puedeEditarRef.current = puedeEditar;
  modoSugerenciasRef.current = modoSugerencias;
  cambioActivoRef.current = cambioActivo;

  const estadoVisible: EstadoGuardado =
    estadoGuardado === 'conflicto' ||
    estadoGuardado === 'recuperacion' ||
    estadoGuardado === 'ocupado' ||
    estadoGuardado === 'error'
      ? estadoGuardado
      : soloLecturaProyecto || bloqueoDocumento
        ? 'solo-lectura'
        : estadoGuardado;

  const recordarProyecto = (
    resumen: ProyectoDto,
    palabras = 0,
    color?: string,
  ) => {
    if (resumen.raiz === '(modo demo)') {
      return;
    }
    setRecientes((lista) => {
      const anterior = lista.find((item) => item.ruta === resumen.raiz);
      const siguiente = [
        {
          ruta: resumen.raiz,
          titulo: resumen.titulo,
          autor: resumen.autor ?? anterior?.autor,
          palabras: palabras ?? anterior?.palabras ?? 0,
          color: color ?? anterior?.color ?? colorDesdeTitulo(resumen.titulo),
        },
        ...lista.filter((item) => item.ruta !== resumen.raiz),
      ].slice(0, 8);
      guardarRecientes(siguiente);
      return siguiente;
    });
  };

  const cargarMetas = async (ruta: string) => {
    setErrorMetas(null);
    const [metasResultado, sesionesResultado, gridResultado, formatoResultado] =
      await Promise.allSettled([
        leerMetas(ruta),
        listarSesiones(ruta),
        leerPlotGrid(ruta),
        leerFormatoPdf(ruta),
      ]);
    const errores: string[] = [];
    if (metasResultado.status === 'fulfilled') {
      setMetas(metasResultado.value);
      setMetaDiaria(metasResultado.value.diaria?.objetivo.toString() ?? '');
      setMetaLibro(metasResultado.value.libro?.objetivo.toString() ?? '');
      setDesafioObjetivo(metasResultado.value.desafio?.objetivo.toString() ?? '');
      setDesafioLimite(metasResultado.value.desafio?.limite ?? '');
    } else {
      setMetas(null);
      setMetaDiaria('');
      setMetaLibro('');
      setDesafioObjetivo('');
      setDesafioLimite('');
      errores.push(`Metas: ${errorDe(metasResultado.reason).mensaje}`);
    }
    if (sesionesResultado.status === 'fulfilled') {
      setSesiones(sesionesResultado.value);
    } else {
      setSesiones([]);
      errores.push(`Sesiones: ${errorDe(sesionesResultado.reason).mensaje}`);
    }
    if (gridResultado.status === 'fulfilled') {
      setPlotGrid(gridResultado.value);
      setTramaSeleccionada((actual) =>
        gridResultado.value.tramas.some((trama) => trama.id === actual)
          ? actual
          : (gridResultado.value.tramas[0]?.id ?? ''),
      );
    } else {
      setPlotGrid({ version: 1, tramas: [] });
      setTramaSeleccionada('');
      errores.push(`Planificación: ${errorDe(gridResultado.reason).mensaje}`);
    }
    if (formatoResultado.status === 'fulfilled') {
      setFormatoPdf(formatoResultado.value);
    } else {
      errores.push(`Formato PDF: ${errorDe(formatoResultado.reason).mensaje}`);
    }
    if (errores.length > 0) {
      setErrorMetas(errores.join(' · '));
    }
  };

  const cargarCambiosDeProyecto = async (ruta: string, documentoId?: string | null) => {
    setCargandoCambios(true);
    setErrorCambios(null);
    try {
      const lista = await listarCambios(ruta);
      if (proyectoRef.current?.raiz !== ruta) {
        return;
      }
      setCambiosProyecto(lista);
      const resumen = lista.find((cambio) => cambio.documento === documentoId);
      if (!resumen) {
        setCambioActivo(null);
        return;
      }
      const detalle = await leerCambio(ruta, resumen.id);
      if (proyectoRef.current?.raiz === ruta) {
        setCambioActivo(detalle);
      }
    } catch (fallo) {
      if (proyectoRef.current?.raiz === ruta) {
        setCambiosProyecto([]);
        setCambioActivo(null);
        setErrorCambios(errorDe(fallo).mensaje);
      }
    } finally {
      setCargandoCambios(false);
    }
  };

  const cargar = async (ruta: string, objetivoPreferido?: string | null) => {
    if (cargando.current) {
      return;
    }
    cargando.current = true;
    setEstado('cargando');
    setTemporizadorActivo(false);
    setTemporizadorPausado(false);
    setMetas(null);
    setSesiones([]);
    setPlotGrid({ version: 1, tramas: [] });
    setTramaSeleccionada('');
    setErrorMetas(null);
    inicioSesionRef.current = null;
    documentoInicioRef.current = null;
    pausaInicioRef.current = null;
    pausasRef.current = [];
    setError(null);
    setMensaje(null);
    setErrorGuardado(null);
    setConsulta('');
    setFiltroArchivo('activo');
    setResultados([]);
    setErrorBusqueda(null);
    setCuarentena([]);
    setConfirmarDescarte(null);
    setRevisiones([]);
    setRevisionSeleccionada(null);
    setDetalleRevision(null);
    setErrorDetalle(null);
    setCargandoDetalle(false);
    detalleSeq.current += 1;
    busquedaSeq.current += 1;
    setConfirmarReversion(null);
    setModoSugerencias(false);
    modoSugerenciasRef.current = false;
    setCambiosProyecto([]);
    setCambioActivo(null);
    setErrorCambios(null);
    setComparacion(null);
    setErrorComparacion(null);
    try {
      const resumen = await abrirProyecto(ruta);
      const lista = await listar(ruta, true);
      const preferido = objetivoPreferido
        ? lista.find((documento) => documento.id === objetivoPreferido)
        : null;
      const primero =
        preferido ?? lista.find((documento) => documento.tipo === 'escena') ?? lista[0] ?? null;
      const leido = primero ? await leer(ruta, primero.id) : null;
      const palabras = lista.reduce(
        (total, documento) => total + contarPalabras(textoSinMarcas(documento.cuerpo)),
        0,
      );
      fijarProyecto(resumen);
      recordarProyecto(resumen, palabras);
      setDocumentos(lista);
      limpiarPendiente();
      fijarActivo(leido);
      cuerpoEditorRef.current = leido?.cuerpo ?? '';
      setRevisionEditor((valor) => valor + 1);
      setEstadoGuardado(resumen.errores.length > 0 ? 'solo-lectura' : 'guardado');
      setConfirmarRecarga(false);
      try {
        setCuarentena(await pedirCuarentena(ruta));
      } catch {
        setCuarentena([]);
      }
      await cargarMetas(ruta);
      void cargarCambiosDeProyecto(ruta, leido?.id);
      setEstado('listo');
    } catch (fallo) {
      const detalle = errorDe(fallo);
      setError(detalle.mensaje);
      setEstado(proyectoRef.current ? 'listo' : 'error');
    } finally {
      cargando.current = false;
    }
  };

  const guardarUnaVez = async (): Promise<boolean> => {
    const pendiente = pendienteRef.current;
    const proyectoActual = proyectoRef.current;
    if (!pendiente || !proyectoActual) {
      return true;
    }
    if (proyectoActual.errores.length > 0) {
      setEstadoGuardado('solo-lectura');
      setErrorGuardado(
        'El proyecto tiene errores de verificación y no se puede escribir hasta corregirlos.',
      );
      return false;
    }
    const activoActual = activoRef.current;
    const propuestaActual = cambioActivoRef.current;
    const enSugerencia =
      modoSugerenciasRef.current ||
      (propuestaActual !== null && propuestaActual.documento === pendiente.id);
    if (enSugerencia) {
      setEstadoGuardado('guardando');
      try {
        const cambio = await registrarCambio(
          proyectoActual.raiz,
          pendiente.id,
          pendiente.cuerpo,
          proyectoActual.autor?.trim() || 'local',
        );
        const cuerpoActual = cuerpoEditorRef.current;
        if (cuerpoActual === pendiente.cuerpo) {
          pendienteRef.current = null;
        } else {
          pendienteRef.current = { id: pendiente.id, cuerpo: cuerpoActual };
        }
        if (cambio) {
          setCambioActivo(cambio);
          const resumen: CambioResumenDto = {
            id: cambio.id,
            documento: cambio.documento,
            ruta: cambio.ruta,
            autor: cambio.autor,
            ts: cambio.ts,
            baseRevision: cambio.baseRevision,
            insertadas: cambio.insertadas,
            borradas: cambio.borradas,
          };
          setCambiosProyecto((lista) => [
            ...lista.filter((item) => item.documento !== cambio.documento),
            resumen,
          ]);
          setEstadoGuardado(
            cuerpoActual === pendiente.cuerpo ? 'sugerencia' : 'pendiente',
          );
        } else {
          setCambioActivo(null);
          setCambiosProyecto((lista) =>
            lista.filter((item) => item.documento !== pendiente.id),
          );
          setEstadoGuardado(
            cuerpoActual === pendiente.cuerpo ? 'guardado' : 'pendiente',
          );
        }
        setErrorCambios(null);
        setErrorGuardado(null);
        setConfirmarRecarga(false);
        return true;
      } catch (fallo) {
        const detalle = errorDe(fallo);
        setErrorGuardado(detalle.mensaje);
        setErrorCambios(detalle.mensaje);
        setEstadoGuardado(estadoDeError(detalle.tipo));
        return false;
      }
    }
    setEstadoGuardado('guardando');
    try {
      const hashBase =
        activoActual && activoActual.id === pendiente.id ? activoActual.hashArchivo : null;
      const guardado = await escribir(
        proyectoActual.raiz,
        pendiente.id,
        pendiente.cuerpo,
        hashBase,
      );
      fijarProyecto({ ...proyectoActual, revision: guardado.revision });
      setDocumentos((lista) =>
        lista.map((documento) =>
          documento.id === pendiente.id
            ? { ...documento, cuerpo: pendiente.cuerpo, hashArchivo: guardado.hashArchivo }
            : documento,
        ),
      );
      if (activoActual && activoActual.id === pendiente.id) {
        fijarActivo({
          ...activoActual,
          cuerpo: pendiente.cuerpo,
          hashArchivo: guardado.hashArchivo,
        });
      }
      // El editor pudo cambiar mientras el guardado estaba en vuelo. Solo se
      // limpia el pendiente si lo que hay ahora mismo en el editor es justo lo
      // que se acaba de persistir; si no, se conserva la versión más reciente.
      const cuerpoActual = cuerpoEditorRef.current;
      if (cuerpoActual === pendiente.cuerpo) {
        pendienteRef.current = null;
        setEstadoGuardado('guardado');
      } else {
        pendienteRef.current = { id: pendiente.id, cuerpo: cuerpoActual };
        setEstadoGuardado('pendiente');
      }
      setErrorGuardado(null);
      setConfirmarRecarga(false);
      return true;
    } catch (fallo) {
      const detalle = errorDe(fallo);
      setErrorGuardado(detalle.mensaje);
      setEstadoGuardado(estadoDeError(detalle.tipo));
      return false;
    }
  };

  const drenar = async (): Promise<boolean> => {
    while (pendienteRef.current !== null) {
      const correcto = await guardarUnaVez();
      if (!correcto) {
        return false;
      }
    }
    return true;
  };

  const guardarAhora = async (): Promise<boolean> => {
    cancelarTemporizador();
    if (drenajeRef.current) {
      const anterior = await drenajeRef.current;
      if (!anterior) {
        return false;
      }
    }
    if (!pendienteRef.current) {
      return true;
    }
    const promesa = drenar();
    drenajeRef.current = promesa;
    try {
      return await promesa;
    } finally {
      if (drenajeRef.current === promesa) {
        drenajeRef.current = null;
      }
    }
  };

  const guardarMetasPanel = async () => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    if (errorMetas) {
      setMensaje('No se guardan metas mientras haya un sidecar ilegible.');
      return;
    }
    const numero = (texto: string) => {
      const limpio = texto.trim();
      if (!limpio) {
        return null;
      }
      const valor = Number(limpio);
      return Number.isFinite(valor) && valor >= 0 ? Math.floor(valor) : null;
    };
    const diaria = numero(metaDiaria);
    const libro = numero(metaLibro);
    const desafio = numero(desafioObjetivo);
    const nuevo: MetasDto = {
      version: 1,
      diaria: diaria === null ? null : { objetivo: diaria, desde: metas?.diaria?.desde ?? fechaHoy() },
      libro: libro === null ? null : { objetivo: libro },
      desafio:
        desafio === null || !desafioLimite
          ? null
          : { objetivo: desafio, limite: desafioLimite },
      archivadas: metas?.archivadas ?? [],
    };
    try {
      await guardarMetas(actual.raiz, nuevo);
      setMetas(nuevo);
      setErrorMetas(null);
      setMensaje('Metas guardadas.');
    } catch (fallo) {
      setErrorMetas(errorDe(fallo).mensaje);
    }
  };

  const iniciarTemporizador = () => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento) {
      setErrorMetas('Abre un documento antes de iniciar el temporizador.');
      return;
    }
    inicioSesionRef.current = new Date().toISOString();
    documentoInicioRef.current = {
      id: documento.id,
      ruta: documento.ruta,
      palabras: palabrasDocumento,
    };
    palabrasInicioRef.current = palabrasDocumento;
    pausasRef.current = [];
    pausaInicioRef.current = null;
    setRestante(preset * 60);
    setTemporizadorActivo(true);
    setTemporizadorPausado(false);
    setErrorMetas(null);
  };

  const pausarTemporizador = () => {
    if (!temporizadorActivo || temporizadorPausado) {
      return;
    }
    pausaInicioRef.current = new Date().toISOString();
    setTemporizadorPausado(true);
  };

  const reanudarTemporizador = () => {
    if (!temporizadorPausado) {
      return;
    }
    const inicio = pausaInicioRef.current;
    if (inicio) {
      pausasRef.current = [...pausasRef.current, { inicio, fin: new Date().toISOString() }];
    }
    pausaInicioRef.current = null;
    setTemporizadorPausado(false);
  };

  const finalizarTemporizador = async () => {
    const actual = proyectoRef.current;
    const inicio = inicioSesionRef.current;
    const documentoInicio = documentoInicioRef.current;
    if (!actual || !inicio || !documentoInicio) {
      return;
    }
    if (temporizadorPausado && pausaInicioRef.current) {
      pausasRef.current = [
        ...pausasRef.current,
        { inicio: pausaInicioRef.current, fin: new Date().toISOString() },
      ];
    }
    const actualEnLista = documentos.find((candidato) => candidato.id === documentoInicio.id);
    const palabrasFinales = actualEnLista
      ? contarPalabras(textoSinMarcas(actualEnLista.cuerpo))
      : palabrasDocumento;
    const sesion: SesionDto = {
      id: '',
      inicio,
      fin: new Date().toISOString(),
      preset,
      pausas: pausasRef.current,
      palabras: Math.max(0, palabrasFinales - documentoInicio.palabras),
      documento: documentoInicio.ruta,
      autor: actual.autor?.trim() || 'local',
    };
    try {
      await agregarSesion(actual.raiz, sesion);
      setSesiones(await listarSesiones(actual.raiz));
      setMensaje(`Sesión de ${preset} min registrada.`);
      setErrorMetas(null);
    } catch (fallo) {
      setErrorMetas(errorDe(fallo).mensaje);
    }
    setTemporizadorActivo(false);
    setTemporizadorPausado(false);
    inicioSesionRef.current = null;
    pausaInicioRef.current = null;
    pausasRef.current = [];
  };

  const alCambiar = (cuerpo: string) => {
    const actual = activoRef.current;
    if (!actual || !puedeEditarRef.current || revirtiendoRef.current) {
      return;
    }
    cancelarTemporizador();
    cuerpoEditorRef.current = cuerpo;
    // Si hay un guardado en vuelo, no se puede declarar «guardado» aunque el
    // cuerpo coincida con la última versión de `activo`: el disco todavía puede
    // recibir una versión intermedia. Se deja pendiente para que el drenaje la
    // vuelva a escribir después.
    if (cuerpo === actual.cuerpo && !cambioActivoRef.current && drenajeRef.current === null) {
      pendienteRef.current = null;
      setEstadoGuardado('guardado');
      setErrorGuardado(null);
      setConfirmarRecarga(false);
      return;
    }
    pendienteRef.current = { id: actual.id, cuerpo };
    setEstadoGuardado('pendiente');
    setErrorGuardado(null);
    setConfirmarRecarga(false);
    temporizadorRef.current = window.setTimeout(() => {
      temporizadorRef.current = null;
      void guardarAhora();
    }, RETARDO_AUTOGUARDADO);
  };

  const seleccionar = async (documento: DocumentoDto) => {
    if (navegando.current || revirtiendoRef.current) {
      return;
    }
    if (activoRef.current?.id === documento.id) {
      setVista('editor');
      return;
    }
    navegando.current = true;
    try {
      const guardado = await guardarAhora();
      if (!guardado) {
        setMensaje('No se cambió de documento: primero resuelve el guardado pendiente.');
        return;
      }
      const proyectoActual = proyectoRef.current;
      if (!proyectoActual) {
        return;
      }
      setMensaje(null);
      setVista('editor');
      try {
        const leido = await leer(proyectoActual.raiz, documento.id);
        fijarActivo(leido);
        cuerpoEditorRef.current = leido.cuerpo;
        setRevisionEditor((valor) => valor + 1);
        setErrorGuardado(null);
        setConfirmarRecarga(false);
        setEstadoGuardado(proyectoActual.errores.length > 0 ? 'solo-lectura' : 'guardado');
      } catch (fallo) {
        const detalle = errorDe(fallo);
        setErrorGuardado(detalle.mensaje);
        setEstadoGuardado(estadoDeError(detalle.tipo));
      }
    } finally {
      navegando.current = false;
    }
  };

  const renombrarActivo = async () => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    const titulo = tituloEdicion.trim();
    if (!actual || !documento) {
      return;
    }
    if (!titulo) {
      setMensaje('El título no puede estar vacío.');
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se renombró: primero resuelve el guardado pendiente.');
      return;
    }
    try {
      const actualizado = await renombrarDocumento(actual.raiz, documento.id, titulo);
      setDocumentos((lista) =>
        lista.map((candidato) =>
          candidato.id === actualizado.id ? { ...candidato, ...actualizado } : candidato,
        ),
      );
      fijarActivo(actualizado);
      setTituloEdicion(actualizado.titulo);
      setMensaje(`Documento renombrado: ${actualizado.titulo}.`);
    } catch (fallo) {
      setMensaje(`No se pudo renombrar: ${errorDe(fallo).mensaje}`);
    }
  };

  const moverDocumento = async (id: string, libro: string, capitulo: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se movió: primero resuelve el guardado pendiente.');
      return;
    }
    try {
      const movido = await moverDocumentoApi(actual.raiz, id, libro, capitulo, null);
      setDocumentos(await listar(actual.raiz, true));
      if (activoRef.current?.id === id) {
        fijarActivo(movido);
        cuerpoEditorRef.current = movido.cuerpo;
        setRevisionEditor((valor) => valor + 1);
      }
      setMensaje(`Documento movido a ${libro}/${capitulo}.`);
    } catch (fallo) {
      setMensaje(`No se pudo mover: ${errorDe(fallo).mensaje}`);
    } finally {
      setDestinoArrastre(null);
    }
  };

  const reordenarActivo = async (delta: number) => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento || documento.tipo !== 'escena' || documento.orden === null) {
      return;
    }
    const hermanas = escenas
      .filter((candidata) => {
        const partes = candidata.ruta.split('/');
        const propias = documento.ruta.split('/');
        return partes[1] === propias[1] && partes[2] === propias[2];
      })
      .sort((a, b) => (a.orden ?? 0) - (b.orden ?? 0));
    const indice = hermanas.findIndex((candidata) => candidata.id === documento.id);
    const vecina = hermanas[indice + delta];
    if (!vecina || vecina.orden === null || documento.orden === null) {
      return;
    }
    try {
      await intercambiarOrden(actual.raiz, documento.id, vecina.id);
      setDocumentos(await listar(actual.raiz, true));
      setMensaje('Orden de escenas actualizado.');
    } catch (fallo) {
      setMensaje(`No se pudo reordenar: ${errorDe(fallo).mensaje}`);
    }
  };

  const convertirIdeaActiva = async () => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento || documento.clase !== 'ideas') {
      return;
    }
    const padre = await elegirCarpetaCreacion();
    if (!padre) {
      return;
    }
    const titulo = documento.titulo;
    const destino = `${padre.replace(/[\\/]+$/, '')}/${slugProyecto(titulo)}`;
    try {
      const resumen = await convertirIdeaEnProyecto(actual.raiz, documento.id, destino, titulo);
      recordarProyecto(resumen, 0);
      setMensaje(`Idea convertida en proyecto: ${resumen.titulo}.`);
    } catch (fallo) {
      setMensaje(`No se pudo convertir la idea: ${errorDe(fallo).mensaje}`);
    }
  };

  const guardarMetadatosActivo = async () => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento) {
      return;
    }
    const etiquetas = etiquetasEdicion
      .split(',')
      .map((etiqueta) => etiqueta.trim())
      .filter(Boolean);
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se guardaron los metadatos: primero resuelve el guardado pendiente.');
      return;
    }
    try {
      const actualizado =
        documento.tipo === 'escena'
          ? await actualizarEscena(actual.raiz, documento.id, estadoEdicion || null, etiquetas)
          : documento.clase === 'personajes'
            ? await actualizarPersonaje(
                actual.raiz,
                documento.id,
                rolPersonaje || null,
                documento.cast ?? [],
                aliasPersonaje
                  .split(',')
                  .map((alias) => alias.trim())
                  .filter(Boolean),
              )
            : await actualizarNota(actual.raiz, documento.id, etiquetas);
      setDocumentos((lista) =>
        lista.map((candidato) =>
          candidato.id === actualizado.id ? { ...candidato, ...actualizado } : candidato,
        ),
      );
      fijarActivo(actualizado);
      setMensaje('Metadatos del documento guardados.');
    } catch (fallo) {
      setMensaje(`No se pudieron guardar los metadatos: ${errorDe(fallo).mensaje}`);
    }
  };

  const crearStickyActivo = async () => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    const texto = textoSticky.trim();
    if (!actual || !documento || !texto) {
      return;
    }
    try {
      const nuevo = await crearSticky(
        actual.raiz,
        documento.ruta,
        texto,
        actual.autor?.trim() || 'local',
      );
      setComentarios((lista) => [...lista, nuevo]);
      setTextoSticky('');
      setErrorComentarios(null);
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const responderHilo = async (id: string) => {
    const actual = proyectoRef.current;
    const texto = (respuestas[id] ?? '').trim();
    if (!actual || !texto) {
      return;
    }
    try {
      const actualizado = await agregarMensajeComentario(
        actual.raiz,
        id,
        'respuesta',
        actual.autor?.trim() || 'local',
        texto,
      );
      setComentarios((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
      setRespuestas((valor) => ({ ...valor, [id]: '' }));
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const reabrirHilo = async (id: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    try {
      const actualizado = await reabrirComentario(actual.raiz, id, actual.autor?.trim() || 'local');
      setComentarios((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
      setComentariosProyecto((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const archivarHilo = async (id: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    try {
      await archivarComentario(actual.raiz, id);
      setComentarios((lista) => lista.filter((comentario) => comentario.id !== id));
      setComentariosProyecto((lista) => lista.filter((comentario) => comentario.id !== id));
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const editarMensaje = async (id: string, mensajeId: string) => {
    const actual = proyectoRef.current;
    const texto = textoEdicionMensaje.trim();
    if (!actual || !texto) {
      return;
    }
    try {
      const actualizado = await editarMensajeComentario(
        actual.raiz,
        id,
        mensajeId,
        actual.autor?.trim() || 'local',
        texto,
      );
      setComentarios((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
      setComentariosProyecto((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
      setMensajeEditando(null);
      setTextoEdicionMensaje('');
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const borrarMensaje = async (id: string, mensajeId: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    try {
      const actualizado = await borrarMensajeComentario(
        actual.raiz,
        id,
        mensajeId,
        actual.autor?.trim() || 'local',
      );
      setComentarios((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
      setComentariosProyecto((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const copiarTexto = async (conComentarios: boolean) => {
    const texto = conComentarios
      ? cuerpoVisible
      : cuerpoVisible.replace(
          /\[\[comentario:(?:[0-9A-HJKMNP-TV-Z]{26}\|[^|]*\|)?([\s\S]*?)\]\]/g,
          '$1',
        );
    try {
      await navigator.clipboard.writeText(texto);
      setMensaje(conComentarios ? 'Texto copiado con comentarios.' : 'Texto copiado sin comentarios.');
    } catch {
      setMensaje('No se pudo copiar al portapapeles.');
    }
  };

  const resolverHilo = async (id: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    try {
      const actualizado = await resolverComentario(
        actual.raiz,
        id,
        actual.autor?.trim() || 'local',
      );
      setComentarios((lista) =>
        lista.map((comentario) => (comentario.id === id ? actualizado : comentario)),
      );
    } catch (fallo) {
      setErrorComentarios(errorDe(fallo).mensaje);
    }
  };

  const cambiarArchivoActivo = async (archivo: 'archivado' | 'papelera' | null) => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se cambió de ubicación: primero resuelve el guardado pendiente.');
      return;
    }
    try {
      const actualizado = await cambiarArchivoApi(actual.raiz, documento.id, archivo);
      setDocumentos((lista) =>
        lista.map((candidato) =>
          candidato.id === actualizado.id ? { ...candidato, ...actualizado } : candidato,
        ),
      );
      fijarActivo(actualizado);
      setMensaje(
        archivo === null
          ? 'Documento restaurado a activos.'
          : archivo === 'archivado'
            ? 'Documento archivado.'
            : 'Documento enviado a la papelera.',
      );
    } catch (fallo) {
      setMensaje(`No se pudo cambiar de ubicación: ${errorDe(fallo).mensaje}`);
    }
  };

  const duplicarPorId = async (id: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se duplicó: primero resuelve el guardado pendiente.');
      return;
    }
    try {
      const copia = await duplicarDocumento(actual.raiz, id);
      setDocumentos(await listar(actual.raiz, true));
      await seleccionar(copia);
      setMensaje(`Documento duplicado: ${copia.titulo}.`);
    } catch (fallo) {
      setMensaje(`No se pudo duplicar: ${errorDe(fallo).mensaje}`);
    }
  };

  const crearNotaNueva = async () => {
    const actual = proyectoRef.current;
    const titulo = nuevaNotaTitulo.trim();
    if (!actual || !titulo) {
      return;
    }
    try {
      const nota = await crearNotaApi(actual.raiz, nuevaNotaTipo, titulo);
      setDocumentos(await listar(actual.raiz, true));
      await seleccionar(nota);
      setNuevaNotaTitulo('');
      setMensaje(`Nota creada: ${nota.titulo}.`);
    } catch (fallo) {
      setMensaje(`No se pudo crear la nota: ${errorDe(fallo).mensaje}`);
    }
  };

  const crearEscenaNueva = async () => {
    const actual = proyectoRef.current;
    const titulo = nuevaEscenaTitulo.trim();
    if (!actual || !titulo) {
      return;
    }
    try {
      const escena = await crearEscenaApi(actual.raiz, titulo, null, null);
      setDocumentos(await listar(actual.raiz, true));
      await seleccionar(escena);
      setNuevaEscenaTitulo('');
      setMensaje(`Escena creada: ${escena.titulo}.`);
    } catch (fallo) {
      setMensaje(`No se pudo crear la escena: ${errorDe(fallo).mensaje}`);
    }
  };

  const crearTramaGrid = async () => {
    const actual = proyectoRef.current;
    const nombre = nuevaTrama.trim();
    if (!actual || !nombre) {
      return;
    }
    if (errorMetas) {
      setMensaje('No se modifica el Plot Grid mientras haya un sidecar ilegible.');
      return;
    }
    try {
      const nota = await crearNotaApi(actual.raiz, 'tramas', nombre);
      const grid: PlotGridDto = {
        version: 1,
        tramas: [
          ...plotGrid.tramas,
          { id: nota.id, nombre, color: colorTrama, puntos: [] },
        ],
      };
      await guardarPlotGrid(actual.raiz, grid);
      setPlotGrid(grid);
      setTramaSeleccionada(nota.id);
      setDocumentos(await listar(actual.raiz, true));
      setNuevaTrama('');
      setMensaje(`Trama creada: ${nombre}.`);
    } catch (fallo) {
      setMensaje(`No se pudo crear la trama: ${errorDe(fallo).mensaje}`);
    }
  };

  const actualizarTrama = async () => {
    const actual = proyectoRef.current;
    const trama = plotGrid.tramas.find((candidata) => candidata.id === tramaSeleccionada);
    if (!actual || !trama || !nombreTramaEdit.trim()) {
      return;
    }
    const nombre = nombreTramaEdit.trim();
    if (nombre !== trama.nombre) {
      try {
        await renombrarDocumento(actual.raiz, trama.id, nombre);
      } catch (fallo) {
        setMensaje(`No se pudo renombrar la nota de la trama: ${errorDe(fallo).mensaje}`);
        return;
      }
    }
    const grid: PlotGridDto = {
      version: 1,
      tramas: plotGrid.tramas.map((candidata) =>
        candidata.id === trama.id
          ? { ...candidata, nombre, color: colorTramaEdit }
          : candidata,
      ),
    };
    try {
      await guardarPlotGrid(actual.raiz, grid);
      setPlotGrid(grid);
      setDocumentos(await listar(actual.raiz, true));
      setMensaje('Trama actualizada.');
    } catch (fallo) {
      setMensaje(`No se pudo actualizar la trama: ${errorDe(fallo).mensaje}`);
    }
  };

  const quitarPuntoGrid = async (tramaId: string, escenaId: string) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const grid: PlotGridDto = {
      version: 1,
      tramas: plotGrid.tramas.map((trama) =>
        trama.id === tramaId
          ? { ...trama, puntos: trama.puntos.filter((punto) => punto.escena !== escenaId) }
          : trama,
      ),
    };
    try {
      await guardarPlotGrid(actual.raiz, grid);
      setPlotGrid(grid);
    } catch (fallo) {
      setMensaje(`No se pudo quitar el punto: ${errorDe(fallo).mensaje}`);
    }
  };

  const moverPuntoGrid = async (tramaId: string, escenaId: string, delta: number) => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const trama = plotGrid.tramas.find((candidata) => candidata.id === tramaId);
    if (!trama) {
      return;
    }
    const puntos = [...trama.puntos].sort((a, b) => a.orden - b.orden);
    const indice = puntos.findIndex((punto) => punto.escena === escenaId);
    const destino = indice + delta;
    if (indice < 0 || destino < 0 || destino >= puntos.length) {
      return;
    }
    [puntos[indice], puntos[destino]] = [puntos[destino], puntos[indice]];
    const grid: PlotGridDto = {
      version: 1,
      tramas: plotGrid.tramas.map((candidata) =>
        candidata.id === tramaId
          ? { ...candidata, puntos: puntos.map((punto, posicion) => ({ ...punto, orden: posicion + 1 })) }
          : candidata,
      ),
    };
    try {
      await guardarPlotGrid(actual.raiz, grid);
      setPlotGrid(grid);
    } catch (fallo) {
      setMensaje(`No se pudo mover el punto: ${errorDe(fallo).mensaje}`);
    }
  };

  useEffect(() => {
    const trama = plotGrid.tramas.find((candidata) => candidata.id === tramaSeleccionada);
    if (trama) {
      setNombreTramaEdit(trama.nombre);
      setColorTramaEdit(trama.color);
    }
  }, [tramaSeleccionada, plotGrid.tramas]);

  const anadirPuntoGrid = async () => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento || !tramaSeleccionada || documento.tipo !== 'escena') {
      if (documento && documento.tipo !== 'escena') {
        setMensaje('Solo se pueden añadir escenas al Plot Grid.');
      }
      return;
    }
    if (errorMetas) {
      setMensaje('No se modifica el Plot Grid mientras haya un sidecar ilegible.');
      return;
    }
    const trama = plotGrid.tramas.find((candidata) => candidata.id === tramaSeleccionada);
    if (!trama) {
      return;
    }
    if (trama.puntos.some((punto) => punto.escena === documento.id)) {
      setMensaje('La escena activa ya está en esta trama.');
      return;
    }
    const grid: PlotGridDto = {
      version: 1,
      tramas: plotGrid.tramas.map((candidata) =>
        candidata.id === tramaSeleccionada
          ? {
              ...candidata,
              puntos: [
                ...candidata.puntos,
                { escena: documento.id, orden: candidata.puntos.length + 1, etiqueta: null },
              ],
            }
          : candidata,
      ),
    };
    try {
      await guardarPlotGrid(actual.raiz, grid);
      setPlotGrid(grid);
      setMensaje(`Escena añadida a ${trama.nombre}.`);
    } catch (fallo) {
      setMensaje(`No se pudo añadir el punto: ${errorDe(fallo).mensaje}`);
    }
  };

  const importarArchivo = async () => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const archivos = await elegirArchivosImportacion();
    if (archivos.length === 0) {
      return;
    }
    try {
      const previa = await previsualizarImportacion(archivos[0]);
      setVistaPreviaImportacion({ archivos, ...previa });
    } catch (fallo) {
      setMensaje(`No se pudo previsualizar: ${errorDe(fallo).mensaje}`);
    }
  };

  const confirmarImportacion = async () => {
    const actual = proyectoRef.current;
    const pendiente = vistaPreviaImportacion;
    if (!actual || !pendiente) {
      return;
    }
    setVistaPreviaImportacion(null);
    try {
      const tipoNota = tipoImportacion || null;
      let ultimo = null;
      for (const archivo of pendiente.archivos) {
        const nombre = archivo.split(/[\\/]/).at(-1) ?? 'importado';
        const titulo = nombre.replace(/\.(md|markdown|docx|html?|rtf)$/i, '');
        ultimo = /\.docx$/i.test(archivo)
          ? await importarDocx(actual.raiz, archivo, titulo, tipoNota)
          : /\.html?$/i.test(archivo)
            ? await importarHtml(actual.raiz, archivo, titulo, tipoNota)
            : /\.rtf$/i.test(archivo)
              ? await importarRtf(actual.raiz, archivo, titulo, tipoNota)
              : await importarMarkdown(actual.raiz, archivo, titulo, tipoNota);
      }
      setDocumentos(await listar(actual.raiz, true));
      if (ultimo) {
        await seleccionar(ultimo);
      }
      setMensaje(`${pendiente.archivos.length} documento(s) importado(s).`);
    } catch (fallo) {
      setMensaje(`No se pudo importar: ${errorDe(fallo).mensaje}`);
    }
  };

  const hacerBackup = async () => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se creó el backup: primero resuelve el guardado pendiente.');
      return;
    }
    const destino = await elegirDestinoBackup(actual.titulo);
    if (!destino) {
      return;
    }
    try {
      const ruta = await crearBackupApi(actual.raiz, destino);
      setMensaje(`Backup creado: ${ruta}.`);
    } catch (fallo) {
      setMensaje(`No se pudo crear el backup: ${errorDe(fallo).mensaje}`);
    }
  };

  const restaurarProyectoDesdeBackup = async () => {
    const archivo = await elegirArchivoBackup();
    if (!archivo) {
      return;
    }
    const destino = await elegirCarpetaRestauracion();
    if (!destino) {
      return;
    }
    try {
      await restaurarBackupApi(archivo, destino);
      setMensaje(`Backup restaurado en ${destino}.`);
    } catch (fallo) {
      setMensaje(`No se pudo restaurar: ${errorDe(fallo).mensaje}`);
    }
  };

  const duplicarActivo = async () => {
    const documento = activoRef.current;
    if (documento) {
      await duplicarPorId(documento.id);
    }
  };

  const copiarActivo = () => {
    const documento = activoRef.current;
    if (documento) {
      setDocumentoCopiado(documento.id);
      setMensaje(`Documento copiado: ${documento.titulo}.`);
    }
  };

  const pegarDocumento = async () => {
    if (documentoCopiado) {
      await duplicarPorId(documentoCopiado);
    }
  };

  const abrirResultado = async (resultado: BusquedaDto) => {
    const documento =
      documentos.find((candidato) => candidato.id === resultado.id) ??
      ({
        tipo: resultado.tipo,
        id: resultado.id,
        ruta: resultado.ruta,
        titulo: resultado.titulo,
        cuerpo: '',
        hashArchivo: '',
        orden: null,
        estado: null,
        archivo: null,
        etiquetas: [],
        rol: null,
        cast: [],
        alias: [],
        clase: null,
      } satisfies DocumentoDto);
    await seleccionar(documento);
  };

  const verRevision = async (revision: number) => {
    if (revirtiendoRef.current) {
      return;
    }
    const proyectoActual = proyectoRef.current;
    if (!proyectoActual) {
      return;
    }
    const raiz = proyectoActual.raiz;
    const secuencia = detalleSeq.current + 1;
    detalleSeq.current = secuencia;
    setRevisionSeleccionada(revision);
    setDetalleRevision(null);
    setErrorDetalle(null);
    setConfirmarReversion(null);
    setComparacion(null);
    setErrorComparacion(null);
    setCargandoDetalle(true);
    try {
      const detalle = await pedirRevision(raiz, revision);
      if (detalleSeq.current !== secuencia || proyectoRef.current?.raiz !== raiz) {
        return;
      }
      if (detalle.revision !== revision) {
        setErrorDetalle('La respuesta del historial no corresponde a la revisión solicitada.');
        return;
      }
      setDetalleRevision(detalle);
    } catch (fallo) {
      if (detalleSeq.current === secuencia && proyectoRef.current?.raiz === raiz) {
        setErrorDetalle(errorDe(fallo).mensaje);
      }
    } finally {
      if (detalleSeq.current === secuencia && proyectoRef.current?.raiz === raiz) {
        setCargandoDetalle(false);
      }
    }
  };

  useEffect(() => {
    if (vista !== 'historial' || !proyecto || revisionSeleccionada !== null) {
      return;
    }
    void verRevision(proyecto.revision);
  }, [vista, proyecto?.revision, revisionSeleccionada]);

  const revertirARevision = async () => {
    const detalle = detalleRevision;
    const proyectoActual = proyectoRef.current;
    if (!detalle || !proyectoActual || revirtiendoRef.current) {
      return;
    }
    if (confirmarReversion === null) {
      setConfirmarReversion({
        raiz: proyectoActual.raiz,
        revision: detalle.revision,
        opId: detalle.opId,
      });
      return;
    }
    if (
      confirmarReversion.raiz !== proyectoActual.raiz ||
      confirmarReversion.revision !== detalle.revision ||
      confirmarReversion.opId !== detalle.opId
    ) {
      setConfirmarReversion(null);
      setErrorDetalle('La revisión seleccionada cambió. Vuelve a elegirla.');
      return;
    }

    const confirmacion = confirmarReversion;
    revirtiendoRef.current = true;
    setRevirtiendo(true);
    setErrorDetalle(null);
    try {
      const guardado = await guardarAhora();
      if (!guardado) {
        setConfirmarReversion(null);
        setErrorDetalle('No se puede revertir mientras haya cambios sin guardar.');
        return;
      }
      if (proyectoRef.current?.raiz !== confirmacion.raiz) {
        setConfirmarReversion(null);
        setErrorDetalle('El proyecto cambió antes de completar la reversión.');
        return;
      }
      const resultado = await revertirRevision(confirmacion.raiz, confirmacion.revision);
      const activoId = activoRef.current?.id ?? null;
      detalleSeq.current += 1;
      setRevisionSeleccionada(null);
      setDetalleRevision(null);
      setConfirmarReversion(null);
      await cargar(confirmacion.raiz, activoId);
      setMensaje(
        esTauri
          ? `Revisión ${confirmacion.revision} revertida en la revisión ${resultado.revision}.`
          : `Demo: reversión simulada de la revisión ${confirmacion.revision}; no se modificó ningún archivo.`,
      );
      if (resultado.erroresPendientes.length > 0) {
        setError(
          `Quedan ${resultado.erroresPendientes.length} error(es) después de la reversión.`,
        );
      }
    } catch (fallo) {
      setErrorDetalle(errorDe(fallo).mensaje);
      setConfirmarReversion(null);
    } finally {
      revirtiendoRef.current = false;
      setRevirtiendo(false);
    }
  };

  const recargarActivo = async () => {
    if (revirtiendoRef.current) {
      return;
    }
    const proyectoActual = proyectoRef.current;
    const actual = activoRef.current;
    if (!proyectoActual || !actual) {
      return;
    }
    try {
      const leido = await leer(proyectoActual.raiz, actual.id);
      limpiarPendiente();
      fijarActivo(leido);
      cuerpoEditorRef.current = leido.cuerpo;
      setDocumentos((lista) =>
        lista.map((documento) => (documento.id === leido.id ? leido : documento)),
      );
      setRevisionEditor((valor) => valor + 1);
      setErrorGuardado(null);
      setConfirmarRecarga(false);
      setEstadoGuardado(proyectoActual.errores.length > 0 ? 'solo-lectura' : 'guardado');
      setMensaje('Documento recargado desde disco; se descartaron los cambios locales.');
    } catch (fallo) {
      const detalle = errorDe(fallo);
      setErrorGuardado(detalle.mensaje);
      setEstadoGuardado(estadoDeError(detalle.tipo));
    }
  };

  const leerCambioDePanel = async (id: string) => {
    const proyectoActual = proyectoRef.current;
    if (!proyectoActual) {
      return;
    }
    setCargandoCambios(true);
    setErrorCambios(null);
    try {
      const detalle = await leerCambio(proyectoActual.raiz, id);
      setCambioActivo(detalle);
    } catch (fallo) {
      setErrorCambios(errorDe(fallo).mensaje);
      setCambioActivo(null);
    } finally {
      setCargandoCambios(false);
    }
  };

  const abrirDocumentoDeCambio = (id: string) => {
    const documento = documentos.find((candidato) => candidato.id === id);
    if (documento) {
      void seleccionar(documento);
    }
  };

  const aceptarCambioActivo = async () => {
    const actual = proyectoRef.current;
    if (!actual || procesandoCambio) {
      return;
    }
    setProcesandoCambio(true);
    setErrorCambios(null);
    try {
      const id = cambioActivoRef.current?.id;
      const teniaPendiente = pendienteRef.current !== null;
      if (teniaPendiente) {
        const guardadoPendiente = await guardarAhora();
        if (!guardadoPendiente) {
          return;
        }
      }
      let cambio = cambioActivoRef.current;
      if (teniaPendiente && id) {
        try {
          cambio = await leerCambio(actual.raiz, id);
        } catch {
          setCambioActivo(null);
          setEstadoGuardado('guardado');
          return;
        }
      }
      if (!cambio) {
        return;
      }
      const guardado = await aplicarCambio(actual.raiz, cambio.id);
      const cuerpo = cambio.propuesta.replace(/\n$/, '');
      fijarProyecto({ ...actual, revision: guardado.revision });
      setDocumentos((lista) =>
        lista.map((documento) =>
          documento.id === cambio.documento
            ? { ...documento, cuerpo, hashArchivo: guardado.hashArchivo }
            : documento,
        ),
      );
      const documentoActivo = activoRef.current;
      if (documentoActivo?.id === cambio.documento) {
        fijarActivo({
          ...documentoActivo,
          cuerpo,
          hashArchivo: guardado.hashArchivo,
        });
        cuerpoEditorRef.current = cuerpo;
        setRevisionEditor((valor) => valor + 1);
      }
      limpiarPendienteDe(cambio.documento);
      setCambioActivo(null);
      setCambiosProyecto((lista) =>
        lista.filter((item) => item.id !== cambio.id),
      );
      setEstadoGuardado('guardado');
      setErrorGuardado(null);
      setDiffAbierto(false);
      enfocarTrasCambio();
      setMensaje(`Cambio aceptado en la revisión ${guardado.revision}.`);
    } catch (fallo) {
      setErrorCambios(errorDe(fallo).mensaje);
    } finally {
      setProcesandoCambio(false);
    }
  };

  const descartarCambioActivo = async () => {
    const actual = proyectoRef.current;
    const cambio = cambioActivoRef.current;
    if (!actual || !cambio || procesandoCambio) {
      return;
    }
    setProcesandoCambio(true);
    setErrorCambios(null);
    try {
      await descartarCambio(actual.raiz, cambio.id);
      setCambiosProyecto((lista) => lista.filter((item) => item.id !== cambio.id));
      setCambioActivo(null);
      limpiarPendienteDe(cambio.documento);
      const documentoActivo = activoRef.current;
      if (documentoActivo?.id === cambio.documento) {
        cuerpoEditorRef.current = documentoActivo.cuerpo;
        setRevisionEditor((valor) => valor + 1);
      }
      setEstadoGuardado('guardado');
      setDiffAbierto(false);
      enfocarTrasCambio();
      setMensaje('Cambio rechazado; el documento conserva su versión anterior.');
    } catch (fallo) {
      setErrorCambios(errorDe(fallo).mensaje);
    } finally {
      setProcesandoCambio(false);
    }
  };

  const compararConActual = async (rutaDocumento: string, revision: number) => {
    const actual = proyectoRef.current;
    if (!actual || comparando) {
      return;
    }
    setComparando(true);
    setComparacion(null);
    setErrorComparacion(null);
    try {
      const resultado = await compararDocumento(actual.raiz, rutaDocumento, revision);
      setComparacion(resultado);
    } catch (fallo) {
      setErrorComparacion(errorDe(fallo).mensaje);
    } finally {
      setComparando(false);
    }
  };

  const copiarLocal = async () => {
    const texto = pendienteRef.current?.cuerpo ?? activoRef.current?.cuerpo ?? '';
    try {
      await navigator.clipboard.writeText(texto);
      setMensaje('Texto copiado al portapapeles.');
    } catch {
      setMensaje('No se pudo copiar. Abre la pestaña Markdown y selecciona el texto.');
    }
  };

  const abrir = async () => {
    if (revirtiendoRef.current) {
      setMensaje('Espera a que termine la reversión antes de abrir otro proyecto.');
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se abrió otro proyecto: hay cambios sin guardar.');
      return;
    }
    try {
      const ruta = await elegirProyecto();
      if (ruta === null) {
        return;
      }
      await cargar(ruta.trim() === '' ? '(demo)' : ruta);
    } catch (fallo) {
      setError(errorDe(fallo).mensaje);
    }
  };

  const crearProyectoDesdeInicio = async (
    datos: DatosNuevoProyecto,
    abrir: boolean,
  ) => {
    const resumen = datos.plantilla
      ? await crearDesdePlantillaApi(
          datos.plantilla,
          datos.ruta,
          datos.titulo,
          datos.autor,
          datos.idioma,
        )
      : await crearProyectoApi(
          datos.ruta,
          datos.titulo,
          datos.autor,
          datos.idioma,
          datos.notasIniciales,
        );
    setError(null);
    recordarProyecto(resumen, 0, datos.color);
    if (abrir) {
      await cargar(resumen.raiz);
      setMensaje(`Proyecto «${resumen.titulo}» creado y abierto.`);
    }
  };

  const volverAlInicio = async () => {
    if (revirtiendoRef.current) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se vuelve al panel: hay cambios sin guardar.');
      return;
    }
    limpiarPendiente();
    fijarProyecto(null);
    setDocumentos([]);
    fijarActivo(null);
    setEstado('inicio');
    setVista('editor');
    setConsulta('');
    setResultados([]);
    setRevisiones([]);
    setDetalleRevision(null);
    setModoSugerencias(false);
    modoSugerenciasRef.current = false;
    setCambiosProyecto([]);
    setCambioActivo(null);
    setMensaje(null);
  };

  const guardarFormato = async () => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    try {
      await guardarFormatoPdf(actual.raiz, formatoPdf);
      setMensaje('Formato de PDF guardado.');
    } catch (fallo) {
      setMensaje(`No se pudo guardar el formato: ${errorDe(fallo).mensaje}`);
    }
  };

  const exportarPdfActivo = async () => {
    const actual = proyectoRef.current;
    if (!actual) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se exporta mientras haya cambios sin guardar.');
      return;
    }
    const destino = await elegirDestinoPdf(actual.titulo);
    if (!destino) {
      return;
    }
    try {
      const ruta = await exportarPdf(actual.raiz, destino, true);
      setMensaje(`PDF exportado: ${ruta}.`);
    } catch (fallo) {
      setMensaje(`No se pudo exportar el PDF: ${errorDe(fallo).mensaje}`);
    }
  };

  const exportarDocumento = async () => {
    if (exportando || revirtiendoRef.current) {
      return;
    }
    const proyectoActual = proyectoRef.current;
    if (!proyectoActual) {
      return;
    }
    const guardado = await guardarAhora();
    if (!guardado) {
      setMensaje('No se exporta mientras haya cambios sin guardar.');
      return;
    }
    setExportando(true);
    setError(null);
    try {
      const destino = await elegirDestinoDocx(proyectoActual.titulo);
      if (destino === null) {
        return;
      }
      const exportado = await exportarDocx(
        proyectoActual.raiz,
        destino,
        incluirNotas,
        true,
      );
      setMensaje(
        esTauri
          ? `DOCX exportado en ${exportado}.`
          : `Demo: exportación simulada a ${exportado}; no se escribió ningún archivo.`,
      );
    } catch (fallo) {
      setError(errorDe(fallo).mensaje);
    } finally {
      setExportando(false);
    }
  };

  const descartarCopia = async (archivo: string) => {
    const proyectoActual = proyectoRef.current;
    if (!proyectoActual || descartando !== null) {
      return;
    }
    setDescartando(archivo);
    setError(null);
    try {
      await descartarCuarentena(proyectoActual.raiz, archivo);
      setCuarentena((items) => items.filter((item) => item.archivo !== archivo));
      setMensaje(`Copia de cuarentena descartada: ${archivo}.`);
    } catch (fallo) {
      setError(errorDe(fallo).mensaje);
    } finally {
      setDescartando(null);
      setConfirmarDescarte(null);
    }
  };

  accionesRef.current = { guardar: guardarAhora, abrir };

  useEffect(() => {
    if (!esTauri) {
      return;
    }
    let cancelado = false;
    void proyectoE2e()
      .then((ruta) => {
        if (!cancelado && ruta) {
          void cargar(ruta);
        }
      })
      .catch(() => {});
    return () => {
      cancelado = true;
    };
  }, []);

  useEffect(() => {
    const termino = consulta.trim();
    if (!proyecto || termino.length < 2) {
      busquedaSeq.current += 1;
      setResultados([]);
      setErrorBusqueda(null);
      setBuscando(false);
      return;
    }
    const secuencia = busquedaSeq.current + 1;
    busquedaSeq.current = secuencia;
    setResultados([]);
    setErrorBusqueda(null);
    setBuscando(true);
    const temporizador = window.setTimeout(() => {
      void buscarEnProyecto(proyecto.raiz, termino, 30)
        .then((lista) => {
          if (busquedaSeq.current === secuencia) {
            setResultados(lista);
          }
        })
        .catch((fallo) => {
          if (busquedaSeq.current === secuencia) {
            setResultados([]);
            setErrorBusqueda(errorDe(fallo).mensaje);
          }
        })
        .finally(() => {
          if (busquedaSeq.current === secuencia) {
            setBuscando(false);
          }
        });
    }, 300);
    return () => window.clearTimeout(temporizador);
  }, [consulta, proyecto?.raiz, proyecto?.revision]);

  useEffect(() => {
    if (vista !== 'historial' || !proyecto) {
      return;
    }
    let cancelado = false;
    setCargandoHistorial(true);
    setErrorHistorial(null);
    void pedirHistorial(proyecto.raiz)
      .then((lista) => {
        if (!cancelado) {
          setRevisiones([...lista].reverse());
        }
      })
      .catch((fallo) => {
        if (!cancelado) {
          setRevisiones([]);
          setErrorHistorial(errorDe(fallo).mensaje);
        }
      })
      .finally(() => {
        if (!cancelado) {
          setCargandoHistorial(false);
        }
      });
    return () => {
      cancelado = true;
    };
  }, [vista, proyecto?.raiz, proyecto?.revision, historialVersion]);

  useEffect(() => {
    const alPulsar = (evento: KeyboardEvent) => {
      if (hayDialogoAbierto() || !(evento.ctrlKey || evento.metaKey)) {
        return;
      }
      if (evento.key.toLowerCase() === 's') {
        evento.preventDefault();
        void accionesRef.current?.guardar();
      } else if (evento.key.toLowerCase() === 'o') {
        evento.preventDefault();
        void accionesRef.current?.abrir();
      } else if (evento.key.toLowerCase() === 'f' && proyectoRef.current) {
        evento.preventDefault();
        buscadorRef.current?.focus();
        buscadorRef.current?.select();
      } else if (evento.shiftKey && evento.key.toLowerCase() === 'h') {
        evento.preventDefault();
        setRevisionSeleccionada(null);
        setDetalleRevision(null);
        setVista('historial');
      } else if (evento.altKey && evento.key.toLowerCase() === 'e') {
        evento.preventDefault();
        setVista('editor');
        window.setTimeout(() => {
          (document.querySelector('.ProseMirror') as HTMLElement | null)?.focus();
        }, 0);
      }
    };
    window.addEventListener('keydown', alPulsar);
    return () => window.removeEventListener('keydown', alPulsar);
  }, []);

  useEffect(() => {
    const alPulsar = (evento: KeyboardEvent) => {
      if (hayDialogoAbierto()) {
        return;
      }
      if (
        evento.altKey &&
        !evento.ctrlKey &&
        !evento.metaKey &&
        evento.key.toLowerCase() === 'e'
      ) {
        evento.preventDefault();
        setVista('editor');
        window.setTimeout(() => {
          (document.querySelector('.ProseMirror') as HTMLElement | null)?.focus();
        }, 0);
        return;
      }
      if ((evento.ctrlKey || evento.metaKey) && evento.key.toLowerCase() === 'p') {
        evento.preventDefault();
        setPaleta(evento.shiftKey ? 'comandos' : 'documentos');
        return;
      }
      if (evento.altKey && evento.key.toLowerCase() === 'z') {
        evento.preventDefault();
        setPreferencias((valor) => ({ ...valor, modoEnfoque: !valor.modoEnfoque }));
        return;
      }
      if (evento.key === 'Escape' && panelAbierto && esVistaEstrecha()) {
        cerrarPanel();
        return;
      }
      if (
        evento.key === 'Escape' &&
        paleta === null &&
        !preferenciasAbiertas &&
        !diffAbierto &&
        preferencias.modoEnfoque
      ) {
        setPreferencias((valor) => ({ ...valor, modoEnfoque: false }));
      }
    };
    window.addEventListener('keydown', alPulsar);
    return () => window.removeEventListener('keydown', alPulsar);
  }, [paleta, preferenciasAbiertas, preferencias.modoEnfoque, panelAbierto, diffAbierto]);

  useEffect(() => {
    const alSalir = (evento: BeforeUnloadEvent) => {
      if (pendienteRef.current !== null || estadoGuardado === 'guardando') {
        evento.preventDefault();
        evento.returnValue = '';
      }
    };
    window.addEventListener('beforeunload', alSalir);
    return () => window.removeEventListener('beforeunload', alSalir);
  }, [estadoGuardado]);

  useEffect(() => {
    if (!esTauri) {
      return;
    }
    let cancelado = false;
    let desregistrar: (() => void) | undefined;
    let cerrando = false;
    void import('@tauri-apps/api/window')
      .then(async ({ getCurrentWindow }) => {
        if (cancelado) {
          return;
        }
        const ventana = getCurrentWindow();
        desregistrar = await ventana.onCloseRequested(async (evento) => {
          // Tauri no ejecuta `beforeunload`; hay que interceptar el cierre y
          // vaciar el autosave antes de destruir la ventana.
          evento.preventDefault();
          if (cerrando) {
            return;
          }
          if (revirtiendoRef.current) {
            setMensaje('Espera a que termine la reversión antes de cerrar la ventana.');
            return;
          }
          const guardado = (await accionesRef.current?.guardar()) ?? true;
          if (!guardado) {
            setMensaje(
              'No se cerró la ventana: hay cambios sin guardar. Resuelve el error y vuelve a intentarlo.',
            );
            return;
          }
          cerrando = true;
          try {
            await ventana.destroy();
          } catch (fallo) {
            cerrando = false;
            setMensaje(
              `No se pudo cerrar la ventana: ${errorDe(fallo).mensaje}. Vuelve a intentarlo.`,
            );
          }
        });
      })
      .catch(() => {
        // En un navegador normal el efecto no se ejecuta; si Tauri no expone la
        // ventana, `beforeunload` sigue siendo la última barrera.
      });
    return () => {
      cancelado = true;
      desregistrar?.();
    };
  }, []);

  useEffect(() => {
    setTituloEdicion(activo?.titulo ?? '');
    setEstadoEdicion(activo?.estado ?? 'borrador');
    setEtiquetasEdicion((activo?.etiquetas ?? []).join(', '));
    setRolPersonaje(activo?.rol ?? '');
    setAliasPersonaje((activo?.alias ?? []).join(', '));
  }, [
    activo?.id,
    activo?.titulo,
    activo?.estado,
    activo?.etiquetas,
    activo?.rol,
    activo?.alias,
  ]);

  useEffect(() => {
    const actual = proyectoRef.current;
    if (!actual) {
      setComentariosProyecto([]);
      return;
    }
    let cancelado = false;
    listarComentarios(actual.raiz)
      .then((lista) => {
        if (!cancelado) {
          setComentariosProyecto(lista);
        }
      })
      .catch(() => {
        if (!cancelado) {
          setComentariosProyecto([]);
        }
      });
    return () => {
      cancelado = true;
    };
  }, [proyecto?.raiz]);

  useEffect(() => {
    const actual = proyectoRef.current;
    const documento = activoRef.current;
    if (!actual || !documento) {
      setCambioActivo(null);
      return;
    }
    const resumen = cambiosProyecto.find(
      (cambio) => cambio.documento === documento.id,
    );
    if (!resumen) {
      setCambioActivo(null);
      setEstadoGuardado((valor) => (valor === 'sugerencia' ? 'guardado' : valor));
      return;
    }
    if (cambioActivoRef.current?.id === resumen.id) {
      setEstadoGuardado((valor) => (valor === 'guardado' ? 'sugerencia' : valor));
      return;
    }
    let cancelado = false;
    void leerCambio(actual.raiz, resumen.id)
      .then((detalle) => {
        if (!cancelado) {
          setCambioActivo(detalle);
          setEstadoGuardado((valor) =>
            valor === 'guardado' ? 'sugerencia' : valor,
          );
          setErrorCambios(null);
        }
      })
      .catch((fallo) => {
        if (!cancelado) {
          setCambioActivo(null);
          setErrorCambios(errorDe(fallo).mensaje);
        }
      });
    return () => {
      cancelado = true;
    };
  }, [activo?.id, proyecto?.raiz, cambiosProyecto]);

  useEffect(() => {
    const actual = proyectoRef.current;
    if (!actual || !activo) {
      setComentarios([]);
      return;
    }
    let cancelado = false;
    sincronizarComentariosHuerfanos(actual.raiz, activo.ruta, activo.cuerpo)
      .then((lista) => {
        if (!cancelado) {
          setComentarios(lista);
          setComentariosProyecto((todas) => [
            ...todas.filter((comentario) => comentario.documento !== activo.ruta),
            ...lista,
          ]);
          setErrorComentarios(null);
        }
      })
      .catch((fallo) => {
        if (!cancelado) {
          setComentarios([]);
          setErrorComentarios(errorDe(fallo).mensaje);
        }
      });
    return () => {
      cancelado = true;
    };
  }, [activo?.id, activo?.ruta, proyecto?.raiz]);

  useEffect(() => {
    if (!temporizadorActivo || temporizadorPausado) {
      return;
    }
    const intervalo = window.setInterval(() => {
      setRestante((valor) => Math.max(0, valor - 1));
    }, 1000);
    return () => window.clearInterval(intervalo);
  }, [temporizadorActivo, temporizadorPausado]);

  useEffect(() => {
    if (temporizadorActivo && restante === 0 && !temporizadorPausado) {
      setTemporizadorPausado(true);
      pausaInicioRef.current = new Date().toISOString();
    }
  }, [restante, temporizadorActivo, temporizadorPausado]);

  useEffect(() => {
    if (errorGuardado) {
      // El aviso está en el flujo normal para no tapar el documento con zoom
      // alto. Al aparecer, el panel vuelve arriba y el foco va al aviso.
      if (panelRef.current) {
        panelRef.current.scrollTop = 0;
      }
      bannerErrorRef.current?.focus({ preventScroll: true });
    }
  }, [errorGuardado]);

  const propuestaVisible =
    activo && cambioActivo?.documento === activo.id ? cambioActivo.propuesta : null;
  const cuerpoVisible =
    activo && pendienteRef.current?.id === activo.id
      ? pendienteRef.current.cuerpo
      : (propuestaVisible ?? activo?.cuerpo ?? '');
  const palabrasDocumento = contarPalabras(textoSinMarcas(cuerpoVisible));
  const comentariosVisibles = comentarios.filter((comentario) => {
    if (filtroComentarios === 'abiertos') {
      return comentario.estado === 'abierto' && !comentario.huerfano;
    }
    if (filtroComentarios === 'resueltos') {
      return comentario.estado === 'resuelto';
    }
    if (filtroComentarios === 'huerfanos') {
      return comentario.huerfano;
    }
    return true;
  });
  const palabrasHoy = sesiones
    .filter((sesion) => sesion.inicio.startsWith(fechaHoy()))
    .reduce((total, sesion) => total + sesion.palabras, 0);
  const ultimosDias = useMemo(() => {
    const dias: Array<{ fecha: string; palabras: number }> = [];
    for (let atras = 29; atras >= 0; atras -= 1) {
      const fecha = new Date();
      fecha.setDate(fecha.getDate() - atras);
      const clave = fecha.toISOString().slice(0, 10);
      dias.push({
        fecha: clave,
        palabras: sesiones
          .filter((sesion) => sesion.inicio.startsWith(clave))
          .reduce((total, sesion) => total + sesion.palabras, 0),
      });
    }
    return dias;
  }, [sesiones, metas?.diaria?.objetivo]);
  const maximoDia = Math.max(1, ...ultimosDias.map((dia) => dia.palabras));
  const racha = useMemo(() => {
    let total = 0;
    for (let atras = 0; atras < 365; atras += 1) {
      const fecha = new Date();
      fecha.setDate(fecha.getDate() - atras);
      const clave = fecha.toISOString().slice(0, 10);
      const palabras = sesiones
        .filter((sesion) => sesion.inicio.startsWith(clave))
        .reduce((suma, sesion) => suma + sesion.palabras, 0);
      const objetivo = metas?.diaria?.objetivo ?? 0;
      const cumple = objetivo > 0 ? palabras >= objetivo : palabras > 0;
      if (cumple) {
        total += 1;
      } else {
        break;
      }
    }
    return total;
  }, [sesiones]);
  const modoBusqueda = consulta.trim().length >= 2;
  const lecturaDisponible = soportaLectura();
  const textoVisible = textoSinMarcas(cuerpoVisible);
  const caracteresDocumento = textoParaLectura(cuerpoVisible).length;
  const parrafosDocumento = textoVisible
    .split(/\n\s*\n/)
    .filter((parrafo) => parrafo.trim().length > 0).length;
  const minutosLectura = Math.ceil(palabrasDocumento / 200);
  const palabrasSeleccion = contarPalabras(textoSinMarcas(seleccionTexto));

  const iniciarLectura = (texto: string) => {
    const limpio = textoParaLectura(texto);
    if (!limpio) {
      setErrorLectura('No hay texto para leer.');
      return;
    }
    const secuencia = lecturaSeq.current + 1;
    lecturaSeq.current = secuencia;
    const esActual = () => lecturaSeq.current === secuencia;
    const iniciada = hablar(limpio, {
      voz: vozLectura || undefined,
      velocidad: velocidadLectura,
      onFin: () => {
        if (esActual()) {
          setLeyendo(false);
          setPausadoLectura(false);
        }
      },
      onError: (mensaje) => {
        if (esActual()) {
          setErrorLectura(mensaje);
          setLeyendo(false);
          setPausadoLectura(false);
        }
      },
    });
    if (!iniciada) {
      setErrorLectura(
        'Este WebView no pudo iniciar la lectura con las voces del sistema.',
      );
      return;
    }
    setErrorLectura(null);
    setLeyendo(true);
    setPausadoLectura(false);
  };

  const leerDocumento = () => iniciarLectura(cuerpoVisible);

  const leerSeleccion = () => {
    const seleccion = window.getSelection()?.toString() ?? '';
    if (!seleccion.trim()) {
      setErrorLectura('Selecciona texto en el documento para leerlo.');
      return;
    }
    iniciarLectura(seleccion);
  };

  const pausarLecturaActual = () => {
    pausarLectura();
    setPausadoLectura(true);
  };

  const reanudarLecturaActual = () => {
    reanudarLectura();
    setPausadoLectura(false);
  };

  const detenerLecturaActual = () => {
    lecturaSeq.current += 1;
    detenerLectura();
    setLeyendo(false);
    setPausadoLectura(false);
    setErrorLectura(null);
  };

  const comandos: ComandoPaleta[] = [
    { id: 'guardar', etiqueta: 'Guardar', detalle: 'Escribe los cambios pendientes', atajo: 'Ctrl+S' },
    { id: 'buscar', etiqueta: 'Buscar en el proyecto', atajo: 'Ctrl+F' },
    { id: 'historial', etiqueta: 'Abrir historial', atajo: 'Ctrl+Shift+H' },
    { id: 'cambios', etiqueta: 'Revisar cambios propuestos' },
    { id: 'herramientas', etiqueta: 'Herramientas de escritura' },
    { id: 'leer-documento', etiqueta: 'Leer documento en voz alta' },
    { id: 'detener-lectura', etiqueta: 'Detener lectura' },
    { id: 'preferencias', etiqueta: 'Preferencias de escritura' },
    { id: 'enfoque', etiqueta: 'Alternar modo enfoque', atajo: 'Alt+Z' },
    { id: 'tema-claro', etiqueta: 'Tema claro' },
    { id: 'tema-sepia', etiqueta: 'Tema sepia' },
    { id: 'tema-oscuro', etiqueta: 'Tema oscuro' },
    { id: 'exportar-docx', etiqueta: 'Exportar DOCX' },
    { id: 'exportar-pdf', etiqueta: 'Exportar PDF' },
  ];

  const ejecutarComando = (id: string) => {
    setPaleta(null);
    switch (id) {
      case 'guardar':
        void guardarAhora();
        break;
      case 'buscar':
        buscadorRef.current?.focus();
        buscadorRef.current?.select();
        break;
      case 'historial':
        setRevisionSeleccionada(null);
        setDetalleRevision(null);
        setVista('historial');
        break;
      case 'cambios':
        setPanelDerecho('cambios');
        setPanelAbierto(true);
        break;
      case 'herramientas':
        setPanelDerecho('herramientas');
        setPanelAbierto(true);
        break;
      case 'leer-documento':
        leerDocumento();
        break;
      case 'detener-lectura':
        detenerLecturaActual();
        break;
      case 'preferencias':
        setPreferenciasAbiertas(true);
        break;
      case 'enfoque':
        setPreferencias((valor) => ({ ...valor, modoEnfoque: !valor.modoEnfoque }));
        break;
      case 'tema-claro':
      case 'tema-sepia':
      case 'tema-oscuro':
        setPreferencias((valor) => ({
          ...valor,
          tema: id.replace('tema-', '') as Preferencias['tema'],
        }));
        break;
      case 'exportar-docx':
        void exportarDocumento();
        break;
      case 'exportar-pdf':
        void exportarPdfActivo();
        break;
      default:
        break;
    }
  };

  const abrirDocumentoDesdePaleta = (documento: DocumentoDto) => {
    setPaleta(null);
    void seleccionar(documento).then(() => {
      window.setTimeout(() => {
        (document.querySelector('.ProseMirror') as HTMLElement | null)?.focus();
      }, 0);
    });
  };

  const botonDocumento = (documento: DocumentoDto, icono: string) => (
    <button
      key={documento.id}
      id={`doc-${documento.id}`}
      className={`dabble-doc ${activo?.id === documento.id ? 'activo' : ''}`}
      tabIndex={idArbolTabbable === documento.id ? 0 : -1}
      aria-current={activo?.id === documento.id ? 'page' : undefined}
      onFocus={() => setFocoArbol(documento.id)}
      onKeyDown={(evento) => manejarTeclaArbol(evento, documento)}
      onClick={() => void seleccionar(documento)}
      draggable={documento.tipo === 'escena'}
      onDragStart={(evento) => {
        if (documento.tipo !== 'escena') {
          return;
        }
        evento.dataTransfer.setData('text/plain', documento.id);
        evento.dataTransfer.effectAllowed = 'move';
      }}
      onDragEnd={() => setDestinoArrastre(null)}
    >
      <span className="dabble-doc-icono" aria-hidden="true">
        {icono}
      </span>
      <span className="dabble-doc-titulo">{documento.titulo}</span>
    </button>
  );

  if (estado === 'inicio' || estado === 'error') {
    return (
      <Inicio
        recientes={recientes}
        esTauri={esTauri}
        errorInicial={error}
        onAbrir={() => void abrir()}
        onDemo={() => void cargar('(demo)')}
        onAbrirReciente={(ruta) => void cargar(ruta)}
        onCrear={crearProyectoDesdeInicio}
      />
    );
  }

  if (estado === 'cargando' && !proyecto) {
    return (
      <div className="bienvenida">
        <div className="bienvenida-tarjeta">
          <p className="kicker">motor + CLI + interfaz</p>
          <h1>writer</h1>
          <p className="entrada">Abriendo el proyecto…</p>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`dabble-app ${preferencias.modoEnfoque ? 'modo-enfoque' : ''} ${
        panelAbierto ? 'panel-abierto' : ''
      }`}
    >
      <a className="saltar-contenido" href="#panel-principal">
        Saltar al contenido
      </a>
      <header className="dabble-topbar">
        <button
          className="dabble-home"
          aria-label="Volver a Proyectos"
          title="Volver a Proyectos"
          onClick={() => void volverAlInicio()}
        >
          ⌂
        </button>
        <div className="dabble-tabs">
          <button
            className={`dabble-tab ${vista === 'editor' ? 'activo' : ''}`}
            onClick={() => setVista('editor')}
          >
            <span className="dabble-tab-icono" aria-hidden="true">▤</span>
            <strong>{proyecto?.titulo ?? 'Proyecto'}</strong>
          </button>
          <button
            className="dabble-tab"
            onClick={() => {
              const nota = notas.find((documento) =>
                ['lugares', 'cronologia', 'objetos'].includes(documento.clase ?? ''),
              );
              if (nota) {
                void seleccionar(nota);
              } else {
                setPanelDerecho('notas');
              }
            }}
          >
            <span className="dabble-tab-icono" aria-hidden="true">◈</span>
            Construcción del mundo
          </button>
          <button
            className={`dabble-tab ${vista !== 'editor' ? 'activo' : ''}`}
            aria-label={`Editor: ${activo?.titulo ?? 'sin documento'}`}
            onClick={() => setVista('editor')}
            disabled={!activo}
          >
            {activo?.titulo ?? 'Sin documento'}
          </button>
        </div>
        <div className="dabble-top-acciones">
          <span
            className={`estado-guardado estado-${estadoVisible}`}
            role="status"
            aria-live="polite"
            aria-label="Estado de guardado"
            title={errorGuardado ?? undefined}
          >
            {etiquetaGuardado(estadoVisible)}
          </span>
          <label className="notas-exportar">
            <input
              type="checkbox"
              checked={incluirNotas}
              onChange={(evento) => setIncluirNotas(evento.target.checked)}
              disabled={exportando}
            />
            notas
          </label>
          <button
            className="dabble-compartir"
            aria-label="Exportar DOCX"
            onClick={() => void exportarDocumento()}
            disabled={exportando || revirtiendo}
          >
            {exportando ? 'Exportando…' : 'Compartir'}
          </button>
          <button
            className={`icono-dabble ${vista === 'fuente' ? 'activo' : ''}`}
            aria-label="Ver Markdown"
            title="Ver Markdown"
            onClick={() => setVista(vista === 'fuente' ? 'editor' : 'fuente')}
            disabled={!activo}
          >
            Aa
          </button>
          <button
            className={`icono-dabble ${vista === 'historial' ? 'activo' : ''}`}
            aria-label="Historial"
            title="Historial de versiones"
            onClick={() => {
              setRevisionSeleccionada(null);
              setDetalleRevision(null);
              setVista('historial');
            }}
          >
            ◷
          </button>
          <button
            className={`icono-dabble ${
              panelDerecho === 'cambios' ? 'activo' : ''
            } ${cambiosProyecto.length > 0 ? 'con-badge' : ''}`}
            aria-label={`Control de cambios${
              cambiosProyecto.length > 0
                ? `: ${cambiosProyecto.length} pendiente(s)`
                : ''
            }`}
            title="Control de cambios"
            onClick={() => {
              setPanelDerecho('cambios');
              setPanelAbierto(true);
            }}
          >
            ±
            {cambiosProyecto.length > 0 && (
              <span className="dabble-badge" aria-hidden="true">
                {cambiosProyecto.length}
              </span>
            )}
          </button>
          <button className="icono-dabble" aria-label="Ayuda" title="Ayuda">
            ?
          </button>
          <button
            className="dabble-panel-toggle"
            aria-label="Abrir panel lateral"
            aria-expanded={panelAbierto}
            title="Panel lateral"
            ref={panelToggleRef}
            onClick={() => setPanelAbierto((valor) => !valor)}
          >
            ▤
          </button>
          <button
            className="icono-dabble"
            aria-label="Preferencias"
            title="Preferencias de escritura"
            onClick={() => setPreferenciasAbiertas(true)}
          >
            ⚙
          </button>
          <span className="dabble-avatar" aria-label={`Sesión de ${proyecto?.autor || 'escritor'}`}>
            {(proyecto?.autor || 'E').charAt(0).toLocaleUpperCase()}
          </span>
        </div>
      </header>

      {estado === 'cargando' && <div className="cargando">Abriendo proyecto…</div>}

      <div className="dabble-workspace">
        <aside className="dabble-sidebar">
          <div className="dabble-filtro">
            <span aria-hidden="true">▽</span>
            <input
              ref={buscadorRef}
              type="text"
              value={consulta}
              onChange={(evento) => setConsulta(evento.target.value)}
              onKeyDown={(evento) => {
                if (evento.nativeEvent.isComposing) {
                  return;
                }
                if (evento.key === 'Escape') {
                  setConsulta('');
                } else if (evento.key === 'Enter' && resultados.length > 0) {
                  evento.preventDefault();
                  void abrirResultado(resultados[0]);
                }
              }}
              placeholder="Filtrar archivos del proyecto"
              aria-label="Buscar en el proyecto"
            />
            <button className="dabble-mas" aria-label="Crear archivo" title="Crear archivo próximamente" disabled>
              +
            </button>
          </div>
          <div className="dabble-filtro-etiqueta">
            <input
              value={filtroEtiqueta}
              onChange={(evento) => setFiltroEtiqueta(evento.target.value)}
              placeholder="Filtrar por etiqueta"
              aria-label="Filtrar por etiqueta"
            />
          </div>

          {modoBusqueda ? (
            <div className="resultados-busqueda" aria-live="polite">
              <div className="sidebar-titulo">
                <span>Resultados</span>
                <span className="contador">{resultados.length}</span>
              </div>
              {buscando && <p className="vacio">Buscando…</p>}
              {errorBusqueda && (
                <pre className="banner-error" role="alert">
                  {errorBusqueda}
                </pre>
              )}
              {!buscando && !errorBusqueda && resultados.length === 0 && (
                <p className="vacio">Sin resultados para «{consulta.trim()}».</p>
              )}
              {resultados.map((resultado) => (
                <button
                  key={resultado.id}
                  className={`resultado ${activo?.id === resultado.id ? 'activo' : ''}`}
                  onClick={() => void abrirResultado(resultado)}
                >
                  <span className="resultado-cabecera">
                    <span className="documento-titulo">{resultado.titulo}</span>
                    <span className="resultado-tipo">{resultado.tipo}</span>
                  </span>
                  <span className="resultado-ruta">{resultado.ruta}</span>
                  <span className="resultado-fragmento">
                    <Fragmento texto={resultado.fragmento} />
                  </span>
                </button>
              ))}
            </div>
          ) : (
            <div className="dabble-arbol">
              {filtroArchivo !== 'activo' && (
                <button
                  className="dabble-utilidad"
                  onClick={() => setFiltroArchivo('activo')}
                >
                  ← Volver a activos
                </button>
              )}
              <h3 className="dabble-seccion">Manuscritos</h3>
              {libros.length === 0 && <p className="dabble-vacio">Sin escenas.</p>}
              {libros.map((libro) => (
                <div className="dabble-libro" key={libro.etiqueta}>
                  <div className="dabble-libro-titulo">{libro.etiqueta}</div>
                  {libro.capitulos.map((capitulo) => (
                    <div
                      className={`dabble-capitulo ${
                        destinoArrastre === capitulo.clave ? 'arrastre-activo' : ''
                      }`}
                      key={capitulo.clave}
                      onDragOver={(evento) => {
                        evento.preventDefault();
                        evento.dataTransfer.dropEffect = 'move';
                        setDestinoArrastre(capitulo.clave);
                      }}
                      onDragLeave={() => setDestinoArrastre(null)}
                      onDrop={(evento) => {
                        evento.preventDefault();
                        const id = evento.dataTransfer.getData('text/plain');
                        const [libro, capituloDestino] = capitulo.clave.split('/');
                        if (id && libro && capituloDestino) {
                          void moverDocumento(id, libro, capituloDestino);
                        }
                      }}
                    >
                      <div className="dabble-capitulo-titulo">{capitulo.etiqueta}</div>
                      {capitulo.documentos.map((documento) => botonDocumento(documento, '▤'))}
                    </div>
                  ))}
                </div>
              ))}

              <h3 className="dabble-seccion">Tramas</h3>
              {notasVisibles.filter((documento) => documento.clase === 'tramas').length === 0 && (
                <p className="dabble-vacio">Sin tramas.</p>
              )}
              {notasVisibles
                .filter((documento) => documento.clase === 'tramas')
                .map((documento) => botonDocumento(documento, '◈'))}

              <h3 className="dabble-seccion">Personajes</h3>
              {notasVisibles.filter((documento) => documento.clase === 'personajes').length === 0 && (
                <p className="dabble-vacio">Sin personajes.</p>
              )}
              {notasVisibles
                .filter((documento) => documento.clase === 'personajes')
                .map((documento) => botonDocumento(documento, '♙'))}

              <h3 className="dabble-seccion">Cuaderno</h3>
              {notasVisibles.filter(
                (documento) =>
                  !['tramas', 'personajes', 'ideas'].includes(documento.clase ?? ''),
              ).length === 0 && <p className="dabble-vacio">Sin notas de cuaderno.</p>}
              {notasVisibles
                .filter(
                  (documento) =>
                    !['tramas', 'personajes', 'ideas'].includes(documento.clase ?? ''),
                )
                .map((documento) => botonDocumento(documento, '▢'))}

              <h3 className="dabble-seccion">Ideas</h3>
              {notasVisibles.filter((documento) => documento.clase === 'ideas').length === 0 && (
                <p className="dabble-vacio">Sin ideas.</p>
              )}
              {notasVisibles
                .filter((documento) => documento.clase === 'ideas')
                .map((documento) => botonDocumento(documento, '✦'))}

              <h3 className="dabble-seccion">Utilidades</h3>
              <button className="dabble-utilidad" disabled>▢ Plantillas</button>
              <button
                className={`dabble-utilidad ${
                  filtroArchivo === 'archivado' ? 'activo' : ''
                }`}
                onClick={() => setFiltroArchivo('archivado')}
              >
                ⌂ Archivo
              </button>
              <button
                className={`dabble-utilidad ${
                  filtroArchivo === 'papelera' ? 'activo' : ''
                }`}
                onClick={() => setFiltroArchivo('papelera')}
              >
                ♲ Papelera
              </button>
              <button
                className="dabble-config"
                onClick={() => setPreferenciasAbiertas(true)}
                title="Preferencias de escritura"
              >
                ⚙ Preferencias de escritura
              </button>
            </div>
          )}
        </aside>

        <main className="dabble-centro" id="panel-principal" tabIndex={-1} ref={panelRef}>
          {error && (
            <pre className="banner-error" role="alert">
              {error}
            </pre>
          )}
          {mensaje && (
            <div className="banner-info" role="status" aria-live="polite">
              <span>{mensaje}</span>
              <button className="secundario pequeno" onClick={() => setMensaje(null)}>
                Cerrar
              </button>
            </div>
          )}
          {revirtiendo && (
            <div className="banner-info">
              <span>Revirtiendo la revisión… el editor queda bloqueado hasta terminar.</span>
            </div>
          )}
          {errorGuardado && (
            <div className="banner-error" ref={bannerErrorRef} role="alert" tabIndex={-1}>
              <strong>No se pudo guardar.</strong>
              <pre>{errorGuardado}</pre>
              <div className="banner-acciones">
                <button className="secundario pequeno" onClick={() => void guardarAhora()}>
                  Reintentar
                </button>
                <button
                  className="secundario pequeno"
                  onClick={() => {
                    if (!confirmarRecarga) {
                      setConfirmarRecarga(true);
                      return;
                    }
                    setConfirmarRecarga(false);
                    void recargarActivo();
                  }}
                >
                  {confirmarRecarga ? 'Confirmar y descartar' : 'Recargar desde disco'}
                </button>
                <button className="secundario pequeno" onClick={() => void copiarLocal()}>
                  Copiar mi texto
                </button>
              </div>
            </div>
          )}
          {bloqueoDocumento && (
            <div className="diagnostico">
              <strong>Este documento usa Markdown que el editor no conserva.</strong>
              <ul>
                {hallazgos.map((hallazgo) => (
                  <li key={hallazgo.construccion}>
                    <b>{hallazgo.construccion}:</b> {hallazgo.detalle}
                  </li>
                ))}
              </ul>
              <p>Se abrió en solo lectura para no perder contenido.</p>
            </div>
          )}
          {proyecto && proyecto.errores.length > 0 && (
            <details className="diagnostico" open>
              <summary>
                {proyecto.errores.length} error(es) de verificación · el proyecto no es escribible
              </summary>
              <ul>
                {proyecto.errores.map((detalle) => (
                  <li key={detalle}>{detalle}</li>
                ))}
              </ul>
            </details>
          )}
          {proyecto && proyecto.avisos.length > 0 && (
            <details className="diagnostico avisos">
              <summary>{proyecto.avisos.length} aviso(s) de verificación</summary>
              <ul>
                {proyecto.avisos.map((detalle) => (
                  <li key={detalle}>{detalle}</li>
                ))}
              </ul>
            </details>
          )}
          {cuarentena.length > 0 && (
            <details className="cuarentena">
              <summary className="cuarentena-cabecera">
                <div>
                  <p className="kicker">copia de diagnóstico</p>
                  <h3>Cuarentena ({cuarentena.length})</h3>
                </div>
                <p>Los originales no se tocan. Descartar elimina solo la copia.</p>
              </summary>
              <ul>
                {cuarentena.map((item) => (
                  <li key={item.archivo}>
                    <div className="cuarentena-datos">
                      <strong>{item.archivo}</strong>
                      <span>
                        {item.motivo ?? 'sin motivo'} ·{' '}
                        {item.rutaOriginal ?? 'ruta original desconocida'} ·{' '}
                        {formatearFecha(item.ts)} · {item.tamano} bytes
                      </span>
                    </div>
                    <button
                      className="peligro pequeno"
                      disabled={descartando !== null}
                      onClick={() => {
                        if (confirmarDescarte !== item.archivo) {
                          setConfirmarDescarte(item.archivo);
                          return;
                        }
                        void descartarCopia(item.archivo);
                      }}
                    >
                      {descartando === item.archivo
                        ? 'Descartando…'
                        : confirmarDescarte === item.archivo
                          ? 'Confirmar descarte'
                          : 'Descartar'}
                    </button>
                  </li>
                ))}
              </ul>
            </details>
          )}

          {vista === 'historial' ? (
            <section className="dabble-historial">
              <header className="dabble-historial-cabecera">
                <div>
                  <p className="kicker">máquina del tiempo</p>
                  <h1>{activo?.titulo ?? proyecto?.titulo ?? 'Proyecto'}</h1>
                </div>
                <button
                  className="dabble-boton"
                  onClick={() => {
                    setVista('editor');
                    setConfirmarReversion(null);
                  }}
                >
                  Salir de la máquina del tiempo
                </button>
              </header>
              <div className="dabble-historial-pagina">
                {cargandoDetalle && <p className="vacio">Cargando revisión…</p>}
                {errorDetalle && (
                  <pre className="banner-error" role="alert">
                    {errorDetalle}
                  </pre>
                )}
                {detalleRevision ? (
                  <>
                    <div className="dabble-version-cabecera">
                      <h2>Revisión {detalleRevision.revision}</h2>
                      <p>
                        {formatearFecha(detalleRevision.ts)} · {detalleRevision.autor} ·{' '}
                        {detalleRevision.scope}
                      </p>
                    </div>
                    {detalleRevision.contenidos.map((contenido, indice) => (
                      <details
                        key={contenido.ruta}
                        open={indice === 0 || detalleRevision.contenidos.length === 1}
                      >
                        <summary>{contenido.ruta}</summary>
                        {contenido.contenido !== null ? (
                          <pre className="fuente">{contenido.contenido}</pre>
                        ) : (
                          <p className="vacio">(archivo eliminado)</p>
                        )}
                        <button
                          className="dabble-boton"
                          disabled={comparando}
                          onClick={() =>
                            void compararConActual(contenido.ruta, detalleRevision.revision)
                          }
                        >
                          {comparando ? 'Comparando…' : 'Comparar con el documento actual'}
                        </button>
                        {comentariosProyecto
                          .filter(
                            (comentario) =>
                              comentario.documento === contenido.ruta &&
                              (!comentario.ancla ||
                                (contenido.contenido ?? '').includes(comentario.ancla.texto)),
                          )
                          .map((comentario) => (
                            <div key={comentario.id} className="dabble-hilo">
                              <strong>{comentario.estado}</strong>
                              {comentario.mensajes.map((mensaje) => (
                                <p key={mensaje.id} className="dabble-mensaje">
                                  {mensaje.texto}
                                </p>
                              ))}
                            </div>
                          ))}
                      </details>
                    ))}
                    {errorComparacion && (
                      <pre className="banner-error" role="alert">
                        {errorComparacion}
                      </pre>
                    )}
                    {comparacion && (
                      <div className="dabble-comparacion">
                        <div className="dabble-version-cabecera">
                          <h3>Diferencias con la revisión {comparacion.revision}</h3>
                          <p>
                            {comparacion.ruta} · +{comparacion.insertadas} / −
                            {comparacion.borradas}
                          </p>
                        </div>
                        <DiffLineas
                          lineas={comparacion.lineas}
                          etiquetaBase={`Revisión ${comparacion.revision}`}
                          etiquetaPropuesta="Documento actual"
                        />
                        <button
                          className="dabble-boton"
                          onClick={() => setComparacion(null)}
                        >
                          Cerrar comparación
                        </button>
                      </div>
                    )}
                  </>
                ) : (
                  !cargandoDetalle &&
                  !errorDetalle && (
                    <p className="vacio">
                      Selecciona una versión en el panel derecho para ver su contenido.
                    </p>
                  )
                )}
              </div>
              <div className="dabble-timeline">
                <button
                  className="dabble-boton"
                  onClick={() => {
                    setVista('editor');
                    setConfirmarReversion(null);
                  }}
                >
                  Salir de la máquina del tiempo
                </button>
                <div className="dabble-timeline-control">
                  <span aria-hidden="true">◀</span>
                  <input
                    type="range"
                    min={1}
                    max={Math.max(1, proyecto?.revision ?? 1)}
                    value={revisionSeleccionada ?? proyecto?.revision ?? 1}
                    onChange={(evento) => void verRevision(Number(evento.target.value))}
                    aria-label="Línea de tiempo de revisiones"
                  />
                  <span aria-hidden="true">▶</span>
                </div>
                <button
                  className="dabble-boton-primario"
                  onClick={() => void revertirARevision()}
                  disabled={!detalleRevision || revirtiendo}
                >
                  {revirtiendo
                    ? 'Recuperando…'
                    : confirmarReversion !== null
                      ? 'Confirmar recuperación'
                      : 'Recuperar'}
                </button>
                {confirmarReversion !== null && (
                  <button
                    className="dabble-boton"
                    onClick={() => setConfirmarReversion(null)}
                  >
                    Cancelar
                  </button>
                )}
                <label className="dabble-ir-cambio">
                  <input type="checkbox" disabled /> Ir al cambio
                </label>
              </div>
            </section>
          ) : activo ? (
            <article className="dabble-documento">
              <header className="dabble-doc-cabecera">
                <span className="dabble-doc-tipo">
                  {activo.tipo === 'escena' ? 'escena' : activo.clase ?? 'nota'}
                </span>
                <h1 className="dabble-titulo">{activo.titulo}</h1>
                <div className="dabble-doc-meta">
                  <span>{etiquetaTipo(activo)}</span>
                  <span>{palabrasDocumento} palabras</span>
                  <span>{estadoVisible}</span>
                </div>
              </header>
              <div className="dabble-hoja">
                {vista === 'editor' ? (
                  <EditorMarkdown
                    key={`${activo.id}:${revisionEditor}`}
                    markdown={cuerpoVisible}
                    editable={puedeEditar}
                    personajes={personajes}
                    tramas={tramas}
                    onCambio={alCambiar}
                    ancla={anclaComentario}
                    corrector={preferencias.corrector}
                    onComentario={async (texto, textoMarcado) => {
                      const actual = proyectoRef.current;
                      const documento = activoRef.current;
                      if (!actual || !documento) {
                        return null;
                      }
                      try {
                        const nuevo = await crearComentario(
                          actual.raiz,
                          documento.ruta,
                          texto,
                          textoMarcado,
                          actual.autor?.trim() || 'local',
                        );
                        setComentarios((lista) => [...lista, nuevo]);
                        setErrorComentarios(null);
                        return nuevo.id;
                      } catch (fallo) {
                        setErrorComentarios(errorDe(fallo).mensaje);
                        return null;
                      }
                    }}
                  />
                ) : (
                  <pre className="fuente">{cuerpoVisible || '(cuerpo vacío)'}</pre>
                )}
              </div>
              <footer className="dabble-doc-pie">
                <span>{palabrasDocumento} Palabra{palabrasDocumento === 1 ? '' : 's'}</span>
                <span>{palabrasHoy} hoy</span>
                <span className={`estado-guardado estado-${estadoVisible}`}>
                  {etiquetaGuardado(estadoVisible)}
                </span>
              </footer>
            </article>
          ) : (
            <div className="vacio-panel">
              <p>El proyecto no tiene documentos válidos.</p>
              <p>Abre Historial para revertir la revisión que los eliminó, o usa la CLI.</p>
            </div>
          )}
        </main>

        {panelAbierto && (
          <button
            className="dabble-panel-fondo"
            aria-label="Cerrar panel lateral"
            onClick={cerrarPanel}
          />
        )}

        {vista === 'historial' ? (
          <aside className="dabble-derecha dabble-derecha-historial">
            <button
              className="dabble-panel-cerrar"
              aria-label="Cerrar panel lateral"
              onClick={cerrarPanel}
            >
              ×
            </button>
            <header className="dabble-derecha-cabecera">
              <strong>Historial de versiones</strong>
              <button
                className="icono-dabble"
                aria-label="Cerrar historial"
                onClick={() => {
                  setVista('editor');
                  setConfirmarReversion(null);
                }}
              >
                ×
              </button>
            </header>
            <button
              className="dabble-actualizar"
              onClick={() => {
                setConfirmarReversion(null);
                setHistorialVersion((valor) => valor + 1);
              }}
            >
              Actualizar
            </button>
            {cargandoHistorial && <p className="vacio">Cargando historial…</p>}
            {errorHistorial && (
              <pre className="banner-error" role="alert">
                {errorHistorial}
              </pre>
            )}
            <ul className="dabble-versiones">
              {revisiones.map((revision) => (
                <li key={revision.revision}>
                  <button
                    className={`dabble-version ${
                      revisionSeleccionada === revision.revision ? 'activa' : ''
                    } ${revision.revision === proyecto?.revision ? 'actual' : ''}`}
                    onClick={() => void verRevision(revision.revision)}
                    disabled={revirtiendo}
                  >
                    <span className="dabble-version-punto" aria-hidden="true">◉</span>
                    <span>
                      <strong>
                        {revision.revision === proyecto?.revision
                          ? 'Versión actual'
                          : `Revisión ${revision.revision}`}
                      </strong>
                      <small>{formatearFecha(revision.ts)}</small>
                      <small>{revision.efectos} archivo(s) · {revision.scope}</small>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </aside>
        ) : (
          <>
        <aside className={`dabble-derecha ${panelAbierto ? 'abierto' : ''}`}>
            <button
              className="dabble-panel-cerrar"
              aria-label="Cerrar panel lateral"
              onClick={cerrarPanel}
            >
              ×
            </button>
            <div className="dabble-derecha-tabs" role="tablist" aria-label="Panel lateral">
              <button
                role="tab"
                aria-selected={panelDerecho === 'metas'}
                className={panelDerecho === 'metas' ? 'activo' : ''}
                onClick={() => setPanelDerecho('metas')}
              >
                Metas
              </button>
              <button
                role="tab"
                aria-selected={panelDerecho === 'notas'}
                className={panelDerecho === 'notas' ? 'activo' : ''}
                onClick={() => setPanelDerecho('notas')}
              >
                Notas
              </button>
              <button
                role="tab"
                aria-selected={panelDerecho === 'comentarios'}
                className={panelDerecho === 'comentarios' ? 'activo' : ''}
                onClick={() => setPanelDerecho('comentarios')}
              >
                Comentarios
              </button>
              <button
                role="tab"
                aria-selected={panelDerecho === 'plan'}
                className={panelDerecho === 'plan' ? 'activo' : ''}
                onClick={() => setPanelDerecho('plan')}
              >
                Plan
              </button>
              <button
                role="tab"
                aria-selected={panelDerecho === 'cambios'}
                className={panelDerecho === 'cambios' ? 'activo' : ''}
                onClick={() => setPanelDerecho('cambios')}
              >
                Cambios
              </button>
              <button
                role="tab"
                aria-selected={panelDerecho === 'herramientas'}
                className={panelDerecho === 'herramientas' ? 'activo' : ''}
                onClick={() => setPanelDerecho('herramientas')}
              >
                Herramientas
              </button>
            </div>
            {panelDerecho === 'metas' ? (
              <>
                <section className="dabble-metas">
                  <h3>Tus metas</h3>
                  <p>Sigue tu progreso con hábitos diarios, metas de libro o desafíos a plazo.</p>
                  <label className="dabble-campo-meta">
                    <span>Meta diaria (palabras)</span>
                    <input
                      type="number"
                      min={0}
                      value={metaDiaria}
                      onChange={(evento) => setMetaDiaria(evento.target.value)}
                      aria-label="Meta diaria de palabras"
                    />
                  </label>
                  <label className="dabble-campo-meta">
                    <span>Meta del libro (palabras)</span>
                    <input
                      type="number"
                      min={0}
                      value={metaLibro}
                      onChange={(evento) => setMetaLibro(evento.target.value)}
                      aria-label="Meta de palabras del libro"
                    />
                  </label>
                  <label className="dabble-campo-meta">
                    <span>Desafío (palabras)</span>
                    <input
                      type="number"
                      min={0}
                      value={desafioObjetivo}
                      onChange={(evento) => setDesafioObjetivo(evento.target.value)}
                      aria-label="Objetivo del desafío"
                    />
                  </label>
                  <label className="dabble-campo-meta">
                    <span>Fecha límite</span>
                    <input
                      type="date"
                      value={desafioLimite}
                      onChange={(evento) => setDesafioLimite(evento.target.value)}
                      aria-label="Fecha límite del desafío"
                    />
                  </label>
                  <button className="dabble-boton" onClick={() => void guardarMetasPanel()}>
                    Guardar metas
                  </button>
                  {errorMetas && (
                    <p className="dabble-error" role="alert">
                      {errorMetas}
                    </p>
                  )}
                  <p className="dabble-resumen">
                    {palabrasHoy.toLocaleString('es')} palabras hoy ·{' '}
                    {
                      sesiones.filter((sesion) =>
                        sesion.inicio.startsWith(new Date().toISOString().slice(0, 7)),
                      ).length
                    }{' '}
                    sesiones este mes · racha {racha} día{racha === 1 ? '' : 's'}
                  </p>
                  <div className="dabble-grafico" aria-hidden="true">
                    {ultimosDias.map((dia) => (
                      <span
                        key={dia.fecha}
                        style={{
                          height: `${Math.max(6, Math.round((dia.palabras / maximoDia) * 100))}%`,
                        }}
                        title={`${dia.fecha}: ${dia.palabras} palabras`}
                      />
                    ))}
                  </div>
                </section>
                <section className="dabble-timer">
                  <div className="dabble-timer-cabecera">
                    <strong>{formatearTiempo(restante)}</strong>
                    {!temporizadorActivo ? (
                      <button
                        className="dabble-play"
                        aria-label="Iniciar temporizador"
                        onClick={iniciarTemporizador}
                      >
                        ▶
                      </button>
                    ) : temporizadorPausado ? (
                      <button
                        className="dabble-play"
                        aria-label="Reanudar temporizador"
                        onClick={reanudarTemporizador}
                      >
                        ▶
                      </button>
                    ) : (
                      <button
                        className="dabble-play"
                        aria-label="Pausar temporizador"
                        onClick={pausarTemporizador}
                      >
                        ❚❚
                      </button>
                    )}
                    {temporizadorActivo && (
                      <button
                        className="dabble-boton"
                        onClick={() => void finalizarTemporizador()}
                      >
                        Finalizar
                      </button>
                    )}
                  </div>
                  <div className="dabble-timer-opciones">
                    {[5, 10, 15, 25, 30].map((minutos) => (
                      <button
                        key={minutos}
                        className={minutos === preset ? 'activo' : ''}
                        aria-pressed={minutos === preset}
                        disabled={temporizadorActivo}
                        onClick={() => {
                          setPreset(minutos);
                          setRestante(minutos * 60);
                        }}
                      >
                        {minutos}
                      </button>
                    ))}
                  </div>
                </section>
              </>
            ) : panelDerecho === 'notas' ? (
              <section className="dabble-notas-panel">
                <h3>Documento</h3>
                {activo && (
                  <div className="dabble-editor-titulo">
                    <label className="dabble-campo-meta">
                      <span>Título</span>
                      <input
                        value={tituloEdicion}
                        onChange={(evento) => setTituloEdicion(evento.target.value)}
                        aria-label="Título del documento"
                      />
                    </label>
                    {activo.tipo === 'escena' && (
                      <label className="dabble-campo-meta">
                        <span>Estado</span>
                        <select
                          value={estadoEdicion}
                          onChange={(evento) => setEstadoEdicion(evento.target.value)}
                          aria-label="Estado de la escena"
                        >
                          <option value="borrador">Borrador</option>
                          <option value="revision">Revisión</option>
                          <option value="final">Final</option>
                        </select>
                      </label>
                    )}
                    <label className="dabble-campo-meta">
                      <span>Etiquetas</span>
                      <input
                        value={etiquetasEdicion}
                        onChange={(evento) => setEtiquetasEdicion(evento.target.value)}
                        placeholder="separadas por comas"
                        aria-label="Etiquetas del documento"
                      />
                    </label>
                    {activo.clase === 'personajes' && (
                      <>
                        <label className="dabble-campo-meta">
                          <span>Rol</span>
                          <select
                            value={rolPersonaje}
                            onChange={(evento) => setRolPersonaje(evento.target.value)}
                            aria-label="Rol del personaje"
                          >
                            <option value="">Sin clasificar</option>
                            <option value="principal">Principal</option>
                            <option value="secundario">Secundario</option>
                          </select>
                        </label>
                        <label className="dabble-campo-meta">
                          <span>Alias</span>
                          <input
                            value={aliasPersonaje}
                            onChange={(evento) => setAliasPersonaje(evento.target.value)}
                            placeholder="separados por comas"
                            aria-label="Alias del personaje"
                          />
                        </label>
                      </>
                    )}
                    <div className="dabble-acciones-documento">
                      <button
                        className="dabble-boton"
                        onClick={() => void guardarMetadatosActivo()}
                      >
                        Guardar metadatos
                      </button>
                    </div>
                    <div className="dabble-acciones-documento">
                      <button
                        className="dabble-boton"
                        onClick={() => void renombrarActivo()}
                      >
                        Renombrar
                      </button>
                      <button
                        className="dabble-boton"
                        onClick={() => void duplicarActivo()}
                      >
                        Duplicar
                      </button>
                    </div>
                    {activo.tipo === 'escena' && (
                      <div className="dabble-acciones-documento">
                        <button className="dabble-boton" onClick={() => void reordenarActivo(-1)}>
                          Subir
                        </button>
                        <button className="dabble-boton" onClick={() => void reordenarActivo(1)}>
                          Bajar
                        </button>
                      </div>
                    )}
                    <div className="dabble-acciones-documento">
                      <button className="dabble-boton" onClick={copiarActivo}>
                        Copiar
                      </button>
                      {documentoCopiado && (
                        <button
                          className="dabble-boton"
                          onClick={() => void pegarDocumento()}
                        >
                          Pegar copia
                        </button>
                      )}
                    </div>
                    <div className="dabble-acciones-documento">
                      <button className="dabble-boton" onClick={() => window.print()}>
                        Imprimir
                      </button>
                      <button className="dabble-boton" onClick={() => void exportarPdfActivo()}>
                        Exportar PDF
                      </button>
                    </div>
                    <label className="dabble-campo-meta">
                      <span>Fuente PDF (pt)</span>
                      <input
                        type="number"
                        min={6}
                        max={36}
                        value={formatoPdf.fuente}
                        onChange={(evento) =>
                          setFormatoPdf((valor) => ({
                            ...valor,
                            fuente: Number(evento.target.value),
                          }))
                        }
                        aria-label="Fuente del PDF"
                      />
                    </label>
                    <label className="dabble-campo-meta">
                      <span>Interlineado PDF (pt)</span>
                      <input
                        type="number"
                        min={6}
                        max={72}
                        value={formatoPdf.interlineado}
                        onChange={(evento) =>
                          setFormatoPdf((valor) => ({
                            ...valor,
                            interlineado: Number(evento.target.value),
                          }))
                        }
                        aria-label="Interlineado del PDF"
                      />
                    </label>
                    <label className="dabble-campo-meta">
                      <span>Margen PDF (pt)</span>
                      <input
                        type="number"
                        min={36}
                        max={144}
                        value={formatoPdf.margen}
                        onChange={(evento) =>
                          setFormatoPdf((valor) => ({
                            ...valor,
                            margen: Number(evento.target.value),
                          }))
                        }
                        aria-label="Margen del PDF"
                      />
                    </label>
                    <button className="dabble-boton" onClick={() => void guardarFormato()}>
                      Guardar formato PDF
                    </button>
                    <div className="dabble-acciones-documento secundarias">
                      {activo.archivo ? (
                        <button
                          className="dabble-boton"
                          onClick={() => void cambiarArchivoActivo(null)}
                        >
                          Restaurar
                        </button>
                      ) : (
                        <>
                          <button
                            className="dabble-boton"
                            onClick={() => void cambiarArchivoActivo('archivado')}
                          >
                            Archivar
                          </button>
                          <button
                            className="dabble-boton"
                            onClick={() => void cambiarArchivoActivo('papelera')}
                          >
                            A papelera
                          </button>
                        </>
                      )}
                    </div>
                  </div>
                )}
                <dl>
                  <dt>Tipo</dt><dd>{activo ? etiquetaTipo(activo) : '—'}</dd>
                  <dt>Título</dt><dd>{activo?.titulo ?? '—'}</dd>
                  <dt>Ruta</dt><dd>{activo?.ruta ?? '—'}</dd>
                  <dt>ID</dt><dd>{activo?.id ?? '—'}</dd>
                  <dt>Palabras</dt><dd>{palabrasDocumento}</dd>
                </dl>
                <h3>Proyecto</h3>
                <dl>
                  <dt>Revisión</dt><dd>{proyecto?.revision ?? 0}</dd>
                  <dt>Escenas</dt><dd>{proyecto?.escenas ?? 0}</dd>
                  <dt>Notas</dt><dd>{proyecto?.notas ?? 0}</dd>
                  <dt>Ruta</dt><dd>{proyecto?.raiz ?? '—'}</dd>
                </dl>
              </section>
            ) : panelDerecho === 'comentarios' ? (
              <section className="dabble-comentarios-panel">
                <h3>Comentarios</h3>
                {activo ? (
                  <>
                    <label className="dabble-campo-meta">
                      <span>Filtro</span>
                      <select
                        value={filtroComentarios}
                        onChange={(evento) =>
                          setFiltroComentarios(
                            evento.target.value as
                              | 'todos'
                              | 'abiertos'
                              | 'resueltos'
                              | 'huerfanos',
                          )
                        }
                        aria-label="Filtrar comentarios"
                      >
                        <option value="todos">Todos</option>
                        <option value="abiertos">Abiertos</option>
                        <option value="resueltos">Resueltos</option>
                        <option value="huerfanos">Huérfanos</option>
                      </select>
                    </label>
                    <div className="dabble-acciones-documento">
                      <button className="dabble-boton" onClick={() => void copiarTexto(true)}>
                        Copiar con comentarios
                      </button>
                      <button className="dabble-boton" onClick={() => void copiarTexto(false)}>
                        Copiar sin comentarios
                      </button>
                    </div>
                    <label className="dabble-campo-meta">
                      <span>Nota rápida</span>
                      <input
                        value={textoSticky}
                        onChange={(evento) => setTextoSticky(evento.target.value)}
                        placeholder="Escribe una nota"
                        aria-label="Texto de la nota rápida"
                      />
                    </label>
                    <button
                      className="dabble-boton"
                      onClick={() => void crearStickyActivo()}
                      disabled={textoSticky.trim() === ''}
                    >
                      Agregar nota
                    </button>
                    {errorComentarios && (
                      <p className="dabble-error" role="alert">
                        {errorComentarios}
                      </p>
                    )}
                    {comentariosVisibles.length === 0 ? (
                      <p className="dabble-vacio">Sin comentarios en este documento.</p>
                    ) : (
                      <ul className="dabble-hilos">
                        {comentariosVisibles.map((comentario) => (
                          <li key={comentario.id} className="dabble-hilo">
                            <div className="dabble-hilo-cabecera">
                              <strong>
                                {comentario.tipo === 'sticky' ? 'Nota' : 'Comentario'}
                              </strong>
                              <span>{comentario.estado}</span>
                            </div>
                            {comentario.ancla && (
                              <button
                                className="dabble-boton"
                                onClick={() => {
                                  setVista('editor');
                                  setAnclaComentario(comentario.ancla?.texto ?? null);
                                }}
                              >
                                Ir al ancla
                              </button>
                            )}
                            <div className="dabble-acciones-documento">
                              {comentario.estado === 'abierto' ? (
                                <button
                                  className="dabble-boton"
                                  onClick={() => void resolverHilo(comentario.id)}
                                >
                                  Resolver
                                </button>
                              ) : (
                                <button
                                  className="dabble-boton"
                                  onClick={() => void reabrirHilo(comentario.id)}
                                >
                                  Reabrir
                                </button>
                              )}
                              <button
                                className="dabble-boton"
                                onClick={() => void archivarHilo(comentario.id)}
                              >
                                Archivar hilo
                              </button>
                            </div>
                            {comentario.mensajes.map((mensaje) => (
                              <div key={mensaje.id} className="dabble-mensaje-fila">
                                <p className="dabble-mensaje">
                                  <small>{mensaje.tipo}</small> {mensaje.texto}
                                </p>
                                {['comentario', 'respuesta'].includes(mensaje.tipo) && (
                                  <div className="dabble-acciones-documento">
                                    <button
                                      className="dabble-boton"
                                      onClick={() => {
                                        setMensajeEditando(mensaje.id);
                                        setTextoEdicionMensaje(mensaje.texto);
                                      }}
                                    >
                                      Editar
                                    </button>
                                    <button
                                      className="dabble-boton"
                                      onClick={() => void borrarMensaje(comentario.id, mensaje.id)}
                                    >
                                      Borrar mensaje
                                    </button>
                                  </div>
                                )}
                              </div>
                            ))}
                            {mensajeEditando &&
                              comentario.mensajes.some(
                                (mensaje) => mensaje.id === mensajeEditando,
                              ) && (
                                <>
                                  <input
                                    value={textoEdicionMensaje}
                                    onChange={(evento) =>
                                      setTextoEdicionMensaje(evento.target.value)
                                    }
                                    aria-label="Editar mensaje"
                                  />
                                  <div className="dabble-acciones-documento">
                                    <button
                                      className="dabble-boton"
                                      onClick={() =>
                                        void editarMensaje(comentario.id, mensajeEditando)
                                      }
                                    >
                                      Guardar edición
                                    </button>
                                    <button
                                      className="dabble-boton"
                                      onClick={() => setMensajeEditando(null)}
                                    >
                                      Cancelar
                                    </button>
                                  </div>
                                </>
                              )}
                            {comentario.estado === 'abierto' && (
                              <>
                                <input
                                  value={respuestas[comentario.id] ?? ''}
                                  onChange={(evento) =>
                                    setRespuestas((valor) => ({
                                      ...valor,
                                      [comentario.id]: evento.target.value,
                                    }))
                                  }
                                  placeholder="Responder"
                                  aria-label={`Responder a ${comentario.id}`}
                                />
                                <div className="dabble-acciones-documento">
                                  <button
                                    className="dabble-boton"
                                    onClick={() => void responderHilo(comentario.id)}
                                    disabled={(respuestas[comentario.id] ?? '').trim() === ''}
                                  >
                                    Responder
                                  </button>
                                </div>
                              </>
                            )}
                          </li>
                        ))}
                      </ul>
                    )}
                  </>
                ) : (
                  <p className="dabble-vacio">Abre un documento para ver sus comentarios.</p>
                )}
              </section>
            ) : panelDerecho === 'plan' ? (
              <section className="dabble-plan-panel">
                <h3>Plot Grid</h3>
                {errorMetas && (
                  <p className="dabble-error" role="alert">
                    {errorMetas}
                  </p>
                )}
                <label className="dabble-campo-meta">
                  <span>Nueva trama</span>
                  <input
                    value={nuevaTrama}
                    onChange={(evento) => setNuevaTrama(evento.target.value)}
                    placeholder="Nombre de la trama"
                    aria-label="Nombre de la nueva trama"
                  />
                </label>
                <label className="dabble-campo-meta">
                  <span>Color</span>
                  <input
                    type="color"
                    value={colorTrama}
                    onChange={(evento) => setColorTrama(evento.target.value)}
                    aria-label="Color de la trama"
                  />
                </label>
                <button
                  className="dabble-boton"
                  onClick={() => void crearTramaGrid()}
                  disabled={nuevaTrama.trim() === ''}
                >
                  Crear trama
                </button>

                <label className="dabble-campo-meta">
                  <span>Trama</span>
                  <select
                    value={tramaSeleccionada}
                    onChange={(evento) => setTramaSeleccionada(evento.target.value)}
                    aria-label="Trama seleccionada"
                  >
                    {plotGrid.tramas.map((trama) => (
                      <option key={trama.id} value={trama.id}>
                        {trama.nombre}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="dabble-campo-meta">
                  <span>Nombre de la trama</span>
                  <input
                    value={nombreTramaEdit}
                    onChange={(evento) => setNombreTramaEdit(evento.target.value)}
                    aria-label="Nombre de la trama seleccionada"
                  />
                </label>
                <label className="dabble-campo-meta">
                  <span>Color de la trama</span>
                  <input
                    type="color"
                    value={colorTramaEdit}
                    onChange={(evento) => setColorTramaEdit(evento.target.value)}
                    aria-label="Color de la trama seleccionada"
                  />
                </label>
                <button className="dabble-boton" onClick={() => void actualizarTrama()}>
                  Guardar trama
                </button>
                <button
                  className="dabble-boton"
                  onClick={() => void anadirPuntoGrid()}
                  disabled={!tramaSeleccionada || activo?.tipo !== 'escena'}
                >
                  Añadir escena activa
                </button>

                {plotGrid.tramas.length === 0 ? (
                  <p className="dabble-vacio">Sin tramas en el Plot Grid.</p>
                ) : (
                  <ul className="dabble-tramas">
                    {plotGrid.tramas.map((trama) => {
                      const puntos = [...trama.puntos].sort((a, b) => a.orden - b.orden);
                      return (
                        <li key={trama.id} className="dabble-trama">
                          <span
                            className="dabble-trama-color"
                            style={{ background: trama.color }}
                            aria-hidden="true"
                          />
                          <strong>{trama.nombre}</strong>
                          <small>{trama.puntos.length} escena(s)</small>
                          {puntos.length > 0 && (
                            <div className="dabble-trama-linea" aria-hidden="true">
                              {puntos.map((punto) => (
                                <span
                                  key={punto.escena}
                                  title={punto.etiqueta ?? undefined}
                                  style={{ background: trama.color }}
                                />
                              ))}
                            </div>
                          )}
                          {trama.id === tramaSeleccionada && puntos.length > 0 && (
                            <ul className="dabble-puntos">
                              {puntos.map((punto) => {
                                const escena = documentos.find(
                                  (documento) => documento.id === punto.escena,
                                );
                                return (
                                  <li key={punto.escena} className="dabble-punto">
                                    <span>{escena?.titulo ?? 'Escena desconocida'}</span>
                                    <div className="dabble-acciones-documento">
                                      <button
                                        className="dabble-boton"
                                        onClick={() =>
                                          void moverPuntoGrid(trama.id, punto.escena, -1)
                                        }
                                      >
                                        ↑
                                      </button>
                                      <button
                                        className="dabble-boton"
                                        onClick={() =>
                                          void moverPuntoGrid(trama.id, punto.escena, 1)
                                        }
                                      >
                                        ↓
                                      </button>
                                      <button
                                        className="dabble-boton"
                                        onClick={() =>
                                          void quitarPuntoGrid(trama.id, punto.escena)
                                        }
                                      >
                                        Quitar
                                      </button>
                                    </div>
                                  </li>
                                );
                              })}
                            </ul>
                          )}
                        </li>
                      );
                    })}
                  </ul>
                )}

                {activo?.clase === 'ideas' && (
                  <button className="dabble-boton" onClick={() => void convertirIdeaActiva()}>
                    Convertir idea en proyecto
                  </button>
                )}
                <h3>Crear documento</h3>
                <label className="dabble-campo-meta">
                  <span>Título de la escena</span>
                  <input
                    value={nuevaEscenaTitulo}
                    onChange={(evento) => setNuevaEscenaTitulo(evento.target.value)}
                    aria-label="Título de la nueva escena"
                  />
                </label>
                <button className="dabble-boton" onClick={() => void crearEscenaNueva()}>
                  Nueva escena
                </button>
                <label className="dabble-campo-meta">
                  <span>Tipo de nota</span>
                  <select
                    value={nuevaNotaTipo}
                    onChange={(evento) => setNuevaNotaTipo(evento.target.value)}
                    aria-label="Tipo de la nueva nota"
                  >
                    <option value="cuaderno">Cuaderno</option>
                    <option value="ideas">Ideas</option>
                    <option value="personajes">Personajes</option>
                    <option value="tramas">Tramas</option>
                    <option value="lugares">Lugares</option>
                    <option value="cronologia">Cronología</option>
                    <option value="objetos">Objetos</option>
                  </select>
                </label>
                <label className="dabble-campo-meta">
                  <span>Título de la nota</span>
                  <input
                    value={nuevaNotaTitulo}
                    onChange={(evento) => setNuevaNotaTitulo(evento.target.value)}
                    aria-label="Título de la nueva nota"
                  />
                </label>
                <button className="dabble-boton" onClick={() => void crearNotaNueva()}>
                  Nueva nota
                </button>

                <h3>Importar y respaldar</h3>
                <label className="dabble-campo-meta">
                  <span>Importar como</span>
                  <select
                    value={tipoImportacion}
                    onChange={(evento) => setTipoImportacion(evento.target.value)}
                    aria-label="Importar como"
                  >
                    <option value="">Escena</option>
                    <option value="cuaderno">Nota de cuaderno</option>
                    <option value="ideas">Idea</option>
                    <option value="personajes">Personaje</option>
                    <option value="tramas">Trama</option>
                    <option value="lugares">Lugar</option>
                    <option value="cronologia">Cronología</option>
                    <option value="objetos">Objeto</option>
                  </select>
                </label>
                <button className="dabble-boton" onClick={() => void importarArchivo()}>
                  Importar documento
                </button>
                <button className="dabble-boton" onClick={() => void hacerBackup()}>
                  Crear backup ZIP
                </button>
                <button
                  className="dabble-boton"
                  onClick={() => void restaurarProyectoDesdeBackup()}
                >
                  Restaurar backup
                </button>
              </section>
            ) : panelDerecho === 'cambios' ? (
              <PanelCambios
                modo={modoSugerencias || cambioActivo !== null}
                cambios={cambiosProyecto}
                activo={cambioActivo}
                documentos={documentos}
                cargando={cargandoCambios}
                error={errorCambios}
                procesando={procesandoCambio}
                onModo={(valor) => {
                  setModoSugerencias(valor);
                  modoSugerenciasRef.current = valor;
                }}
                onLeer={(id) => void leerCambioDePanel(id)}
                onAceptar={() => void aceptarCambioActivo()}
                onDescartar={() => void descartarCambioActivo()}
                onAbrirDocumento={abrirDocumentoDeCambio}
                onVerDiff={() => setDiffAbierto(true)}
              />
            ) : (
              <PanelHerramientas
                soporta={lecturaDisponible}
                voces={voces}
                voz={vozLectura}
                velocidad={velocidadLectura}
                leyendo={leyendo}
                pausado={pausadoLectura}
                error={errorLectura}
                palabras={palabrasDocumento}
                caracteres={caracteresDocumento}
                parrafos={parrafosDocumento}
                minutosLectura={minutosLectura}
                palabrasSeleccion={palabrasSeleccion}
                enfoque={preferencias.modoEnfoque}
                onVoz={setVozLectura}
                onVelocidad={setVelocidadLectura}
                onLeer={leerDocumento}
                onLeerSeleccion={leerSeleccion}
                onPausar={pausarLecturaActual}
                onReanudar={reanudarLecturaActual}
                onDetener={detenerLecturaActual}
                onEnfoque={(activo) =>
                  setPreferencias((valor) => ({ ...valor, modoEnfoque: activo }))
                }
              />
            )}
          </aside>
          </>
        )}
      </div>

      <footer className="dabble-statusbar">
        <span>revisión {proyecto?.revision ?? 0}</span>
        <span>{proyecto?.escenas ?? 0} escenas · {proyecto?.notas ?? 0} notas</span>
        <span>{palabrasDocumento} palabras · {palabrasHoy} hoy</span>
        <span className={proyecto && proyecto.errores.length > 0 ? 'estado-error' : 'estado-ok'}>
          {proyecto && proyecto.errores.length > 0
            ? `${proyecto.errores.length} error(es)`
            : 'sin errores'}
        </span>
        {proyecto && proyecto.avisos.length > 0 && (
          <span className="estado-aviso">{proyecto.avisos.length} aviso(s)</span>
        )}
        <span className={`estado-guardado estado-${estadoVisible}`}>{etiquetaGuardado(estadoVisible)}</span>
        <span className="crece" />
        <span>{esTauri ? 'escritorio' : 'demo en navegador'}</span>
      </footer>

      {vistaPreviaImportacion && (
        <div className="modal-fondo">
          <div
            className="modal-proyecto"
            role="dialog"
            aria-modal="true"
            aria-label="Previsualizar importación"
          >
            <header className="modal-proyecto-cabecera">
              <h2>Previsualizar importación</h2>
              <button
                className="cerrar-modal"
                aria-label="Cerrar previsualización"
                onClick={() => setVistaPreviaImportacion(null)}
              >
                ×
              </button>
            </header>
            <div className="modal-proyecto-cuerpo">
              <div className="modal-formulario">
                <p>
                  {vistaPreviaImportacion.archivos.length} archivo(s) ·{' '}
                  {vistaPreviaImportacion.formato}
                </p>
                <ul>
                  {vistaPreviaImportacion.archivos.map((archivo) => (
                    <li key={archivo}>{archivo.split(/[\\/]/).at(-1)}</li>
                  ))}
                </ul>
                <pre className="fuente">{vistaPreviaImportacion.texto}</pre>
              </div>
            </div>
            <footer className="modal-proyecto-pie">
              <button
                className="boton-inicio"
                onClick={() => setVistaPreviaImportacion(null)}
              >
                Cancelar
              </button>
              <button className="boton-primario" onClick={() => void confirmarImportacion()}>
                Importar
              </button>
            </footer>
          </div>
        </div>
      )}

      {preferenciasAbiertas && (
        <ModalPreferencias
          preferencias={preferencias}
          onCambio={setPreferencias}
          onCerrar={() => setPreferenciasAbiertas(false)}
        />
      )}

      {diffAbierto && cambioActivo && (
        <ModalDiff
          lineas={cambioActivo.lineas}
          etiquetaBase="Documento actual"
          etiquetaPropuesta="Propuesta"
          onCerrar={() => setDiffAbierto(false)}
        />
      )}

      {paleta && (
        <Paleta
          modo={paleta}
          documentos={documentos}
          comandos={comandos}
          onCerrar={() => setPaleta(null)}
          onDocumento={abrirDocumentoDesdePaleta}
          onComando={ejecutarComando}
        />
      )}
    </div>
  );
}
