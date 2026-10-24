import { expect, test } from '@playwright/test';

test('abre documentos y comandos desde la paleta', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.keyboard.press('Control+P');
  const paleta = page.getByRole('dialog', { name: 'Abrir documento' });
  await expect(paleta).toBeVisible();
  await page.getByRole('textbox', { name: 'Buscar documento' }).fill('La llave');
  await page.keyboard.press('Enter');
  await expect(page.getByRole('heading', { name: 'La llave' })).toBeVisible();

  await page.keyboard.press('Control+Shift+P');
  await expect(page.getByRole('dialog', { name: 'Paleta de comandos' })).toBeVisible();
  await page.getByRole('textbox', { name: 'Buscar comando' }).fill('sepia');
  await page.keyboard.press('Enter');
  await expect(page.locator('html')).toHaveAttribute('data-tema', 'sepia');
});

test('cambia preferencias y activa el modo enfoque', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: 'Preferencias', exact: true }).click();
  const dialogo = page.getByRole('dialog', { name: 'Preferencias de escritura' });
  await expect(dialogo).toBeVisible();
  await dialogo.getByRole('combobox', { name: 'Tema', exact: true }).selectOption('oscuro');
  await dialogo.getByRole('combobox', { name: 'Tipografía', exact: true }).selectOption('mono');
  await expect(page.locator('html')).toHaveAttribute('data-tema', 'oscuro');
  await dialogo.getByRole('button', { name: 'Listo' }).click();

  await page.keyboard.press('Alt+Z');
  await expect(page.locator('.dabble-app')).toHaveClass(/modo-enfoque/);
  await page.keyboard.press('Alt+Z');
  await expect(page.locator('.dabble-app')).not.toHaveClass(/modo-enfoque/);
});
