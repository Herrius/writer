import { useEffect, useMemo, useRef, useState } from 'react';
import {
  BookMarked,
  BookOpen,
  ChevronDown,
  ChevronRight,
  Download,
  FilePlus2,
  FileText,
  Folder,
  FolderOpen,
  FolderPlus,
  MoreHorizontal,
  Network,
  NotebookPen,
  Pencil,
  Trash2,
  Users,
  UserRound,
} from 'lucide-react';
import type { DocumentoDto } from '../../types/DocumentoDto';
import { useTrampaTab } from '../inicio/useTrampaTab';

export type TipoCreacion = 'libro' | 'capitulo' | 'seccion' | 'escena' | 'trama' | 'personaje' | 'nota';

type Acciones = {
  onCrear: (tipo: TipoCreacion, padre?: string) => void;
  onRenombrarCarpeta: (ruta: string, nombre: string) => Promise<void>;
  onEliminarCarpeta: (ruta: string, nombre: string) => Promise<void>;
  onExportarCarpeta: (ruta: string, nombre: string) => void;
  onRenombrarDocumento: (documento: DocumentoDto, titulo: string) => Promise<void>;
  onEliminarDocumento: (documento: DocumentoDto, titulo: string) => Promise<void>;
  onExportarDocumento: (documento: DocumentoDto) => void;
  onMoverDocumento: (documento: DocumentoDto, ruta: string) => void;
  gruposPersonajes: string[];
  gruposPersonajesOcultos: string[];
  carpetasCuadernoOcultas: string[];
  contenedorActivo: string | null;
  onSeleccionarGrupo: (grupo: string, etiqueta: string) => void;
  onSeleccionarCarpetaCuaderno: (ruta: string, nombre: string) => void;
  onCrearGrupoPersonaje: (nombre: string) => Promise<void>;
  onRenombrarGrupoPersonaje: (grupo: string, nombre: string) => Promise<void>;
  onEliminarGrupoPersonaje: (grupo: string) => Promise<void>;
  onCrearPersonajeEnGrupo: (grupo: string) => void;
  onCrearNotaCuaderno: (ruta: string) => void;
  onCrearCarpetaCuaderno: (padre: string, nombre: string) => Promise<void>;
  onRenombrarCarpetaCuaderno: (ruta: string, nombre: string) => Promise<void>;
  onEliminarCarpetaCuaderno: (ruta: string, nombre: string) => Promise<void>;
};

type Props = Acciones & {
  carpetas: string[];
  documentos: DocumentoDto[];
  activoId: string | null;
  filtroEtiqueta: string;
  tituloProyecto: string;
  onSeleccionar: (documento: DocumentoDto) => void;
};

type CarpetaNodo = {
  ruta: string;
  nombre: string;
  nivel: number;
  hijos: CarpetaNodo[];
  documentos: DocumentoDto[];
};

type Dialogo =
  | { tipo: 'renombrar-carpeta'; ruta: string; nombre: string; valor: string }
  | { tipo: 'eliminar-carpeta'; ruta: string; nombre: string }
  | { tipo: 'renombrar-documento'; documento: DocumentoDto; valor: string }
  | { tipo: 'eliminar-documento'; documento: DocumentoDto; nombre: string }
  | { tipo: 'crear-grupo'; valor: string }
  | { tipo: 'renombrar-grupo'; grupo: string; valor: string }
  | { tipo: 'eliminar-grupo'; grupo: string; nombre: string }
  | { tipo: 'crear-carpeta-cuaderno'; padre: string; valor: string }
  | { tipo: 'renombrar-carpeta-cuaderno'; ruta: string; valor: string }
  | { tipo: 'eliminar-carpeta-cuaderno'; ruta: string; nombre: string };

function etiquetaCarpeta(carpeta: string): string {
  const coincidencia = /^(\d+)[-_](.+)$/.exec(carpeta);
  const numero = coincidencia?.[1] ? Number(coincidencia[1]) : null;
  const nombre = (coincidencia?.[2] ?? carpeta)
    .replace(/[-_]/g, ' ')
    .replace(/\bcapitulo\b/gi, 'capítulo')
    .replace(/\blibro\b/gi, 'libro')
    .replace(/\bseccion\b/gi, 'sección');
  const capitalizada = nombre
    ? nombre.charAt(0).toUpperCase() + nombre.slice(1)
    : nombre;
  return numero === null ? capitalizada : `${capitalizada} ${numero}`;
}

function crearCarpetaRaiz(): CarpetaNodo {
  return { ruta: 'manuscrito', nombre: 'Manuscritos', nivel: 0, hijos: [], documentos: [] };
}

function insertarCarpeta(raiz: CarpetaNodo, ruta: string) {
  if (!ruta.startsWith('manuscrito/')) {
    return;
  }
  const partes = ruta.split('/').slice(1);
  let actual = raiz;
  let acumulada = 'manuscrito';
  for (const parte of partes) {
    acumulada = `${acumulada}/${parte}`;
    let hija = actual.hijos.find((nodo) => nodo.ruta === acumulada);
    if (!hija) {
      hija = {
        ruta: acumulada,
        nombre: parte,
        nivel: actual.nivel + 1,
        hijos: [],
        documentos: [],
      };
      actual.hijos.push(hija);
    }
    actual = hija;
  }
}

