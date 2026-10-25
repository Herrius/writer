import { useEffect, useMemo, useRef, useState } from 'react';
import {
  elegirCarpetaCreacion,
  eliminarPlantilla,
  errorDe,
  guardarPlantilla,
  listarPlantillas,
  moverProyectoAPapelera,
  purgarProyectoDePapelera,
  restaurarProyectoDePapelera,
} from './api';
import type { PlantillaDto } from './types/PlantillaDto';
import { colorDesdeTitulo, slugProyecto, type Reciente } from './recientes';
import {
  autoresRecientes,
  cargarFiltros,
  filtrarRecientes,
  filtrosActivos,
  guardarColecciones,
  guardarFiltros,
  guardarPapelera,
  leerColecciones,
  leerPapelera,
  nuevaColeccion,
  type Coleccion,
  type EntradaPapelera,
  type FiltrosProyectos,
} from './organizacion';
import { ModalFiltros } from './features/inicio/ModalFiltros';
import { ModalColeccion } from './features/inicio/ModalColeccion';
import { ModalGestionColecciones } from './features/inicio/ModalGestionColecciones';
import { ModalAccionesProyecto } from './features/inicio/ModalAccionesProyecto';
import { ModalPlantilla } from './features/inicio/ModalPlantilla';
import { ModalConfirmar } from './features/inicio/ModalConfirmar';
import portadaArbol from './assets/portada-arbol.jpg';

export type DatosNuevoProyecto = {
  ruta: string;
  nombre: string;
  titulo: string;
  autor: string | null;
  idioma: string;
  notasIniciales: boolean;
  color: string;
  plantilla: string | null;
};

type Props = {
  recientes: Reciente[];
  esTauri: boolean;
  errorInicial: string | null;
  onAbrir: () => void;
  onDemo: () => void;
  onAbrirReciente: (ruta: string) => void;
  onQuitarReciente: (ruta: string) => void;
  onAgregarReciente: (reciente: Reciente) => void;
  onCrear: (datos: DatosNuevoProyecto, abrir: boolean) => Promise<void>;
  vistaInicial?: 'activo' | 'plantillas' | 'papelera';
};

const COLORES = [
  '#4d6b8a',
  '#5a8fa3',
  '#4f9b8a',
  '#7ba05b',
  '#c2a24a',
  '#d18b47',
  '#c97a63',
  '#b65f62',
  '#8f5d7a',
  '#9b6aa8',
  '#6f6fa8',
  '#6b6f76',
];

const NOMBRES_COLOR: Record<string, string> = {
  '#4d6b8a': 'Azul profundo',
  '#5a8fa3': 'Azul petróleo',
  '#4f9b8a': 'Verde azulado',
  '#7ba05b': 'Verde oliva',
  '#c2a24a': 'Mostaza',
  '#d18b47': 'Naranja',
  '#c97a63': 'Terracota',
  '#b65f62': 'Rojo ladrillo',
  '#8f5d7a': 'Ciruela',
  '#9b6aa8': 'Malva',
  '#6f6fa8': 'Índigo',
  '#6b6f76': 'Grafito',
};

function Portada({
  color,
  titulo,
  autor,
  compacta = false,
}: {
  color: string;
  titulo: string;
  autor: string;
  compacta?: boolean;
}) {
  return (
    <div
      className={`portada ${compacta ? 'portada-compacta' : ''}`}
      style={{ backgroundColor: color }}
      aria-hidden="true"
    >
      <img className="portada-arte" src={portadaArbol} alt="" draggable={false} />
      <span
        className="portada-tinte"
        style={{ backgroundColor: color }}
        aria-hidden="true"
      />
      <span className="portada-velo" />
      <span className="portada-linea" />
      <strong className="portada-titulo">{titulo || 'Proyecto sin título'}</strong>
      <span className="portada-autor">{autor || 'Sin autor'}</span>
    </div>
  );
}

function fechaModificada(reciente: Reciente): string {
  const fecha = reciente.modificado ? new Date(reciente.modificado) : new Date();
  const valida = Number.isNaN(fecha.getTime()) ? new Date() : fecha;
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(valida);
}

