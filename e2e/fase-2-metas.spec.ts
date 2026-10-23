import { expect, test } from '@playwright/test';

test('muestra, guarda y ejecuta metas y temporizador', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await expect(page.getByRole('heading', { name: 'Tus metas' })).toBeVisible();
  const diaria = page.getByLabel('Meta diaria de palabras');
  await expect(diaria).toHaveValue('500');
  await diaria.fill('750');
  await page.getByRole('button', { name: 'Guardar metas' }).click();
  await expect(page.getByText('Metas guardadas.')).toBeVisible();

  await page.getByRole('button', { name: 'Iniciar temporizador' }).click();
  await expect(page.getByRole('button', { name: 'Pausar temporizador' })).toBeVisible();
  await page.getByRole('button', { name: 'Pausar temporizador' }).click();
  await expect(page.getByRole('button', { name: 'Reanudar temporizador' })).toBeVisible();
  await page.getByRole('button', { name: 'Reanudar temporizador' }).click();
  await page.getByRole('button', { name: 'Finalizar' }).click();
  await expect(page.getByText('Sesión de 5 min registrada.')).toBeVisible();
});
