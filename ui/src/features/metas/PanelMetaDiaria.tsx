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
  onAgregar: () => void;
};

export function PanelMetaDiaria({
  meta,
  hechas,
  objetivo,
  porcentaje,
  restantes,
  libre,
  dias,
  onEditar,
  onAgregar,
}: Props) {
  const maximo = Math.max(1, objetivo, ...dias.map((dia) => dia.palabras));
  const alturaObjetivo = objetivo > 0 ? Math.min(100, (objetivo / maximo) * 100) : 0;

  return (
    <section className="dabble-metas-global" aria-labelledby="metas-diarias-titulo">
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
        <span>
          {meta
            ? libre
              ? 'Hoy es un día libre'
              : 'Hoy es un día de escritura'
            : 'Configura tu primera meta diaria'}
        </span>
        <span className="meta-info" title="Los días libres no rompen la racha">
          ⓘ
        </span>
      </div>

      {meta ? (
        <>
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
        <p className="meta-sin-meta">
          Aún no tienes una meta diaria. Crea una para empezar a medir tu hábito.
        </p>
      )}

      <div className="meta-cta">
        <div className="meta-cta-ilustracion" aria-hidden="true">
          <span className="meta-cta-linea" />
          <span className="meta-cta-punto" />
          <span className="meta-cta-punto meta-cta-punto-2" />
        </div>
        <h3>Establece tus metas de escritura</h3>
        <p>
          Sigue tu progreso con hábitos diarios, metas de libro o desafíos de
          escritura a plazo.
        </p>
        <button className="meta-agregar" onClick={meta ? onEditar : onAgregar}>
          <span aria-hidden="true">+</span>{' '}
          {meta ? 'Ajustar meta diaria' : 'Agregar una meta'}
        </button>
      </div>
    </section>
  );
}
