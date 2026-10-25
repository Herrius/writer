import { expect, test } from '@playwright/test';

test('crea la jerarquía, renombra y elimina desde los menús del árbol', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await expect(page.getByText('Ideas')).toHaveCount(0);

  await page.getByRole('button', { name: 'Acciones de Manuscritos' }).click();
  await page.getByRole('menuitem', { name: 'Libro' }).click();
  await expect(page.getByRole('button', { name: 'Libro 2', exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Acciones de Libro 2' }).click();
  await page.getByRole('menuitem', { name: 'Capítulo' }).click();
  await expect(page.getByRole('button', { name: 'Capítulo 1', exact: true })).toHaveCount(2);

  await page.getByRole('button', { name: 'Acciones de Capítulo 1' }).last().click();
  await page.getByRole('menuitem', { name: 'Sección' }).click();
  await expect(page.getByText('Sección 1')).toBeVisible();

  await page.getByRole('button', { name: 'Acciones de Sección 1' }).click();
  await page.getByRole('menuitem', { name: 'Escena' }).click();
  await expect(page.locator('.dabble-doc').filter({ hasText: 'Escena sin título' })).toBeVisible();

  await page.getByRole('button', { name: 'Acciones de Sección 1' }).click();
  await page.getByRole('menuitem', { name: 'Renombrar' }).click();
  const dialogo = page.getByRole('dialog', { name: 'Renombrar' });
  await dialogo.locator('input').fill('Parte nueva');
  await dialogo.getByRole('button', { name: 'Guardar' }).click();
  await expect(page.getByRole('button', { name: 'Parte Nueva', exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Acciones de Libro 2' }).click();
  await page.getByRole('menuitem', { name: 'Eliminar' }).click();
  await page
    .getByRole('dialog', { name: 'Eliminar' })
    .getByRole('button', { name: 'Eliminar' })
    .click();
  await expect(page.getByRole('heading', { name: 'Papelera', level: 1 })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Libro 2', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Restaurar Escena sin título' })).toBeVisible();

  await page.getByRole('button', { name: 'Restaurar Escena sin título' }).click();
  await expect(page.getByRole('button', { name: 'Libro 2', exact: true })).toBeVisible();
});

test('sincroniza la ruta superior con el árbol y las vistas', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  const migas = page.locator('.dabble-breadcrumb');
  await expect(migas).toContainText('El umbral');
  await expect(migas).toContainText('Capítulo 1');
  await expect(migas).toContainText('El umbral');

  await page.locator('.dabble-doc').filter({ hasText: 'Marta' }).click();
  await expect(migas).toContainText('Personajes');
  await expect(migas).toContainText('Personajes sin clasificar');
  await expect(migas).toContainText('Marta');

  await page.getByRole('button', { name: /^Papelera/ }).click();
  await expect(migas).toContainText('Papelera');
  await page.getByRole('button', { name: 'Volver al editor' }).click();
  await expect(migas).toContainText('Marta');

  await page.getByRole('button', { name: /^Archivo/ }).click();
  await expect(migas).toContainText('Archivo');
});
