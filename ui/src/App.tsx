import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
} from 'react';
import {
  abrirProyecto,
  buscar as buscarEnProyecto,
  crearProyecto as crearProyectoApi,
  cuarentena as pedirCuarentena,
  descartarCuarentena,
  elegirDestinoDocx,
  elegirProyecto,
  errorDe,
  esTauri,
  escribir,
  exportarDocx,
  historial as pedirHistorial,
  leer,
  listar,
  mostrar as pedirRevision,
  proyectoE2e,
  revertir as revertirRevision,
} from './api';
import type { BusquedaDto } from './types/BusquedaDto';
import type { CuarentenaDto } from './types/CuarentenaDto';
import type { DocumentoDto } from './types/DocumentoDto';
import type { ProyectoDto } from './types/ProyectoDto';
import type { RevisionDetalleDto } from './types/RevisionDetalleDto';
import type { RevisionDto } from './types/RevisionDto';
import type { TipoError } from './types/TipoError';
import { EditorMarkdown } from './EditorMarkdown';
import { Inicio, type DatosNuevoProyecto } from './Inicio';
import { analizarCompatibilidad } from './markdown/compatibilidad';
import { contarPalabras, textoSinMarcas } from './palabras';
import { colorDesdeTitulo, guardarRecientes, leerRecientes, type Reciente } from './recientes';

