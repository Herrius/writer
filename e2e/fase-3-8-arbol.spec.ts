import { expect, test } from '@playwright/test';

test('crea la jerarquía, renombra y elimina desde los menús del árbol', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await expect(page.getByText('Ideas')).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Acciones de Manuscritos' })).toHaveCount(0);

  await page.getByRole('button', { name: 'Acciones de El umbral' }).first().click();
  await expect(page.getByRole('menuitem', { name: 'Libro' })).toHaveCount(0);
  await expect(page.getByRole('menuitem', { name: 'Trama' })).toHaveCount(0);
  await expect(page.getByRole('menuitem', { name: 'Personaje' })).toHaveCount(0);
  await expect(page.getByRole('menuitem', { name: 'Nota de cuaderno' })).toHaveCount(0);
  await page.getByRole('menuitem', { name: 'Capítulo' }).click();
  await expect(page.getByRole('button', { name: 'Capítulo 3', exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Acciones de Capítulo 3' }).click();
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
  await expect(page.getByRole('button', { name: 'Parte nueva', exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Acciones de Capítulo 3' }).click();
  await page.getByRole('menuitem', { name: 'Eliminar' }).click();
  await page
    .getByRole('dialog', { name: 'Eliminar' })
    .getByRole('button', { name: 'Eliminar' })
    .click();
  await expect(page.getByRole('heading', { name: 'Papelera', level: 1 })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Capítulo 3', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Restaurar Escena sin título' })).toBeVisible();
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
  await expect(migas).toContainText('Personajes sin clasificar');
  await expect(migas).toContainText('Marta');

  await page.getByRole('button', { name: /^Papelera/ }).click();
  await expect(migas).toContainText('Papelera');
  await page.getByRole('button', { name: 'Volver al editor' }).click();
  await expect(migas).toContainText('Marta');

  await page.getByRole('button', { name: /^Archivo/ }).click();
  await expect(migas).toContainText('Archivo');
});

test('agrupa personajes y crea notas dentro de carpetas de cuaderno', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('button', { name: 'Acciones de Tramas' }).click();
  await expect(page.getByRole('menuitem', { name: 'Trama' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Personaje' })).toHaveCount(0);
  await expect(page.getByRole('menuitem', { name: 'Capítulo' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Acciones de Tramas' }).click();

  await page.getByRole('button', { name: 'Acciones de Personajes', exact: true }).click();
  await page.getByRole('menuitem', { name: 'Nueva sección de personajes' }).click();
  const dialogo = page.getByRole('dialog', { name: 'Nueva sección' });
  await dialogo.locator('input').fill('Antagonistas');
  await dialogo.getByRole('button', { name: 'Guardar' }).click();
  await expect(page.getByRole('heading', { name: 'Personajes: Antagonistas' })).toBeVisible();
  await expect(page.locator('.dabble-breadcrumb')).toContainText('Antagonistas');
  await page.getByRole('button', { name: 'Acciones de Antagonistas' }).click();
  await expect(page.getByRole('menuitem', { name: 'Personaje' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Renombrar' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Eliminar' })).toBeVisible();
  await page.getByRole('button', { name: 'Acciones de Antagonistas' }).click();

  await page.getByRole('button', { name: 'Crear un personaje' }).click();
  await expect(page.getByText('Personaje creado.')).toBeVisible();
  await page.getByRole('button', { name: 'Antagonistas', exact: true }).click();
  await expect(page.locator('.tarjeta-personaje').filter({ hasText: 'Personaje sin título' })).toBeVisible();

  await page.getByRole('button', { name: 'Cuaderno', exact: true }).first().click();
  await page.getByRole('button', { name: 'Acciones de Cuaderno' }).click();
  await expect(page.getByRole('menuitem', { name: 'Nota' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Carpeta' })).toBeVisible();
  await page.getByRole('button', { name: 'Acciones de Cuaderno' }).click();
  await page.getByRole('button', { name: 'Crear una carpeta' }).click();
  await page.locator('.vista-crear-carpeta input').fill('Construcción del mundo');
  await page.locator('.vista-crear-carpeta').getByRole('button', { name: 'Crear' }).click();
  await expect(page.getByRole('heading', { name: 'Cuaderno: Construcción del mundo' })).toBeVisible();
  await page.getByRole('button', { name: 'Acciones de Construcción del mundo' }).click();
  await expect(page.getByRole('menuitem', { name: 'Nota' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Renombrar' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'Eliminar' })).toBeVisible();
  await page.getByRole('button', { name: 'Acciones de Construcción del mundo' }).click();

  await page.getByRole('button', { name: 'Crear una nota' }).click();
  await expect(page.getByText('Nota creada.')).toBeVisible();
  await page.getByRole('button', { name: 'Cuaderno', exact: true }).first().click();
  await page.getByRole('button', { name: 'Construcción del mundo', exact: true }).click();
  await expect(page.locator('.tarjeta-nota').filter({ hasText: 'Nota sin título' })).toBeVisible();
});
