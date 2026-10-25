import { useEffect, useRef, useState } from 'react';
import { fechaLocal, type MetaDiariaGlobal } from '../../metasGlobales';
import { useTrampaTab } from '../inicio/useTrampaTab';

const DIAS = [
  { valor: 0, etiqueta: 'D', nombre: 'domingo' },
  { valor: 1, etiqueta: 'L', nombre: 'lunes' },
  { valor: 2, etiqueta: 'M', nombre: 'martes' },
  { valor: 3, etiqueta: 'X', nombre: 'miércoles' },
  { valor: 4, etiqueta: 'J', nombre: 'jueves' },
  { valor: 5, etiqueta: 'V', nombre: 'viernes' },
  { valor: 6, etiqueta: 'S', nombre: 'sábado' },
];

type Props = {
  meta: MetaDiariaGlobal | null;
  onGuardar: (meta: MetaDiariaGlobal) => void;
  onEliminar: () => void;
  onCerrar: () => void;
};

export function ModalMetaDiaria({ meta, onGuardar, onEliminar, onCerrar }: Props) {
  const [objetivo, setObjetivo] = useState(String(meta?.objetivo ?? 500));
  const [diasLibres, setDiasLibres] = useState<number[]>(meta?.diasLibres ?? []);
  const [soloManuscrito, setSoloManuscrito] = useState(meta?.contarSoloManuscrito ?? true);
  const [pegadas, setPegadas] = useState(meta?.contarPegadas ?? true);
  const [eliminaciones, setEliminaciones] = useState(meta?.contarEliminaciones ?? true);
  const [recordatorio, setRecordatorio] = useState(meta?.recordatorio ?? false);
  const [modificado, setModificado] = useState(false);
  const [confirmarEliminar, setConfirmarEliminar] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(
    typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null),
  );
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  useTrampaTab(dialogoRef);

  useEffect(() => {
    dialogoRef.current?.querySelector<HTMLInputElement>('input[type="number"]')?.focus();
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

  const cambiar = () => setModificado(true);
  const alternarDia = (dia: number) => {
    setDiasLibres((actual) =>
      actual.includes(dia) ? actual.filter((valor) => valor !== dia) : [...actual, dia],
    );
    cambiar();
  };
  const guardar = () => {
    const numero = Number(objetivo);
    if (!Number.isFinite(numero) || numero < 1 || numero > 1_000_000) {
      setError('La meta debe estar entre 1 y 1.000.000 palabras.');
      return;
    }
    onGuardar({
      objetivo: Math.floor(numero),
      desde: meta?.desde ?? fechaLocal(),
      diasLibres,
      contarSoloManuscrito: soloManuscrito,
      contarPegadas: pegadas,
      contarEliminaciones: eliminaciones,
      recordatorio,
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
        className="modal-meta-diaria"
        role="dialog"
        aria-modal="true"
        aria-labelledby="meta-diaria-titulo"
        ref={dialogoRef}
        tabIndex={-1}
      >
        <header className="modal-meta-cabecera">
          <h2 id="meta-diaria-titulo">Meta diaria</h2>
          <button className="cerrar-modal" aria-label="Cerrar meta diaria" onClick={onCerrar}>
            ×
          </button>
        </header>
        <div className="modal-meta-cuerpo">
          <p className="modal-meta-intro">
            Establece una meta diaria de escritura para crear un hábito constante.
            Sigue tu racha y mantente motivado.
          </p>

          <label className="modal-meta-objetivo">
            <span>Palabras por día</span>
            <span className="modal-meta-input">
              <input
                type="number"
                min={1}
                max={1_000_000}
                value={objetivo}
                aria-label="Palabras por día"
                onChange={(evento) => {
                  setObjetivo(evento.target.value);
                  setError(null);
                  cambiar();
                }}
              />
              <span>palabras</span>
            </span>
          </label>

          <fieldset className="modal-meta-grupo">
            <legend>Qué cuenta para tu objetivo</legend>
            <label className="fila-check">
              <input
                type="checkbox"
                checked={soloManuscrito}
                onChange={(evento) => {
                  setSoloManuscrito(evento.target.checked);
                  cambiar();
                }}
              />
              Contar solo las palabras del manuscrito
            </label>
            <label className="fila-check">
              <input
                type="checkbox"
                checked={pegadas}
                onChange={(evento) => {
                  setPegadas(evento.target.checked);
                  cambiar();
                }}
              />
              Contar palabras pegadas
            </label>
            <label className="fila-check">
              <input
                type="checkbox"
                checked={eliminaciones}
                onChange={(evento) => {
                  setEliminaciones(evento.target.checked);
                  cambiar();
                }}
              />
              Contar eliminaciones (progreso neto)
            </label>
          </fieldset>

          <fieldset className="modal-meta-grupo">
            <legend>
              Días libres <small>OPCIONAL</small>
            </legend>
            <p className="modal-meta-ayuda">
              Selecciona los días en los que no planeas escribir.
            </p>
            <div className="dias-libres" role="group" aria-label="Días libres">
              {DIAS.map((dia) => (
                <button
                  key={dia.valor}
                  type="button"
                  className={diasLibres.includes(dia.valor) ? 'activo' : ''}
                  aria-label={dia.nombre}
                  aria-pressed={diasLibres.includes(dia.valor)}
                  onClick={() => alternarDia(dia.valor)}
                >
                  {dia.etiqueta}
                </button>
              ))}
              <span className="dias-calendario" aria-hidden="true">▣</span>
            </div>
          </fieldset>

          <fieldset className="modal-meta-grupo">
            <legend>Recordatorios</legend>
            <label className="fila-check">
              <input
                type="checkbox"
                checked={recordatorio}
                onChange={(evento) => {
                  setRecordatorio(evento.target.checked);
                  cambiar();
                }}
              />
              Mostrar un recordatorio local si no cumplo mi objetivo
            </label>
            <p className="modal-meta-ayuda">
              writer es local/offline: no envía correos.
            </p>
          </fieldset>

          {error && (
            <p className="dabble-error" role="alert">
              {error}
            </p>
          )}
        </div>
        <footer className="modal-meta-pie">
          <button
            className="dabble-boton-primario"
            disabled={!modificado}
            onClick={guardar}
          >
            Guardar meta
          </button>
          {meta && (
            <button
              className="boton-eliminar-meta"
              onClick={() => {
                if (!confirmarEliminar) {
                  setConfirmarEliminar(true);
                  return;
                }
                onEliminar();
              }}
            >
              {confirmarEliminar ? 'Confirmar eliminación' : 'Eliminar meta'}
            </button>
          )}
        </footer>
      </div>
    </div>
  );
}
