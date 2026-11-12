import { expect, test } from '@playwright/test';

test('crea un capítulo desde Crear y edita el título de la hoja', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: 'Crear documento' }).click();
  await expect(page.getByRole('tab', { name: 'Crear' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(page.getByRole('tab', { name: 'Plan' })).toHaveCount(0);
  await expect(page.getByRole('tab', { name: 'Cambios' })).toHaveCount(0);
  await page.getByLabel('Tipo de documento a crear').selectOption('capitulo');
  await page.getByRole('button', { name: 'Crear capítulo' }).click();
  await expect(page.getByRole('button', { name: 'Capítulo 3', exact: true })).toBeVisible();

  const titulo = page.getByRole('heading', { name: 'El umbral' }).first();
  await titulo.click();
  const entrada = page.getByRole('textbox', { name: 'Título de la hoja' });
  await expect(entrada).toBeVisible();
  await entrada.fill('El umbral renovado');
  await entrada.press('Enter');
  await expect(page.getByRole('heading', { name: 'El umbral renovado' })).toBeVisible();
});

test('busca dentro de la hoja y filtra la lista de hojas', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: 'Buscar en esta hoja' }).click();
  const buscarHoja = page.getByRole('textbox', { name: 'Buscar en esta hoja' });
  await buscarHoja.fill('casa');
  await expect(page.getByText(/\d+ de \d+/)).toBeVisible();
  await buscarHoja.press('Escape');
  await expect(buscarHoja).toHaveCount(0);

  await page.getByRole('textbox', { name: 'Filtrar hojas por título' }).fill('llave');
  await expect(page.locator('.dabble-doc').filter({ hasText: 'La llave' })).toBeVisible();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toHaveCount(0);
});
