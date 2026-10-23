import { expect, test } from '@playwright/test';

test('crea una nota, responde y resuelve el hilo', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Comentarios' }).click();
  await page.getByLabel('Texto de la nota rápida').fill('Revisar el ritmo del primer párrafo.');
  await page.getByRole('button', { name: 'Agregar nota' }).click();
  await expect(page.getByText('Revisar el ritmo del primer párrafo.')).toBeVisible();

  await page.getByPlaceholder('Responder').fill('De acuerdo, lo reviso.');
  await page.getByRole('button', { name: 'Responder' }).click();
  await expect(page.getByText('De acuerdo, lo reviso.')).toBeVisible();

  await page.getByRole('button', { name: 'Resolver' }).click();
  await expect(page.getByText('resuelto', { exact: true })).toBeVisible();
});
