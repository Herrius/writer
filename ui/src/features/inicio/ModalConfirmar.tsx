import { useEffect, useRef } from 'react';

type Props = {
  titulo: string;
  descripcion: string;
  confirmar: string;
  peligro?: boolean;
  onConfirmar: () => void;
  onCerrar: () => void;
};

export function ModalConfirmar({
  titulo,
  descripcion,
  confirmar,
  peligro = false,
  onConfirmar,
  onCerrar,
}: Props) {
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(null);
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;

  useEffect(() => {
    anteriorFoco.current = document.activeElement as HTMLElement | null;
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
      anteriorFoco.current?.focus?.();
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
        ref={dialogoRef}
        tabIndex={-1}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">confirmación</p>
            <h2 id="confirmar-titulo">{titulo}</h2>
          </div>
          <button className="cerrar-modal" aria-label="Cerrar confirmación" onClick={onCerrar}>
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          <p>{descripcion}</p>
        </div>
        <footer className="modal-inicio-pie">
          <button className="dabble-boton" onClick={onCerrar}>
            Cancelar
          </button>
          <button
            className={`dabble-boton-primario ${peligro ? 'peligro' : ''}`}
            onClick={onConfirmar}
          >
            {confirmar}
          </button>
        </footer>
      </div>
    </div>
  );
}
