import { useEffect, useRef } from 'react';
import type { LineaDiffDto } from '../../types/LineaDiffDto';
import { DiffLineas } from './DiffLineas';

type Props = {
  lineas: LineaDiffDto[];
  etiquetaBase: string;
  etiquetaPropuesta: string;
  onCerrar: () => void;
};

export function ModalDiff({ lineas, etiquetaBase, etiquetaPropuesta, onCerrar }: Props) {
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  const anteriorFoco = useRef<HTMLElement | null>(null);

  useEffect(() => {
    anteriorFoco.current = document.activeElement as HTMLElement | null;
    dialogoRef.current?.querySelector<HTMLElement>('button')?.focus();
    const alPulsar = (evento: KeyboardEvent) => {
      if (evento.key === 'Escape') {
        evento.preventDefault();
        evento.stopPropagation();
        cerrarRef.current();
        return;
      }
      if (evento.key !== 'Tab' || !dialogoRef.current) {
        return;
      }
      const enfocables = dialogoRef.current.querySelectorAll<HTMLElement>(
        'button, [tabindex]:not([tabindex="-1"])',
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
        className="modal-diff"
        role="dialog"
        aria-modal="true"
        aria-labelledby="diff-titulo"
        tabIndex={-1}
        ref={dialogoRef}
      >
        <header className="modal-diff-cabecera">
          <div>
            <p className="kicker">revisión de cambios</p>
            <h2 id="diff-titulo">Diferencias del documento</h2>
          </div>
          <button className="dabble-boton" onClick={onCerrar} aria-label="Cerrar diferencias">
            Cerrar
          </button>
        </header>
        <div className="modal-diff-cuerpo">
          <DiffLineas
            lineas={lineas}
            etiquetaBase={etiquetaBase}
            etiquetaPropuesta={etiquetaPropuesta}
          />
        </div>
      </div>
    </div>
  );
}