function ordenarArbol(nodo: CarpetaNodo) {
  nodo.hijos.sort((a, b) => a.ruta.localeCompare(b.ruta, 'es'));
  nodo.documentos.sort((a, b) => a.ruta.localeCompare(b.ruta, 'es'));
  nodo.hijos.forEach(ordenarArbol);
}

function construirArbol(
  carpetas: string[],
  documentos: DocumentoDto[],
  todos: DocumentoDto[],
): CarpetaNodo {
  const raiz = crearCarpetaRaiz();
  for (const carpeta of carpetas) {
    insertarCarpeta(raiz, carpeta);
  }
  for (const documento of documentos) {
    if (documento.tipo !== 'escena' || documento.archivo) {
      continue;
    }
    const partes = documento.ruta.split('/');
    const carpeta = partes.slice(0, -1).join('/');
    insertarCarpeta(raiz, carpeta);
    let actual = raiz;
    for (const parte of partes.slice(1, -1)) {
      actual = actual.hijos.find((nodo) => nodo.ruta === `${actual.ruta}/${parte}`) ?? actual;
    }
    actual.documentos.push(documento);
  }
  const conArchivos = new Set<string>();
  for (const documento of todos) {
    if (documento.tipo !== 'escena') {
      continue;
    }
    const partes = documento.ruta.split('/');
    for (let indice = 2; indice < partes.length; indice += 1) {
      conArchivos.add(partes.slice(0, indice).join('/'));
    }
  }
  const podar = (nodo: CarpetaNodo): boolean => {
    nodo.hijos = nodo.hijos.filter(podar);
    if (
      nodo.documentos.length === 0 &&
      nodo.hijos.length === 0 &&
      conArchivos.has(nodo.ruta)
    ) {
      return false;
    }
    return true;
  };
  raiz.hijos = raiz.hijos.filter(podar);
  ordenarArbol(raiz);
  return raiz;
}

function documentosDelArbol(nodo: CarpetaNodo): DocumentoDto[] {
  return [
    ...nodo.hijos.flatMap(documentosDelArbol),
    ...nodo.documentos,
  ];
}

function IconoCarpeta({ nivel, abierta }: { nivel: number; abierta: boolean }) {
  if (nivel === 1) {
    return <BookOpen size={15} strokeWidth={1.7} aria-hidden="true" />;
  }
  if (nivel === 2) {
    return <BookMarked size={15} strokeWidth={1.7} aria-hidden="true" />;
  }
  return abierta ? (
    <FolderOpen size={15} strokeWidth={1.7} aria-hidden="true" />
  ) : (
    <Folder size={15} strokeWidth={1.7} aria-hidden="true" />
  );
}

function BotonMenu({
  etiqueta,
  abierto,
  onClick,
}: {
  etiqueta: string;
  abierto: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      className={`arbol-menu-boton ${abierto ? 'abierto' : ''}`}
      aria-label={`Acciones de ${etiqueta}`}
      aria-expanded={abierto}
      title={`Acciones de ${etiqueta}`}
      onClick={(evento) => {
        evento.stopPropagation();
        onClick();
      }}
    >
      <MoreHorizontal size={16} strokeWidth={1.8} aria-hidden="true" />
    </button>
  );
}

function Menu({
  onCerrar,
  onCrear,
  tipoFiltro,
  onRenombrar,
  onEliminar,
  onExportar,
  onNuevaSeccion,
}: {
  onCerrar: () => void;
  onCrear?: (tipo: TipoCreacion) => void;
  tipoFiltro?: TipoCreacion[];
  onRenombrar?: () => void;
  onEliminar?: () => void;
  onExportar?: () => void;
  onNuevaSeccion?: () => void;
}) {
  const opciones: Array<{ tipo: TipoCreacion; etiqueta: string }> = [
    { tipo: 'libro', etiqueta: 'Libro' },
    { tipo: 'capitulo', etiqueta: 'Capítulo' },
    { tipo: 'seccion', etiqueta: 'Sección' },
    { tipo: 'escena', etiqueta: 'Escena' },
    { tipo: 'trama', etiqueta: 'Trama' },
    { tipo: 'personaje', etiqueta: 'Personaje' },
    { tipo: 'nota', etiqueta: 'Nota de cuaderno' },
  ];
  const visibles = onCrear
    ? opciones.filter((opcion) => !tipoFiltro || tipoFiltro.includes(opcion.tipo))
    : [];
  return (
    <div className="arbol-menu" role="menu" onMouseDown={(evento) => evento.stopPropagation()}>
      {visibles.length > 0 && (
        <>
          <span className="arbol-menu-titulo">Agregar nuevo…</span>
          {visibles.map((opcion) => (
            <button
              key={opcion.tipo}
              type="button"
              role="menuitem"
              onClick={() => {
                onCrear?.(opcion.tipo);
                onCerrar();
              }}
            >
              <FilePlus2 size={15} strokeWidth={1.7} aria-hidden="true" />
              {opcion.etiqueta}
            </button>
          ))}
          <div className="arbol-menu-separador" />
        </>
      )}
      {onNuevaSeccion && (
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onNuevaSeccion();
            onCerrar();
          }}
        >
          <FolderPlus size={15} strokeWidth={1.7} aria-hidden="true" />
          Nueva sección de personajes
        </button>
      )}
      {onRenombrar && (
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onRenombrar();
            onCerrar();
          }}
        >
          <Pencil size={15} strokeWidth={1.7} aria-hidden="true" />
          Renombrar
        </button>
      )}
      {onExportar && (
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onExportar();
            onCerrar();
          }}
        >
          <Download size={15} strokeWidth={1.7} aria-hidden="true" />
          Exportar
        </button>
      )}
      {onEliminar && (
        <button
          type="button"
          role="menuitem"
          className="peligro-menu"
          onClick={() => {
            onEliminar();
            onCerrar();
          }}
        >
          <Trash2 size={15} strokeWidth={1.7} aria-hidden="true" />
          Eliminar
        </button>
      )}
    </div>
  );
}

