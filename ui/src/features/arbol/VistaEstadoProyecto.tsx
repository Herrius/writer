import {
  Archive,
  BookOpen,
  FileText,
  FolderOpen,
  LayoutTemplate,
  Network,
  NotebookPen,
  RotateCcw,
  Trash2,
  Users,
} from 'lucide-react';
import type { DocumentoDto } from '../../types/DocumentoDto';

type Props = {
  estado: 'archivo' | 'papelera';
  documentos: DocumentoDto[];
  onAbrir: (documento: DocumentoDto) => void;
  onRestaurar: (documento: DocumentoDto) => void;
  onVolver: () => void;
};

function IconoDocumento({ documento }: { documento: DocumentoDto }) {
  if (documento.tipo === 'escena') {
    return <BookOpen size={17} strokeWidth={1.7} aria-hidden="true" />;
  }
  if (documento.clase === 'tramas') {
    return <Network size={17} strokeWidth={1.7} aria-hidden="true" />;
  }
  if (documento.clase === 'personajes') {
    return <Users size={17} strokeWidth={1.7} aria-hidden="true" />;
  }
  if (documento.clase === 'ideas') {
    return <FileText size={17} strokeWidth={1.7} aria-hidden="true" />;
  }
  return <NotebookPen size={17} strokeWidth={1.7} aria-hidden="true" />;
}

function tipoDocumento(documento: DocumentoDto): string {
  if (documento.tipo === 'escena') {
    return 'Escena';
  }
  const etiquetas: Record<string, string> = {
    tramas: 'Trama',
    personajes: 'Personaje',
    ideas: 'Idea',
    cuaderno: 'Cuaderno',
    lugares: 'Lugar',
    cronologia: 'Cronología',
    objetos: 'Objeto',
  };
  return etiquetas[documento.clase ?? ''] ?? 'Nota';
}

export function VistaEstadoProyecto({
  estado,
  documentos,
  onAbrir,
  onRestaurar,
  onVolver,
}: Props) {
  const esPapelera = estado === 'papelera';
  const titulo = esPapelera ? 'Papelera' : 'Archivo';
  const vacio = esPapelera ? 'La papelera está vacía' : 'El archivo está vacío';
  const descripcionVacia = esPapelera
    ? 'Los elementos que envíes a la papelera aparecerán aquí.'
    : 'Los elementos que archives aparecerán aquí.';

  return (
    <section className="vista-proyecto-pagina" aria-labelledby="vista-proyecto-titulo">
      <header className="vista-proyecto-cabecera">
        <div className="vista-proyecto-titulo">
          {esPapelera ? (
            <Trash2 size={21} strokeWidth={1.6} aria-hidden="true" />
          ) : (
            <Archive size={21} strokeWidth={1.6} aria-hidden="true" />
          )}
          <div>
            <h1 id="vista-proyecto-titulo">{titulo}</h1>
            <p>
              {esPapelera
                ? 'Documentos apartados del manuscrito; puedes restaurarlos cuando quieras.'
                : 'Documentos guardados fuera del manuscrito; puedes recuperarlos cuando los necesites.'}
            </p>
          </div>
        </div>
        <div className="vista-proyecto-acciones">
          <span className="vista-proyecto-contador">
            {documentos.length} {documentos.length === 1 ? 'elemento' : 'elementos'}
          </span>
          <button className="dabble-boton" onClick={onVolver}>
            ← Volver al editor
          </button>
        </div>
      </header>

      {documentos.length === 0 ? (
        <div className="vista-proyecto-vacia">
          <div className="vista-proyecto-vacia-icono" aria-hidden="true">
            <FolderOpen size={34} strokeWidth={1.3} />
          </div>
          <h2>{vacio}</h2>
          <p>{descripcionVacia}</p>
        </div>
      ) : (
        <ul className="vista-proyecto-lista">
          {documentos.map((documento) => (
            <li key={documento.id}>
              <button className="vista-proyecto-fila" onClick={() => onAbrir(documento)}>
                <span className="vista-proyecto-icono">
                  <IconoDocumento documento={documento} />
                </span>
                <span className="vista-proyecto-texto">
                  <strong>{documento.titulo}</strong>
                  <small>
                    {tipoDocumento(documento)} · {documento.ruta}
                  </small>
                </span>
              </button>
              <button
                className="vista-proyecto-restaurar"
                aria-label={`Restaurar ${documento.titulo}`}
                title="Restaurar a activos"
                onClick={() => onRestaurar(documento)}
              >
                <RotateCcw size={16} strokeWidth={1.8} aria-hidden="true" />
                Restaurar
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

type PropsPlantillas = {
  onGestionar: () => void;
  onVolver: () => void;
};

export function VistaPlantillas({ onGestionar, onVolver }: PropsPlantillas) {
  return (
    <section className="vista-proyecto-pagina" aria-labelledby="vista-plantillas-titulo">
      <header className="vista-proyecto-cabecera">
        <div className="vista-proyecto-titulo">
          <LayoutTemplate size={21} strokeWidth={1.6} aria-hidden="true" />
          <div>
            <h1 id="vista-plantillas-titulo">Plantillas</h1>
            <p>Las plantillas se gestionan desde el panel de Proyectos.</p>
          </div>
        </div>
        <div className="vista-proyecto-acciones">
          <button className="dabble-boton" onClick={onVolver}>
            ← Volver al editor
          </button>
        </div>
      </header>
      <div className="vista-proyecto-vacia">
        <div className="vista-proyecto-vacia-icono" aria-hidden="true">
          <LayoutTemplate size={34} strokeWidth={1.3} />
        </div>
        <h2>Plantillas de proyecto</h2>
        <p>Crea, revisa y elimina plantillas locales desde la pestaña Plantillas de Proyectos.</p>
        <button className="dabble-boton-primario" onClick={onGestionar}>
          Abrir gestor de plantillas
        </button>
      </div>
    </section>
  );
}
