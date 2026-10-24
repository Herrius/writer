import type { VozSistema } from '../lectura/lectura';

type Props = {
  soporta: boolean;
  voces: VozSistema[];
  voz: string;
  velocidad: number;
  leyendo: boolean;
  pausado: boolean;
  error: string | null;
  palabras: number;
  caracteres: number;
  parrafos: number;
  minutosLectura: number;
  palabrasSeleccion: number;
  enfoque: boolean;
  onVoz: (voz: string) => void;
  onVelocidad: (velocidad: number) => void;
  onLeer: () => void;
  onLeerSeleccion: () => void;
  onPausar: () => void;
  onReanudar: () => void;
  onDetener: () => void;
  onEnfoque: (activo: boolean) => void;
};

export function PanelHerramientas({
  soporta,
  voces,
  voz,
  velocidad,
  leyendo,
  pausado,
  error,
  palabras,
  caracteres,
  parrafos,
  minutosLectura,
  palabrasSeleccion,
  enfoque,
  onVoz,
  onVelocidad,
  onLeer,
  onLeerSeleccion,
  onPausar,
  onReanudar,
  onDetener,
  onEnfoque,
}: Props) {
  return (
    <section className="dabble-herramientas" aria-labelledby="herramientas-titulo">
      <h3 id="herramientas-titulo">Herramientas de escritura</h3>

      <dl className="dabble-estadisticas">
        <div>
          <dt>Palabras</dt>
          <dd>{palabras.toLocaleString('es')}</dd>
        </div>
        <div>
          <dt>Caracteres</dt>
          <dd>{caracteres.toLocaleString('es')}</dd>
        </div>
        <div>
          <dt>Párrafos</dt>
          <dd>{parrafos.toLocaleString('es')}</dd>
        </div>
        <div>
          <dt>Lectura</dt>
          <dd>{minutosLectura === 0 ? '—' : `${minutosLectura} min`}</dd>
        </div>
      </dl>
      {palabrasSeleccion > 0 && (
        <p className="dabble-resumen">
          Selección actual: {palabrasSeleccion.toLocaleString('es')} palabra
          {palabrasSeleccion === 1 ? '' : 's'}.
        </p>
      )}

      <label className="dabble-interruptor">
        <input
          type="checkbox"
          checked={enfoque}
          onChange={(evento) => onEnfoque(evento.target.checked)}
        />
        <span>
          <strong>Modo enfoque</strong>
          <small>Oculta el árbol y el panel lateral. Alt+Z.</small>
        </span>
      </label>

      <h4>Lectura en voz alta</h4>
      {!soporta ? (
        <p className="dabble-vacio">
          Este WebView no expone voces del sistema. La lectura en voz alta queda
          desactivada; el texto no se envía a ningún servicio.
        </p>
      ) : (
        <>
          <label className="dabble-campo-meta">
            <span>Voz</span>
            <select
              value={voz}
              onChange={(evento) => onVoz(evento.target.value)}
              aria-label="Voz del sistema"
            >
              <option value="">Voz predeterminada</option>
              {voces.map((opcion) => (
                <option key={opcion.nombre} value={opcion.nombre}>
                  {opcion.nombre} ({opcion.idioma})
                </option>
              ))}
            </select>
          </label>
          {voces.length === 0 && (
            <p className="dabble-vacio">
              No hay voces disponibles. Instala una voz en el sistema operativo y
              vuelve a abrir la app.
            </p>
          )}
          <label className="dabble-campo-meta">
            <span>
              Velocidad: <strong>{velocidad.toFixed(2)}×</strong>
            </span>
            <input
              type="range"
              min={0.5}
              max={2}
              step={0.05}
              value={velocidad}
              onChange={(evento) => onVelocidad(Number(evento.target.value))}
              aria-label="Velocidad de lectura"
            />
          </label>
          {error && (
            <p className="dabble-error" role="alert">
              {error}
            </p>
          )}
          <div className="dabble-acciones-documento">
            {!leyendo || pausado ? (
              <button
                className="dabble-boton-primario"
                onClick={pausado ? onReanudar : onLeer}
                disabled={!pausado && voces.length === 0}
              >
                {pausado ? 'Reanudar' : 'Leer documento'}
              </button>
            ) : (
              <button className="dabble-boton" onClick={onPausar}>
                Pausar
              </button>
            )}
            <button
              className="dabble-boton"
              onClick={onLeerSeleccion}
              disabled={palabrasSeleccion === 0 || voces.length === 0}
            >
              Leer selección
            </button>
            {leyendo && (
              <button className="dabble-boton" onClick={onDetener}>
                Detener
              </button>
            )}
          </div>
          <p className="dabble-nota-pie">
            Usa las voces instaladas en el sistema. writer no graba audio ni
            envía el texto fuera del equipo.
          </p>
        </>
      )}

      <h4>Corrección</h4>
      <p className="dabble-nota-pie">
        El corrector nativo del WebView se activa desde Preferencias. No se
        incluye diccionario ni gramática propios en esta versión.
      </p>
    </section>
  );
}