function MenuCuaderno({
  onCerrar,
  onNota,
  onCarpeta,
  onRenombrar,
  onEliminar,
}: {
  onCerrar: () => void;
  onNota: () => void;
  onCarpeta: () => void;
  onRenombrar?: () => void;
  onEliminar?: () => void;
}) {
  return (
    <div className="arbol-menu" role="menu" onMouseDown={(evento) => evento.stopPropagation()}>
      <span className="arbol-menu-titulo">Agregar nuevo…</span>
      <button
        type="button"
        role="menuitem"
        onClick={() => {
          onNota();
          onCerrar();
        }}
      >
        <FilePlus2 size={15} strokeWidth={1.7} aria-hidden="true" />
        Nota
      </button>
      <button
        type="button"
        role="menuitem"
        onClick={() => {
          onCarpeta();
          onCerrar();
        }}
      >
        <FolderPlus size={15} strokeWidth={1.7} aria-hidden="true" />
        Carpeta
      </button>
      {onRenombrar && (
        <button
          type="button"
          role="menuitem"
          onClick={() => {
            onRenombrar();
            onCerrar();
          }}
        >
          <Pencil size={15} strokeWidth={1.7} aria-hidden="true" />
          Renombrar
        </button>
      )}
      {onEliminar && (
        <button
          type="button"
          role="menuitem"
          className="peligro-menu"
          onClick={() => {
            onEliminar();
            onCerrar();
          }}
        >
          <Trash2 size={15} strokeWidth={1.7} aria-hidden="true" />
          Eliminar
        </button>
      )}
    </div>
  );
}

