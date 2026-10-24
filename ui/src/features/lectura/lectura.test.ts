import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  detenerLectura,
  hablar,
  listarVoces,
  textoParaLectura,
} from './lectura';

type VozFalsa = {
  name: string;
  lang: string;
  localService: boolean;
};

class UtteranceFalsa {
  text: string;
  rate = 1;
  lang = '';
  voice: VozFalsa | null = null;
  onend: (() => void) | null = null;
  onerror: ((evento: { error: string }) => void) | null = null;

  constructor(texto: string) {
    this.text = texto;
  }
}

const voces: VozFalsa[] = [
  { name: 'Voz española', lang: 'es-ES', localService: true },
  { name: 'English voice', lang: 'en-US', localService: true },
  { name: 'Voz remota', lang: 'es-ES', localService: false },
];

function instalarSintesis() {
  const hablarFn = vi.fn();
  const cancelarFn = vi.fn();
  const sintesis = {
    speaking: false,
    getVoices: () => voces,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    speak: hablarFn,
    cancel: cancelarFn,
    pause: vi.fn(),
    resume: vi.fn(),
  };
  Object.defineProperty(window, 'speechSynthesis', {
    configurable: true,
    value: sintesis,
  });
  vi.stubGlobal('SpeechSynthesisUtterance', UtteranceFalsa);
  return { sintesis, hablarFn, cancelarFn };
}

afterEach(() => {
  vi.unstubAllGlobals();
  delete (window as { speechSynthesis?: unknown }).speechSynthesis;
});

describe('lectura en voz alta', () => {
  it('convierte Markdown en texto plano sin ejecutar nada', () => {
    const texto = textoParaLectura(
      '# Capítulo\n\n**Marta** abrió la puerta.\n\n> Una cita.\n\n' +
        '[[personaje:01ABC|Marta]] miró el [faro](https://ejemplo.test).',
    );
    expect(texto).toBe('Capítulo Marta abrió la puerta. Una cita. Marta miró el faro.');
  });

  it('lista voces y elige una voz española', async () => {
    const { hablarFn, cancelarFn } = instalarSintesis();
    await expect(listarVoces()).resolves.toEqual([
      { nombre: 'Voz española', idioma: 'es-ES', local: true },
      { nombre: 'English voice', idioma: 'en-US', local: true },
    ]);

    expect(hablar('Hola mundo.', { velocidad: 1.5 })).toBe(true);
    expect(cancelarFn).toHaveBeenCalledTimes(1);
    expect(hablarFn).toHaveBeenCalledTimes(1);
    const utterance = hablarFn.mock.calls[0][0] as UtteranceFalsa;
    expect(utterance.text).toBe('Hola mundo.');
    expect(utterance.rate).toBe(1.5);
    expect(utterance.voice?.name).toBe('Voz española');
  });

  it('no usa una voz remota si no hay ninguna local', () => {
    const { sintesis, hablarFn } = instalarSintesis();
    sintesis.getVoices = () => [voces[2]];
    expect(hablar('Hola')).toBe(false);
    expect(hablarFn).not.toHaveBeenCalled();
  });

  it('detiene la lectura y no falla sin soporte', () => {
    const { cancelarFn } = instalarSintesis();
    detenerLectura();
    expect(cancelarFn).toHaveBeenCalledTimes(1);
    delete (window as { speechSynthesis?: unknown }).speechSynthesis;
    expect(hablar('Hola')).toBe(false);
  });
});
