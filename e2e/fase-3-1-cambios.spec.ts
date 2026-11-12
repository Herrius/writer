import { expect, test } from '@playwright/test';

test('la línea de tiempo recorre solo las revisiones disponibles', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('button', { name: 'Historial' }).click();
  await expect(page.getByRole('heading', { name: 'Revisión 3' })).toBeVisible();

  await page
    .getByRole('slider', { name: 'Línea de tiempo de revisiones' })
    .fill('0');
  await expect(page.getByRole('heading', { name: 'Revisión 1' })).toBeVisible();
  await expect(page.getByText('Revisión 1 · 1/3')).toBeVisible();

  await page.getByRole('button', { name: 'Revisión siguiente' }).click();
  await expect(page.getByRole('heading', { name: 'Revisión 2' })).toBeVisible();
  await expect(page.getByText('Revisión 2 · 2/3')).toBeVisible();
});

test('compara una revisión con el documento actual', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: 'Historial' }).click();
  await expect(page.getByRole('heading', { name: /Revisión/ })).toBeVisible();
  await page.getByRole('button', { name: /Revisión 2/ }).click();
  await expect(page.getByRole('heading', { name: 'Revisión 2' })).toBeVisible();
  await page.getByRole('button', { name: 'Comparar con el documento actual' }).first().click();
  await expect(page.getByText('Diferencias con la revisión 2')).toBeVisible();
  await expect(page.getByText('[Versión 2 de la demo]', { exact: true })).toBeVisible();
});
