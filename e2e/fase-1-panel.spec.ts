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

async function abrirPanel(
  page: import('@playwright/test').Page,
  datos: typeof recientes = recientes,
) {
  await page.addInitScript((proyectos) => {
    localStorage.setItem('writer.recientes.v1', JSON.stringify(proyectos));
  }, datos);
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Proyectos', level: 1 })).toBeVisible();
}

async function mostrarAcciones(page: import('@playwright/test').Page) {
  await page.locator('.mis-proyectos-cabecera').hover();
}

test('la composición reproduce la referencia de Mis Proyectos', async ({ page }) => {
  await page.setViewportSize({ width: 807, height: 700 });
  await abrirPanel(page);

  const medidas = await page.evaluate(() => {
    const panel = document.querySelector('.mis-proyectos')!.getBoundingClientRect();
    const cabecera = document
      .querySelector('.mis-proyectos-cabecera')!
      .getBoundingClientRect();
    const caja = (selector: string) => {
      const rect = document.querySelector(selector)!.getBoundingClientRect();
      return {
        x: rect.x - panel.x,
        y: rect.y - panel.y,
        width: rect.width,
        height: rect.height,
      };
    };
    const portadas = [...document.querySelectorAll('.portada')].map((elemento) => {
      const rect = elemento.getBoundingClientRect();
      return { width: rect.width, height: rect.height, y: rect.y - panel.y };
    });
    return {
      panel: { width: panel.width, height: panel.height },
      cabecera: { height: cabecera.height },
      crear: caja('.crear-tarjeta'),
      tarjetas: [...document.querySelectorAll('.tarjeta-proyecto')].map((elemento) => {
        const rect = elemento.getBoundingClientRect();
        return { x: rect.x - panel.x, width: rect.width, height: rect.height };
      }),
      portadas,
      plantilla: caja('.crear-plantilla'),
    };
  });

  expect(medidas.panel.width).toBeGreaterThan(760);
  expect(medidas.panel.width).toBeLessThan(776);
  expect(medidas.panel.height).toBeGreaterThan(495);
  expect(medidas.panel.height).toBeLessThan(512);
  expect(medidas.cabecera.height).toBeGreaterThan(50);
  expect(medidas.cabecera.height).toBeLessThan(61);

  expect(medidas.crear.x).toBeGreaterThan(42);
  expect(medidas.crear.x).toBeLessThan(52);
  expect(medidas.crear.width).toBeGreaterThan(157);
  expect(medidas.crear.width).toBeLessThan(167);
  expect(medidas.crear.height).toBeGreaterThan(259);
  expect(medidas.crear.height).toBeLessThan(272);

  expect(medidas.tarjetas[0].x).toBeGreaterThan(290);
  expect(medidas.tarjetas[0].x).toBeLessThan(301);
  expect(medidas.tarjetas[1].x).toBeGreaterThan(536);
  expect(medidas.tarjetas[1].x).toBeLessThan(548);
  expect(medidas.portadas[0].width).toBeGreaterThan(157);
  expect(medidas.portadas[0].width).toBeLessThan(167);
  expect(medidas.portadas[0].height).toBeGreaterThan(259);
  expect(medidas.portadas[0].height).toBeLessThan(272);
  expect(medidas.portadas[0].y).toBeGreaterThan(90);
  expect(medidas.portadas[0].y).toBeLessThan(108);
  expect(medidas.plantilla.y).toBeGreaterThan(medidas.crear.y + medidas.crear.height);
});

test('la tarjeta de creación y la portada comparten medida', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page);
  const crear = await page.locator('.crear-tarjeta').boundingBox();
  const portada = await page.locator('.portada').first().boundingBox();
  const datos = await page.locator('.tarjeta-datos').first().boundingBox();
  expect(crear).not.toBeNull();
  expect(portada).not.toBeNull();
  expect(datos).not.toBeNull();
  expect(Math.abs(crear!.height - portada!.height)).toBeLessThanOrEqual(2);
  expect(portada!.width / portada!.height).toBeCloseTo(0.61, 2);
  expect(datos!.height).toBeGreaterThanOrEqual(30);
  expect(datos!.height).toBeLessThanOrEqual(44);
  const borde = await page.locator('.crear-tarjeta').evaluate((elemento) => {
    const estilos = getComputedStyle(elemento);
    return {
      estilo: estilos.borderTopStyle,
      ancho: estilos.borderTopWidth,
      color: estilos.borderTopColor,
    };
  });
  expect(borde.estilo).toBe('dashed');
  expect(borde.ancho).toBe('2px');
  expect(borde.color).toBe('rgb(201, 201, 198)');
});

test('las portadas muestran arte, título y autor; los datos muestran fecha', async ({ page }) => {
  await abrirPanel(page);
  const tarjeta = page.locator('.tarjeta-proyecto').first();
  await expect(tarjeta.getByRole('img')).toHaveCount(0);
  await expect(tarjeta.locator('.portada-arte')).toHaveCount(1);
  await expect(tarjeta.locator('.portada-titulo')).toHaveText('El umbral');
  await expect(tarjeta.locator('.portada-autor')).toHaveText('Marta Ruiz');
  await expect(tarjeta.locator('.tarjeta-datos strong')).toHaveText('El umbral');
  await expect(tarjeta.getByText('Modificado el Oct 22, 2026')).toBeVisible();
  await expect(tarjeta.locator('.tarjeta-menu')).toHaveText('⋯');
  await expect(tarjeta.locator('.tarjeta-abrir')).toHaveAttribute('aria-label', 'Abrir El umbral de Marta Ruiz');
  await expect(tarjeta.getByText('12.345 palabras')).toHaveCount(0);
  const fecha = tarjeta.locator('.tarjeta-meta > span').first();
  expect(
    await fecha.evaluate((elemento) => elemento.scrollWidth <= elemento.clientWidth + 1),
  ).toBe(true);
});

