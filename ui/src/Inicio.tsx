import { useMemo, useState } from 'react';
import { elegirCarpetaCreacion, errorDe } from './api';
import { colorDesdeTitulo, slugProyecto, type Reciente } from './recientes';

export type DatosNuevoProyecto = {
  ruta: string;
  nombre: string;
  titulo: string;
  autor: string | null;
  idioma: string;
  notasIniciales: boolean;
  color: string;
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
      style={{ background: `linear-gradient(155deg, ${color}, #2f2732)` }}
    >
      <span className="portada-linea" />
      <strong>{titulo || 'Proyecto sin título'}</strong>
      <span>{autor || 'Sin autor'}</span>
    </div>
  );
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
  const [nombreEditado, setNombreEditado] = useState(false);
  const [guardando, setGuardando] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
    setModalAbierto(true);
    setNuevoNombre(tituloInicial ? slugProyecto(tituloInicial) : '');
    setNuevoTitulo(tituloInicial);
    setNuevoAutor('');
    setNuevoIdioma('es');
    setCarpetaPadre('');
    setColor(colorDesdeTitulo(tituloInicial || 'writer'));
    setNotasIniciales(true);
    setNombreEditado(false);
    setError(null);
  };

  const cerrarModal = () => {
    if (!guardando) {
      setModalAbierto(false);
      setError(null);
    }
  };

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
        },
        abrir,
      );
      setModalAbierto(false);
    } catch (fallo) {
      setError(errorDe(fallo).mensaje);
    } finally {
      setGuardando(false);
    }
  };

  return (
    <div className="inicio">
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
          <button className="icono-inicio" aria-label="Notificaciones" title="Notificaciones">
            ◔
          </button>
          <button className="icono-inicio" aria-label="Ayuda" title="Ayuda">
            ?
          </button>
          <span className="avatar-inicio" aria-label={`Sesión de ${autorBienvenida}`}>
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

      <div className="inicio-cuerpo">
        <main className="inicio-proyectos">
          <section className="mis-proyectos">
            <header className="mis-proyectos-cabecera">
              <button className="plegar-proyectos" aria-label="Mis proyectos">
                <span aria-hidden="true">⌄</span> <strong>Mis Proyectos</strong>
              </button>
              <div className="mis-proyectos-acciones">
                <span>{filtrados.length} proyectos</span>
                <button
                  className={vista === 'lista' ? 'vista-activa' : ''}
                  aria-label="Vista de lista"
                  title="Vista de lista"
                  onClick={() => setVista('lista')}
                >
                  ☰
                </button>
                <button
                  className={vista === 'cuadricula' ? 'vista-activa' : ''}
                  aria-label="Vista de cuadrícula"
                  title="Vista de cuadrícula"
                  onClick={() => setVista('cuadricula')}
                >
                  ▦
                </button>
                <button className="icono-inicio" aria-label="Más opciones" title="Más opciones">
                  …
                </button>
              </div>
            </header>

            <div className={`proyectos ${vista === 'lista' ? 'proyectos-lista' : ''}`}>
              <button className="crear-tarjeta" onClick={() => abrirModal()}>
                <span className="crear-mas" aria-hidden="true">
                  +
                </span>
                <span>Crear nuevo proyecto</span>
              </button>
              {filtrados.map((reciente) => (
                <button
                  key={reciente.ruta}
                  className="tarjeta-proyecto"
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
                    <span>{reciente.autor || 'Sin autor'}</span>
                    <small>{reciente.palabras ?? 0} palabras</small>
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

            <button
              className="crear-plantilla"
              disabled
              title="Las plantillas llegarán en una próxima fase"
            >
              ◫ Crear desde plantilla
            </button>
          </section>
        </main>

        <aside className="inicio-sidebar">
          <section className="tarjeta-bienvenida">
            <p>Bienvenido, {autorBienvenida}</p>
            <span className="avatar-bienvenida" aria-hidden="true">
              {autorBienvenida.charAt(0).toLocaleUpperCase()}
            </span>
            <div className="estadisticas">
              <span>
                <strong>{recientes.length}</strong>
                <small>proyectos</small>
              </span>
              <span>
                <strong>{totalPalabras.toLocaleString('es')}</strong>
                <small>palabras escritas</small>
              </span>
            </div>
          </section>

          <section className="tarjeta-meta">
            <span className="meta-punto" aria-hidden="true" />
            <p>
              Estos son tus conteos de palabras de los últimos 30 días. Establece una meta
              diaria para seguir tu progreso.
            </p>
            <button className="boton-inicio" disabled title="Meta diaria próximamente">
              + Configurar meta diaria
            </button>
          </section>

          <section className="proyectos-personales">
            <h2>Proyectos personales</h2>
            <button className="cuaderno-ideas" onClick={() => abrirModal('Ideas')}>
              <span className="cuaderno-espiral" aria-hidden="true" />
              <strong>Ideas</strong>
              <span className="cuaderno-bombillas" aria-hidden="true">
                ◌ ◌ ◌
              </span>
            </button>
          </section>
        </aside>
      </div>

      {modalAbierto && (
        <div className="modal-fondo">
          <form
            className="modal-proyecto"
            role="dialog"
            aria-modal="true"
            aria-labelledby="crear-proyecto-titulo"
            onSubmit={(evento) => void enviar(evento, true)}
            onKeyDown={(evento) => {
              if (evento.key === 'Escape' && !guardando) {
                evento.stopPropagation();
                cerrarModal();
              }
            }}
          >
            <header className="modal-proyecto-cabecera">
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
            </header>
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
                <label className="campo-ancho">
                  Nombre del proyecto
                  <span className="nota-campo">Nombre de la carpeta del proyecto</span>
                  <input
                    id="campo-nombre-proyecto"
                    autoFocus
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
                            aria-label={`Color ${tono}`}
                            aria-pressed={tono === color}
                            onClick={() => setColor(tono)}
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
                    <label>
                      Carpeta padre
                      <span className="modal-ruta">
                        <input readOnly value={carpetaPadre} placeholder="Elige una carpeta…" />
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
                          Elegir…
                        </button>
                      </span>
                    </label>
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
                  </div>
                </div>
              </div>
            </div>
            {error && (
              <pre className="banner-error modal-error" role="alert">
                {error}
              </pre>
            )}
            <footer className="modal-proyecto-pie">
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
            </footer>
          </form>
        </div>
      )}
    </div>
  );
}
