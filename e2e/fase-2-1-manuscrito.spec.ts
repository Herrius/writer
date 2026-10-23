import { expect, test } from '@playwright/test';

test('renombra y duplica el documento activo desde Notas', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Notas' }).click();
  const titulo = page.getByRole('textbox', { name: 'Título del documento' });
  await titulo.fill('Nuevo título');
  await page.getByRole('button', { name: 'Renombrar' }).click();
  await expect(page.getByText(/Documento renombrado: Nuevo título/)).toBeVisible();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'Nuevo título' })).toBeVisible();

  await page.getByRole('button', { name: 'Duplicar' }).click();
  await expect(page.getByText(/Documento duplicado: Copia de Nuevo título/)).toBeVisible();
  await expect(
    page.locator('.dabble-doc').filter({ hasText: 'Copia de Nuevo título' }),
  ).toBeVisible();
});
