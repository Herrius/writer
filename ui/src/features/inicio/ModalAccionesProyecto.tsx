import { useTrampaTab } from './useTrampaTab';
import { useEffect, useRef, useState } from 'react';
import type { Coleccion } from '../../organizacion';
import type { Reciente } from '../../recientes';

type Props = {
  reciente: Reciente;
  colecciones: Coleccion[];
  onCerrar: () => void;
  onAbrir: () => void;
  onGuardarColecciones: (ids: string[]) => boolean;
  onCrearColeccion: () => void;
  onMoverPapelera: () => void;
  error?: string | null;
};

export function ModalAccionesProyecto({
  reciente,
  colecciones,
  onCerrar,
  onAbrir,
  onGuardarColecciones,
  onCrearColeccion,
  onMoverPapelera,
  error = null,
}: Props) {
  const [seleccion, setSeleccion] = useState(
    () => new Set(colecciones.filter((c) => c.rutas.includes(reciente.ruta)).map((c) => c.id)),
  );
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

  const alternar = (id: string) => {
    setSeleccion((actual) => {
      const siguiente = new Set(actual);
      if (siguiente.has(id)) {
        siguiente.delete(id);
      } else {
        siguiente.add(id);
      }
      return siguiente;
    });
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
        aria-labelledby="acciones-proyecto-titulo"
        ref={dialogoRef}
        tabIndex={-1}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">proyecto</p>
            <h2 id="acciones-proyecto-titulo">{reciente.titulo}</h2>
          </div>
          <button className="cerrar-modal" aria-label="Cerrar acciones" onClick={onCerrar}>
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          <button
            className="dabble-boton"
            onClick={() => {
              if (onGuardarColecciones(Array.from(seleccion))) {
                onAbrir();
              }
            }}
          >
            Abrir proyecto
          </button>

          {error && (
            <p className="dabble-error" role="alert">
              {error}
            </p>
          )}
          <h3>Colecciones</h3>
          {colecciones.length === 0 ? (
            <p className="dabble-vacio">Todavía no hay colecciones.</p>
          ) : (
            <div className="lista-checks">
              {colecciones.map((coleccion) => (
                <label key={coleccion.id} className="dabble-campo-meta fila-check">
                  <input
                    type="checkbox"
                    checked={seleccion.has(coleccion.id)}
                    onChange={() => alternar(coleccion.id)}
                  />
                  <span className="punto-color" style={{ background: coleccion.color }} aria-hidden="true" />
                  <span>{coleccion.nombre}</span>
                </label>
              ))}
            </div>
          )}
          <button
            className="dabble-boton"
            onClick={() => {
              if (onGuardarColecciones(Array.from(seleccion))) {
                onCrearColeccion();
              }
            }}
          >
            + Nueva colección
          </button>

          <h3>Papelera</h3>
          <p className="dabble-nota-pie">
            El proyecto se moverá a una papelera local recuperable. No se borra del disco.
          </p>
          <button
            className="dabble-boton peligro"
            onClick={() => {
              if (onGuardarColecciones(Array.from(seleccion))) {
                onMoverPapelera();
              }
            }}
          >
            Mover proyecto a la papelera
          </button>
        </div>
        <footer className="modal-inicio-pie">
          <button className="dabble-boton" onClick={onCerrar}>
            Cancelar
          </button>
          <button
            className="dabble-boton-primario"
            onClick={() => onGuardarColecciones(Array.from(seleccion))}
          >
            Guardar colecciones
          </button>
        </footer>
      </div>
    </div>
  );
}
