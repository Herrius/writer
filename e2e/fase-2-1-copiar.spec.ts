import { expect, test } from '@playwright/test';

test('copia un documento y lo pega como una copia nueva', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Notas' }).click();
  await page.getByRole('button', { name: 'Copiar', exact: true }).click();
  await expect(page.getByText(/Documento copiado: El umbral/)).toBeVisible();
  await page.getByRole('button', { name: 'Pegar copia' }).click();
  await expect(page.getByText(/Documento duplicado: Copia de El umbral/)).toBeVisible();
  await expect(
    page.locator('.dabble-doc').filter({ hasText: 'Copia de El umbral' }),
  ).toBeVisible();
});
