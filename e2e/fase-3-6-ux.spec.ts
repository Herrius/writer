import { expect, test } from '@playwright/test';

test('a 960 px con texto al 200 % las pestañas no se solapan', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 700 });
  await page.goto('/');
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '32px';
  });
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('tab', { name: 'Herramientas' }).click();
  await expect(page.getByRole('heading', { name: 'Herramientas de escritura' })).toBeVisible();
  const cajas = await page.getByRole('tab').evaluateAll((tabs) =>
    tabs.map((tab) => {
      const caja = tab.getBoundingClientRect();
      return { x: caja.x, y: caja.y, width: caja.width, height: caja.height };
    }),
  );
  expect(cajas.every((caja) => caja.width > 20 && caja.height > 20)).toBe(true);
  for (let izquierda = 0; izquierda < cajas.length; izquierda += 1) {
    for (let derecha = izquierda + 1; derecha < cajas.length; derecha += 1) {
      const a = cajas[izquierda];
      const b = cajas[derecha];
      const mismaFila = a.y < b.y + b.height && b.y < a.y + a.height;
      const seSolapan = a.x < b.x + b.width && b.x < a.x + a.width;
      expect(mismaFila && seSolapan).toBe(false);
    }
  }
});

test('los diálogos atrapan el foco y bloquean atajos globales', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: 'Preferencias', exact: true }).click();
  const dialogo = page.getByRole('dialog', { name: 'Preferencias de escritura' });
  await expect(dialogo).toBeVisible();
  await page.keyboard.press('Shift+Tab');
  expect(
    await page.evaluate(() => Boolean(document.activeElement?.closest('[role="dialog"]'))),
  ).toBe(true);
  await page.keyboard.press('Control+P');
  await expect(page.getByRole('dialog', { name: 'Abrir documento' })).toHaveCount(0);
  await page.keyboard.press('Escape');
  await expect(dialogo).toHaveCount(0);

  await page.keyboard.press('Control+P');
  const paleta = page.getByRole('dialog', { name: 'Abrir documento' });
  await expect(paleta).toBeVisible();
  await page.keyboard.press('Tab');
  expect(
    await page.evaluate(() => Boolean(document.activeElement?.closest('[role="dialog"]'))),
  ).toBe(true);
  await page.keyboard.press('Escape');
  await expect(paleta).toHaveCount(0);
});

test('el drawer de historial se cierra y el foco vuelve al botón', async ({ page }) => {
  await page.setViewportSize({ width: 720, height: 600 });
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('button', { name: 'Historial' }).click();
  await expect(page.getByRole('heading', { name: /Revisión/ })).toBeVisible();
  await page.getByRole('button', { name: 'Abrir panel lateral' }).click();
  const cerrar = page.locator('.dabble-panel-cerrar').first();
  await expect(cerrar).toBeFocused();
  await cerrar.click();
  await expect(page.getByRole('button', { name: 'Abrir panel lateral' })).toBeFocused();
});
