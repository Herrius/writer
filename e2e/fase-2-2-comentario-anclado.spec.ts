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
