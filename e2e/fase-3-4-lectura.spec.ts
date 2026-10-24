import { expect, test } from '@playwright/test';

test('lee el documento con una voz local y muestra estadísticas', async ({ page }) => {
  await page.addInitScript(() => {
    class VozFalsa {
      text;
      rate = 1;
      lang = 'es-ES';
      voice = null;
      onend = null;
      onerror = null;
      constructor(texto) {
        this.text = texto;
      }
    }
    Object.defineProperty(window, 'SpeechSynthesisUtterance', {
      configurable: true,
      value: VozFalsa,
    });
    Object.defineProperty(window, 'speechSynthesis', {
      configurable: true,
      value: {
        speaking: false,
        getVoices: () => [
          { name: 'Voz de prueba', lang: 'es-ES', localService: true },
        ],
        addEventListener: () => {},
        removeEventListener: () => {},
        speak: () => {},
        cancel: () => {},
        pause: () => {},
        resume: () => {},
      },
    });
  });

  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('tab', { name: 'Herramientas' }).click();

  await expect(page.getByRole('heading', { name: 'Herramientas de escritura' })).toBeVisible();
  await expect(page.getByText('Palabras', { exact: true })).toBeVisible();
  await expect(page.getByText('Caracteres', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Leer documento' }).click();
  await expect(page.getByRole('button', { name: 'Pausar' })).toBeVisible();
  await page.getByRole('button', { name: 'Pausar' }).click();
  await expect(page.getByRole('button', { name: 'Reanudar' })).toBeVisible();
  await page.getByRole('button', { name: 'Reanudar' }).click();
  await page.getByRole('button', { name: 'Detener' }).click();
  await expect(page.getByRole('button', { name: 'Leer documento' })).toBeVisible();
});
