import { expect, test } from '@playwright/test';

test('edita nombre y color de una trama y mueve sus puntos', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('tab', { name: 'Plan' }).click();
  await page.getByLabel('Nombre de la nueva trama').fill('El faro');
  await page.getByRole('button', { name: 'Crear trama' }).click();
  await page.getByLabel('Nombre de la trama seleccionada').fill('El faro nuevo');
  await page.getByRole('button', { name: 'Guardar trama' }).click();
  await expect(page.getByText('Trama actualizada.')).toBeVisible();
  await expect(page.locator('.dabble-trama').filter({ hasText: 'El faro nuevo' })).toBeVisible();
});
