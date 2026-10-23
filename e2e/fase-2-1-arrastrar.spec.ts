import { expect, test } from '@playwright/test';

test('mueve una escena a otro capítulo con arrastrar y soltar', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  const origen = page.locator('.dabble-doc').filter({ hasText: 'La llave' });
  const destino = page
    .locator('.dabble-capitulo')
    .filter({ hasText: 'El umbral' })
    .first();
  await origen.dragTo(destino);

  await expect(page.getByText(/Documento movido a 01-libro\/01-capitulo/)).toBeVisible();
  await expect(
    page.locator('.dabble-capitulo').filter({ hasText: 'El umbral' }).filter({
      hasText: 'La llave',
    }),
  ).toBeVisible();
});
