import { useEffect, useMemo, useRef, useState } from 'react';
import { elegirCarpetaCreacion, errorDe, listarPlantillas } from './api';
import type { PlantillaDto } from './types/PlantillaDto';
import { colorDesdeTitulo, slugProyecto, type Reciente } from './recientes';
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
  onCrear: (datos: DatosNuevoProyecto, abrir: boolean) => Promise<void>;
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
  onCrear,
}: Props) {
  const [consulta, setConsulta] = useState('');
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

  useEffect(() => {
    listarPlantillas()
      .then(setPlantillas)
      .catch(() => setPlantillas([]));
  }, []);

  const filtrados = useMemo(() => {
    const termino = consulta.trim().toLocaleLowerCase();
    if (!termino) {
      return recientes;
    }
    return recientes.filter(
      (reciente) =>
        reciente.titulo.toLocaleLowerCase().includes(termino) ||
        reciente.ruta.toLocaleLowerCase().includes(termino),
    );
  }, [consulta, recientes]);

  const totalPalabras = useMemo(
    () => recientes.reduce((total, reciente) => total + (reciente.palabras ?? 0), 0),
    [recientes],
  );
  const autorBienvenida =
    recientes.find((reciente) => reciente.autor?.trim())?.autor?.trim() || 'escritor';

  const abrirModal = (tituloInicial = '') => {
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
    setPlantilla('');
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
          <button className="boton-inicio" disabled title="Filtros disponibles próximamente">
            Filtros
          </button>
          <button className="boton-inicio" disabled title="Colecciones disponibles próximamente">
            + Agregar una colección
          </button>
          <div className="tabs-inicio" role="tablist" aria-label="Vistas de proyectos">
            <button className="tab-inicio activo" role="tab" aria-selected="true">
              Activo
            </button>
            <button
              className="tab-inicio"
              role="tab"
              aria-selected="false"
              disabled
              title="Plantillas próximamente"
            >
              Plantillas
            </button>
            <button
              className="tab-inicio"
              role="tab"
              aria-selected="false"
              disabled
              title="Papelera próximamente"
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
                <button
                  key={reciente.ruta}
                  className="tarjeta-proyecto"
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
                      <span className="tarjeta-mas" aria-hidden="true">
                        ⋯
                      </span>
                    </span>
                  </span>
                </button>
              ))}
              {filtrados.length === 0 && (
                <div className="proyectos-vacio">
                  <strong>{consulta ? 'Sin coincidencias' : 'No proyectos'}</strong>
                  <p>
                    {consulta
                      ? 'Prueba con otro título o ruta.'
                      : 'Crea tu primer proyecto y comienza a escribir tu obra maestra.'}
                  </p>
                  {!consulta && (
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
                    <label className="modal-check">
                      <input
                        type="checkbox"
                        checked={notasIniciales}
                        onChange={(evento) => setNotasIniciales(evento.target.checked)}
                      />
                      Crear notas iniciales de personajes, tramas y lugares
                    </label>
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
