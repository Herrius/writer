import { expect, test } from '@playwright/test';

const recientes = [
  {
    ruta: '/tmp/el-umbral',
    titulo: 'El umbral',
    autor: 'Marta Ruiz',
    palabras: 12345,
    color: '#4d6b8a',
    modificado: '2026-10-22T10:00:00Z',
  },
  {
    ruta: '/tmp/la-casa',
    titulo: 'La casa de las mareas',
    autor: 'Lucía Ferrer',
    palabras: 80210,
    color: '#5a8fa3',
    modificado: '2026-10-23T10:00:00Z',
  },
];

async function abrirPanel(page: import('@playwright/test').Page) {
  await page.addInitScript((proyectos) => {
    localStorage.setItem('writer.recientes.v1', JSON.stringify(proyectos));
  }, recientes);
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Proyectos', level: 1 })).toBeVisible();
}

test('filtra y ordena los proyectos', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('button', { name: 'Filtros' }).click();
  const filtros = page.getByRole('dialog', { name: 'Filtrar proyectos' });
  await filtros.getByLabel('Autor').selectOption('Marta Ruiz');
  await filtros.getByRole('button', { name: 'Aplicar' }).click();

  await expect(page.getByRole('button', { name: 'Filtros (1)' })).toBeVisible();
  await expect(page.locator('.tarjeta-proyecto')).toHaveCount(1);
  await expect(page.locator('.tarjeta-proyecto .portada-titulo')).toHaveText('El umbral');

  await page.getByRole('button', { name: 'Filtros (1)' }).click();
  await filtros.getByRole('button', { name: 'Limpiar filtros' }).click();
  await filtros.getByRole('button', { name: 'Aplicar' }).click();
  await expect(page.locator('.tarjeta-proyecto')).toHaveCount(2);

  await page.getByRole('button', { name: 'Filtros' }).click();
  await filtros.getByLabel('Ordenar por').selectOption('titulo');
  await filtros.getByRole('button', { name: 'Aplicar' }).click();
  await expect(page.locator('.tarjeta-proyecto .tarjeta-datos strong').first()).toHaveText(
    'El umbral',
  );
});

test('crea y asigna una colección local', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('button', { name: '+ Agregar una colección' }).click();
  const crear = page.getByRole('dialog', { name: 'Nueva colección' });
  await crear.getByLabel('Nombre').fill('Prioridad');
  await crear.getByRole('button', { name: 'Crear colección' }).click();
  await expect(page.getByRole('button', { name: /Prioridad/ })).toBeVisible();

  await page.getByRole('button', { name: 'Opciones de El umbral' }).click();
  const acciones = page.getByRole('dialog', { name: 'El umbral' });
  await acciones.getByRole('checkbox', { name: 'Prioridad' }).check();
  await acciones.getByRole('button', { name: 'Guardar colecciones' }).click();

  await page.getByRole('button', { name: /Prioridad/ }).click();
  await expect(page.locator('.tarjeta-proyecto')).toHaveCount(1);
  await expect(page.locator('.tarjeta-proyecto .portada-titulo')).toHaveText('El umbral');

  await page.getByRole('button', { name: 'Gestionar' }).click();
  const gestion = page.getByRole('dialog', { name: 'Gestionar colecciones' });
  await gestion.getByLabel(/Nombre de la colección/).fill('Prioridad alta');
  await gestion.getByRole('button', { name: 'Guardar cambios' }).click();
  await expect(page.getByRole('button', { name: /Prioridad alta/ })).toBeVisible();
});

test('persiste las colecciones al recargar', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('button', { name: '+ Agregar una colección' }).click();
  const crear = page.getByRole('dialog', { name: 'Nueva colección' });
  await crear.getByLabel('Nombre').fill('Persistente');
  await crear.getByRole('button', { name: 'Crear colección' }).click();
  await expect(page.getByRole('button', { name: /Persistente/ })).toBeVisible();

  await page.reload();
  await expect(page.getByRole('heading', { name: 'Proyectos', level: 1 })).toBeVisible();
  await expect(page.getByRole('button', { name: /Persistente/ })).toBeVisible();
});

test('gestiona plantillas locales y preselecciona una al crear', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('tab', { name: 'Plantillas' }).click();
  await expect(page.getByRole('heading', { name: 'Plantillas' })).toBeVisible();
  await expect(page.getByText('Novela en tres actos')).toBeVisible();

  await page.getByRole('button', { name: '+ Nueva plantilla' }).click();
  const modal = page.getByRole('dialog', { name: 'Nueva plantilla' });
  await modal.getByLabel('Nombre').fill('Cuatro actos');
  await modal.getByLabel('Descripción').fill('Una estructura breve de prueba.');
  await modal.getByRole('button', { name: 'Guardar plantilla' }).click();
  await expect(page.getByRole('heading', { name: 'Cuatro actos' })).toBeVisible();

  const tarjeta = page.locator('.plantilla-tarjeta').filter({ hasText: 'Cuatro actos' });
  await tarjeta.getByRole('button', { name: 'Usar plantilla' }).click();
  const crear = page.getByRole('dialog', { name: 'Crear nuevo proyecto' });
  await expect(crear.getByLabel('Plantilla del proyecto')).toHaveValue('cuatro-actos');
  await page.keyboard.press('Escape');

  await tarjeta.getByRole('button', { name: /Eliminar plantilla/ }).click();
  await tarjeta.getByRole('button', { name: /Confirmar eliminación/ }).click();
  await expect(page.getByRole('heading', { name: 'Cuatro actos' })).toHaveCount(0);
});

