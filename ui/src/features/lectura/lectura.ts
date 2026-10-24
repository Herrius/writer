import { textoSinMarcas } from '../../palabras';

export type VozSistema = {
  nombre: string;
  idioma: string;
  local: boolean;
};

export type OpcionesLectura = {
  voz?: string;
  velocidad?: number;
  onFin?: () => void;
  onError?: (mensaje: string) => void;
};

/** `true` si el WebView expone la síntesis de voz del sistema. */
export function soportaLectura(): boolean {
  return typeof window !== 'undefined' && 'speechSynthesis' in window;
}

/** Voces del sistema, sin red. Puede devolver una lista vacía. */
export async function listarVoces(): Promise<VozSistema[]> {
  if (!soportaLectura()) {
    return [];
  }
  const sintesis = window.speechSynthesis;
  const actuales = sintesis.getVoices().filter((voz) => voz.localService);
  if (actuales.length > 0) {
    return actuales.map((voz) => ({
      nombre: voz.name,
      idioma: voz.lang,
      local: true,
    }));
  }
  return new Promise((resolver) => {
    let terminado = false;
    const finalizar = () => {
      if (terminado) {
        return;
      }
      terminado = true;
      sintesis.removeEventListener?.('voiceschanged', finalizar);
      resolver(
        sintesis
          .getVoices()
          .filter((voz) => voz.localService)
          .map((voz) => ({
            nombre: voz.name,
            idioma: voz.lang,
            local: true,
          })),
      );
    };
    sintesis.addEventListener?.('voiceschanged', finalizar, { once: true });
    window.setTimeout(finalizar, 1200);
  });
}

/** Texto plano para leer en voz alta; no altera el documento. */
export function textoParaLectura(markdown: string): string {
  return textoSinMarcas(markdown)
    .replace(/```[\s\S]*?```/g, ' ')
    .replace(/`([^`]+)`/g, '$1')
    .replace(/!\[[^\]]*\]\([^)]*\)/g, ' ')
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/^#{1,6}\s+/gm, '')
    .replace(/^>\s?/gm, '')
    .replace(/[*_~]{1,3}/g, '')
    .replace(/<[^>]+>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

/**
 * Lee texto con la voz del sistema. Cancela cualquier lectura anterior.
 * Devuelve `false` si el WebView no soporta síntesis de voz.
 */
export function hablar(texto: string, opciones: OpcionesLectura = {}): boolean {
  if (!soportaLectura()) {
    return false;
  }
  const limpio = texto.trim();
  if (!limpio) {
    return false;
  }
  const sintesis = window.speechSynthesis;
  sintesis.cancel();
  const utterance = new SpeechSynthesisUtterance(limpio);
  const velocidad = Math.min(2, Math.max(0.5, opciones.velocidad ?? 1));
  utterance.rate = velocidad;
  utterance.lang = 'es-ES';
  const voces = sintesis.getVoices().filter((voz) => voz.localService);
  if (voces.length === 0) {
    return false;
  }
  const elegida =
    (opciones.voz && voces.find((voz) => voz.name === opciones.voz)) ||
    voces.find((voz) => voz.lang.toLowerCase().startsWith('es')) ||
    voces[0];
  utterance.voice = elegida;
  utterance.lang = elegida.lang;
  utterance.onend = () => opciones.onFin?.();
  utterance.onerror = (evento) => {
    const mensaje =
      evento.error === 'interrupted' || evento.error === 'canceled'
        ? ''
        : `No se pudo leer el texto (${evento.error}).`;
    if (mensaje) {
      opciones.onError?.(mensaje);
    } else {
      opciones.onFin?.();
    }
  };
  try {
    sintesis.speak(utterance);
    return true;
  } catch {
    return false;
  }
}

export function pausarLectura(): void {
  if (soportaLectura()) {
    window.speechSynthesis.pause();
  }
}

export function reanudarLectura(): void {
  if (soportaLectura()) {
    window.speechSynthesis.resume();
  }
}

export function detenerLectura(): void {
  if (soportaLectura()) {
    window.speechSynthesis.cancel();
  }
}

export function estaLeyendo(): boolean {
  return soportaLectura() && window.speechSynthesis.speaking;
}
