import { useTrampaTab } from './useTrampaTab';
import { useEffect, useRef, useState } from 'react';
import {
  FILTROS_POR_DEFECTO,
  type FiltrosProyectos,
  type OrdenProyectos,
  type PeriodoFiltro,
} from '../../organizacion';

type Props = {
  filtros: FiltrosProyectos;
  autores: string[];
  onAplicar: (filtros: FiltrosProyectos) => void;
  onCerrar: () => void;
};

export function ModalFiltros({ filtros, autores, onAplicar, onCerrar }: Props) {
  const [borrador, setBorrador] = useState(filtros);
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(
    typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null),
  );
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  useTrampaTab(dialogoRef);

  useEffect(() => {
    dialogoRef.current?.querySelector<HTMLElement>('.modal-inicio-cuerpo select, .modal-inicio-cuerpo input, .modal-inicio-cuerpo button')?.focus();
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

  const autoresVisibles =
    borrador.autor && !autores.includes(borrador.autor)
      ? [borrador.autor, ...autores]
      : autores;

  const actualizar = <K extends keyof FiltrosProyectos>(
    clave: K,
    valor: FiltrosProyectos[K],
  ) => setBorrador((actual) => ({ ...actual, [clave]: valor }));

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
        aria-labelledby="filtros-titulo"
        ref={dialogoRef}
        tabIndex={-1}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">Mis Proyectos</p>
            <h2 id="filtros-titulo">Filtrar proyectos</h2>
          </div>
          <button className="cerrar-modal" aria-label="Cerrar filtros" onClick={onCerrar}>
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          <label className="dabble-campo-meta">
            <span>Autor</span>
            <select
              value={borrador.autor}
              onChange={(evento) => actualizar('autor', evento.target.value)}
            >
              <option value="">Cualquier autor</option>
              {autoresVisibles.map((autor) => (
                <option key={autor} value={autor}>
                  {autor}
                </option>
              ))}
            </select>
          </label>
          <label className="dabble-campo-meta">
            <span>Modificación</span>
            <select
              value={borrador.periodo}
              onChange={(evento) =>
                actualizar('periodo', evento.target.value as PeriodoFiltro)
              }
            >
              <option value="todos">Cualquier fecha</option>
              <option value="7d">Últimos 7 días</option>
              <option value="30d">Últimos 30 días</option>
              <option value="ano">Último año</option>
            </select>
          </label>
          <label className="dabble-campo-meta">
            <span>Palabras mínimas</span>
            <input
              type="number"
              min={0}
              step={100}
              value={borrador.minimoPalabras}
              onChange={(evento) =>
                actualizar('minimoPalabras', Math.max(0, Number(evento.target.value) || 0))
              }
            />
          </label>
          <label className="dabble-campo-meta">
            <span>Ordenar por</span>
            <select
              value={borrador.orden}
              onChange={(evento) =>
                actualizar('orden', evento.target.value as OrdenProyectos)
              }
            >
              <option value="recientes">Recientes primero</option>
              <option value="titulo">Título A–Z</option>
              <option value="palabras">Más palabras</option>
            </select>
          </label>
        </div>
        <footer className="modal-inicio-pie">
          <button
            className="dabble-boton"
            onClick={() => setBorrador({ ...FILTROS_POR_DEFECTO })}
          >
            Limpiar filtros
          </button>
          <button className="dabble-boton" onClick={onCerrar}>
            Cancelar
          </button>
          <button className="dabble-boton-primario" onClick={() => onAplicar(borrador)}>
            Aplicar
          </button>
        </footer>
      </div>
    </div>
  );
}