type Estado = 'inicio' | 'cargando' | 'listo' | 'error';
type Vista = 'editor' | 'fuente' | 'historial';
type EstadoGuardado =
  | 'guardado'
  | 'pendiente'
  | 'guardando'
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

  const cargando = useRef(false);
  const navegando = useRef(false);
  const revirtiendoRef = useRef(false);
  const detalleSeq = useRef(0);
  const busquedaSeq = useRef(0);
  const buscadorRef = useRef<HTMLInputElement | null>(null);
  const bannerErrorRef = useRef<HTMLDivElement | null>(null);
  const panelRef = useRef<HTMLElement | null>(null);
  const proyectoRef = useRef<ProyectoDto | null>(null);
  const activoRef = useRef<DocumentoDto | null>(null);
  const puedeEditarRef = useRef(false);
  const pendienteRef = useRef<Pendiente | null>(null);
  const cuerpoEditorRef = useRef('');
  const drenajeRef = useRef<Promise<boolean> | null>(null);
  const temporizadorRef = useRef<number | null>(null);
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

  const escenas = useMemo(
    () => documentos.filter((documento) => documento.tipo === 'escena'),
    [documentos],
  );
  const notas = useMemo(
    () => documentos.filter((documento) => documento.tipo === 'nota'),
    [documentos],
  );
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
        escenas,
        (documento) => grupoEscena(documento).clave,
        (documento) => grupoEscena(documento).etiqueta,
      ),
    [escenas],
  );
  const gruposNotas = useMemo(
    () =>
      agrupar(
        notas,
        (documento) => documento.clase ?? 'notas',
        (documento) => etiquetaCarpeta(documento.clase ?? 'notas'),
      ),
    [notas],
  );
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
    !revirtiendo;
  puedeEditarRef.current = puedeEditar;

  const estadoVisible: EstadoGuardado =
    estadoGuardado === 'conflicto' ||
    estadoGuardado === 'recuperacion' ||
    estadoGuardado === 'ocupado' ||
    estadoGuardado === 'error'
      ? estadoGuardado
      : soloLecturaProyecto || bloqueoDocumento
        ? 'solo-lectura'
        : estadoGuardado;

  const puedeGuardar =
    estadoGuardado === 'pendiente' ||
    estadoGuardado === 'error' ||
    estadoGuardado === 'ocupado';

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
          palabras: palabras || anterior?.palabras || 0,
          color: color ?? anterior?.color ?? colorDesdeTitulo(resumen.titulo),
        },
        ...lista.filter((item) => item.ruta !== resumen.raiz),
      ].slice(0, 8);
      guardarRecientes(siguiente);
      return siguiente;
    });
  };

  const cargar = async (ruta: string, objetivoPreferido?: string | null) => {
    if (cargando.current) {
      return;
    }
    cargando.current = true;
    setEstado('cargando');
    setError(null);
    setMensaje(null);
    setErrorGuardado(null);
    setConsulta('');
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
    setEstadoGuardado('guardando');
    try {
      const activoActual = activoRef.current;
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
    if (cuerpo === actual.cuerpo && drenajeRef.current === null) {
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
    const resumen = await crearProyectoApi(
      datos.ruta,
      datos.titulo,
      datos.autor,
      datos.idioma,
      datos.notasIniciales,
    );
    recordarProyecto(resumen, 0, datos.color);
    if (abrir) {
      await cargar(resumen.raiz);
      setMensaje(`Proyecto «${resumen.titulo}» creado y abierto.`);
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
      if (!(evento.ctrlKey || evento.metaKey)) {
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
    if (errorGuardado) {
      // El aviso está en el flujo normal para no tapar el documento con zoom
      // alto. Al aparecer, el panel vuelve arriba y el foco va al aviso.
      if (panelRef.current) {
        panelRef.current.scrollTop = 0;
      }
      bannerErrorRef.current?.focus({ preventScroll: true });
    }
  }, [errorGuardado]);

  const cuerpoVisible =
    activo && pendienteRef.current?.id === activo.id
      ? pendienteRef.current.cuerpo
      : (activo?.cuerpo ?? '');
  const modoBusqueda = consulta.trim().length >= 2;

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
    <div className="aplicacion">
      <a className="saltar-contenido" href="#panel-principal">
        Saltar al contenido
      </a>
      <header className="topbar">
        <div className="marca">
          writer <span>v0.2</span>
        </div>
        <div className="proyecto-cabecera">
          <strong>{proyecto?.titulo ?? 'Proyecto'}</strong>
          <span>{proyecto?.raiz}</span>
        </div>
        <div className="topbar-acciones">
          <span className={`modo ${esTauri ? 'modo-tauri' : 'modo-demo'}`}>
            {esTauri ? 'Tauri' : 'Demo'}
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
            className="secundario"
            onClick={() => void exportarDocumento()}
            disabled={exportando || revirtiendo}
          >
            {exportando ? 'Exportando…' : 'Exportar DOCX'}
          </button>
          <button className="secundario" onClick={() => void abrir()}>
            Abrir otro
          </button>
        </div>
      </header>

      {estado === 'cargando' && <div className="cargando">Abriendo proyecto…</div>}

      <div className="workspace">
        <aside className="sidebar">
          <div className="buscador">
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
              placeholder="Buscar en el proyecto…"
              aria-label="Buscar en el proyecto"
            />
            {consulta !== '' && (
              <button
                className="limpiar-busqueda"
                onClick={() => setConsulta('')}
                aria-label="Limpiar búsqueda"
              >
                ×
              </button>
            )}
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
            <>
              <div className="sidebar-titulo">
                <span>Manuscrito</span>
                <span className="contador">{escenas.length}</span>
              </div>
              {gruposEscenas.length === 0 && <p className="vacio">Sin escenas.</p>}
              {gruposEscenas.map(({ clave, etiqueta, documentos: lista }) => (
                <section className="grupo" key={clave}>
                  <h3>{etiqueta}</h3>
                  {lista.map((documento) => (
                    <button
                      key={documento.id}
                      id={`doc-${documento.id}`}
                      className={`documento ${activo?.id === documento.id ? 'activo' : ''}`}
                      tabIndex={idArbolTabbable === documento.id ? 0 : -1}
                      aria-current={activo?.id === documento.id ? 'page' : undefined}
                      onFocus={() => setFocoArbol(documento.id)}
                      onKeyDown={(evento) => manejarTeclaArbol(evento, documento)}
                      onClick={() => void seleccionar(documento)}
                    >
                      <span className="documento-titulo">{documento.titulo}</span>
                      <span className="documento-meta">
                        {documento.orden !== null ? `orden ${documento.orden}` : 'sin orden'} ·{' '}
                        {documento.estado ?? 'borrador'}
                      </span>
                    </button>
                  ))}
                </section>
              ))}

              <div className="sidebar-titulo sidebar-notas">
                <span>Notas</span>
                <span className="contador">{notas.length}</span>
              </div>
              {gruposNotas.length === 0 && <p className="vacio">Sin notas.</p>}
              {gruposNotas.map(({ clave, etiqueta, documentos: lista }) => (
                <section className="grupo" key={clave}>
                  <h3>{etiqueta}</h3>
                  {lista.map((documento) => (
                    <button
                      key={documento.id}
                      id={`doc-${documento.id}`}
                      className={`documento ${activo?.id === documento.id ? 'activo' : ''}`}
                      tabIndex={idArbolTabbable === documento.id ? 0 : -1}
                      aria-current={activo?.id === documento.id ? 'page' : undefined}
                      onFocus={() => setFocoArbol(documento.id)}
                      onKeyDown={(evento) => manejarTeclaArbol(evento, documento)}
                      onClick={() => void seleccionar(documento)}
                    >
                      <span className="documento-titulo">{documento.titulo}</span>
                      <span className="documento-meta">{documento.clase}</span>
                    </button>
                  ))}
                </section>
              ))}
            </>
          )}
        </aside>

        <main className="panel" id="panel-principal" tabIndex={-1} ref={panelRef}>
          {error && (
            <pre className="banner-error" role="alert">
              {error}
            </pre>
          )}
          {mensaje && (
            <div className="banner-info">
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
            <div
              className="banner-error"
              ref={bannerErrorRef}
              role="alert"
              tabIndex={-1}
            >
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
              <p>
                Se abrió en solo lectura para no perder contenido. Edítalo desde la CLI o
                conviértelo a las construcciones soportadas.
              </p>
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
            <section className="cuarentena">
              <div className="cuarentena-cabecera">
                <div>
                  <p className="kicker">copia de diagnóstico</p>
                  <h3>Cuarentena ({cuarentena.length})</h3>
                </div>
                <p>
                  Los originales no se tocan. Descartar elimina solo la copia.
                </p>
              </div>
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
            </section>
          )}

          {proyecto ? (
            <article className="documento-panel">
              {activo ? (
                <header className="documento-cabecera">
                  <div>
                    <p className="kicker">{activo.tipo === 'escena' ? 'escena' : 'nota'}</p>
                    <h2>{activo.titulo}</h2>
                  </div>
                  <div className="chips">
                    <span>{etiquetaTipo(activo)}</span>
                    {activo.orden !== null && <span>orden {activo.orden}</span>}
                    <span className="chip-id">{activo.id}</span>
                  </div>
                </header>
              ) : (
                <header className="documento-cabecera">
                  <div>
                    <p className="kicker">proyecto</p>
                    <h2>{proyecto.titulo}</h2>
                  </div>
                  <div className="chips">
                    <span>{proyecto.escenas} escenas</span>
                    <span>{proyecto.notas} notas</span>
                  </div>
                </header>
              )}
              <div className="tabs" role="tablist">
                <button
                  className={vista === 'editor' ? 'tab activo' : 'tab'}
                  onClick={() => setVista('editor')}
                  disabled={!activo}
                >
                  Editor
                </button>
                <button
                  className={vista === 'fuente' ? 'tab activo' : 'tab'}
                  onClick={() => setVista('fuente')}
                  disabled={!activo}
                >
                  Markdown
                </button>
                <button
                  className={vista === 'historial' ? 'tab activo' : 'tab'}
                  onClick={() => setVista('historial')}
                >
                  Historial
                </button>
                <span
                  className={`estado-guardado estado-${estadoVisible}`}
                  role="status"
                  aria-live="polite"
                  aria-label="Estado de guardado"
                  title={errorGuardado ?? undefined}
                >
                  {etiquetaGuardado(estadoVisible)}
                </span>
                <button
                  className="secundario pequeno guardar"
                  onClick={() => void guardarAhora()}
                  disabled={!puedeGuardar || !activo}
                >
                  Guardar
                </button>
              </div>
              {vista === 'historial' ? (
                <section className="historial">
                  <div className="historial-cabecera">
                    <div>
                      <p className="kicker">línea de tiempo</p>
                      <h3>Revisiones del proyecto</h3>
                    </div>
                    <button
                      className="secundario pequeno"
                      onClick={() => {
                        setConfirmarReversion(null);
                        setHistorialVersion((valor) => valor + 1);
                      }}
                    >
                      Actualizar
                    </button>
                  </div>
                  {cargandoHistorial && <p className="vacio">Cargando historial…</p>}
                  {errorHistorial && (
                    <pre className="banner-error" role="alert">
                      {errorHistorial}
                    </pre>
                  )}
                  {!cargandoHistorial && !errorHistorial && revisiones.length === 0 && (
                    <p className="vacio">Sin revisiones.</p>
                  )}
                  <div className="historial-cuerpo">
                    <ol className="revisiones">
                      {revisiones.map((revision) => (
                        <li key={revision.revision}>
                          <button
                            className={`revision ${
                              revisionSeleccionada === revision.revision ? 'activa' : ''
                            } ${revision.revision === proyecto?.revision ? 'actual' : ''}`}
                            onClick={() => void verRevision(revision.revision)}
                            disabled={revirtiendo}
                          >
                            <span className="revision-numero">
                              r{revision.revision}
                              {revision.revision === proyecto?.revision && <em>actual</em>}
                            </span>
                            <span className="revision-meta">
                              {formatearFecha(revision.ts)} · {revision.autor} · {revision.scope}
                            </span>
                            <span className="revision-rutas">
                              {revision.efectos} archivo(s): {revision.rutas.join(', ')}
                            </span>
                          </button>
                        </li>
                      ))}
                    </ol>
                    <div className="detalle-revision">
                      {cargandoDetalle && <p className="vacio">Cargando revisión…</p>}
                      {errorDetalle && (
                        <pre className="banner-error" role="alert">
                          {errorDetalle}
                        </pre>
                      )}
                      {detalleRevision ? (
                        <>
                          <header className="detalle-cabecera">
                            <h4>Revisión {detalleRevision.revision}</h4>
                            <p>
                              {formatearFecha(detalleRevision.ts)} · {detalleRevision.autor} ·{' '}
                              {detalleRevision.scope}
                            </p>
                          </header>
                          <div className="detalle-acciones">
                            <button
                              className={`peligro ${confirmarReversion !== null ? 'confirmado' : ''}`}
                              onClick={() => void revertirARevision()}
                              disabled={revirtiendo}
                            >
                              {revirtiendo
                                ? 'Revirtiendo…'
                                : confirmarReversion !== null
                                  ? 'Confirmar reversión'
                                  : 'Revertir esta revisión'}
                            </button>
                            {confirmarReversion !== null && (
                              <button
                                className="secundario pequeno"
                                onClick={() => setConfirmarReversion(null)}
                              >
                                Cancelar
                              </button>
                            )}
                            {detalleRevision.revision === proyecto?.revision && (
                              <span className="nota-modo">Es la revisión actual.</span>
                            )}
                            {confirmarReversion !== null && (
                              <span className="nota-modo">
                                Deshace los cambios de esta revisión y crea una revisión nueva.
                              </span>
                            )}
                          </div>
                          <div className="detalle-archivos">
                            {detalleRevision.contenidos.map((contenido) => (
                              <details
                                key={contenido.ruta}
                                open={detalleRevision.contenidos.length === 1}
                              >
                                <summary>{contenido.ruta}</summary>
                                {contenido.contenido !== null ? (
                                  <pre className="fuente">{contenido.contenido}</pre>
                                ) : (
                                  <p className="vacio">(archivo eliminado)</p>
                                )}
                              </details>
                            ))}
                          </div>
                        </>
                      ) : (
                        !cargandoDetalle &&
                        !errorDetalle && (
                          <p className="vacio">
                            Selecciona una revisión para ver los archivos que cambió.
                          </p>
                        )
                      )}
                    </div>
                  </div>
                </section>
              ) : vista === 'editor' && activo ? (
                <EditorMarkdown
                  key={`${activo.id}:${revisionEditor}`}
                  markdown={cuerpoVisible}
                  editable={puedeEditar}
                  personajes={personajes}
                  tramas={tramas}
                  onCambio={alCambiar}
                />
              ) : vista === 'fuente' && activo ? (
                <pre className="fuente">{cuerpoVisible || '(cuerpo vacío)'}</pre>
              ) : (
                <div className="vacio-panel">
                  <p>El proyecto no tiene documentos válidos.</p>
                  <p>
                    Abre <b>Historial</b> para revertir la revisión que los eliminó, o usa la CLI.
                  </p>
                </div>
              )}
            </article>
          ) : (
            <p className="vacio-panel">Selecciona una escena o nota del árbol.</p>
          )}
        </main>
      </div>

      <footer className="statusbar">
        <span>revisión {proyecto?.revision ?? 0}</span>
        <span>
          {proyecto?.escenas ?? 0} escenas · {proyecto?.notas ?? 0} notas
        </span>
        <span className={proyecto && proyecto.errores.length > 0 ? 'estado-error' : 'estado-ok'}>
          {proyecto && proyecto.errores.length > 0
            ? `${proyecto.errores.length} error(es)`
            : 'sin errores'}
        </span>
        {proyecto && proyecto.avisos.length > 0 && (
          <span className="estado-aviso">{proyecto.avisos.length} aviso(s)</span>
        )}
        <span className={`estado-guardado estado-${estadoVisible}`}>
          {etiquetaGuardado(estadoVisible)}
        </span>
        <span className="crece" />
        <span>{esTauri ? 'escritorio' : 'demo en navegador'}</span>
      </footer>
    </div>
  );
}
