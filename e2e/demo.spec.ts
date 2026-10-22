import { expect, test } from '@playwright/test';

async function abrirDemo(page: import('@playwright/test').Page) {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
}

test('abre la demo, edita y guarda con el autosave', async ({ page }) => {
  await abrirDemo(page);
  const editor = page.locator('.ProseMirror');
  await expect(editor).toContainText('La casa olía');
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.keyboard.type('Texto E2E de prueba.');
  await expect(page.getByRole('status', { name: 'Estado de guardado' })).toHaveText(
    'guardado',
    { timeout: 5000 },
  );

  // Cambiar de documento y volver: el texto debe venir de la persistencia,
  // no solo del estado visual del editor.
  await page.locator('.documento').filter({ hasText: 'La llave' }).click();
  await expect(page.getByRole('heading', { name: 'La llave' })).toBeVisible();
  await page.locator('.documento').filter({ hasText: 'El umbral' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await expect(page.locator('.ProseMirror')).toHaveText('Texto E2E de prueba.');
});

test('busca y salta a otro documento', async ({ page }) => {
  await abrirDemo(page);
  await page.getByRole('textbox', { name: 'Buscar en el proyecto' }).fill('llave');
  const resultado = page.locator('.resultado').filter({ hasText: 'La llave' });
  await expect(resultado).toBeVisible();
  await expect(resultado).toContainText('llave');
  await resultado.click();
  await expect(page.getByRole('heading', { name: 'La llave' })).toBeVisible();
  await expect(page.locator('.ProseMirror')).toContainText('cajón de la cocina');
});

test('aplica formato y muestra el recuento de palabras', async ({ page }) => {
  await abrirDemo(page);
  const editor = page.locator('.ProseMirror');
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.getByRole('button', { name: 'Negrita' }).click();
  await expect(page.getByRole('button', { name: 'Negrita' })).toHaveClass(/activo/);
  await expect(page.locator('.editor-contador')).toHaveText(/\d+ palabras/);
  await page.getByRole('button', { name: 'Markdown' }).click();
  await expect(page.locator('pre.fuente')).toContainText('**');
});

test('muestra el historial y el detalle de una revisión', async ({ page }) => {
  await abrirDemo(page);
  await page.getByRole('button', { name: 'Historial' }).click();
  await expect(page.getByText('Revisiones del proyecto')).toBeVisible();
  await page.getByRole('button', { name: /r2/ }).click();
  await expect(page.getByText('Revisión 2')).toBeVisible();
  await page.locator('.detalle-archivos summary').first().click();
  await expect(page.getByText(/La casa olía/)).toBeVisible();
});

test('aplica y sustituye marcas sin anidarlas', async ({ page }) => {
  await abrirDemo(page);
  const editor = page.locator('.ProseMirror');
  await editor.click();
  await page.keyboard.press('Control+A');
  await page.getByRole('combobox', { name: 'Aplicar personaje' }).selectOption({
    label: 'Marta',
  });
  await page.getByRole('button', { name: 'Markdown' }).click();
  await expect(page.locator('pre.fuente')).toContainText('[[personaje:');

  await page.getByRole('button', { name: 'Editor' }).click();
  const editor2 = page.locator('.ProseMirror');
  await editor2.click();
  await page.keyboard.press('Control+A');
  await page.getByRole('combobox', { name: 'Aplicar trama' }).selectOption({
    label: 'El mapa incompleto',
  });
  await page.getByRole('button', { name: 'Markdown' }).click();
  const fuente = page.locator('pre.fuente');
  await expect(fuente).toContainText('[[trama:');
  await expect(fuente).not.toContainText('[[personaje:');
});

test('muestra y descarta una copia de cuarentena', async ({ page }) => {
  await abrirDemo(page);
  await expect(page.getByText('Cuarentena (1)')).toBeVisible();
  await page.getByRole('button', { name: 'Descartar' }).click();
  await page.getByRole('button', { name: 'Confirmar descarte' }).click();
  await expect(page.getByText(/Copia de cuarentena descartada/)).toBeVisible();
  await expect(page.getByText('Cuarentena (1)')).toHaveCount(0);
});

test('permite saltar al contenido y navegar el árbol con flechas', async ({ page }) => {
  await abrirDemo(page);
  await page.keyboard.press('Tab');
  const salto = page.getByRole('link', { name: 'Saltar al contenido' });
  await expect(salto).toBeFocused();
  await expect(salto).toBeVisible();
  await page.keyboard.press('Enter');
  await expect(page.locator('#panel-principal')).toBeFocused();

  const activo = page.locator('.documento.activo');
  await expect(activo).toHaveAttribute('tabindex', '0');
  await expect(page.locator('.documento:not(.activo)').first()).toHaveAttribute(
    'tabindex',
    '-1',
  );
  await activo.focus();
  await page.keyboard.press('ArrowDown');
  await expect(page.locator('.documento').nth(1)).toBeFocused();
});

test('muestra un anillo de foco en el editor', async ({ page }) => {
  await abrirDemo(page);
  const editor = page.locator('.ProseMirror');
  await editor.focus();
  await page.keyboard.press('Shift');
  const estilos = await editor.evaluate((elemento) => {
    const calculado = getComputedStyle(elemento);
    return { estilo: calculado.outlineStyle, ancho: calculado.outlineWidth };
  });
  expect(estilos.estilo).not.toBe('none');
  expect(parseFloat(estilos.ancho)).toBeGreaterThan(0);
});

test('exporta DOCX desde la demo', async ({ page }) => {
  page.on('dialog', async (dialogo) => {
    await dialogo.accept('/tmp/writer-e2e.docx');
  });
  await abrirDemo(page);
  await page.getByRole('button', { name: 'Exportar DOCX' }).click();
  await expect(
    page.getByText(/Demo: exportación simulada a \/tmp\/writer-e2e.docx/),
  ).toBeVisible();
});
