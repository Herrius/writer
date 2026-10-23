import { expect, test } from '@playwright/test';

test('archiva, filtra y restaura un documento', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Notas' }).click();
  await page.getByRole('button', { name: 'Archivar' }).click();
  await expect(page.getByText('Documento archivado.')).toBeVisible();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toHaveCount(0);

  await page.getByRole('button', { name: /Archivo/ }).click();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toBeVisible();
  await page.locator('.dabble-doc').filter({ hasText: 'El umbral' }).click();
  await page.getByRole('tab', { name: 'Notas' }).click();
  await page.getByRole('button', { name: 'Restaurar' }).click();
  await expect(page.getByText('Documento restaurado a activos.')).toBeVisible();
  await page.getByRole('button', { name: /Volver a activos/ }).click();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toBeVisible();
});
