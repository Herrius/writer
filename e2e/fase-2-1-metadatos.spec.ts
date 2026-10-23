import { expect, test } from '@playwright/test';

test('actualiza estado y etiquetas y filtra por etiqueta', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Notas' }).click();
  await page.getByLabel('Estado de la escena').selectOption('final');
  await page.getByLabel('Etiquetas del documento').fill('clave, faro');
  await page.getByRole('button', { name: 'Guardar metadatos' }).click();
  await expect(page.getByText('Metadatos del documento guardados.')).toBeVisible();

  await page.getByLabel('Filtrar por etiqueta').fill('clave');
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toBeVisible();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'La llave' })).toHaveCount(0);
});
