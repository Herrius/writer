import { expect, test } from '@playwright/test';

test('propone un cambio, lo revisa y lo acepta', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: /Control de cambios/ }).click();
  await page.getByRole('switch').check();
  const editor = page.locator('.ProseMirror');
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.keyboard.type('Propuesta escrita en el editor.');

  const estado = page.getByRole('status', { name: 'Estado de guardado' });
  await expect(estado).toHaveText('cambio propuesto', { timeout: 5000 });
  await expect(page.getByText('Pendientes (1)')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Aceptar cambio' })).toBeVisible();
  await page.getByRole('button', { name: 'Aceptar cambio' }).click();
  await expect(page.getByText(/Cambio aceptado en la revisión/)).toBeVisible();
  await expect(estado).toHaveText('guardado');
  await expect(page.locator('.ProseMirror')).toBeFocused();
});

test('rechaza una propuesta y conserva el texto anterior', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();

  await page.getByRole('button', { name: /Control de cambios/ }).click();
  await page.getByRole('switch').check();
  const editor = page.locator('.ProseMirror');
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.keyboard.type('Propuesta que se rechazará.');
  const estado = page.getByRole('status', { name: 'Estado de guardado' });
  await expect(estado).toHaveText('cambio propuesto', { timeout: 5000 });

  await page.getByRole('button', { name: 'Rechazar' }).click();
  await page.getByRole('button', { name: 'Confirmar descarte' }).click();
  await expect(page.getByText(/Cambio rechazado/)).toBeVisible();
  await expect(editor).toContainText('La casa olía a lluvia vieja');
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
