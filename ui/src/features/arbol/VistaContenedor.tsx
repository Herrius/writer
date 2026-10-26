import { useState } from 'react';
import { FilePlus2, FolderPlus, MoreHorizontal, UserRound, UserPlus } from 'lucide-react';
import type { DocumentoDto } from '../../types/DocumentoDto';

const COLORES = ['#7d9d55', '#536f91', '#a97854', '#9a6f8e', '#5c8b86', '#b08a53'];

function colorDocumento(documento: DocumentoDto): string {
  let hash = 0;
  for (const caracter of documento.titulo) {
    hash = (hash * 31 + caracter.charCodeAt(0)) | 0;
  }
  return COLORES[Math.abs(hash) % COLORES.length];
}

function vistaPrevia(texto: string, vacio: string): string {
  const limpio = texto.replace(/[#*_`>\[\]]/g, ' ').replace(/\s+/g, ' ').trim();
  return limpio || vacio;
}

type PersonajesProps = {
  etiqueta: string;
  personajes: DocumentoDto[];
  onAbrir: (documento: DocumentoDto) => void;
  onCrear: () => void;
};

export function VistaGrupoPersonajes({
  etiqueta,
  personajes,
  onAbrir,
  onCrear,
}: PersonajesProps) {
  return (
    <section className="vista-contenedor" aria-labelledby="vista-grupo-titulo">
      <header className="vista-contenedor-cabecera">
        <div>
          <p className="vista-contenedor-migas">Personajes</p>
          <h1 id="vista-grupo-titulo">Personajes: {etiqueta}</h1>
        </div>
        <button className="dabble-boton-primario" onClick={onCrear}>
          <UserPlus size={15} strokeWidth={1.8} aria-hidden="true" />
          Crear un personaje
        </button>
      </header>
      {personajes.length === 0 ? (
        <div className="vista-contenedor-vacia">
          <UserRound size={32} strokeWidth={1.4} aria-hidden="true" />
          <strong>No hay personajes en esta sección</strong>
          <p>Crea el primero para empezar a desarrollar su ficha.</p>
        </div>
      ) : (
        <div className="vista-tarjetas">
          {personajes.map((personaje) => (
            <article className="tarjeta-personaje" key={personaje.id}>
              <header className="tarjeta-cabecera">
                <span className="tarjeta-tipo">
                  <UserRound size={14} strokeWidth={1.7} aria-hidden="true" />
                  Personaje
                </span>
                <button
                  className="tarjeta-menu"
                  aria-label={`Acciones de ${personaje.titulo}`}
                  title="Acciones"
                  onClick={() => onAbrir(personaje)}
                >
                  <MoreHorizontal size={16} aria-hidden="true" />
                </button>
              </header>
              <button
                className="tarjeta-cuerpo"
                onClick={() => onAbrir(personaje)}
                aria-label={`Abrir ${personaje.titulo}`}
              >
                <span
                  className="tarjeta-portada"
                  style={{ backgroundColor: colorDocumento(personaje) }}
                  aria-hidden="true"
                />
                <strong>{personaje.titulo}</strong>
                <span className="tarjeta-preview">
                  {vistaPrevia(personaje.cuerpo, 'Agrega notas para este personaje…')}
                </span>
              </button>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}

type CuadernoProps = {
  ruta: string;
  nombre: string;
  notas: DocumentoDto[];
  onAbrir: (documento: DocumentoDto) => void;
  onCrearNota: () => void;
  onCrearCarpeta: (nombre: string) => Promise<void>;
};

export function VistaCarpetaCuaderno({
  ruta,
  nombre,
  notas,
  onAbrir,
  onCrearNota,
  onCrearCarpeta,
}: CuadernoProps) {
  const [creando, setCreando] = useState(false);
  const [nombreCarpeta, setNombreCarpeta] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [procesando, setProcesando] = useState(false);

  const crearCarpeta = async () => {
    const limpio = nombreCarpeta.trim();
    if (!limpio) {
      setError('Escribe un nombre para la carpeta.');
      return;
    }
    setProcesando(true);
    setError(null);
    try {
      await onCrearCarpeta(limpio);
      setCreando(false);
      setNombreCarpeta('');
    } catch (fallo) {
      setError(fallo instanceof Error ? fallo.message : 'No se pudo crear la carpeta.');
    } finally {
      setProcesando(false);
    }
  };

  return (
    <section className="vista-contenedor" aria-labelledby="vista-cuaderno-titulo">
      <header className="vista-contenedor-cabecera">
        <div>
          <p className="vista-contenedor-migas">Cuaderno</p>
          <h1 id="vista-cuaderno-titulo">
            {nombre === 'Cuaderno' ? 'Cuaderno' : `Cuaderno: ${nombre}`}
          </h1>
        </div>
        <div className="vista-contenedor-acciones">
          <button className="dabble-boton-primario" onClick={onCrearNota}>
            <FilePlus2 size={15} strokeWidth={1.8} aria-hidden="true" />
            Crear una nota
          </button>
          <button className="dabble-boton" onClick={() => setCreando(true)}>
            <FolderPlus size={15} strokeWidth={1.8} aria-hidden="true" />
            Crear una carpeta
          </button>
        </div>
      </header>

      {creando && (
        <div className="vista-crear-carpeta">
          <label className="dabble-campo-meta">
            <span>Nombre de la carpeta</span>
            <input
              autoFocus
              value={nombreCarpeta}
              onChange={(evento) => setNombreCarpeta(evento.target.value)}
              onKeyDown={(evento) => {
                if (evento.key === 'Enter') {
                  evento.preventDefault();
                  void crearCarpeta();
                }
              }}
            />
          </label>
          <button className="dabble-boton-primario" disabled={procesando} onClick={() => void crearCarpeta()}>
            {procesando ? 'Creando…' : 'Crear'}
          </button>
          <button className="dabble-boton" onClick={() => setCreando(false)}>
            Cancelar
          </button>
          {error && (
            <p className="dabble-error" role="alert">
              {error}
            </p>
          )}
        </div>
      )}

      {notas.length === 0 ? (
        <div className="vista-contenedor-vacia">
          <FilePlus2 size={32} strokeWidth={1.4} aria-hidden="true" />
          <strong>Esta carpeta está vacía</strong>
          <p>Crea una nota para empezar a escribir dentro de {nombre}.</p>
        </div>
      ) : (
        <div className="vista-tarjetas">
          {notas.map((nota) => (
            <article className="tarjeta-nota" key={nota.id}>
              <header className="tarjeta-cabecera">
                <span className="tarjeta-tipo">
                  <FilePlus2 size={14} strokeWidth={1.7} aria-hidden="true" />
                  Nota
                </span>
                <button
                  className="tarjeta-menu"
                  aria-label={`Acciones de ${nota.titulo}`}
                  title="Acciones"
                  onClick={() => onAbrir(nota)}
                >
                  <MoreHorizontal size={16} aria-hidden="true" />
                </button>
              </header>
              <button
                className="tarjeta-cuerpo tarjeta-cuerpo-nota"
                onClick={() => onAbrir(nota)}
                aria-label={`Abrir ${nota.titulo}`}
              >
                <strong>{nota.titulo}</strong>
                <span className="tarjeta-preview">
                  {vistaPrevia(nota.cuerpo, 'Agregar una nota…')}
                </span>
              </button>
            </article>
          ))}
        </div>
      )}
      <p className="vista-cuaderno-ruta">{ruta}</p>
    </section>
  );
}
