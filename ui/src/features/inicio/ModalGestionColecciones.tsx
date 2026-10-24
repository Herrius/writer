import { useTrampaTab } from './useTrampaTab';
import { useEffect, useRef, useState } from 'react';
import type { Coleccion } from '../../organizacion';

type Props = {
  colecciones: Coleccion[];
  onGuardar: (colecciones: Coleccion[]) => void;
  onCerrar: () => void;
};

export function ModalGestionColecciones({ colecciones, onGuardar, onCerrar }: Props) {
  const [borrador, setBorrador] = useState(colecciones);
  const [confirmar, setConfirmar] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(
    typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null),
  );
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  useTrampaTab(dialogoRef);

  useEffect(() => {
    dialogoRef.current?.querySelector<HTMLElement>('input, button')?.focus();
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

  const guardar = () => {
    if (borrador.some((coleccion) => !coleccion.nombre.trim())) {
      setError('Las colecciones deben tener nombre.');
      return;
    }
    onGuardar(
      borrador.map((coleccion) => ({
        ...coleccion,
        nombre: coleccion.nombre.trim().slice(0, 120),
      })),
    );
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
        className="modal-inicio"
        role="dialog"
        aria-modal="true"
        aria-labelledby="gestion-colecciones-titulo"
        ref={dialogoRef}
        tabIndex={-1}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">organización local</p>
            <h2 id="gestion-colecciones-titulo">Gestionar colecciones</h2>
          </div>
          <button className="cerrar-modal" aria-label="Cerrar colecciones" onClick={onCerrar}>
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          {borrador.length === 0 && <p className="dabble-vacio">No hay colecciones.</p>}
          {borrador.map((coleccion) => (
            <div key={coleccion.id} className="fila-coleccion">
              <span className="punto-color" style={{ background: coleccion.color }} aria-hidden="true" />
              <input
                aria-label={`Nombre de la colección ${coleccion.nombre}`}
                value={coleccion.nombre}
                maxLength={120}
                onChange={(evento) =>
                  setBorrador((actual) =>
                    actual.map((item) =>
                      item.id === coleccion.id ? { ...item, nombre: evento.target.value } : item,
                    ),
                  )
                }
              />
              <small>{coleccion.rutas.length} proyecto(s)</small>
              <button
                className="dabble-boton peligro"
                onClick={() => {
                  if (confirmar !== coleccion.id) {
                    setConfirmar(coleccion.id);
                    return;
                  }
                  setBorrador((actual) => actual.filter((item) => item.id !== coleccion.id));
                  setConfirmar(null);
                  window.requestAnimationFrame(() => {
                    dialogoRef.current?.querySelector<HTMLElement>('input, button')?.focus();
                  });
                }}
              >
                {confirmar === coleccion.id ? 'Confirmar' : 'Eliminar'}
              </button>
            </div>
          ))}
          {error && <p className="dabble-error" role="alert">{error}</p>}
          <p className="dabble-nota-pie">
            Eliminar una colección no elimina ni mueve ningún proyecto.
          </p>
        </div>
        <footer className="modal-inicio-pie">
          <button className="dabble-boton" onClick={onCerrar}>
            Cancelar
          </button>
          <button className="dabble-boton-primario" onClick={guardar}>
            Guardar cambios
          </button>
        </footer>
      </div>
    </div>
  );
}
