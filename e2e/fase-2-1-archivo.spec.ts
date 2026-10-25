import { expect, test } from '@playwright/test';

test('archiva, restaura, vacía la papelera y crea desde la barra lateral', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('tab', { name: 'Notas' }).click();
  await page.getByRole('button', { name: 'Archivar' }).click();
  await expect(page.getByText('Documento archivado.')).toBeVisible();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toHaveCount(0);

  await page.getByRole('button', { name: /^Archivo/ }).click();
  await expect(page.getByRole('heading', { name: 'Archivo', level: 1 })).toBeVisible();
  await expect(page.getByText('1 elemento')).toBeVisible();
  await expect(page.locator('.vista-proyecto-fila').filter({ hasText: 'El umbral' })).toBeVisible();
  await page.getByRole('button', { name: 'Restaurar El umbral' }).click();
  await expect(page.getByText('El archivo está vacío')).toBeVisible();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: /^Papelera/ }).click();
  await expect(page.getByRole('heading', { name: 'Papelera', level: 1 })).toBeVisible();
  await expect(page.getByText('La papelera está vacía')).toBeVisible();

  await page.getByRole('button', { name: 'Crear documento' }).click();
  await expect(page.getByRole('tab', { name: 'Plan' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.locator('#nueva-escena-titulo')).toBeFocused();

  await page.getByRole('button', { name: /Plantillas/ }).click();
  await expect(page.getByRole('heading', { name: 'Plantillas', level: 1 })).toBeVisible();
  await page.getByRole('button', { name: 'Abrir gestor de plantillas' }).click();
  await expect(page.getByRole('tab', { name: 'Plantillas' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
});
