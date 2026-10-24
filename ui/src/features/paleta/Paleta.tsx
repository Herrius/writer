import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react';
import type { DocumentoDto } from '../../types/DocumentoDto';

export type ComandoPaleta = {
  id: string;
  etiqueta: string;
  detalle?: string;
  atajo?: string;
};

type Props = {
  modo: 'documentos' | 'comandos';
  documentos: DocumentoDto[];
  comandos: ComandoPaleta[];
  onCerrar: () => void;
  onDocumento: (documento: DocumentoDto) => void;
  onComando: (id: string) => void;
};

export function Paleta({
  modo,
  documentos,
  comandos,
  onCerrar,
  onDocumento,
  onComando,
}: Props) {
  const [consulta, setConsulta] = useState('');
  const [seleccion, setSeleccion] = useState(0);
  const entradaRef = useRef<HTMLInputElement | null>(null);
  const dialogoRef = useRef<HTMLDivElement | null>(null);
  const anteriorFoco = useRef<HTMLElement | null>(null);
  const cerrarRef = useRef(onCerrar);
  cerrarRef.current = onCerrar;

  useEffect(() => {
    anteriorFoco.current = document.activeElement as HTMLElement | null;
    entradaRef.current?.focus();
    const alPulsar = (evento: globalThis.KeyboardEvent) => {
      if (evento.key === 'Escape') {
        evento.preventDefault();
        evento.stopPropagation();
        cerrarRef.current();
        return;
      }
      if (evento.key !== 'Tab' || !dialogoRef.current) {
        return;
      }
      const enfocables = dialogoRef.current.querySelectorAll<HTMLElement>(
        'input, button, [tabindex]:not([tabindex="-1"])',
      );
      if (enfocables.length === 0) {
        return;
      }
      const primero = enfocables[0];
      const ultimo = enfocables[enfocables.length - 1];
      if (evento.shiftKey && document.activeElement === primero) {
        evento.preventDefault();
        ultimo.focus();
      } else if (!evento.shiftKey && document.activeElement === ultimo) {
        evento.preventDefault();
        primero.focus();
      }
    };
    document.addEventListener('keydown', alPulsar);
    return () => {
      document.removeEventListener('keydown', alPulsar);
      anteriorFoco.current?.focus?.();
    };
  }, [modo]);

  const termino = consulta.trim().toLocaleLowerCase();
  const documentosFiltrados = useMemo(
    () =>
      documentos.filter(
        (documento) =>
          !termino ||
          documento.titulo.toLocaleLowerCase().includes(termino) ||
          documento.ruta.toLocaleLowerCase().includes(termino),
      ),
    [documentos, termino],
  );
  const comandosFiltrados = useMemo(
    () =>
      comandos.filter(
        (comando) =>
          !termino ||
          comando.etiqueta.toLocaleLowerCase().includes(termino) ||
          (comando.detalle ?? '').toLocaleLowerCase().includes(termino),
      ),
    [comandos, termino],
  );
  const total = modo === 'documentos' ? documentosFiltrados.length : comandosFiltrados.length;

  useEffect(() => {
    setSeleccion(0);
  }, [modo, termino]);

  const ejecutar = (indice: number) => {
    if (modo === 'documentos') {
      const documento = documentosFiltrados[indice];
      if (documento) {
        onDocumento(documento);
      }
    } else {
      const comando = comandosFiltrados[indice];
      if (comando) {
        onComando(comando.id);
      }
    }
  };

  const alPulsar = (evento: KeyboardEvent<HTMLInputElement>) => {
    if (evento.key === 'Escape') {
      evento.preventDefault();
      onCerrar();
    } else if (evento.key === 'ArrowDown') {
      evento.preventDefault();
      setSeleccion((valor) => (total === 0 ? 0 : (valor + 1) % total));
    } else if (evento.key === 'ArrowUp') {
      evento.preventDefault();
      setSeleccion((valor) => (total === 0 ? 0 : (valor - 1 + total) % total));
    } else if (evento.key === 'Enter') {
      evento.preventDefault();
      ejecutar(seleccion);
    }
  };

  return (
    <div
      className="paleta-fondo"
      role="presentation"
      onMouseDown={(evento) => {
        if (evento.target === evento.currentTarget) {
          onCerrar();
        }
      }}
    >
      <div
        className="paleta-comandos"
        role="dialog"
        aria-modal="true"
        aria-label={modo === 'documentos' ? 'Abrir documento' : 'Paleta de comandos'}
        ref={dialogoRef}
      >
        <input
          ref={entradaRef}
          value={consulta}
          onChange={(evento) => setConsulta(evento.target.value)}
          onKeyDown={alPulsar}
          placeholder={
            modo === 'documentos'
              ? 'Buscar un documento…'
              : 'Buscar un comando…'
          }
          aria-label={modo === 'documentos' ? 'Buscar documento' : 'Buscar comando'}
          aria-controls="paleta-lista"
          aria-activedescendant={
            total > 0 ? `paleta-opcion-${seleccion}` : undefined
          }
        />
        <ul id="paleta-lista" role="listbox" aria-label="Resultados">
          {modo === 'documentos' &&
            documentosFiltrados.map((documento, indice) => (
              <li
                key={documento.id}
                id={`paleta-opcion-${indice}`}
                role="option"
                aria-selected={seleccion === indice}
                className={seleccion === indice ? 'activo' : ''}
                onMouseEnter={() => setSeleccion(indice)}
                onClick={() => onDocumento(documento)}
              >
                <span>
                  <strong>{documento.titulo}</strong>
                  <small>{documento.ruta}</small>
                </span>
                <small>{documento.tipo}</small>
              </li>
            ))}
          {modo === 'comandos' &&
            comandosFiltrados.map((comando, indice) => (
              <li
                key={comando.id}
                id={`paleta-opcion-${indice}`}
                role="option"
                aria-selected={seleccion === indice}
                className={seleccion === indice ? 'activo' : ''}
                onMouseEnter={() => setSeleccion(indice)}
                onClick={() => onComando(comando.id)}
              >
                <span>
                  <strong>{comando.etiqueta}</strong>
                  {comando.detalle && <small>{comando.detalle}</small>}
                </span>
                {comando.atajo && <kbd>{comando.atajo}</kbd>}
              </li>
            ))}
          {total === 0 && <li className="paleta-vacio">Sin resultados.</li>}
        </ul>
        <footer>
          <span>↑↓ moverse</span>
          <span>Enter abrir</span>
          <span>Esc cerrar</span>
        </footer>
      </div>
    </div>
  );
}
