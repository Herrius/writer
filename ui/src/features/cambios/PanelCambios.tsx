import { useState } from 'react';
import type { CambioDetalleDto } from '../../types/CambioDetalleDto';
import type { CambioResumenDto } from '../../types/CambioResumenDto';
import type { DocumentoDto } from '../../types/DocumentoDto';
import { DiffLineas } from './DiffLineas';

type Props = {
  modo: boolean;
  cambios: CambioResumenDto[];
  activo: CambioDetalleDto | null;
  documentos: DocumentoDto[];
  cargando: boolean;
  error: string | null;
  procesando: boolean;
  onModo: (activo: boolean) => void;
  onLeer: (id: string) => void;
  onAceptar: (id: string) => void;
  onDescartar: (id: string) => void;
  onAbrirDocumento: (id: string) => void;
};

function tituloDocumento(documentos: DocumentoDto[], id: string): string {
  return documentos.find((documento) => documento.id === id)?.titulo ?? 'Documento';
}

export function PanelCambios({
  modo,
  cambios,
  activo,
  documentos,
  cargando,
  error,
  procesando,
  onModo,
  onLeer,
  onAceptar,
  onDescartar,
  onAbrirDocumento,
}: Props) {
  const [confirmarDescarte, setConfirmarDescarte] = useState<string | null>(null);

  return (
    <section className="dabble-cambios-panel" aria-labelledby="cambios-titulo">
      <h3 id="cambios-titulo">Control de cambios</h3>
      <label className="dabble-interruptor">
        <input
          type="checkbox"
          role="switch"
          checked={modo}
          onChange={(evento) => onModo(evento.target.checked)}
        />
        <span>
          <strong>Proponer cambios</strong>
          <small>
            Lo que escribas se guarda como propuesta; el documento no cambia hasta
            aceptarla.
          </small>
        </span>
      </label>

      {error && (
        <p className="dabble-error" role="alert">
          {error}
        </p>
      )}

      <h4>Pendientes ({cambios.length})</h4>
      {cargando && <p className="dabble-vacio">Cargando cambios…</p>}
      {!cargando && cambios.length === 0 && (
        <p className="dabble-vacio">
          No hay cambios propuestos. Activa «Proponer cambios» y edita el documento.
        </p>
      )}
      <ul className="dabble-lista-cambios">
        {cambios.map((cambio) => (
          <li key={cambio.id}>
            <button
              className={`dabble-cambio-item ${activo?.id === cambio.id ? 'activo' : ''}`}
              onClick={() => {
                onLeer(cambio.id);
                onAbrirDocumento(cambio.documento);
              }}
            >
              <span>
                <strong>{tituloDocumento(documentos, cambio.documento)}</strong>
                <small>{cambio.ruta}</small>
              </span>
              <span className="dabble-cambio-cuenta">
                +{cambio.insertadas} / −{cambio.borradas}
              </span>
            </button>
          </li>
        ))}
      </ul>

      {activo && (
        <div className="dabble-cambio-detalle">
          <div className="dabble-cambio-cabecera">
            <div>
              <h4>{tituloDocumento(documentos, activo.documento)}</h4>
              <p>
                {activo.autor} · revisión base {activo.baseRevision} ·{' '}
                {activo.insertadas} inserción(es), {activo.borradas} borrado(s)
              </p>
            </div>
            <button className="dabble-boton" onClick={() => onAbrirDocumento(activo.documento)}>
              Abrir documento
            </button>
          </div>

          <DiffLineas
            lineas={activo.lineas}
            etiquetaBase="Documento actual"
            etiquetaPropuesta="Propuesta"
          />

          <div className="dabble-acciones-documento">
            <button
              className="dabble-boton-primario"
              disabled={procesando}
              onClick={() => onAceptar(activo.id)}
            >
              {procesando ? 'Aplicando…' : 'Aceptar cambio'}
            </button>
            <button
              className="dabble-boton"
              disabled={procesando}
              onClick={() => {
                if (confirmarDescarte !== activo.id) {
                  setConfirmarDescarte(activo.id);
                  return;
                }
                setConfirmarDescarte(null);
                onDescartar(activo.id);
              }}
            >
              {confirmarDescarte === activo.id ? 'Confirmar descarte' : 'Rechazar'}
            </button>
            {confirmarDescarte === activo.id && (
              <button className="dabble-boton" onClick={() => setConfirmarDescarte(null)}>
                Cancelar
              </button>
            )}
          </div>
          <p className="dabble-nota-pie">
            Aceptar escribe una nueva revisión. Rechazar descarta la propuesta y
            conserva el documento tal como está.
          </p>
        </div>
      )}
    </section>
  );
}
