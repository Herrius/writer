import { useTrampaTab } from './useTrampaTab';
import { useEffect, useRef, useState } from 'react';

const COLORES = [
  '#6b5747',
  '#4d6b8a',
  '#4f9b8a',
  '#7ba05b',
  '#c2a24a',
  '#c97a63',
  '#8f5d7a',
  '#6b6f76',
];

type Props = {
  limiteAlcanzado: boolean;
  onCrear: (nombre: string, color: string) => void;
  onCerrar: () => void;
};

export function ModalColeccion({ limiteAlcanzado, onCrear, onCerrar }: Props) {
  const [nombre, setNombre] = useState('');
  const [color, setColor] = useState(COLORES[0]);
  const [error, setError] = useState<string | null>(null);
  const dialogoRef = useRef<HTMLFormElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(
    typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null),
  );
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  useTrampaTab(dialogoRef);

  useEffect(() => {
    dialogoRef.current?.querySelector<HTMLInputElement>('input')?.focus();
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

  const enviar = (evento: { preventDefault: () => void }) => {
    evento.preventDefault();
    const limpio = nombre.trim();
    if (!limpio) {
      setError('Escribe un nombre para la colección.');
      return;
    }
    if (limpio.length > 120) {
      setError('El nombre no puede superar 120 caracteres.');
      return;
    }
    if (limiteAlcanzado) {
      setError('Has alcanzado el máximo de 50 colecciones.');
      return;
    }
    onCrear(limpio, color);
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
      <form
        className="modal-inicio"
        role="dialog"
        aria-modal="true"
        aria-labelledby="coleccion-titulo"
        ref={dialogoRef}
        onSubmit={enviar}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">organización local</p>
            <h2 id="coleccion-titulo">Nueva colección</h2>
          </div>
          <button type="button" className="cerrar-modal" aria-label="Cerrar colección" onClick={onCerrar}>
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          <label className="dabble-campo-meta">
            <span>Nombre</span>
            <input
              value={nombre}
              onChange={(evento) => {
                setNombre(evento.target.value);
                setError(null);
              }}
              placeholder="Novela negra, ideas, pendientes…"
              maxLength={120}
            />
          </label>
          <div className="modal-inicio-colores" role="group" aria-label="Color de la colección">
            {COLORES.map((tono) => (
              <button
                key={tono}
                type="button"
                className={`tono ${color === tono ? 'activo' : ''}`}
                style={{ background: tono }}
                aria-label={`Color ${tono}`}
                aria-pressed={color === tono}
                onClick={() => setColor(tono)}
              />
            ))}
          </div>
          {limiteAlcanzado && (
            <p className="dabble-error" role="alert">
              Has alcanzado el máximo de 50 colecciones. Elimina una para crear otra.
            </p>
          )}
          {error && <p className="dabble-error" role="alert">{error}</p>}
          <p className="dabble-nota-pie">
            Las colecciones se guardan en este equipo. No mueven ni copian los proyectos.
          </p>
        </div>
        <footer className="modal-inicio-pie">
          <button type="button" className="dabble-boton" onClick={onCerrar}>
            Cancelar
          </button>
          <button type="submit" className="dabble-boton-primario" disabled={limiteAlcanzado}>
            Crear colección
          </button>
        </footer>
      </form>
    </div>
  );
}
