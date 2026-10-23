import { expect, test } from '@playwright/test';

test('reordena escenas dentro del capítulo', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.locator('.dabble-doc').filter({ hasText: 'La llave' }).click();
  await page.getByRole('tab', { name: 'Notas' }).click();
  await page.getByRole('button', { name: 'Subir' }).click();
  await expect(page.getByText('Orden de escenas actualizado.')).toBeVisible();
});
