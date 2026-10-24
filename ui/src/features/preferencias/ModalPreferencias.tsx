import { useEffect, useRef } from 'react';
import {
  FUENTES,
  PREFERENCIAS_POR_DEFECTO,
  type Preferencias,
  type Tema,
  type FuenteEditor,
} from './preferencias';

type Props = {
  preferencias: Preferencias;
  onCambio: (preferencias: Preferencias) => void;
  onCerrar: () => void;
};

export function ModalPreferencias({ preferencias, onCambio, onCerrar }: Props) {
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(null);
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;

  useEffect(() => {
    anteriorFoco.current = document.activeElement as HTMLElement | null;
    dialogoRef.current?.focus();
    const alPulsar = (evento: KeyboardEvent) => {
      if (evento.key === 'Escape') {
        evento.preventDefault();
        cerrarRef.current();
        return;
      }
      if (evento.key !== 'Tab' || !dialogoRef.current) {
        return;
      }
      const enfocables = dialogoRef.current.querySelectorAll<HTMLElement>(
        'button, input, select, textarea, [tabindex]:not([tabindex="-1"])',
      );
      if (enfocables.length === 0) {
        return;
      }
      const primero = enfocables[0];
      const ultimo = enfocables[enfocables.length - 1];
      if (evento.shiftKey && document.activeElement === primero) {
        evento.preventDefault();
        ultimo.focus();
      } else if (!evento.shiftKey && document.activeElement === ultimo) {
        evento.preventDefault();
        primero.focus();
      }
    };
    document.addEventListener('keydown', alPulsar);
    return () => {
      document.removeEventListener('keydown', alPulsar);
      anteriorFoco.current?.focus?.();
    };
  }, []);

  const actualizar = <K extends keyof Preferencias>(clave: K, valor: Preferencias[K]) => {
    onCambio({ ...preferencias, [clave]: valor });
  };

  return (
    <div
      className="modal-fondo"
      role="presentation"
      onMouseDown={(evento) => {
        if (evento.target === evento.currentTarget) {
          onCerrar();
        }
      }}
    >
      <div
        className="modal-preferencias"
        role="dialog"
        aria-modal="true"
        aria-labelledby="preferencias-titulo"
        tabIndex={-1}
        ref={dialogoRef}
      >
        <header className="modal-preferencias-cabecera">
          <div>
            <p className="kicker">este equipo</p>
            <h2 id="preferencias-titulo">Preferencias de escritura</h2>
          </div>
          <button className="dabble-boton" onClick={onCerrar} aria-label="Cerrar preferencias">
            Cerrar
          </button>
        </header>

        <div className="modal-preferencias-cuerpo">
          <label className="dabble-campo-meta">
            <span>Tema</span>
            <select
              value={preferencias.tema}
              onChange={(evento) => actualizar('tema', evento.target.value as Tema)}
            >
              <option value="claro">Claro</option>
              <option value="sepia">Sepia</option>
              <option value="oscuro">Oscuro</option>
            </select>
          </label>

          <label className="dabble-campo-meta">
            <span>Tipografía</span>
            <select
              value={preferencias.fuente}
              onChange={(evento) =>
                actualizar('fuente', evento.target.value as FuenteEditor)
              }
            >
              <option value="serif">Serif de lectura</option>
              <option value="sans">Sans para pantalla</option>
              <option value="mono">Monoespaciada</option>
            </select>
          </label>
          <p className="dabble-preferencias-muestra" style={{ fontFamily: FUENTES[preferencias.fuente] }}>
            La casa olía a lluvia vieja. Marta dejó la maleta en el umbral.
          </p>

          <label className="dabble-campo-meta">
            <span>
              Tamaño del texto: <strong>{preferencias.tamano} px</strong>
            </span>
            <input
              type="range"
              min={12}
              max={32}
              step={1}
              value={preferencias.tamano}
              onChange={(evento) => actualizar('tamano', Number(evento.target.value))}
            />
          </label>

          <label className="dabble-campo-meta">
            <span>
              Interlineado: <strong>{preferencias.interlineado.toFixed(2)}</strong>
            </span>
            <input
              type="range"
              min={1.2}
              max={2.6}
              step={0.05}
              value={preferencias.interlineado}
              onChange={(evento) => actualizar('interlineado', Number(evento.target.value))}
            />
          </label>

          <label className="dabble-campo-meta">
            <span>
              Espacio entre párrafos: <strong>{preferencias.espaciadoParrafo.toFixed(2)} em</strong>
            </span>
            <input
              type="range"
              min={0}
              max={2.5}
              step={0.05}
              value={preferencias.espaciadoParrafo}
              onChange={(evento) =>
                actualizar('espaciadoParrafo', Number(evento.target.value))
              }
            />
          </label>

          <label className="dabble-interruptor">
            <input
              type="checkbox"
              checked={preferencias.modoEnfoque}
              onChange={(evento) => actualizar('modoEnfoque', evento.target.checked)}
            />
            <span>
              <strong>Modo enfoque</strong>
              <small>Oculta las barras laterales. Alt+Z para entrar o salir.</small>
            </span>
          </label>

          <label className="dabble-interruptor">
            <input
              type="checkbox"
              checked={preferencias.corrector}
              onChange={(evento) => actualizar('corrector', evento.target.checked)}
            />
            <span>
              <strong>Corrector ortográfico del sistema</strong>
              <small>
                Usa el corrector nativo del WebView. No envía texto a ningún
                servicio.
              </small>
            </span>
          </label>

          <p className="dabble-nota-pie">
            Las preferencias se guardan en este equipo y no se sincronizan. El
            formato del PDF se guarda aparte, dentro del proyecto.
          </p>
        </div>

        <footer className="modal-preferencias-pie">
          <button
            className="dabble-boton"
            onClick={() => onCambio({ ...PREFERENCIAS_POR_DEFECTO })}
          >
            Restablecer
          </button>
          <button className="dabble-boton-primario" onClick={onCerrar}>
            Listo
          </button>
        </footer>
      </div>
    </div>
  );
}
