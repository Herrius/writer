import { useEffect, type RefObject } from 'react';

/** Mantiene Tab y Shift+Tab dentro del diálogo. */
export function useTrampaTab(ref: RefObject<HTMLElement | null>): void {
  useEffect(() => {
    const alPulsar = (evento: KeyboardEvent) => {
      if (evento.key !== 'Tab' || !ref.current) {
        return;
      }
      const enfocables = Array.from(
        ref.current.querySelectorAll<HTMLElement>(
          'button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [href], [tabindex]:not([tabindex="-1"])',
        ),
      ).filter((elemento) => elemento.offsetParent !== null);
      if (enfocables.length === 0) {
        return;
      }
      const primero = enfocables[0];
      const ultimo = enfocables[enfocables.length - 1];
      const activo = document.activeElement;
      if (!ref.current.contains(activo)) {
        evento.preventDefault();
        primero.focus();
      } else if (evento.shiftKey && activo === primero) {
        evento.preventDefault();
        ultimo.focus();
      } else if (!evento.shiftKey && activo === ultimo) {
        evento.preventDefault();
        primero.focus();
      }
    };
    document.addEventListener('keydown', alPulsar);
    return () => document.removeEventListener('keydown', alPulsar);
  }, [ref]);
}
