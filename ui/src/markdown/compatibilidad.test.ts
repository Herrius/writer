import { describe, expect, it } from 'vitest';
import { analizarCompatibilidad } from './compatibilidad';

function construcciones(markdown: string): string[] {
  return analizarCompatibilidad(markdown).map((hallazgo) => hallazgo.construccion);
}

describe('compatibilidad del editor', () => {
  it('detecta las construcciones que pierden contenido', () => {
    expect(construcciones('Un <span>texto</span>.\n')).toContain('HTML embebido');
    expect(construcciones('Un <br/> y un <!-- comentario -->.\n')).toContain('HTML embebido');
    expect(construcciones('![Un faro](faro.png)\n')).toContain('imágenes');
    expect(construcciones('| A | B |\n|---|---|\n| 1 | 2 |\n')).toContain('tablas');
    expect(construcciones('- [ ] pendiente\n')).toContain('listas de tareas');
    expect(construcciones('Texto[^1]\n\n[^1]: Nota.\n')).toContain('notas al pie');
  });

  it('ignora las construcciones dentro de código', () => {
    expect(construcciones('```html\n<span>literal</span>\n```\n')).toEqual([]);
    expect(construcciones('Usa `<span>` y `![alt](x.png)` literalmente.\n')).toEqual([]);
    expect(construcciones('```\n| A | B |\n|---|---|\n```\n')).toEqual([]);
  });

  it('no confunde prosa normal con HTML, tablas ni enlaces', () => {
    expect(construcciones('1 < 2 y 3 > 2; Tom & Jerry.\n')).toEqual([]);
    expect(construcciones('<https://example.com>\n')).toEqual([]);
    expect(construcciones('[texto](https://example.com)\n')).toEqual([]);
    expect(construcciones('[texto][ref]\n\n[ref]: https://example.com\n')).toEqual([]);
    expect(construcciones('Un texto con | barras sueltas.\n')).toEqual([]);
  });

  it('acepta el esquema soportado sin hallazgos', () => {
    const markdown = [
      '# Título',
      '',
      'Una *cursiva*, una **negrita** y `código`.',
      '',
      '- Uno',
      '- Dos',
      '',
      '> Una cita.',
      '',
      '---',
      '',
      '[Enlace](https://example.com)',
      '',
    ].join('\n');
    expect(construcciones(markdown)).toEqual([]);
  });
});
