import { useTrampaTab } from './useTrampaTab';
import { useEffect, useRef } from 'react';

type Props = {
  titulo: string;
  descripcion: string;
  confirmar: string;
  peligro?: boolean;
  onConfirmar: () => void;
  onCerrar: () => void;
  error?: string | null;
  procesando?: boolean;
};

export function ModalConfirmar({
  titulo,
  descripcion,
  confirmar,
  peligro = false,
  onConfirmar,
  onCerrar,
  error = null,
  procesando = false,
}: Props) {
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(
    typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null),
  );
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  useTrampaTab(dialogoRef);

  useEffect(() => {
    dialogoRef.current?.querySelector<HTMLElement>('button')?.focus();
    const alPulsar = (evento: KeyboardEvent) => {
      if (evento.key === 'Escape') {
        evento.preventDefault();
        evento.stopPropagation();
        cerrarRef.current();
      }
    };
    document.addEventListener('keydown', alPulsar);
    return () => {
      document.removeEventListener('keydown', alPulsar);
      window.setTimeout(() => anteriorFoco.current?.focus?.(), 50);
    };
  }, []);

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
        className="modal-inicio modal-confirmar"
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirmar-titulo"
        aria-describedby="confirmar-descripcion"
        ref={dialogoRef}
        tabIndex={-1}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">confirmación</p>
            <h2 id="confirmar-titulo">{titulo}</h2>
          </div>
          <button
            className="cerrar-modal"
            aria-label="Cerrar confirmación"
            onClick={onCerrar}
            disabled={procesando}
          >
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          <p id="confirmar-descripcion">{descripcion}</p>
          {error && (
            <p className="dabble-error" role="alert">
              {error}
            </p>
          )}
        </div>
        <footer className="modal-inicio-pie">
          <button className="dabble-boton" onClick={onCerrar} disabled={procesando}>
            Cancelar
          </button>
          <button
            className={`dabble-boton-primario ${peligro ? 'peligro' : ''}`}
            onClick={onConfirmar}
            disabled={procesando}
          >
            {procesando ? 'Procesando…' : confirmar}
          </button>
        </footer>
      </div>
    </div>
  );
}
