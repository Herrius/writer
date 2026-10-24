import { afterEach, describe, expect, it } from 'vitest';
import {
  PREFERENCIAS_POR_DEFECTO,
  aplicarPreferencias,
  cargarPreferencias,
  guardarPreferencias,
} from './preferencias';

afterEach(() => {
  window.localStorage.clear();
  document.documentElement.removeAttribute('data-tema');
  document.documentElement.removeAttribute('style');
});

describe('preferencias de escritura', () => {
  it('carga los valores por defecto y aplica el tema y la tipografía', () => {
    expect(cargarPreferencias()).toEqual(PREFERENCIAS_POR_DEFECTO);
    aplicarPreferencias({
      ...PREFERENCIAS_POR_DEFECTO,
      tema: 'sepia',
      fuente: 'mono',
      tamano: 22,
      interlineado: 2,
      espaciadoParrafo: 1.4,
    });
    expect(document.documentElement.dataset.tema).toBe('sepia');
    expect(document.documentElement.style.getPropertyValue('--editor-font-size')).toBe(
      '22px',
    );
    expect(document.documentElement.style.getPropertyValue('--editor-line-height')).toBe(
      '2',
    );
    expect(document.documentElement.style.getPropertyValue('--editor-font-family')).toContain(
      'monospace',
    );
  });

  it('normaliza valores fuera de rango y persiste solo datos válidos', () => {
    guardarPreferencias({
      ...PREFERENCIAS_POR_DEFECTO,
      tema: 'oscuro',
      tamano: 200,
      interlineado: 0,
      espaciadoParrafo: -1,
    });
    const guardadas = cargarPreferencias();
    expect(guardadas.tema).toBe('oscuro');
    expect(guardadas.tamano).toBe(32);
    expect(guardadas.interlineado).toBe(1.2);
    expect(guardadas.espaciadoParrafo).toBe(0);
  });

  it('ignora JSON corrupto o versiones desconocidas', () => {
    window.localStorage.setItem('writer:preferencias:v1', '{no-json');
    expect(cargarPreferencias()).toEqual(PREFERENCIAS_POR_DEFECTO);
    window.localStorage.setItem(
      'writer:preferencias:v1',
      JSON.stringify({ version: 99, tema: 'oscuro' }),
    );
    expect(cargarPreferencias()).toEqual(PREFERENCIAS_POR_DEFECTO);
  });
});
