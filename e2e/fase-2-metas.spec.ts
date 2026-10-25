import { expect, test } from '@playwright/test';

async function abrirDemo(page: import('@playwright/test').Page) {
  await page.goto('/');
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('tab', { name: 'Metas' }).click();
}

test('configura la meta diaria global y muestra el progreso', async ({ page }) => {
  await page.addInitScript(() => {
    const ahora = new Date();
    const hoy = `${ahora.getFullYear()}-${String(ahora.getMonth() + 1).padStart(2, '0')}-${String(ahora.getDate()).padStart(2, '0')}`;
    localStorage.setItem(
      'writer.metas-globales.v1',
      JSON.stringify({
        version: 1,
        diaria: {
          objetivo: 10,
          desde: hoy,
          diasLibres: [],
          contarSoloManuscrito: true,
          contarPegadas: true,
          contarEliminaciones: true,
          recordatorio: false,
        },
        actividad: { [hoy]: 3 },
      }),
    );
  });
  await abrirDemo(page);
  await expect(page.getByText('METAS DIARIAS')).toBeVisible();
  await expect(page.locator('.meta-progreso')).toContainText('3');
  await expect(page.locator('.meta-progreso')).toContainText('30 %');
  await expect(page.locator('.meta-grafico')).toHaveCount(0);
  const panel = await page.locator('.dabble-derecha').boundingBox();
  const timer = await page.locator('.dabble-timer').boundingBox();
  expect(panel).not.toBeNull();
  expect(timer).not.toBeNull();
  expect(timer!.y + timer!.height).toBeGreaterThan(panel!.y + panel!.height - 30);

  await page.getByRole('button', { name: 'Configurar meta diaria' }).click();
  const modal = page.getByRole('dialog', { name: 'Meta diaria' });
  await modal.getByLabel('Palabras por día').fill('20');
  await modal.getByRole('button', { name: 'Guardar meta' }).click();
  await expect(modal).toHaveCount(0);
  await expect(page.locator('.meta-progreso')).toContainText('/ 20 palabras');

  const guardado = await page.evaluate(() =>
    JSON.parse(localStorage.getItem('writer.metas-globales.v1') ?? 'null'),
  );
  expect(guardado.diaria.objetivo).toBe(20);
  expect(guardado.actividad[Object.keys(guardado.actividad)[0]]).toBe(3);

  await page.reload();
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('tab', { name: 'Metas' }).click();
  await expect(page.locator('.meta-progreso')).toContainText('3');
});
test('marca un día libre y lo conserva al recargar', async ({ page }) => {
  await abrirDemo(page);
  await page.getByRole('button', { name: 'Configurar meta diaria' }).click();
  const modal = page.getByRole('dialog', { name: 'Meta diaria' });
  const hoy = new Date().getDay();
  const nombre = ['domingo', 'lunes', 'martes', 'miércoles', 'jueves', 'viernes', 'sábado'][
    hoy
  ];
  await modal.getByRole('button', { name: nombre }).click();
  await modal.getByLabel('Palabras por día').fill('25');
  await modal.getByRole('button', { name: 'Guardar meta' }).click();
  await expect(page.getByText('Hoy es un día libre')).toBeVisible();

  await page.reload();
  await page.getByRole('button', { name: 'Ver demo' }).click();
  await page.getByRole('tab', { name: 'Metas' }).click();
  await expect(page.getByText('Hoy es un día libre')).toBeVisible();
});

test('mantiene el temporizador y registra la sesión', async ({ page }) => {
  await abrirDemo(page);
  await page.getByRole('button', { name: 'Iniciar temporizador' }).click();
  await expect(page.getByRole('button', { name: 'Pausar temporizador' })).toBeVisible();
  await page.getByRole('button', { name: 'Pausar temporizador' }).click();
  await expect(page.getByRole('button', { name: 'Reanudar temporizador' })).toBeVisible();
  await page.getByRole('button', { name: 'Reanudar temporizador' }).click();
  await page.getByRole('button', { name: 'Finalizar' }).click();
  await expect(page.getByText('Sesión de 5 min registrada.')).toBeVisible();
});

test('muestra el estado vacío y el botón solo al pasar el ratón', async ({ page }) => {
  await abrirDemo(page);
  await page.getByRole('button', { name: 'Configurar meta diaria' }).click();
  const modal = page.getByRole('dialog', { name: 'Meta diaria' });
  await modal.getByRole('button', { name: 'Eliminar meta' }).click();
  await modal.getByRole('button', { name: 'Confirmar eliminación' }).click();
  await expect(modal).toHaveCount(0);

  await expect(page.getByText('METAS DIARIAS')).toHaveCount(0);
  await expect(page.getByRole('heading', { name: 'Establece tus metas de escritura' })).toBeVisible();
  await expect(page.getByText('Configura tu primera meta diaria')).toHaveCount(0);
  await expect(page.getByText('Aún no tienes una meta diaria.')).toHaveCount(0);

  const agregar = page.getByRole('button', { name: 'Agregar una meta' });
  expect(
    await agregar.evaluate((elemento) => window.getComputedStyle(elemento).opacity),
  ).toBe('0');
  await page.locator('.meta-vacia').hover();
  await expect
    .poll(() => agregar.evaluate((elemento) => window.getComputedStyle(elemento).opacity))
    .toBe('1');
  await agregar.click();
  await expect(page.getByRole('dialog', { name: 'Meta diaria' })).toBeVisible();
});
