import { expect, test } from '@playwright/test';

test('crea trama, añade escena y crea documentos', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Plan' }).click();
  await page.getByLabel('Nombre de la nueva trama').fill('El faro');
  await page.getByRole('button', { name: 'Crear trama' }).click();
  await expect(page.locator('.dabble-trama').filter({ hasText: 'El faro' })).toBeVisible();

  await page.getByRole('button', { name: 'Añadir escena activa' }).click();
  await expect(page.getByText('Escena añadida a El faro.')).toBeVisible();
  await expect(page.locator('.dabble-trama').filter({ hasText: '1 escena' })).toBeVisible();

  await page.getByLabel('Título de la nueva escena').fill('Escena nueva');
  await page.getByRole('button', { name: 'Nueva escena' }).click();
  await expect(page.getByText('Escena creada: Escena nueva.')).toBeVisible();

  await page.getByLabel('Título de la nueva nota').fill('Idea nueva');
  await page.getByRole('button', { name: 'Nueva nota' }).click();
  await expect(page.getByText('Nota creada: Idea nueva.')).toBeVisible();
});
