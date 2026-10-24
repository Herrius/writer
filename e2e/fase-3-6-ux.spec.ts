import { expect, test } from '@playwright/test';

test('el panel lateral y el diff siguen accesibles en ventana estrecha', async ({ page }) => {
  await page.setViewportSize({ width: 720, height: 600 });
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: 'Abrir panel lateral' }).click();
  await page.getByRole('tab', { name: 'Cambios' }).click();
  await page.getByRole('switch').check();
  await page.locator('.dabble-panel-cerrar').click();

  const editor = page.locator('.ProseMirror');
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.keyboard.type('Propuesta para revisar el diff.');
  await page.getByRole('button', { name: 'Abrir panel lateral' }).click();
  await page.getByRole('tab', { name: 'Cambios' }).click();
  await expect(page.getByText('Pendientes (1)')).toBeVisible({ timeout: 5000 });
  await page.getByRole('button', { name: 'Ver diferencias en grande' }).click();
  const dialogo = page.getByRole('dialog', { name: 'Diferencias del documento' });
  await expect(dialogo).toBeVisible();
  const caja = await dialogo.boundingBox();
  expect(caja?.width ?? 0).toBeGreaterThan(500);
  await expect(dialogo.getByText('Propuesta para revisar el diff.', { exact: true })).toBeVisible();
});

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
});

test('el modo propuesto se reinicia al volver a Proyectos', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('button', { name: /Control de cambios/ }).click();
  await page.getByRole('switch').check();
  await page.getByRole('button', { name: 'Volver a Proyectos' }).click();
  await expect(page.getByRole('heading', { name: 'Proyectos' })).toBeVisible();
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('button', { name: /Control de cambios/ }).click();
  await expect(page.getByRole('switch')).not.toBeChecked();
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
