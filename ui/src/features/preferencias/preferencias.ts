export type Tema = 'claro' | 'sepia' | 'oscuro';
export type FuenteEditor = 'serif' | 'sans' | 'mono';

export type Preferencias = {
  version: 1;
  tema: Tema;
  fuente: FuenteEditor;
  tamano: number;
  interlineado: number;
  espaciadoParrafo: number;
  corrector: boolean;
  modoEnfoque: boolean;
};

const CLAVE = 'writer:preferencias:v1';

export const PREFERENCIAS_POR_DEFECTO: Preferencias = {
  version: 1,
  tema: 'claro',
  fuente: 'serif',
  tamano: 18,
  interlineado: 1.85,
  espaciadoParrafo: 1.05,
  corrector: true,
  modoEnfoque: false,
};

export const FUENTES: Record<FuenteEditor, string> = {
  serif: 'Georgia, "Times New Roman", serif',
  sans: '"Inter", "Segoe UI", system-ui, sans-serif',
  mono: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
};

function limitar(valor: number, minimo: number, maximo: number): number {
  if (!Number.isFinite(valor)) {
    return minimo;
  }
  return Math.min(maximo, Math.max(minimo, valor));
}

function normalizar(valor: Partial<Preferencias> | null): Preferencias {
  if (!valor || valor.version !== 1) {
    return { ...PREFERENCIAS_POR_DEFECTO };
  }
  return {
    version: 1,
    tema: ['claro', 'sepia', 'oscuro'].includes(valor.tema ?? '')
      ? (valor.tema as Tema)
      : PREFERENCIAS_POR_DEFECTO.tema,
    fuente: ['serif', 'sans', 'mono'].includes(valor.fuente ?? '')
      ? (valor.fuente as FuenteEditor)
      : PREFERENCIAS_POR_DEFECTO.fuente,
    tamano: Math.round(limitar(valor.tamano ?? 18, 12, 32)),
    interlineado: Number(limitar(valor.interlineado ?? 1.85, 1.2, 2.6).toFixed(2)),
    espaciadoParrafo: Number(limitar(valor.espaciadoParrafo ?? 1.05, 0, 2.5).toFixed(2)),
    corrector: valor.corrector ?? true,
    modoEnfoque: valor.modoEnfoque ?? false,
  };
}

export function cargarPreferencias(): Preferencias {
  if (typeof window === 'undefined') {
    return { ...PREFERENCIAS_POR_DEFECTO };
  }
  try {
    const texto = window.localStorage.getItem(CLAVE);
    return normalizar(texto ? (JSON.parse(texto) as Partial<Preferencias>) : null);
  } catch {
    return { ...PREFERENCIAS_POR_DEFECTO };
  }
}

export function guardarPreferencias(preferencias: Preferencias): void {
  if (typeof window === 'undefined') {
    return;
  }
  try {
    window.localStorage.setItem(CLAVE, JSON.stringify(normalizar(preferencias)));
  } catch {
    // Un WebView sin almacenamiento sigue usando las preferencias en memoria.
  }
}

export function aplicarPreferencias(
  preferencias: Preferencias,
  raiz: HTMLElement | null = typeof document === 'undefined'
    ? null
    : document.documentElement,
): void {
  if (!raiz) {
    return;
  }
  const normalizadas = normalizar(preferencias);
  raiz.dataset.tema = normalizadas.tema;
  raiz.style.colorScheme = normalizadas.tema === 'oscuro' ? 'dark' : 'light';
  raiz.style.setProperty('--editor-font-family', FUENTES[normalizadas.fuente]);
  raiz.style.setProperty('--editor-font-size', `${normalizadas.tamano}px`);
  raiz.style.setProperty('--editor-line-height', `${normalizadas.interlineado}`);
  raiz.style.setProperty('--editor-paragraph-gap', `${normalizadas.espaciadoParrafo}em`);
}
