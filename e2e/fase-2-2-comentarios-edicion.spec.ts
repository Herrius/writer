import { expect, test } from '@playwright/test';

test('edita un mensaje y archiva el hilo', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('tab', { name: 'Comentarios' }).click();
  await page.getByLabel('Texto de la nota rápida').fill('Primera versión.');
  await page.getByRole('button', { name: 'Agregar nota' }).click();
  await expect(page.getByText('Primera versión.')).toBeVisible();

  await page.getByRole('button', { name: 'Editar' }).click();
  await page.getByLabel('Editar mensaje').fill('Versión editada.');
  await page.getByRole('button', { name: 'Guardar edición' }).click();
  await expect(page.getByText('Versión editada.')).toBeVisible();

  await page.getByRole('button', { name: 'Archivar hilo' }).click();
  await expect(page.getByText('Primera versión.')).toBeHidden();
});