export function ArbolProyecto({
  carpetas,
  documentos,
  activoId,
  filtroEtiqueta,
  tituloProyecto,
  onSeleccionar,
  onMoverDocumento,
  onCrear,
  onRenombrarCarpeta,
  onEliminarCarpeta,
  onExportarCarpeta,
  onRenombrarDocumento,
  onEliminarDocumento,
  onExportarDocumento,
  gruposPersonajes,
  gruposPersonajesOcultos,
  carpetasCuadernoOcultas,
  contenedorActivo,
  onSeleccionarGrupo,
  onSeleccionarCarpetaCuaderno,
  onCrearGrupoPersonaje,
  onRenombrarGrupoPersonaje,
  onEliminarGrupoPersonaje,
  onCrearPersonajeEnGrupo,
  onCrearNotaCuaderno,
  onCrearCarpetaCuaderno,
  onRenombrarCarpetaCuaderno,
  onEliminarCarpetaCuaderno,
}: Props) {
  const [plegados, setPlegados] = useState<Set<string>>(new Set());
  const [menuAbierto, setMenuAbierto] = useState<string | null>(null);
  const [dialogo, setDialogo] = useState<Dialogo | null>(null);
  const [procesando, setProcesando] = useState(false);
  const [errorDialogo, setErrorDialogo] = useState<string | null>(null);
  const dialogoRef = useRef<HTMLFormElement | null>(null);
  useTrampaTab(dialogoRef);

  const visibles = useMemo(() => {
    const termino = filtroEtiqueta.trim().toLocaleLowerCase();
    return documentos.filter(
      (documento) =>
        !termino ||
        documento.etiquetas.some((etiqueta) => etiqueta.toLocaleLowerCase().includes(termino)),
    );
  }, [documentos, filtroEtiqueta]);
  const arbol = useMemo(
    () => construirArbol(carpetas, visibles, documentos),
    [carpetas, visibles, documentos],
  );
  const primerLibro = arbol.hijos[0]?.ruta ?? null;
  const nombreCarpeta = (nodo: CarpetaNodo) => {
    if (
      nodo.nivel === 1 &&
      nodo.ruta === primerLibro &&
      /^\d+-libro$/.test(nodo.nombre) &&
      tituloProyecto.trim()
    ) {
      return tituloProyecto.trim();
    }
    return etiquetaCarpeta(nodo.nombre);
  };
  const documentosArbol = useMemo(() => documentosDelArbol(arbol), [arbol]);
  const notas = useMemo(
    () => visibles.filter((documento) => documento.tipo === 'nota' && !documento.archivo),
    [visibles],
  );
  const tramas = notas.filter((nota) => nota.clase === 'tramas');
  const personajes = notas.filter((nota) => nota.clase === 'personajes');
  useEffect(() => {
    if (!menuAbierto) {
      return;
    }
    const cerrar = () => setMenuAbierto(null);
    document.addEventListener('click', cerrar);
    return () => document.removeEventListener('click', cerrar);
  }, [menuAbierto]);

  useEffect(() => {
    if (!dialogo) {
      return;
    }
    const alPulsar = (evento: KeyboardEvent) => {
      if (evento.key === 'Escape' && !procesando) {
        evento.preventDefault();
        setDialogo(null);
        setErrorDialogo(null);
      }
    };
    document.addEventListener('keydown', alPulsar);
    return () => document.removeEventListener('keydown', alPulsar);
  }, [dialogo, procesando]);

  const alternar = (ruta: string) => {
    setPlegados((actual) => {
      const siguiente = new Set(actual);
      if (siguiente.has(ruta)) {
        siguiente.delete(ruta);
      } else {
        siguiente.add(ruta);
      }
      return siguiente;
    });
  };

  const filtroCreacion = (nivel: number): TipoCreacion[] => {
    if (nivel === 1) {
      return ['capitulo', 'seccion', 'escena'];
    }
    if (nivel === 2) {
      return ['capitulo', 'seccion'];
    }
    return ['seccion', 'escena'];
  };

  const abrirRenombrarCarpeta = (nodo: CarpetaNodo) => {
    setErrorDialogo(null);
    setDialogo({
      tipo: 'renombrar-carpeta',
      ruta: nodo.ruta,
      nombre: etiquetaCarpeta(nodo.nombre),
      valor: nodo.nombre,
    });
  };

  const confirmar = async (evento: React.FormEvent) => {
    evento.preventDefault();
    if (!dialogo) {
      return;
    }
    setProcesando(true);
    setErrorDialogo(null);
    try {
      if (dialogo.tipo === 'renombrar-carpeta') {
        const nombre = dialogo.valor.trim();
        if (!nombre) {
          throw new Error('El nombre no puede estar vacío.');
        }
        await onRenombrarCarpeta(dialogo.ruta, nombre);
      } else if (dialogo.tipo === 'eliminar-carpeta') {
        await onEliminarCarpeta(dialogo.ruta, dialogo.nombre);
      } else if (dialogo.tipo === 'renombrar-documento') {
        const titulo = dialogo.valor.trim();
        if (!titulo) {
          throw new Error('El título no puede estar vacío.');
        }
        await onRenombrarDocumento(dialogo.documento, titulo);
      } else if (dialogo.tipo === 'crear-grupo') {
        const nombre = dialogo.valor.trim();
        if (!nombre) {
          throw new Error('El nombre de la sección no puede estar vacío.');
        }
        await onCrearGrupoPersonaje(nombre);
      } else if (dialogo.tipo === 'crear-carpeta-cuaderno') {
        const nombre = dialogo.valor.trim();
        if (!nombre) {
          throw new Error('El nombre de la carpeta no puede estar vacío.');
        }
        await onCrearCarpetaCuaderno(dialogo.padre, nombre);
      } else if (dialogo.tipo === 'renombrar-carpeta-cuaderno') {
        const nombre = dialogo.valor.trim();
        if (!nombre) {
          throw new Error('El nombre de la carpeta no puede estar vacío.');
        }
        await onRenombrarCarpetaCuaderno(dialogo.ruta, nombre);
      } else if (dialogo.tipo === 'eliminar-carpeta-cuaderno') {
        await onEliminarCarpetaCuaderno(dialogo.ruta, dialogo.nombre);
      } else if (dialogo.tipo === 'renombrar-grupo') {
        const nombre = dialogo.valor.trim();
        if (!nombre) {
          throw new Error('El nombre de la sección no puede estar vacío.');
        }
        await onRenombrarGrupoPersonaje(dialogo.grupo, nombre);
      } else if (dialogo.tipo === 'eliminar-grupo') {
        await onEliminarGrupoPersonaje(dialogo.grupo);
      } else {
        await onEliminarDocumento(dialogo.documento, dialogo.nombre);
      }
      setDialogo(null);
    } catch (fallo) {
      setErrorDialogo(fallo instanceof Error ? fallo.message : 'No se pudo completar la acción.');
    } finally {
      setProcesando(false);
    }
  };

  const renderCarpeta = (nodo: CarpetaNodo) => {
    const abierta = !plegados.has(nodo.ruta);
    const claveMenu = `carpeta:${nodo.ruta}`;
    return (
      <div
        className={`arbol-rama ${
          nodo.nivel === 1 ? 'dabble-libro' : nodo.nivel >= 2 ? 'dabble-capitulo' : ''
        }`}
        key={nodo.ruta}
        onDragOver={(evento) => {
          if (nodo.nivel >= 2) {
            evento.preventDefault();
            evento.dataTransfer.dropEffect = 'move';
          }
        }}
        onDrop={(evento) => {
          if (nodo.nivel < 2) {
            return;
          }
          const id = evento.dataTransfer.getData('text/plain');
          const documento = documentos.find((item) => item.id === id);
          if (documento) {
            evento.preventDefault();
            evento.stopPropagation();
            onMoverDocumento(documento, nodo.ruta);
          }
        }}
      >
        <div
          className={`arbol-fila arbol-carpeta nivel-${Math.min(nodo.nivel, 4)}`}
          style={{ paddingLeft: `${0.35 + nodo.nivel * 1.05}rem` }}
        >
          <button
            type="button"
            className="arbol-plegar"
            aria-label={abierta ? `Plegar ${nombreCarpeta(nodo)}` : `Desplegar ${nombreCarpeta(nodo)}`}
            onClick={() => alternar(nodo.ruta)}
          >
            {nodo.hijos.length + nodo.documentos.length > 0 ? (
              abierta ? (
                <ChevronDown size={14} aria-hidden="true" />
              ) : (
                <ChevronRight size={14} aria-hidden="true" />
              )
            ) : (
              <span className="arbol-plegar-hueco" />
            )}
          </button>
          <span className="arbol-icono">
            <IconoCarpeta nivel={nodo.nivel} abierta={abierta} />
          </span>
          <button
            type="button"
            className="arbol-nombre"
            onClick={() => alternar(nodo.ruta)}
            aria-expanded={abierta}
          >
            {nombreCarpeta(nodo)}
          </button>
          <BotonMenu
            etiqueta={nombreCarpeta(nodo)}
            abierto={menuAbierto === claveMenu}
            onClick={() => setMenuAbierto(menuAbierto === claveMenu ? null : claveMenu)}
          />
          {menuAbierto === claveMenu && (
            <Menu
              onCerrar={() => setMenuAbierto(null)}
              onCrear={(tipo) => onCrear(tipo, nodo.ruta)}
              tipoFiltro={filtroCreacion(nodo.nivel)}
              onRenombrar={() => abrirRenombrarCarpeta(nodo)}
              onEliminar={() =>
                setDialogo({
                  tipo: 'eliminar-carpeta',
                  ruta: nodo.ruta,
                  nombre: nombreCarpeta(nodo),
                })
              }
              onExportar={() => onExportarCarpeta(nodo.ruta, nombreCarpeta(nodo))}
            />
          )}
        </div>
        {abierta && (
          <div className="arbol-hijos">
            {nodo.hijos.map(renderCarpeta)}
            {nodo.documentos.map((documento) => renderDocumento(documento, nodo.nivel + 1))}
          </div>
        )}
      </div>
    );
  };

  const renderDocumento = (documento: DocumentoDto, nivel: number) => {
    const claveMenu = `doc:${documento.id}`;
    return (
      <div
        className="arbol-fila arbol-documento"
        key={documento.id}
        style={{ paddingLeft: `${0.35 + nivel * 1.05}rem` }}
      >
        <span className="arbol-icono">
          <FileText size={14} strokeWidth={1.7} aria-hidden="true" />
        </span>
        <button
          type="button"
          id={`doc-${documento.id}`}
          className={`dabble-doc arbol-doc ${activoId === documento.id ? 'activo' : ''}`}
          aria-current={activoId === documento.id ? 'page' : undefined}
          tabIndex={
            documento.id === activoId ||
            (!activoId && documentosArbol[0]?.id === documento.id)
              ? 0
              : -1
          }
          onClick={() => onSeleccionar(documento)}
          onKeyDown={(evento) => {
            if (!['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(evento.key)) {
              return;
            }
            evento.preventDefault();
            const indice = documentosArbol.findIndex((item) => item.id === documento.id);
            if (indice < 0) {
              return;
            }
            const siguiente =
              evento.key === 'Home'
                ? 0
                : evento.key === 'End'
                  ? documentosArbol.length - 1
                  : evento.key === 'ArrowDown'
                    ? Math.min(documentosArbol.length - 1, indice + 1)
                    : Math.max(0, indice - 1);
            const destino = documentosArbol[siguiente];
            if (destino) {
              document.getElementById(`doc-${destino.id}`)?.focus();
            }
          }}
          draggable={documento.tipo === 'escena'}
          onDragStart={(evento) => {
            if (documento.tipo !== 'escena') {
              return;
            }
            evento.dataTransfer.setData('text/plain', documento.id);
            evento.dataTransfer.effectAllowed = 'move';
          }}
        >
          <span className="dabble-doc-titulo">{documento.titulo}</span>
        </button>
        <BotonMenu
          etiqueta={documento.titulo}
          abierto={menuAbierto === claveMenu}
          onClick={() => setMenuAbierto(menuAbierto === claveMenu ? null : claveMenu)}
        />
        {menuAbierto === claveMenu && (
          <Menu
            onCerrar={() => setMenuAbierto(null)}
            onRenombrar={() =>
              setDialogo({
                tipo: 'renombrar-documento',
                documento,
                valor: documento.titulo,
              })
            }
            onEliminar={() =>
              setDialogo({
                tipo: 'eliminar-documento',
                documento,
                nombre: documento.titulo,
              })
            }
            onExportar={() => onExportarDocumento(documento)}
          />
        )}
      </div>
    );
  };

  const renderPersonajes = () => {
    const ids = [
      'principal',
      'secundario',
      '',
      ...gruposPersonajes,
      ...personajes.map((personaje) => personaje.rol ?? ''),
    ].filter((valor, indice, lista) => lista.indexOf(valor) === indice);
    const etiqueta = (grupo: string) => {
      if (grupo === 'principal') {
        return 'Personajes principales';
      }
      if (grupo === 'secundario') {
        return 'Personajes secundarios';
      }
      if (!grupo) {
        return 'Personajes sin clasificar';
      }
      return grupo;
    };
    const grupos = ids
      .map((grupo) => ({
        grupo,
        etiqueta: etiqueta(grupo),
        lista: personajes.filter((personaje) => (personaje.rol ?? '') === grupo),
      }))
      .filter(
        (item) =>
          item.lista.length > 0 ||
          gruposPersonajes.includes(item.grupo) ||
          !gruposPersonajesOcultos.includes(item.grupo),
      );
    const claveMenu = 'seccion:Personajes';
    return (
      <section className="arbol-seccion-notas" key="Personajes">
        <div className="arbol-fila arbol-carpeta arbol-categoria nivel-1">
          <span className="arbol-icono">
            <Users size={15} strokeWidth={1.7} aria-hidden="true" />
          </span>
          <span className="arbol-nombre arbol-nombre-estatico">Personajes</span>
          <BotonMenu
            etiqueta="Personajes"
            abierto={menuAbierto === claveMenu}
            onClick={() => setMenuAbierto(menuAbierto === claveMenu ? null : claveMenu)}
          />
          {menuAbierto === claveMenu && (
            <Menu
              onCerrar={() => setMenuAbierto(null)}
              onCrear={() => onCrear('personaje')}
              tipoFiltro={['personaje']}
              onNuevaSeccion={() => {
                setErrorDialogo(null);
                setDialogo({ tipo: 'crear-grupo', valor: '' });
              }}
            />
          )}
        </div>
        {grupos.map(({ grupo, etiqueta: nombre, lista }) => (
          <div key={grupo || 'sin-clasificar'}>
            <div
              className={`arbol-fila arbol-personajes-grupo ${
                contenedorActivo === `personajes:${grupo}` ? 'activo' : ''
              }`}
              style={{ paddingLeft: '1.7rem' }}
            >
              <span className="arbol-icono">
                <UserRound size={14} strokeWidth={1.7} aria-hidden="true" />
              </span>
              <button
                type="button"
                className="arbol-nombre"
                onClick={() => onSeleccionarGrupo(grupo, nombre)}
              >
                {nombre}
              </button>
              <BotonMenu
                etiqueta={nombre}
                abierto={menuAbierto === `grupo:${grupo}`}
                onClick={() =>
                  setMenuAbierto(menuAbierto === `grupo:${grupo}` ? null : `grupo:${grupo}`)
                }
              />
              {menuAbierto === `grupo:${grupo}` && (
                <Menu
                  onCerrar={() => setMenuAbierto(null)}
                  onCrear={() => onCrearPersonajeEnGrupo(grupo)}
                  tipoFiltro={['personaje']}
                  onRenombrar={() =>
                    setDialogo({
                      tipo: 'renombrar-grupo',
                      grupo,
                      valor: nombre,
                    })
                  }
                  onEliminar={() =>
                    setDialogo({
                      tipo: 'eliminar-grupo',
                      grupo,
                      nombre,
                    })
                  }
                />
              )}
            </div>
            {lista.map((documento) => renderDocumento(documento, 2))}
          </div>
        ))}
      </section>
    );
  };

  const renderCuaderno = () => {
    const notasCuaderno = notas.filter(
      (nota) => nota.clase !== 'tramas' && nota.clase !== 'personajes',
    );
    const activas = new Set<string>();
    for (const nota of notasCuaderno) {
      const partes = nota.ruta.split('/');
      for (let indice = 3; indice < partes.length; indice += 1) {
        activas.add(partes.slice(0, indice).join('/'));
      }
    }
    const carpetasCuaderno = [...new Set(
      carpetas.filter((carpeta) => carpeta.startsWith('notas/cuaderno/')),
    )]
      .filter(
        (carpeta) =>
          !carpetasCuadernoOcultas.includes(carpeta) || activas.has(carpeta),
      )
      .sort((a, b) => a.localeCompare(b, 'es'));
    const todas = new Set(carpetasCuaderno);
    for (const nota of notasCuaderno) {
      const partes = nota.ruta.split('/');
      if (partes[0] === 'notas' && partes[1] === 'cuaderno' && partes.length > 3) {
        todas.add(partes.slice(0, -1).join('/'));
      }
    }
    const notasDirectas = (ruta: string) =>
      notasCuaderno.filter(
        (nota) => nota.ruta.split('/').slice(0, -1).join('/') === ruta,
      );
    const raiz = 'notas/cuaderno';
    return (
      <section className="arbol-seccion-notas" key="Cuaderno">
        <div className="arbol-fila arbol-carpeta arbol-categoria nivel-1">
          <span className="arbol-icono">
            <NotebookPen size={15} strokeWidth={1.7} aria-hidden="true" />
          </span>
          <button
            type="button"
            className={`arbol-nombre ${
              contenedorActivo === `cuaderno:${raiz}` ? 'activo' : ''
            }`}
            onClick={() => onSeleccionarCarpetaCuaderno(raiz, 'Cuaderno')}
          >
            Cuaderno
          </button>
          <BotonMenu
            etiqueta="Cuaderno"
            abierto={menuAbierto === `cuaderno:${raiz}`}
            onClick={() =>
              setMenuAbierto(menuAbierto === `cuaderno:${raiz}` ? null : `cuaderno:${raiz}`)
            }
          />
          {menuAbierto === `cuaderno:${raiz}` && (
            <MenuCuaderno
              onCerrar={() => setMenuAbierto(null)}
              onNota={() => onCrearNotaCuaderno(raiz)}
              onCarpeta={() =>
                setDialogo({ tipo: 'crear-carpeta-cuaderno', padre: raiz, valor: '' })
              }
            />
          )}
        </div>
        {[...todas]
          .sort((a, b) => a.localeCompare(b, 'es'))
          .map((ruta) => {
            const profundidad = ruta.split('/').length - 2;
            const nombre = etiquetaCarpeta(ruta.split('/').at(-1) ?? ruta);
            return (
              <div key={ruta}>
                <div
                  className={`arbol-fila arbol-personajes-grupo ${
                    contenedorActivo === `cuaderno:${ruta}` ? 'activo' : ''
                  }`}
                  style={{ paddingLeft: `${1.35 + profundidad * 1.05}rem` }}
                >
                  <span className="arbol-icono">
                    <Folder size={14} strokeWidth={1.7} aria-hidden="true" />
                  </span>
                  <button
                    type="button"
                    className="arbol-nombre"
                    onClick={() => onSeleccionarCarpetaCuaderno(ruta, nombre)}
                  >
                    {nombre}
                  </button>
                  <BotonMenu
                    etiqueta={nombre}
                    abierto={menuAbierto === `cuaderno:${ruta}`}
                    onClick={() =>
                      setMenuAbierto(
                        menuAbierto === `cuaderno:${ruta}` ? null : `cuaderno:${ruta}`,
                      )
                    }
                  />
                  {menuAbierto === `cuaderno:${ruta}` && (
                    <MenuCuaderno
                      onCerrar={() => setMenuAbierto(null)}
                      onNota={() => onCrearNotaCuaderno(ruta)}
                      onCarpeta={() =>
                        setDialogo({ tipo: 'crear-carpeta-cuaderno', padre: ruta, valor: '' })
                      }
                      onRenombrar={() =>
                        setDialogo({
                          tipo: 'renombrar-carpeta-cuaderno',
                          ruta,
                          valor: ruta.split('/').at(-1) ?? '',
                        })
                      }
                      onEliminar={() =>
                        setDialogo({
                          tipo: 'eliminar-carpeta-cuaderno',
                          ruta,
                          nombre,
                        })
                      }
                    />
                  )}
                </div>
                {notasDirectas(ruta).map((nota) => renderDocumento(nota, profundidad + 2))}
              </div>
            );
          })}
        {notasDirectas(raiz).map((nota) => renderDocumento(nota, 2))}
        {todas.size === 0 && notasDirectas(raiz).length === 0 && (
          <p className="arbol-vacio">Sin notas de cuaderno.</p>
        )}
      </section>
    );
  };

  const renderSeccionNotas = (
    titulo: string,
    icono: React.ReactNode,
    lista: DocumentoDto[],
    tipo: 'trama' | 'personaje' | 'nota',
  ) => {
    const claveMenu = `seccion:${titulo}`;
    return (
      <section className="arbol-seccion-notas" key={titulo}>
        <div className="arbol-fila arbol-carpeta arbol-categoria nivel-1">
          <span className="arbol-icono">{icono}</span>
          <span className="arbol-nombre arbol-nombre-estatico">{titulo}</span>
          <BotonMenu
            etiqueta={titulo}
            abierto={menuAbierto === claveMenu}
            onClick={() => setMenuAbierto(menuAbierto === claveMenu ? null : claveMenu)}
          />
          {menuAbierto === claveMenu && (
            <Menu
              onCerrar={() => setMenuAbierto(null)}
              onCrear={() => onCrear(tipo)}
              tipoFiltro={[tipo]}
            />
          )}
        </div>
        {lista.map((documento) => renderDocumento(documento, 1))}
        {lista.length === 0 && <p className="arbol-vacio">Sin {titulo.toLowerCase()}.</p>}
      </section>
    );
  };

  return (
    <div className="arbol-proyecto">
      <div className="arbol-fila arbol-seccion-principal">
        <span className="arbol-icono">
          <BookOpen size={15} strokeWidth={1.8} aria-hidden="true" />
        </span>
        <span className="arbol-nombre arbol-nombre-estatico">Manuscritos</span>
        {arbol.hijos.length === 0 && (
          <BotonMenu
            etiqueta="Manuscritos"
            abierto={menuAbierto === 'seccion:manuscrito'}
            onClick={() =>
              setMenuAbierto(menuAbierto === 'seccion:manuscrito' ? null : 'seccion:manuscrito')
            }
          />
        )}
        {arbol.hijos.length === 0 && menuAbierto === 'seccion:manuscrito' && (
          <Menu
            onCerrar={() => setMenuAbierto(null)}
            onCrear={(tipo) => onCrear(tipo, 'manuscrito')}
            tipoFiltro={['libro']}
          />
        )}
      </div>
      {arbol.hijos.map(renderCarpeta)}
      {arbol.hijos.length === 0 && arbol.documentos.length === 0 && (
        <p className="arbol-vacio">Sin escenas.</p>
      )}

      {renderSeccionNotas(
        'Tramas',
        <Network size={15} strokeWidth={1.7} aria-hidden="true" />,
        tramas,
        'trama',
      )}
      {renderPersonajes()}
      {renderCuaderno()}

      {dialogo && (
        <div className="modal-fondo" role="presentation">
          <form
            className="modal-arbol"
            ref={dialogoRef}
            onSubmit={(evento) => void confirmar(evento)}
            role="dialog"
            aria-modal="true"
            aria-labelledby="arbol-dialogo-titulo"
          >
            <header className="modal-arbol-cabecera">
              <h2 id="arbol-dialogo-titulo">
                {dialogo.tipo === 'eliminar-carpeta' || dialogo.tipo === 'eliminar-documento'
                  ? 'Eliminar'
                  : dialogo.tipo === 'crear-grupo'
                    ? 'Nueva sección'
                    : dialogo.tipo === 'renombrar-grupo'
                      ? 'Renombrar sección'
                      : dialogo.tipo === 'crear-carpeta-cuaderno'
                        ? 'Nueva carpeta'
                        : dialogo.tipo === 'renombrar-carpeta-cuaderno'
                          ? 'Renombrar carpeta'
                          : 'Renombrar'}
              </h2>
              <button
                type="button"
                className="cerrar-modal"
                aria-label="Cerrar"
                onClick={() => !procesando && setDialogo(null)}
              >
                ×
              </button>
            </header>
            <div className="modal-arbol-cuerpo">
              {dialogo.tipo === 'renombrar-carpeta' ||
              dialogo.tipo === 'renombrar-documento' ||
              dialogo.tipo === 'crear-grupo' ||
              dialogo.tipo === 'renombrar-grupo' ||
              dialogo.tipo === 'crear-carpeta-cuaderno' ||
              dialogo.tipo === 'renombrar-carpeta-cuaderno' ? (
                <label className="dabble-campo-meta">
                  <span>
                    {dialogo.tipo === 'crear-grupo' || dialogo.tipo === 'renombrar-grupo'
                      ? 'Nombre de la sección'
                      : dialogo.tipo === 'crear-carpeta-cuaderno' ||
                          dialogo.tipo === 'renombrar-carpeta-cuaderno'
                        ? 'Nombre de la carpeta'
                        : 'Nombre'}
                  </span>
                  <input
                    autoFocus
                    value={dialogo.valor}
                    onChange={(evento) =>
                      setDialogo({ ...dialogo, valor: evento.target.value } as Dialogo)
                    }
                  />
                </label>
              ) : (
                <p>
                  ¿Eliminar <strong>{dialogo.nombre}</strong>? Los documentos se moverán a la
                  papelera para poder restaurarlos.
                </p>
              )}
              {errorDialogo && (
                <p className="dabble-error" role="alert">
                  {errorDialogo}
                </p>
              )}
            </div>
            <footer className="modal-arbol-pie">
              <button
                type="button"
                className="dabble-boton"
                disabled={procesando}
                onClick={() => setDialogo(null)}
              >
                Cancelar
              </button>
              <button
                type="submit"
                className={dialogo.tipo.startsWith('eliminar') ? 'peligro' : 'dabble-boton-primario'}
                disabled={procesando}
              >
                {procesando ? 'Procesando…' : dialogo.tipo.startsWith('eliminar') ? 'Eliminar' : 'Guardar'}
              </button>
            </footer>
          </form>
        </div>
      )}
    </div>
  );
}