test('la cabecera conserva las vistas sin ensuciar el estado de reposo', async ({ page }) => {
  await abrirPanel(page);
  const lista = page.getByRole('button', { name: 'Vista de lista' });
  expect(
    await page.locator('.mis-proyectos-acciones').evaluate(
      (elemento) => getComputedStyle(elemento).opacity,
    ),
  ).toBe('0');
  await mostrarAcciones(page);
  await expect(lista).toBeVisible();
  await lista.click();
  await expect(page.locator('.proyectos')).toHaveClass(/proyectos-lista/);
  await expect(page.getByRole('button', { name: 'Crear desde plantilla' })).toBeVisible();
});

test('la tarjeta de creación abre el asistente y devuelve el foco', async ({ page }) => {
  await abrirPanel(page);
  const crear = page.getByRole('button', { name: /Crear nuevo proyecto/ });
  await crear.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog', { name: 'Crear nuevo proyecto' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toBeHidden();
  await expect(crear).toBeFocused();
});

test('crear desde plantilla abre el asistente', async ({ page }) => {
  await abrirPanel(page);
  await page.getByRole('button', { name: 'Crear desde plantilla' }).click();
  await expect(page.getByRole('dialog', { name: 'Crear nuevo proyecto' })).toBeVisible();
});

test('el estado vacío conserva la tarjeta de creación y la plantilla', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page, []);
  const crear = await page.locator('.crear-tarjeta').boundingBox();
  const plantilla = await page.getByRole('button', { name: 'Crear desde plantilla' }).boundingBox();
  expect(crear).not.toBeNull();
  expect(plantilla).not.toBeNull();
  expect(crear!.height).toBeGreaterThan(250);
  expect(plantilla!.y).toBeGreaterThanOrEqual(crear!.y + crear!.height - 1);
  await expect(page.getByText('No proyectos')).toBeVisible();
});

test('a 960 y 200 % no hay overflow y las portadas mantienen su proporción', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 640 });
  await abrirPanel(page);
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '32px';
  });

  const portada = await page.locator('.portada').first().boundingBox();
  const crear = await page.locator('.crear-tarjeta').boundingBox();
  expect(portada).not.toBeNull();
  expect(crear).not.toBeNull();
  expect(portada!.width / portada!.height).toBeCloseTo(0.61, 1);
  expect(Math.abs(crear!.height - portada!.height)).toBeLessThanOrEqual(2);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
});

test('la vista de lista es compacta', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page);
  await mostrarAcciones(page);
  await page.getByRole('button', { name: 'Vista de lista' }).click();

  const crear = await page.locator('.crear-tarjeta').boundingBox();
  const tarjeta = await page.locator('.tarjeta-proyecto').first().boundingBox();
  const portada = await page.locator('.portada').first().boundingBox();
  expect(crear).not.toBeNull();
  expect(tarjeta).not.toBeNull();
  expect(portada).not.toBeNull();
  expect(crear!.height).toBeLessThan(120);
  expect(tarjeta!.height).toBeLessThan(120);
  expect(portada!.width).toBeLessThan(60);
  expect(
    await page.locator('.proyectos-lista .tarjeta-meta').first().evaluate(
      (elemento) => getComputedStyle(elemento).color,
    ),
  ).toBe('rgb(116, 112, 106)');
});

test('el buscador muestra un foco visible', async ({ page }) => {
  await abrirPanel(page);
  const buscador = page.getByRole('textbox', { name: 'Buscar proyectos' });
  await buscador.focus();

  const estilos = await page.locator('.buscador-inicio').evaluate((elemento) => {
    const calculado = getComputedStyle(elemento);
    return {
      outlineStyle: calculado.outlineStyle,
      outlineWidth: calculado.outlineWidth,
      borderColor: calculado.borderColor,
    };
  });
  expect(estilos.outlineStyle).toBe('solid');
  expect(estilos.outlineWidth).toBe('2px');
  expect(estilos.borderColor).toBe('rgb(155, 74, 47)');
});

test('no hay overflow horizontal en 1440, 960 y 700', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page);

  for (const [width, height] of [
    [1440, 900],
    [960, 640],
    [700, 800],
  ]) {
    await page.setViewportSize({ width, height });
    await page.waitForTimeout(50);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
    ).toBe(true);
  }
});

test('la topbar crece y no corta Ver demo a 200 %', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 640 });
  await abrirPanel(page);
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '32px';
  });

  const boton = await page.getByRole('button', { name: 'Ver demo' }).boundingBox();
  expect(boton).not.toBeNull();
  expect(boton!.y).toBeGreaterThanOrEqual(0);
  expect(boton!.y + boton!.height).toBeLessThanOrEqual(640);
});
