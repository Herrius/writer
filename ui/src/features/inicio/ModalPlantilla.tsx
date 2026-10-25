import { useTrampaTab } from './useTrampaTab';
import { useEffect, useRef, useState } from 'react';
import type { PlantillaDto } from '../../types/PlantillaDto';
import { slugProyecto } from '../../recientes';

const NOTAS = [
  ['personajes', 'Personajes'],
  ['tramas', 'Tramas'],
  ['lugares', 'Lugares'],
  ['cronologia', 'Cronología'],
  ['objetos', 'Objetos'],
  ['cuaderno', 'Cuaderno'],
] as const;

type Props = {
  onGuardar: (plantilla: PlantillaDto) => Promise<void>;
  onCerrar: () => void;
};

export function ModalPlantilla({ onGuardar, onCerrar }: Props) {
  const [nombre, setNombre] = useState('');
  const [descripcion, setDescripcion] = useState('');
  const [idioma, setIdioma] = useState('es');
  const [libros, setLibros] = useState(1);
  const [capitulos, setCapitulos] = useState(3);
  const [escenas, setEscenas] = useState(1);
  const [notas, setNotas] = useState<string[]>(['personajes', 'tramas', 'lugares']);
  const [error, setError] = useState<string | null>(null);
  const [guardando, setGuardando] = useState(false);
  const dialogoRef = useRef<HTMLFormElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(
    typeof document === 'undefined' ? null : (document.activeElement as HTMLElement | null),
  );
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;
  useTrampaTab(dialogoRef);
  const guardandoRef = useRef(false);
  guardandoRef.current = guardando;

  useEffect(() => {
    dialogoRef.current?.querySelector<HTMLInputElement>('input')?.focus();
    const alPulsar = (evento: KeyboardEvent) => {
      if (evento.key === 'Escape' && !guardandoRef.current) {
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

  const totalEscenas = libros * capitulos * escenas;
  const enviar = async (evento: { preventDefault: () => void }) => {
    evento.preventDefault();
    if (!nombre.trim()) {
      setError('Escribe un nombre para la plantilla.');
      return;
    }
    if (totalEscenas > 500) {
      setError('La plantilla no puede superar 500 escenas.');
      return;
    }
    const plantilla: PlantillaDto = {
      slug: slugProyecto(nombre),
      version: 1,
      nombre: nombre.trim(),
      descripcion: descripcion.trim(),
      idioma,
      estructura: {
        libros,
        capitulosPorLibro: capitulos,
        escenasPorCapitulo: escenas,
      },
      notasIniciales: notas,
    };
    setGuardando(true);
    setError(null);
    try {
      await onGuardar(plantilla);
    } catch (fallo) {
      const mensaje =
        typeof fallo === 'object' && fallo !== null && 'mensaje' in fallo
          ? String(fallo.mensaje)
          : 'No se pudo guardar la plantilla.';
      setError(mensaje);
    } finally {
      setGuardando(false);
    }
  };

  const alternarNota = (tipo: string) =>
    setNotas((actual) =>
      actual.includes(tipo) ? actual.filter((item) => item !== tipo) : [...actual, tipo],
    );

  return (
    <div
      className="modal-fondo"
      role="presentation"
      onMouseDown={(evento) => {
        if (evento.target === evento.currentTarget && !guardando) {
          onCerrar();
        }
      }}
    >
      <form
        className="modal-inicio modal-plantilla"
        role="dialog"
        aria-modal="true"
        aria-labelledby="plantilla-titulo"
        ref={dialogoRef}
        onSubmit={enviar}
      >
        <header className="modal-inicio-cabecera">
          <div>
            <p className="kicker">estructura reutilizable</p>
            <h2 id="plantilla-titulo">Nueva plantilla</h2>
          </div>
          <button
            type="button"
            className="cerrar-modal"
            aria-label="Cerrar plantilla"
            onClick={onCerrar}
            disabled={guardando}
          >
            ×
          </button>
        </header>
        <div className="modal-inicio-cuerpo">
          <label className="dabble-campo-meta">
            <span>Nombre</span>
            <input
              value={nombre}
              onChange={(evento) => setNombre(evento.target.value)}
              placeholder="Novela en tres actos"
              maxLength={200}
            />
          </label>
          <label className="dabble-campo-meta">
            <span>Descripción</span>
            <input
              value={descripcion}
              onChange={(evento) => setDescripcion(evento.target.value)}
              placeholder="Qué estructura genera"
              maxLength={500}
            />
          </label>
          <label className="dabble-campo-meta">
            <span>Idioma</span>
            <select value={idioma} onChange={(evento) => setIdioma(evento.target.value)}>
              <option value="es">Español</option>
              <option value="en">English</option>
            </select>
          </label>
          <div className="modal-plantilla-estructura">
            {[
              { etiqueta: 'Libros', valor: libros, cambiar: setLibros },
              { etiqueta: 'Capítulos/libro', valor: capitulos, cambiar: setCapitulos },
              { etiqueta: 'Escenas/capítulo', valor: escenas, cambiar: setEscenas },
            ].map(({ etiqueta, valor, cambiar }) => (
              <label key={etiqueta} className="dabble-campo-meta">
                <span>{etiqueta}</span>
                <input
                  type="number"
                  min={1}
                  max={999}
                  value={valor}
                  onChange={(evento) =>
                    cambiar(Math.max(1, Math.min(999, Number(evento.target.value) || 1)))
                  }
                />
              </label>
            ))}
          </div>
          <p className="dabble-nota-pie">
            {totalEscenas.toLocaleString('es')} escena(s) al crear un proyecto (máximo 500).
          </p>
          <fieldset className="modal-plantilla-notas">
            <legend>Notas iniciales</legend>
            {NOTAS.map(([tipo, etiqueta]) => (
              <label key={tipo} className="fila-check">
                <input
                  type="checkbox"
                  checked={notas.includes(tipo)}
                  onChange={() => alternarNota(tipo)}
                />
                {etiqueta}
              </label>
            ))}
          </fieldset>
          {error && <p className="dabble-error" role="alert">{error}</p>}
          <p className="dabble-nota-pie">
            La plantilla se guarda en los datos locales de writer; no se sincroniza.
          </p>
        </div>
        <footer className="modal-inicio-pie">
          <button type="button" className="dabble-boton" onClick={onCerrar} disabled={guardando}>
            Cancelar
          </button>
          <button type="submit" className="dabble-boton-primario" disabled={guardando}>
            {guardando ? 'Guardando…' : 'Guardar plantilla'}
          </button>
        </footer>
      </form>
    </div>
  );
}
