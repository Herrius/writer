export type MetaDiariaGlobal = {
  objetivo: number;
  desde: string;
  diasLibres: number[];
  contarSoloManuscrito: boolean;
  contarPegadas: boolean;
  contarEliminaciones: boolean;
  recordatorio: boolean;
};

export type MetasGlobales = {
  version: 1;
  diaria: MetaDiariaGlobal | null;
  actividad: Record<string, number>;
};

const CLAVE = 'writer.metas-globales.v1';

export const METAS_GLOBALES_POR_DEFECTO: MetasGlobales = {
  version: 1,
  diaria: null,
  actividad: {},
};

export function fechaLocal(fecha = new Date()): string {
  const anio = fecha.getFullYear();
  const mes = String(fecha.getMonth() + 1).padStart(2, '0');
  const dia = String(fecha.getDate()).padStart(2, '0');
  return `${anio}-${mes}-${dia}`;
}

function normalizarMeta(valor: unknown): MetaDiariaGlobal | null {
  if (!valor || typeof valor !== 'object') {
    return null;
  }
  const meta = valor as Partial<MetaDiariaGlobal>;
  const objetivo = Math.floor(Number(meta.objetivo));
  if (!Number.isFinite(objetivo) || objetivo < 1 || objetivo > 1_000_000) {
    return null;
  }
  const diasLibres = Array.isArray(meta.diasLibres)
    ? Array.from(
        new Set(
          meta.diasLibres.filter(
            (dia): dia is number =>
              typeof dia === 'number' && Number.isInteger(dia) && dia >= 0 && dia <= 6,
          ),
        ),
      )
    : [];
  return {
    objetivo,
    desde:
      typeof meta.desde === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(meta.desde)
        ? meta.desde
        : fechaLocal(),
    diasLibres,
    contarSoloManuscrito: meta.contarSoloManuscrito !== false,
    contarPegadas: meta.contarPegadas !== false,
    contarEliminaciones: meta.contarEliminaciones !== false,
    recordatorio: meta.recordatorio === true,
  };
}

export function normalizarMetasGlobales(valor: unknown): MetasGlobales {
  if (!valor || typeof valor !== 'object') {
    return { ...METAS_GLOBALES_POR_DEFECTO, actividad: {} };
  }
  const datos = valor as Partial<MetasGlobales>;
  const actividad: Record<string, number> = {};
  if (datos.actividad && typeof datos.actividad === 'object') {
    for (const [fecha, palabras] of Object.entries(datos.actividad)) {
      if (
        /^\d{4}-\d{2}-\d{2}$/.test(fecha) &&
        typeof palabras === 'number' &&
        Number.isFinite(palabras) &&
        palabras >= 0
      ) {
        actividad[fecha] = Math.floor(palabras);
      }
    }
  }
  return {
    version: 1,
    diaria: normalizarMeta(datos.diaria),
    actividad,
  };
}

export function cargarMetasGlobales(): MetasGlobales {
  try {
    return normalizarMetasGlobales(
      JSON.parse(window.localStorage.getItem(CLAVE) ?? 'null'),
    );
  } catch {
    return { ...METAS_GLOBALES_POR_DEFECTO, actividad: {} };
  }
}

export function guardarMetasGlobales(metas: MetasGlobales): void {
  try {
    window.localStorage.setItem(CLAVE, JSON.stringify(normalizarMetasGlobales(metas)));
  } catch {
    // Sin almacenamiento, la sesión sigue usando el estado en memoria.
  }
}

export function registrarActividad(
  metas: MetasGlobales,
  fecha: string,
  deltaPalabras: number,
): MetasGlobales {
  if (!Number.isFinite(deltaPalabras) || deltaPalabras === 0) {
    return metas;
  }
  const anterior = metas.actividad[fecha] ?? 0;
  const siguiente = Math.max(0, anterior + Math.trunc(deltaPalabras));
  return {
    ...metas,
    actividad: { ...metas.actividad, [fecha]: siguiente },
  };
}

export function esDiaLibre(meta: MetaDiariaGlobal | null, fecha = new Date()): boolean {
  return Boolean(meta?.diasLibres.includes(fecha.getDay()));
}

export function progresoDelDia(
  metas: MetasGlobales,
  fecha = new Date(),
): { hechas: number; objetivo: number; porcentaje: number; restantes: number } {
  const hechas = Math.max(0, metas.actividad[fechaLocal(fecha)] ?? 0);
  const objetivo = metas.diaria?.objetivo ?? 0;
  return {
    hechas,
    objetivo,
    porcentaje: objetivo > 0 ? Math.min(100, Math.round((hechas / objetivo) * 100)) : 0,
    restantes: Math.max(0, objetivo - hechas),
  };
}

export function diasRecientes(
  metas: MetasGlobales,
  cantidad: number,
  hasta = new Date(),
): Array<{ fecha: string; palabras: number; etiqueta: string }> {
  const dias: Array<{ fecha: string; palabras: number; etiqueta: string }> = [];
  const etiquetas = ['D', 'L', 'M', 'X', 'J', 'V', 'S'];
  for (let atras = cantidad - 1; atras >= 0; atras -= 1) {
    const fecha = new Date(hasta);
    fecha.setDate(fecha.getDate() - atras);
    const clave = fechaLocal(fecha);
    dias.push({
      fecha: clave,
      palabras: metas.actividad[clave] ?? 0,
      etiqueta: etiquetas[fecha.getDay()],
    });
  }
  return dias;
}

export function rachaGlobal(
  metas: MetasGlobales,
  hasta = new Date(),
): number {
  if (!metas.diaria) {
    return 0;
  }
  let racha = 0;
  const cursor = new Date(hasta);
  for (let atras = 0; atras < 366; atras += 1) {
    const clave = fechaLocal(cursor);
    const palabras = metas.actividad[clave] ?? 0;
    if (esDiaLibre(metas.diaria, cursor)) {
      racha += 1;
    } else if (palabras >= metas.diaria.objetivo) {
      racha += 1;
    } else {
      break;
    }
    cursor.setDate(cursor.getDate() - 1);
  }
  return racha;
}
