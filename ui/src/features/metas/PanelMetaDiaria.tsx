import type { MetaDiariaGlobal } from '../../metasGlobales';

type Dia = { fecha: string; palabras: number; etiqueta: string };

type Props = {
  meta: MetaDiariaGlobal | null;
  hechas: number;
  objetivo: number;
  porcentaje: number;
  restantes: number;
  libre: boolean;
  dias: Dia[];
  onEditar: () => void;
};

const BARRAS_VACIAS = [38, 52, 46, 63, 57, 72, 66, 84, 76, 91];

export function PanelMetaDiaria({
  meta,
  hechas,
  objetivo,
  porcentaje,
  restantes,
  libre,
  dias,
  onEditar,
}: Props) {
  const maximo = Math.max(1, objetivo, ...dias.map((dia) => dia.palabras));
  const alturaObjetivo = objetivo > 0 ? Math.min(100, (objetivo / maximo) * 100) : 0;

  return (
    <section className="dabble-metas-global" aria-label="Metas diarias">
      {meta ? (
        <>
          <header className="metas-global-cabecera">
            <span id="metas-diarias-titulo">METAS DIARIAS</span>
            <button
              className="meta-config"
              aria-label="Configurar meta diaria"
              title="Configurar meta diaria"
              onClick={onEditar}
            >
              ⚙
            </button>
          </header>

          <div className={`meta-hoy ${libre ? 'libre' : ''}`}>
            <span className="meta-icono-calendario" aria-hidden="true">
              ▣
            </span>
            <span>{libre ? 'Hoy es un día libre' : 'Hoy es un día de escritura'}</span>
            <span className="meta-info" title="Los días libres no rompen la racha">
              ⓘ
            </span>
          </div>

          <div className="meta-progreso">
            <span className="meta-etiqueta">PROGRESO TOTAL</span>
            <div className="meta-progreso-numero">
              <strong>{hechas.toLocaleString('es')}</strong>
              <span>/ {objetivo.toLocaleString('es')} palabras</span>
              <b>{porcentaje} %</b>
            </div>
            <div className="meta-barra" aria-hidden="true">
              <span style={{ width: `${porcentaje}%` }} />
            </div>
            <small>{restantes.toLocaleString('es')} restante</small>
          </div>

          <div className="meta-grafico" aria-label="Palabras de los últimos días">
            {dias.map((dia) => (
              <span
                key={dia.fecha}
                className="meta-barra-dia"
                style={{
                  height: `${Math.max(4, Math.round((dia.palabras / maximo) * 100))}%`,
                }}
                title={`${dia.fecha}: ${dia.palabras.toLocaleString('es')} palabras`}
              />
            ))}
            {objetivo > 0 && (
              <span
                className="meta-linea-objetivo"
                style={{ bottom: `${alturaObjetivo}%` }}
                aria-hidden="true"
              />
            )}
          </div>
        </>
      ) : (
        <div className="meta-vacia">
          <div className="meta-vacia-ilustracion" aria-hidden="true">
            <div className="meta-vacia-barras">
              {BARRAS_VACIAS.map((altura, indice) => (
                <span key={indice} style={{ height: `${altura}%` }} />
              ))}
            </div>
            <svg
              className="meta-vacia-lineas"
              viewBox="0 0 240 110"
              preserveAspectRatio="none"
            >
              <line
                x1="8"
                y1="94"
                x2="218"
                y2="40"
                stroke="#c59a62"
                strokeWidth="1.4"
                strokeDasharray="4 4"
              />
              <polyline
                points="8,92 32,86 54,78 78,72 102,63 126,57 150,48 176,42 196,34"
                fill="none"
                stroke="#47769d"
                strokeWidth="2.5"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
              <line
                x1="196"
                y1="34"
                x2="196"
                y2="100"
                stroke="#47769d"
                strokeWidth="2.5"
              />
              <circle cx="196" cy="34" r="4.5" fill="#47769d" />
            </svg>
            <svg
              className="meta-vacia-trofeo"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M8 4h8v4a4 4 0 0 1-8 0V4Z" />
              <path d="M8 6H5v1a4 4 0 0 0 4 4" />
              <path d="M16 6h3v1a4 4 0 0 1-4 4" />
              <path d="M12 12v4" />
              <path d="M9 20h6" />
              <path d="M10 16h4" />
            </svg>
          </div>
          <h3>Establece tus metas de escritura</h3>
          <p>
            Sigue tu progreso con hábitos diarios, metas de libro o desafíos de
            escritura a plazo.
          </p>
          <button className="meta-vacia-agregar" onClick={onEditar}>
            <span aria-hidden="true">+</span> Agregar una meta
          </button>
        </div>
      )}
    </section>
  );
}