export function Inicio({
  recientes,
  esTauri,
  errorInicial,
  onAbrir,
  onDemo,
  onAbrirReciente,
  onQuitarReciente,
  onAgregarReciente,
  onCrear,
  vistaInicial = 'activo',
}: Props) {
  const [consulta, setConsulta] = useState('');
  const [filtros, setFiltros] = useState<FiltrosProyectos>(() => cargarFiltros());
  const [filtrosAbiertos, setFiltrosAbiertos] = useState(false);
  const [colecciones, setColecciones] = useState<Coleccion[]>(() => leerColecciones());
  const [coleccionActiva, setColeccionActiva] = useState<string | null>(null);
  const [modalColeccion, setModalColeccion] = useState(false);
  const [proyectoOrigenColeccion, setProyectoOrigenColeccion] = useState<Reciente | null>(null);
  const [gestionColecciones, setGestionColecciones] = useState(false);
  const [accionesProyecto, setAccionesProyecto] = useState<Reciente | null>(null);
  const [modalPlantilla, setModalPlantilla] = useState(false);
  const [plantillaAEliminar, setPlantillaAEliminar] = useState<string | null>(null);
  const [errorPlantillas, setErrorPlantillas] = useState<string | null>(null);
  const [papelera, setPapelera] = useState<EntradaPapelera[]>(() => leerPapelera());
  const [proyectoAPapelera, setProyectoAPapelera] = useState<Reciente | null>(null);
  const [papeleraAEliminar, setPapeleraAEliminar] = useState<EntradaPapelera | null>(null);
  const [errorPapelera, setErrorPapelera] = useState<string | null>(null);
  const [procesandoPapelera, setProcesandoPapelera] = useState(false);
  const [errorOrganizacion, setErrorOrganizacion] = useState<string | null>(null);
  const [vistaInicio, setVistaInicio] = useState<'activo' | 'plantillas' | 'papelera'>(
    vistaInicial,
  );
  const [vista, setVista] = useState<'cuadricula' | 'lista'>('cuadricula');
  const [modalAbierto, setModalAbierto] = useState(false);
  const [nuevoNombre, setNuevoNombre] = useState('');
  const [nuevoTitulo, setNuevoTitulo] = useState('');
  const [nuevoAutor, setNuevoAutor] = useState('');
  const [nuevoIdioma, setNuevoIdioma] = useState('es');
  const [carpetaPadre, setCarpetaPadre] = useState('');
  const [color, setColor] = useState(COLORES[0]);
  const [notasIniciales, setNotasIniciales] = useState(true);
  const [plantilla, setPlantilla] = useState('');
  const [plantillas, setPlantillas] = useState<PlantillaDto[]>([]);
  const [nombreEditado, setNombreEditado] = useState(false);
  const [guardando, setGuardando] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [exito, setExito] = useState<string | null>(null);
  const [plegado, setPlegado] = useState(false);
  const fondoRef = useRef<HTMLDivElement>(null);
  const modalRef = useRef<HTMLFormElement>(null);
  const focoPrevioRef = useRef<HTMLElement | null>(null);
  const crearRef = useRef<HTMLButtonElement>(null);

  const hayModalInicio =
    filtrosAbiertos ||
    modalColeccion ||
    gestionColecciones ||
    Boolean(accionesProyecto) ||
    modalPlantilla ||
    Boolean(proyectoAPapelera) ||
    Boolean(papeleraAEliminar);

  useEffect(() => {
    const fondo = fondoRef.current;
    if (!fondo) {
      return;
    }
    if (hayModalInicio) {
      fondo.inert = true;
    }
    return () => {
      fondo.inert = false;
    };
  }, [hayModalInicio]);

  useEffect(() => {
    listarPlantillas()
      .then(setPlantillas)
      .catch((fallo: unknown) => {
        setPlantillas([]);
        setErrorPlantillas(
          `No se pudieron leer las plantillas: ${errorDe(fallo).mensaje}`,
        );
      });
  }, []);

  const autores = useMemo(() => autoresRecientes(recientes), [recientes]);
  const rutasColeccionActiva = useMemo(() => {
    if (!coleccionActiva) {
      return null;
    }
    return colecciones.find((coleccion) => coleccion.id === coleccionActiva)?.rutas ?? [];
  }, [coleccionActiva, colecciones]);
  const filtrados = useMemo(
    () =>
      filtrarRecientes(
        recientes,
        filtros,
        consulta,
        rutasColeccionActiva,
      ),
    [coleccionActiva, colecciones, consulta, filtros, recientes],
  );
  const cantidadFiltros = filtrosActivos(filtros);
  const coleccionSinRutas =
    coleccionActiva !== null && (rutasColeccionActiva?.length ?? 0) === 0;
  const guardarListaColecciones = (siguiente: Coleccion[]) => {
    setColecciones(siguiente);
    guardarColecciones(siguiente);
  };
  const guardarAsignacion = (ruta: string, ids: string[]): boolean => {
    const seleccionadas = new Set(ids);
    const excedida = colecciones.find(
      (coleccion) =>
        seleccionadas.has(coleccion.id) &&
        !coleccion.rutas.includes(ruta) &&
        coleccion.rutas.length >= 1000,
    );
    if (excedida) {
      setErrorOrganizacion(
        `La colección «${excedida.nombre}» ya tiene 1.000 proyectos.`,
      );
      return false;
    }
    guardarListaColecciones(
      colecciones.map((coleccion) => ({
        ...coleccion,
        rutas: seleccionadas.has(coleccion.id)
          ? Array.from(new Set([...coleccion.rutas, ruta]))
          : coleccion.rutas.filter((actual) => actual !== ruta),
      })),
    );
    setAccionesProyecto(null);
    setErrorOrganizacion(null);
    setExito('Colecciones del proyecto actualizadas.');
    return true;
  };
  const enfocarOpcionesProyecto = (titulo: string) => {
    window.setTimeout(() => {
      const boton = Array.from(document.querySelectorAll<HTMLElement>('.tarjeta-menu')).find(
        (elemento) => elemento.getAttribute('aria-label') === `Opciones de ${titulo}`,
      );
      boton?.focus();
    }, 50);
  };

  const enfocarPestanaActiva = () => {
    window.setTimeout(() => {
      (document.querySelector('.tabs-inicio [aria-selected="true"]') as HTMLElement | null)?.focus();
    }, 50);
  };

  const moverAPapelera = async (reciente: Reciente) => {
    setErrorPapelera(null);
    if (papelera.length >= 100) {
      setErrorPapelera('La papelera ya tiene 100 proyectos. Restaura o elimina alguno.');
      return;
    }
    setProcesandoPapelera(true);
    try {
      const entrada = await moverProyectoAPapelera(
        reciente.ruta,
        reciente.titulo,
        reciente.autor ?? null,
        reciente.color ?? null,
        reciente.palabras ?? 0,
      );
      const siguiente = [entrada, ...papelera];
      setPapelera(siguiente);
      guardarPapelera(siguiente);
      guardarListaColecciones(
        colecciones.map((coleccion) => ({
          ...coleccion,
          rutas: coleccion.rutas.filter((ruta) => ruta !== reciente.ruta),
        })),
      );
      onQuitarReciente(reciente.ruta);
      setProyectoAPapelera(null);
      setExito(`Proyecto «${reciente.titulo}» movido a la papelera.`);
      enfocarPestanaActiva();
    } catch (fallo) {
      setErrorPapelera(errorDe(fallo).mensaje);
    } finally {
      setProcesandoPapelera(false);
    }
  };

  const restaurarPapelera = async (entrada: EntradaPapelera) => {
    if (procesandoPapelera) {
      return;
    }
    setErrorPapelera(null);
    setProcesandoPapelera(true);
    try {
      const ruta = await restaurarProyectoDePapelera(entrada.rutaPapelera);
      const siguiente = papelera.filter((item) => item.rutaPapelera !== entrada.rutaPapelera);
      setPapelera(siguiente);
      guardarPapelera(siguiente);
      onAgregarReciente({
        ruta,
        titulo: entrada.titulo,
        autor: entrada.autor ?? undefined,
        palabras: entrada.palabras ?? 0,
        color: entrada.color ?? undefined,
        modificado: new Date().toISOString(),
      });
      setExito(`Proyecto «${entrada.titulo}» restaurado en Mis Proyectos.`);
      enfocarPestanaActiva();
    } catch (fallo) {
      setErrorPapelera(errorDe(fallo).mensaje);
    } finally {
      setProcesandoPapelera(false);
    }
  };

  const purgarPapelera = async (entrada: EntradaPapelera) => {
    setErrorPapelera(null);
    setProcesandoPapelera(true);
    try {
      await purgarProyectoDePapelera(entrada.rutaPapelera);
      const siguiente = papelera.filter((item) => item.rutaPapelera !== entrada.rutaPapelera);
      setPapelera(siguiente);
      guardarPapelera(siguiente);
      setPapeleraAEliminar(null);
      setExito(`Proyecto «${entrada.titulo}» eliminado definitivamente.`);
      enfocarPestanaActiva();
    } catch (fallo) {
      setErrorPapelera(errorDe(fallo).mensaje);
    } finally {
      setProcesandoPapelera(false);
    }
  };

  useEffect(() => {
    if (coleccionActiva && !colecciones.some((coleccion) => coleccion.id === coleccionActiva)) {
      setColeccionActiva(null);
    }
  }, [coleccionActiva, colecciones]);

  const totalPalabras = useMemo(
    () => recientes.reduce((total, reciente) => total + (reciente.palabras ?? 0), 0),
    [recientes],
  );
  const autorBienvenida =
    recientes.find((reciente) => reciente.autor?.trim())?.autor?.trim() || 'escritor';

  const abrirModal = (tituloInicial = '', plantillaSlug = '') => {
    focoPrevioRef.current =
      document.activeElement instanceof HTMLElement ? document.activeElement : crearRef.current;
    setModalAbierto(true);
    setNuevoNombre(tituloInicial ? slugProyecto(tituloInicial) : '');
    setNuevoTitulo(tituloInicial);
    setNuevoAutor('');
    setNuevoIdioma('es');
    setCarpetaPadre('');
    setColor(colorDesdeTitulo(tituloInicial || 'writer'));
    setNotasIniciales(true);
    setPlantilla(plantillaSlug);
    setNombreEditado(false);
    setError(null);
    setExito(null);
  };

  const cerrarModal = () => {
    if (!guardando) {
      setModalAbierto(false);
      setError(null);
      window.requestAnimationFrame(() => focoPrevioRef.current?.focus());
    }
  };

  useEffect(() => {
    if (!modalAbierto) {
      return;
    }
    const fondo = fondoRef.current;
    if (fondo) {
      fondo.inert = true;
    }
    const manejarTeclado = (evento: KeyboardEvent) => {
      if (evento.key === 'Escape' && !guardando) {
        evento.preventDefault();
        cerrarModal();
        return;
      }
      if (evento.key !== 'Tab' || guardando) {
        return;
      }
      const enfocables = modalRef.current
        ? Array.from(
            modalRef.current.querySelectorAll<HTMLElement>(
              'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
            ),
          ).filter((elemento) => elemento.offsetParent !== null)
        : [];
      if (enfocables.length === 0) {
        return;
      }
      const primero = enfocables[0];
      const ultimo = enfocables[enfocables.length - 1];
      const activo = document.activeElement;
      if (!modalRef.current?.contains(activo)) {
        evento.preventDefault();
        primero.focus();
      } else if (evento.shiftKey && activo === primero) {
        evento.preventDefault();
        ultimo.focus();
      } else if (!evento.shiftKey && activo === ultimo) {
        evento.preventDefault();
        primero.focus();
      }
    };
    document.addEventListener('keydown', manejarTeclado);
    return () => {
      document.removeEventListener('keydown', manejarTeclado);
      if (fondo) {
        fondo.inert = false;
      }
    };
  }, [modalAbierto, guardando]);

  const actualizarTitulo = (titulo: string) => {
    setNuevoTitulo(titulo);
    if (!nombreEditado) {
      setNuevoNombre(slugProyecto(titulo || 'proyecto'));
    }
  };

  const enviar = async (evento: { preventDefault: () => void }, abrir: boolean) => {
    evento.preventDefault();
    const titulo = nuevoTitulo.trim();
    const nombre = nuevoNombre.trim();
    if (!titulo) {
      setError('Escribe un título para el libro.');
      return;
    }
    if (!carpetaPadre.trim()) {
      setError('Elige la carpeta donde se creará el proyecto.');
      return;
    }
    if (
      !nombre ||
      nombre === '.' ||
      nombre === '..' ||
      /[\\/:*?"<>|]/.test(nombre) ||
      nombre.startsWith(' ')
    ) {
      setError('El nombre del proyecto no es válido como nombre de carpeta.');
      return;
    }
    setGuardando(true);
    setError(null);
    try {
      await onCrear(
        {
          ruta: `${carpetaPadre.replace(/[\\/]+$/, '')}/${nombre}`,
          nombre,
          titulo,
          autor: nuevoAutor.trim() || null,
          idioma: nuevoIdioma,
          notasIniciales,
          color,
          plantilla: plantilla || null,
        },
        abrir,
      );
      setModalAbierto(false);
      setError(null);
      setConsulta('');
      setExito(
        abrir
          ? null
          : `Proyecto «${titulo}» creado y guardado en recientes. Ya aparece al inicio de Mis Proyectos.`,
      );
      window.requestAnimationFrame(() => focoPrevioRef.current?.focus());
    } catch (fallo) {
      setError(errorDe(fallo).mensaje);
    } finally {
      setGuardando(false);
    }
  };

  return (
    <div className="inicio">
      <div className="inicio-fondo" ref={fondoRef}>
      <header className="inicio-topbar">
        <div className="inicio-marca">
          <span className="inicio-logo" aria-hidden="true">
            ✒
          </span>
          <strong>writer</strong>
        </div>
        <div className="inicio-top-acciones">
          {!esTauri && (
            <button className="boton-inicio" onClick={onDemo}>
              Ver demo
            </button>
          )}
          <button
            className="icono-inicio"
            aria-label="Notificaciones"
            title="Notificaciones disponibles próximamente"
            disabled
          >
            ◔
          </button>
          <span className="avatar-inicio" aria-hidden="true">
            {autorBienvenida.charAt(0).toLocaleUpperCase()}
          </span>
        </div>
      </header>

      <div className="inicio-cabecera">
        <h1>Proyectos</h1>
        <div className="inicio-controles">
          <label className="buscador-inicio">
            <span aria-hidden="true">⌕</span>
            <input
              value={consulta}
              onChange={(evento) => setConsulta(evento.target.value)}
              placeholder="Buscar proyectos"
              aria-label="Buscar proyectos"
            />
          </label>
          <button className="boton-inicio" onClick={onAbrir}>
            Abrir proyecto
          </button>
          <button
            className={`boton-inicio ${cantidadFiltros > 0 ? 'activo' : ''}`}
            aria-pressed={cantidadFiltros > 0}
            title="Filtrar y ordenar proyectos"
            onClick={() => setFiltrosAbiertos(true)}
          >
            Filtros{cantidadFiltros > 0 ? ` (${cantidadFiltros})` : ''}
          </button>
          <button
            className="boton-inicio"
            onClick={() => {
              setProyectoOrigenColeccion(null);
              setModalColeccion(true);
            }}
            title="Crear una colección local de proyectos"
          >
            + Agregar una colección
          </button>
          <div className="tabs-inicio" role="tablist" aria-label="Vistas de proyectos">
            <button
              className={`tab-inicio ${vistaInicio === 'activo' ? 'activo' : ''}`}
              role="tab"
              aria-selected={vistaInicio === 'activo'}
              onClick={() => setVistaInicio('activo')}
            >
              Activo
            </button>
            <button
              className={`tab-inicio ${vistaInicio === 'plantillas' ? 'activo' : ''}`}
              role="tab"
              aria-selected={vistaInicio === 'plantillas'}
              onClick={() => setVistaInicio('plantillas')}
            >
              Plantillas
            </button>
            <button
              className={`tab-inicio ${vistaInicio === 'papelera' ? 'activo' : ''}`}
              role="tab"
              aria-selected={vistaInicio === 'papelera'}
              onClick={() => setVistaInicio('papelera')}
            >
              Papelera
            </button>
          </div>
        </div>
      </div>

      {errorInicial && (
        <pre className="banner-error inicio-error" role="alert">
          {errorInicial}
        </pre>
      )}
      {exito && (
        <div className="banner-info inicio-exito" role="status" aria-live="polite">
          <span>{exito}</span>
          <button className="boton-inicio" onClick={() => setExito(null)}>
            Cerrar
          </button>
        </div>
      )}
      <div className="inicio-cuerpo">
        <main className="inicio-proyectos">
          {vistaInicio === 'activo' ? (
          <section className="mis-proyectos">
            <header className="mis-proyectos-cabecera">
              <button
                className="plegar-proyectos"
                aria-label="Mis proyectos"
                aria-expanded={!plegado}
                onClick={() => setPlegado((valor) => !valor)}
              >
                <span className={plegado ? 'chevron-plegado' : ''} aria-hidden="true">
                  ⌄
                </span>{' '}
                <strong>Mis Proyectos</strong>
              </button>
              <div className="mis-proyectos-acciones">
                <span>
                  {filtrados.length} proyecto{filtrados.length === 1 ? '' : 's'}
                </span>
                <button
                  className={vista === 'lista' ? 'vista-activa' : ''}
                  aria-label="Vista de lista"
                  aria-pressed={vista === 'lista'}
                  title="Vista de lista"
                  onClick={() => setVista('lista')}
                >
                  ☰
                </button>
                <button
                  className={vista === 'cuadricula' ? 'vista-activa' : ''}
                  aria-label="Vista de cuadrícula"
                  aria-pressed={vista === 'cuadricula'}
                  title="Vista de cuadrícula"
                  onClick={() => setVista('cuadricula')}
                >
                  ▦
                </button>
                <button
                  className="icono-inicio"
                  aria-label="Más opciones"
                  title="Más opciones disponibles próximamente"
                  disabled
                >
                  …
                </button>
              </div>
            </header>

            {colecciones.length > 0 && (
              <div className="colecciones-barra" role="group" aria-label="Colecciones de proyectos">
                <button
                  className={coleccionActiva === null ? 'activo' : ''}
                  aria-pressed={coleccionActiva === null}
                  onClick={() => setColeccionActiva(null)}
                >
                  Todas
                </button>
                {colecciones.map((coleccion) => (
                  <button
                    key={coleccion.id}
                    className={coleccionActiva === coleccion.id ? 'activo' : ''}
                    aria-pressed={coleccionActiva === coleccion.id}
                    onClick={() => setColeccionActiva(coleccion.id)}
                  >
                    <span
                      className="punto-color"
                      style={{ background: coleccion.color }}
                      aria-hidden="true"
                    />
                    {coleccion.nombre}
                    <small>{coleccion.rutas.length}</small>
                  </button>
                ))}
                <button
                  className="gestionar-colecciones"
                  onClick={() => setGestionColecciones(true)}
                >
                  Gestionar
                </button>
              </div>
            )}

            {errorOrganizacion && !accionesProyecto && (
              <p className="dabble-error colecciones-error" role="alert">
                {errorOrganizacion}
              </p>
            )}

            {!plegado && (
              <>
            <div className={`proyectos ${vista === 'lista' ? 'proyectos-lista' : ''}`}>
              <div className="crear-columna">
                <button className="crear-tarjeta" ref={crearRef} onClick={() => abrirModal()}>
                  <span className="crear-contenido">
                    <span className="crear-mas" aria-hidden="true">
                      +
                    </span>
                    <span>Crear nuevo proyecto</span>
                  </span>
                </button>
                <button
                  className="crear-plantilla"
                  onClick={() => abrirModal()}
                  title="Elige una plantilla en el asistente de creación"
                >
                  <svg
                    className="icono-plantilla"
                    viewBox="0 0 16 16"
                    aria-hidden="true"
                  >
                    <path
                      d="M3 2.5h5l3 3v8H3z"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1.2"
                      strokeLinejoin="round"
                    />
                    <path
                      d="M8 2.5v3h3"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="1.2"
                      strokeLinejoin="round"
                    />
                  </svg>
                  Crear desde plantilla
                </button>
              </div>
              {filtrados.map((reciente) => (
                <div key={reciente.ruta} className="tarjeta-proyecto">
                  <button
                    className="tarjeta-abrir"
                    aria-label={`Abrir ${reciente.titulo}${
                      reciente.autor ? ` de ${reciente.autor}` : ''
                    }`}
                    onClick={() => onAbrirReciente(reciente.ruta)}
                  >
                    <Portada
                      color={reciente.color ?? colorDesdeTitulo(reciente.titulo)}
                      titulo={reciente.titulo}
                      autor={reciente.autor ?? ''}
                      compacta
                    />
                    <span className="tarjeta-datos">
                      <strong>{reciente.titulo}</strong>
                      <span className="tarjeta-meta">
                        <span>Modificado el {fechaModificada(reciente)}</span>
                      </span>
                    </span>
                  </button>
                  <button
                    className="tarjeta-menu"
                    aria-label={`Opciones de ${reciente.titulo}`}
                    title="Opciones del proyecto"
                    onClick={() => {
                      setErrorOrganizacion(null);
                      setAccionesProyecto(reciente);
                    }}
                  >
                    ⋯
                  </button>
                </div>
              ))}
              {filtrados.length === 0 && (
                <div className="proyectos-vacio">
                  <strong>
                    {consulta
                      ? 'Sin coincidencias'
                      : cantidadFiltros > 0
                        ? 'Ningún proyecto coincide'
                        : coleccionSinRutas
                          ? 'Esta colección está vacía'
                          : 'No proyectos'}
                  </strong>
                  <p>
                    {consulta || cantidadFiltros > 0 || coleccionActiva
                      ? 'Prueba con otra búsqueda, colección o limpia los filtros.'
                      : 'Crea tu primer proyecto y comienza a escribir tu obra maestra.'}
                  </p>
                  {(consulta || cantidadFiltros > 0 || coleccionActiva) && (
                    <button
                      className="boton-inicio"
                      onClick={() => {
                        setConsulta('');
                        setColeccionActiva(null);
                        const limpios = {
                          ...filtros,
                          autor: '',
                          periodo: 'todos' as const,
                          minimoPalabras: 0,
                          orden: 'recientes' as const,
                        };
                        setFiltros(limpios);
                        guardarFiltros(limpios);
                      }}
                    >
                      Ver todos los proyectos
                    </button>
                  )}
                  {!consulta && !coleccionActiva && (
                    <button className="boton-inicio" onClick={onAbrir}>
                      Abrir proyecto existente
                    </button>
                  )}
                </div>
              )}
            </div>

              </>
            )}
          </section>
          ) : vistaInicio === 'plantillas' ? (
            <section className="mis-proyectos panel-plantillas">
              <header className="mis-proyectos-cabecera panel-plantillas-cabecera">
                <div>
                  <h2>Plantillas</h2>
                  <p>Estructuras reutilizables para empezar un proyecto.</p>
                </div>
                <button
                  className="boton-inicio"
                  onClick={() => setModalPlantilla(true)}
                >
                  + Nueva plantilla
                </button>
              </header>
              {errorPlantillas && (
                <p className="dabble-error" role="alert">
                  {errorPlantillas}
                </p>
              )}
              {plantillas.length === 0 ? (
                <div className="proyectos-vacio plantillas-vacio">
                  <strong>No hay plantillas</strong>
                  <p>Crea una estructura para reutilizarla en tus próximos proyectos.</p>
                  <button className="boton-inicio" onClick={() => setModalPlantilla(true)}>
                    Crear la primera plantilla
                  </button>
                </div>
              ) : (
                <div className="plantillas-grid">
                  {plantillas.map((opcion) => (
                    <article className="plantilla-tarjeta" key={opcion.slug}>
                      <header>
                        <h3>{opcion.nombre}</h3>
                        <small>{opcion.slug}</small>
                      </header>
                      <p>{opcion.descripcion || 'Sin descripción.'}</p>
                      <dl>
                        <div><dt>Libros</dt><dd>{opcion.estructura.libros}</dd></div>
                        <div><dt>Caps.</dt><dd>{opcion.estructura.capitulosPorLibro}</dd></div>
                        <div><dt>Escenas</dt><dd>{opcion.estructura.escenasPorCapitulo}</dd></div>
                        <div><dt>Notas</dt><dd>{opcion.notasIniciales.length}</dd></div>
                      </dl>
                      <div className="plantilla-acciones">
                        <button
                          className="dabble-boton-primario"
                          onClick={() => abrirModal('', opcion.slug)}
                        >
                          Usar plantilla
                        </button>
                        <button
                          className={`dabble-boton ${plantillaAEliminar === opcion.slug ? 'peligro' : ''}`}
                          onClick={() => {
                            if (plantillaAEliminar !== opcion.slug) {
                              setPlantillaAEliminar(opcion.slug);
                              return;
                            }
                            setPlantillaAEliminar(null);
                            void eliminarPlantilla(opcion.slug)
                              .then(() => {
                                setPlantillas((actual) =>
                                  actual.filter((item) => item.slug !== opcion.slug),
                                );
                                setErrorPlantillas(null);
                                setExito(`Plantilla «${opcion.nombre}» eliminada.`);
                                enfocarPestanaActiva();
                              })
                              .catch((fallo: unknown) => setErrorPlantillas(errorDe(fallo).mensaje));
                          }}
                          aria-label={
                            plantillaAEliminar === opcion.slug
                              ? `Confirmar eliminación de ${opcion.nombre}`
                              : `Eliminar plantilla ${opcion.nombre}`
                          }
                        >
                          {plantillaAEliminar === opcion.slug ? 'Confirmar' : 'Eliminar'}
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              )}
            </section>
          ) : (
            <section className="mis-proyectos panel-papelera">
              <header className="mis-proyectos-cabecera panel-plantillas-cabecera">
                <div>
                  <h2>Papelera</h2>
                  <p>Proyectos retirados de Mis Proyectos. Se pueden restaurar.</p>
                </div>
              </header>
              {errorPapelera && !papeleraAEliminar && !proyectoAPapelera && (
                <p className="dabble-error" role="alert">{errorPapelera}</p>
              )}
              {papelera.length === 0 ? (
                <div className="proyectos-vacio plantillas-vacio">
                  <strong>La papelera está vacía</strong>
                  <p>Los proyectos que muevas aquí se podrán restaurar.</p>
                </div>
              ) : (
                <div className="papelera-lista">
                  {papelera.map((entrada) => (
                    <article className="papelera-item" key={entrada.rutaPapelera}>
                      <div>
                        <h3>{entrada.titulo}</h3>
                        <p>{entrada.rutaOriginal}</p>
                        <small>
                          Eliminado el {new Date(entrada.eliminado).toLocaleDateString('es')}
                        </small>
                      </div>
                      <div className="papelera-acciones">
                        <button
                          className="dabble-boton-primario"
                          disabled={procesandoPapelera}
                          aria-label={`Restaurar «${entrada.titulo}»`}
                          onClick={() => void restaurarPapelera(entrada)}
                        >
                          Restaurar
                        </button>
                        <button
                          className="dabble-boton peligro"
                          disabled={procesandoPapelera}
                          aria-label={`Eliminar definitivamente «${entrada.titulo}»`}
                          onClick={() => {
                            setErrorPapelera(null);
                            setPapeleraAEliminar(entrada);
                          }}
                        >
                          Eliminar definitivamente
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              )}
            </section>
          )}
        </main>

        <aside className="inicio-sidebar">
          <section className="tarjeta-bienvenida">
            <p>Bienvenido, {autorBienvenida}</p>
            <span className="avatar-bienvenida" aria-hidden="true">
              {autorBienvenida.charAt(0).toLocaleUpperCase()}
            </span>
            <div className="estadisticas">
              <span title="Últimos 8 proyectos abiertos">
                <strong>{recientes.length}</strong>
                <small>recientes</small>
              </span>
              <span>
                <strong>{totalPalabras.toLocaleString('es')}</strong>
                <small>palabras escritas</small>
              </span>
            </div>
          </section>
          <div className="inicio-sidebar-reserva" aria-hidden="true" />
        </aside>
      </div>
      </div>

      {modalColeccion && (
        <ModalColeccion
          limiteAlcanzado={colecciones.length >= 50}
          onCrear={(nombre, color) => {
            const coleccion = nuevaColeccion(nombre, color);
            guardarListaColecciones([...colecciones, coleccion]);
            setModalColeccion(false);
            const origen = proyectoOrigenColeccion;
            setProyectoOrigenColeccion(null);
            if (origen) {
              enfocarOpcionesProyecto(origen.titulo);
            }
            setExito(`Colección «${coleccion.nombre}» creada.`);
          }}
          onCerrar={() => {
            const origen = proyectoOrigenColeccion;
            setModalColeccion(false);
            setProyectoOrigenColeccion(null);
            if (origen) {
              enfocarOpcionesProyecto(origen.titulo);
            }
          }}
        />
      )}

      {gestionColecciones && (
        <ModalGestionColecciones
          colecciones={colecciones}
          onGuardar={(siguiente) => {
            guardarListaColecciones(siguiente);
            setGestionColecciones(false);
            if (siguiente.length === 0) {
              enfocarPestanaActiva();
            }
            setExito('Colecciones actualizadas.');
          }}
          onCerrar={() => setGestionColecciones(false)}
        />
      )}

      {accionesProyecto && (
        <ModalAccionesProyecto
          reciente={accionesProyecto}
          colecciones={colecciones}
          onCerrar={() => {
            setAccionesProyecto(null);
            setErrorOrganizacion(null);
          }}
          onAbrir={() => onAbrirReciente(accionesProyecto.ruta)}
          onGuardarColecciones={(ids) => guardarAsignacion(accionesProyecto.ruta, ids)}
          error={errorOrganizacion}
          onCrearColeccion={() => {
            setAccionesProyecto(null);
            setProyectoOrigenColeccion(accionesProyecto);
            setModalColeccion(true);
          }}
          onMoverPapelera={() => {
            setAccionesProyecto(null);
            setErrorPapelera(null);
            setProyectoAPapelera(accionesProyecto);
          }}
        />
      )}

      {proyectoAPapelera && (
        <ModalConfirmar
          titulo="Mover a la papelera"
          descripcion={`«${proyectoAPapelera.titulo}» se moverá a .writer-papelera y podrá restaurarse después.`}
          confirmar="Mover a la papelera"
          error={errorPapelera}
          procesando={procesandoPapelera}
          onConfirmar={() => void moverAPapelera(proyectoAPapelera)}
          onCerrar={() => {
            if (!procesandoPapelera) {
              const titulo = proyectoAPapelera.titulo;
              setProyectoAPapelera(null);
              setErrorPapelera(null);
              enfocarOpcionesProyecto(titulo);
            }
          }}
        />
      )}

      {papeleraAEliminar && (
        <ModalConfirmar
          titulo="Eliminar definitivamente"
          descripcion={`«${papeleraAEliminar.titulo}» se borrará del disco y no se podrá restaurar.`}
          confirmar="Eliminar definitivamente"
          peligro
          error={errorPapelera}
          procesando={procesandoPapelera}
          onConfirmar={() => void purgarPapelera(papeleraAEliminar)}
          onCerrar={() => {
            if (!procesandoPapelera) {
              setPapeleraAEliminar(null);
              setErrorPapelera(null);
            }
          }}
        />
      )}

      {modalPlantilla && (
        <ModalPlantilla
          onGuardar={async (nueva) => {
            const guardada = await guardarPlantilla(nueva);
            setPlantillas((actual) => [...actual, guardada]);
            setErrorPlantillas(null);
            setModalPlantilla(false);
            setExito(`Plantilla «${guardada.nombre}» creada.`);
          }}
          onCerrar={() => setModalPlantilla(false)}
        />
      )}

      {filtrosAbiertos && (
        <ModalFiltros
          filtros={filtros}
          autores={autores}
          onAplicar={(nuevos) => {
            setFiltros(nuevos);
            guardarFiltros(nuevos);
            setFiltrosAbiertos(false);
          }}
          onCerrar={() => setFiltrosAbiertos(false)}
        />
      )}

      {modalAbierto && (
        <div className="modal-fondo">
          <form
            ref={modalRef}
            className="modal-proyecto"
            role="dialog"
            aria-modal="true"
            aria-labelledby="crear-proyecto-titulo"
            aria-describedby="crear-proyecto-descripcion"
            noValidate
            onSubmit={(evento) => void enviar(evento, true)}
          >
            <p id="crear-proyecto-descripcion" className="sr-only">
              Elige el nombre, la carpeta, el título y los datos iniciales del proyecto. Pulsa
              Guardar para dejarlo en recientes o Guardar y abrir para empezar a escribir.
            </p>
            <div className="modal-proyecto-cabecera">
              <h2 id="crear-proyecto-titulo">Crear nuevo proyecto</h2>
              <button
                type="button"
                className="cerrar-modal"
                aria-label="Cerrar"
                onClick={cerrarModal}
                disabled={guardando}
              >
                ×
              </button>
            </div>
            <div className="modal-proyecto-cuerpo">
              <nav className="modal-pasos" aria-label="Secciones del proyecto">
                <button
                  type="button"
                  className="paso activo"
                  onClick={() => document.getElementById('campo-nombre-proyecto')?.focus()}
                >
                  Detalles
                </button>
                <button
                  type="button"
                  className="paso"
                  onClick={() => document.getElementById('campo-titulo-libro')?.focus()}
                >
                  Detalles del libro
                </button>
              </nav>
              <div className="modal-formulario">
                <label className="campo-ancho" htmlFor="campo-nombre-proyecto">
                  Nombre del proyecto
                  <span className="nota-campo">Nombre de la carpeta del proyecto</span>
                  <input
                    id="campo-nombre-proyecto"
                    aria-label="Nombre del proyecto"
                    autoFocus
                    required
                    autoComplete="off"
                    value={nuevoNombre}
                    onChange={(evento) => {
                      setNuevoNombre(evento.target.value);
                      setNombreEditado(true);
                    }}
                    placeholder="mi-novela"
                  />
                </label>

                <div className="modal-columnas">
                  <div className="modal-portada">
                    <Portada color={color} titulo={nuevoTitulo} autor={nuevoAutor} />
                    <div className="portada-acciones">
                      <span className="nota-campo">Color de portada</span>
                      <div className="paleta" role="group" aria-label="Color de portada">
                        {COLORES.map((tono) => (
                          <button
                            key={tono}
                            type="button"
                            className={`tono ${tono === color ? 'activo' : ''}`}
                            style={{ background: tono }}
                            aria-label={NOMBRES_COLOR[tono] ?? 'Color de portada'}
                            aria-pressed={tono === color}
                            tabIndex={tono === color ? 0 : -1}
                            onClick={() => setColor(tono)}
                            onKeyDown={(evento) => {
                              if (!['ArrowRight', 'ArrowLeft', 'Home', 'End'].includes(evento.key)) {
                                return;
                              }
                              evento.preventDefault();
                              const actual = COLORES.indexOf(tono);
                              const siguiente =
                                evento.key === 'Home'
                                  ? 0
                                  : evento.key === 'End'
                                    ? COLORES.length - 1
                                    : evento.key === 'ArrowRight'
                                      ? (actual + 1) % COLORES.length
                                      : (actual - 1 + COLORES.length) % COLORES.length;
                              setColor(COLORES[siguiente]);
                              const botones = evento.currentTarget.parentElement?.querySelectorAll<
                                HTMLButtonElement
                              >('button.tono');
                              botones?.[siguiente]?.focus();
                            }}
                          />
                        ))}
                      </div>
                    </div>
                  </div>

                  <div className="modal-campos">
                    <label>
                      Título
                      <input
                        id="campo-titulo-libro"
                        aria-label="Título"
                        required
                        autoComplete="off"
                        value={nuevoTitulo}
                        onChange={(evento) => actualizarTitulo(evento.target.value)}
                        placeholder="Libro sin título"
                      />
                    </label>
                    <label>
                      Autor
                      <input
                        value={nuevoAutor}
                        onChange={(evento) => setNuevoAutor(evento.target.value)}
                        placeholder="Tu nombre"
                      />
                    </label>
                    <label>
                      Idioma
                      <select
                        value={nuevoIdioma}
                        onChange={(evento) => setNuevoIdioma(evento.target.value)}
                      >
                        <option value="es">Español</option>
                        <option value="en">English</option>
                      </select>
                    </label>
                    <div className="campo-carpeta">
                      <label htmlFor="campo-carpeta-padre">Carpeta padre</label>
                      <span className="modal-ruta">
                        <input
                          id="campo-carpeta-padre"
                          readOnly
                          aria-required="true"
                          value={carpetaPadre}
                          placeholder="Elige una carpeta…"
                        />
                        <button
                          type="button"
                          className="boton-inicio"
                          onClick={() => {
                            elegirCarpetaCreacion()
                              .then((carpeta) => {
                                if (carpeta) {
                                  setCarpetaPadre(carpeta);
                                  setError(null);
                                }
                              })
                              .catch((fallo: unknown) => setError(errorDe(fallo).mensaje));
                          }}
                        >
                          Examinar…
                        </button>
                      </span>
                    </div>
                    <p className="nota-campo">
                      {carpetaPadre
                        ? `Se creará en ${carpetaPadre.replace(/[\\/]+$/, '')}/${
                            nuevoNombre || slugProyecto(nuevoTitulo || 'proyecto')
                          }`
                        : 'Elige la carpeta donde se creará el proyecto.'}
                    </p>
                    {plantilla ? (
                      <p className="nota-campo">
                        Las notas iniciales las define la plantilla seleccionada.
                      </p>
                    ) : (
                      <label className="modal-check">
                        <input
                          type="checkbox"
                          checked={notasIniciales}
                          onChange={(evento) => setNotasIniciales(evento.target.checked)}
                        />
                        Crear notas iniciales de personajes, tramas y lugares
                      </label>
                    )}
                    {plantillas.length > 0 && (
                      <label className="campo-carpeta">
                        <span>Plantilla (opcional)</span>
                        <select
                          value={plantilla}
                          onChange={(evento) => setPlantilla(evento.target.value)}
                          aria-label="Plantilla del proyecto"
                        >
                          <option value="">Sin plantilla</option>
                          {plantillas.map((opcion) => (
                            <option key={opcion.slug} value={opcion.slug}>
                              {opcion.nombre}
                            </option>
                          ))}
                        </select>
                        <span className="nota-campo">
                          La plantilla genera estructura y notas; no copia historial.
                        </span>
                      </label>
                    )}
                  </div>
                </div>
              </div>
            </div>
            {error && (
              <pre className="banner-error modal-error" role="alert">
                {error}
              </pre>
            )}
            <div className="modal-proyecto-pie">
              <button
                type="button"
                className="boton-inicio"
                disabled={guardando}
                onClick={(evento) => void enviar(evento, false)}
              >
                Guardar
              </button>
              <button type="submit" className="boton-primario" disabled={guardando}>
                {guardando ? 'Creando…' : 'Guardar y abrir'}
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}
