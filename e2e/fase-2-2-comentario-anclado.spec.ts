import { expect, test } from '@playwright/test';

test('crea un comentario anclado con ULID v2', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  const editor = page.locator('.ProseMirror');
  await expect(editor).toContainText('La casa olía');
  await editor.click();
  await page.keyboard.press('Control+A');

  await page.getByRole('button', { name: 'Comentario', exact: true }).click();
  await page.getByLabel('Texto del comentario').fill('revisar el ritmo');
  await page.getByRole('button', { name: 'Aplicar' }).click();

  await page.getByRole('button', { name: 'Ver Markdown' }).click();
  await expect(page.locator('pre.fuente')).toContainText(
    /\[\[comentario:[0-9A-HJKMNP-TV-Z]{26}\|revisar el ritmo\|/,
  );
});

test('muestra la nota flotante al seleccionar texto y la aplica', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  const editor = page.locator('.ProseMirror');
  await expect(editor).toContainText('La casa olía');
  await editor.click();
  await page.keyboard.press('Control+A');

  await expect(page.getByRole('button', { name: 'Comentar', exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Comentar', exact: true })).toBeInViewport();
  await page.getByRole('button', { name: 'Comentar', exact: true }).click();
  await page.getByLabel('Texto de la nota').fill('nota desde la selección');
  await page.getByRole('button', { name: 'Añadir nota' }).click();

  await page.locator('.marca-comentario').first().click();
  const nota = page.getByRole('dialog', { name: 'Comentario del texto' });
  await expect(nota).toContainText('nota desde la selección');
  await nota.getByRole('button', { name: 'Ver en Comentarios' }).click();
  await page.getByLabel('Filtrar comentarios').selectOption('abiertos');
  await expect(
    page.locator('.dabble-comentarios-panel').getByText('nota desde la selección'),
  ).toBeVisible();

  await page.getByRole('button', { name: 'Ver Markdown' }).click();
  await expect(page.locator('pre.fuente')).toContainText(
    /\[\[comentario:[0-9A-HJKMNP-TV-Z]{26}\|nota desde la selección\|/,
  );
});