test('mueve, restaura y purga un proyecto de la papelera', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('button', { name: 'Opciones de El umbral' }).click();
  await page.getByRole('button', { name: 'Mover proyecto a la papelera' }).click();
  const confirmar = page.getByRole('dialog', { name: 'Mover a la papelera' });
  await confirmar.getByRole('button', { name: 'Mover a la papelera' }).click();
  await expect(page.locator('.tarjeta-proyecto')).toHaveCount(1);

  await page.getByRole('tab', { name: 'Papelera' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('button', { name: /^Restaurar/ }).click();
  await expect(page.getByText(/restaurado en Mis Proyectos/)).toBeVisible();
  await page.getByRole('tab', { name: 'Activo' }).click();
  await expect(page.locator('.tarjeta-proyecto')).toHaveCount(2);

  await page.getByRole('button', { name: 'Opciones de El umbral' }).click();
  await page.getByRole('button', { name: 'Mover proyecto a la papelera' }).click();
  await page
    .getByRole('dialog', { name: 'Mover a la papelera' })
    .getByRole('button', { name: 'Mover a la papelera' })
    .click();
  await page.getByRole('tab', { name: 'Papelera' }).click();
  await page.locator('.papelera-acciones').getByRole('button', { name: /Eliminar definitivamente/ }).click();
  const purgar = page.getByRole('dialog', { name: 'Eliminar definitivamente' });
  await purgar.getByRole('button', { name: 'Eliminar definitivamente' }).click();
  await expect(page.getByText('La papelera está vacía')).toBeVisible();
});

test('guarda la colección antes de abrir el proyecto', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('button', { name: '+ Agregar una colección' }).click();
  const crear = page.getByRole('dialog', { name: 'Nueva colección' });
  await crear.getByLabel('Nombre').fill('Al abrir');
  await crear.getByRole('button', { name: 'Crear colección' }).click();

  await page.getByRole('button', { name: 'Opciones de El umbral' }).click();
  const acciones = page.getByRole('dialog', { name: 'El umbral' });
  await acciones.getByRole('checkbox', { name: 'Al abrir' }).check();
  await acciones.getByRole('button', { name: 'Abrir proyecto' }).click();
  await expect(page.getByRole('heading', { name: 'El umbral' })).toBeVisible();
  await page.getByRole('button', { name: 'Volver a Proyectos' }).click();
  await page.getByRole('button', { name: /Al abrir/ }).click();
  await expect(page.locator('.tarjeta-proyecto')).toHaveCount(1);
});

test('los modales nuevos restauran el foco y atrapan Tab', async ({ page }) => {
  await abrirPanel(page);
  const filtros = page.getByRole('button', { name: 'Filtros' });
  await filtros.click();
  const dialogo = page.getByRole('dialog', { name: 'Filtrar proyectos' });
  for (let i = 0; i < 10; i += 1) {
    await page.keyboard.press('Tab');
  }
  expect(
    await page.evaluate(() => Boolean(document.activeElement?.closest('[role="dialog"]'))),
  ).toBe(true);
  await page.keyboard.press('Escape');
  await expect(dialogo).toHaveCount(0);
  await expect(filtros).toBeFocused();

  const menu = page.getByRole('button', { name: 'Opciones de El umbral' });
  await menu.click();
  await expect(page.getByRole('dialog', { name: 'El umbral' })).toBeVisible();
  await page.keyboard.press('Shift+Tab');
  expect(
    await page.evaluate(() => Boolean(document.activeElement?.closest('[role="dialog"]'))),
  ).toBe(true);
  await page.keyboard.press('Escape');
  await expect(menu).toBeFocused();
});

test('la papelera no recorta acciones a 960 y 720', async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem(
      'writer.papelera.v1',
      JSON.stringify([
        {
          id: '01K',
          titulo:
            'Un proyecto con un título extremadamente largo para probar el desbordamiento',
          autor: 'Autora',
          color: '#6b5747',
          palabras: 100,
          rutaOriginal:
            '/home/enrique/Documentos/software de escritura/proyectos/un-proyecto-con-un-nombre-de-ruta-muy-largo',
          rutaPapelera:
            '/home/enrique/Documentos/software de escritura/proyectos/.writer-papelera/01K',
          eliminado: '2026-10-24T10:00:00Z',
        },
      ]),
    );
  });
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Proyectos', level: 1 })).toBeVisible();
  await page.getByRole('tab', { name: 'Papelera' }).click();
  for (const [width, height, font] of [
    [960, 900, 16],
    [960, 900, 32],
    [720, 900, 16],
  ]) {
    await page.setViewportSize({ width, height });
    await page.evaluate((tamano) => {
      document.documentElement.style.fontSize = `${tamano}px`;
    }, font);
    const panel = await page.locator('.panel-papelera').boundingBox();
    const eliminar = await page
      .getByRole('button', { name: /Eliminar definitivamente/ })
      .boundingBox();
    expect(panel).not.toBeNull();
    expect(eliminar).not.toBeNull();
    expect(eliminar!.x + eliminar!.width).toBeLessThanOrEqual(panel!.x + panel!.width + 1);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
    ).toBe(true);
  }
});
