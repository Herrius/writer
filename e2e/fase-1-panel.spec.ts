import { expect, test } from '@playwright/test';

const recientes = [
  {
    ruta: '/tmp/el-umbral',
    titulo: 'El umbral',
    autor: 'Marta Ruiz',
    palabras: 12345,
    color: '#4d6b8a',
  },
  {
    ruta: '/tmp/la-casa',
    titulo: 'La casa de las mareas',
    autor: 'Lucía Ferrer',
    palabras: 80210,
    color: '#5a8fa3',
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

async function cajas(page: import('@playwright/test').Page) {
  const crear = await page.locator('.crear-tarjeta').boundingBox();
  const tarjeta = await page.locator('.tarjeta-proyecto').first().boundingBox();
  const portada = await page.locator('.portada').first().boundingBox();
  const datos = await page.locator('.tarjeta-datos').first().boundingBox();
  expect(crear).not.toBeNull();
  expect(tarjeta).not.toBeNull();
  expect(portada).not.toBeNull();
  expect(datos).not.toBeNull();
  return { crear: crear!, tarjeta: tarjeta!, portada: portada!, datos: datos! };
}

test('la tarjeta de creación iguala la tarjeta completa y la portada es 2:3', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page);
  const { crear, tarjeta, portada, datos } = await cajas(page);

  expect(Math.abs(crear.height - tarjeta.height)).toBeLessThanOrEqual(2);
  expect(portada.width / portada.height).toBeCloseTo(2 / 3, 2);
  expect(datos.height).toBeGreaterThanOrEqual(54);
  expect(datos.height).toBeLessThanOrEqual(66);
});

test('el estado vacío conserva la altura de una tarjeta completa', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page, []);
  const crear = await page.locator('.crear-tarjeta').boundingBox();
  expect(crear).not.toBeNull();
  expect(crear!.height).toBeGreaterThanOrEqual(290);
  expect(crear!.height).toBeLessThanOrEqual(305);
});

test('a 960 y 200 % no hay overflow y las alturas coinciden', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 640 });
  await abrirPanel(page);
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '32px';
  });

  const { crear, tarjeta, portada } = await cajas(page);
  expect(Math.abs(crear.height - tarjeta.height)).toBeLessThanOrEqual(2);
  expect(portada.width / portada.height).toBeCloseTo(2 / 3, 2);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
});

test('la barra lateral ocupa la altura del panel y reserva espacio vacío', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page);

  for (const [width, height] of [
    [1440, 900],
    [960, 640],
  ]) {
    await page.setViewportSize({ width, height });
    const panel = await page.locator('.mis-proyectos').boundingBox();
    const sidebar = await page.locator('.inicio-sidebar').boundingBox();
    const reserva = await page.locator('.inicio-sidebar-reserva').boundingBox();
    expect(panel).not.toBeNull();
    expect(sidebar).not.toBeNull();
    expect(reserva).not.toBeNull();
    expect(Math.abs(panel!.height - sidebar!.height)).toBeLessThanOrEqual(1);
    expect(reserva!.height).toBeGreaterThan(50);
    expect(
      await page.locator('.inicio-sidebar-reserva button, .inicio-sidebar-reserva input').count(),
    ).toBe(0);
  }

  await page.setViewportSize({ width: 700, height: 800 });
  await expect(page.locator('.inicio-sidebar-reserva')).toBeHidden();
});

test('la tarjeta de creación iguala al proyecto a 960 y 700', async ({ page }) => {
  for (const [width, height] of [
    [960, 640],
    [700, 800],
  ]) {
    await page.setViewportSize({ width, height });
    await abrirPanel(page);
    const crear = await page.locator('.crear-tarjeta').boundingBox();
    const tarjeta = await page.locator('.tarjeta-proyecto').first().boundingBox();
    expect(crear).not.toBeNull();
    expect(tarjeta).not.toBeNull();
    expect(Math.abs(crear!.height - tarjeta!.height)).toBeLessThanOrEqual(2);
  }
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

test('el vacío a 960/200 conserva la medida de la tarjeta', async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 640 });
  await abrirPanel(page, []);
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '32px';
  });

  const medidas = await page.evaluate(() => {
    const caja = (selector: string) => {
      const elemento = document.querySelector(selector);
      if (!elemento) return null;
      const rect = elemento.getBoundingClientRect();
      return {
        width: rect.width,
        height: rect.height,
        y: rect.y,
        bottom: rect.bottom,
      };
    };
    return {
      crear: caja('.crear-tarjeta'),
      medida: caja('.crear-medida'),
      vacio: caja('.proyectos-vacio'),
    };
  });
  expect(medidas.crear).not.toBeNull();
  expect(medidas.medida).not.toBeNull();
  expect(medidas.vacio).not.toBeNull();
  expect(Math.abs(medidas.crear!.height - medidas.medida!.height)).toBeLessThanOrEqual(1);
  expect(medidas.vacio!.y).toBeGreaterThanOrEqual(medidas.crear!.bottom - 1);
  expect(medidas.vacio!.width).toBeGreaterThan(400);
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
  ).toBe(true);
});

test('los controles usan bordes con contraste y la tarjeta un nombre único', async ({ page }) => {
  await abrirPanel(page);

  const bordes = await page.evaluate(() => {
    const borde = (selector: string) => {
      const elemento = document.querySelector(selector);
      return elemento ? getComputedStyle(elemento).borderTopColor : null;
    };
    const crear = document.querySelector('.crear-tarjeta');
    return {
      buscador: borde('.buscador-inicio'),
      boton: borde('.boton-inicio'),
      tabs: borde('.tabs-inicio'),
      crear: crear ? getComputedStyle(crear, '::after').borderTopColor : null,
    };
  });
  expect(bordes.buscador).toBe('rgb(148, 137, 122)');
  expect(bordes.boton).toBe('rgb(148, 137, 122)');
  expect(bordes.tabs).toBe('rgb(148, 137, 122)');
  expect(bordes.crear).toBe('rgb(148, 137, 122)');

  const nombre = await page.locator('.tarjeta-proyecto').first().ariaSnapshot();
  expect(nombre).toContain('button "El umbral Marta Ruiz 12.345 palabras"');

  await page.getByRole('button', { name: 'Vista de lista' }).click();
  const colorLista = await page
    .locator('.proyectos-lista .tarjeta-datos small')
    .first()
    .evaluate((elemento) => getComputedStyle(elemento).color);
  expect(colorLista).toBe('rgb(117, 107, 95)');
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

test('la lista vacía es compacta y no solapa el mensaje', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page, []);
  await page.getByRole('button', { name: 'Vista de lista' }).click();

  const crear = await page.locator('.crear-tarjeta').boundingBox();
  const vacio = await page.locator('.proyectos-vacio').boundingBox();
  expect(crear).not.toBeNull();
  expect(vacio).not.toBeNull();
  expect(crear!.height).toBeLessThan(120);
  expect(vacio!.y).toBeGreaterThanOrEqual(crear!.y + crear!.height - 1);
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

test('la tarjeta de creación se abre con teclado y devuelve el foco', async ({ page }) => {
  await abrirPanel(page);
  const crear = page.getByRole('button', { name: /Crear nuevo proyecto/ });
  await crear.focus();
  await page.keyboard.press('Enter');
  await expect(page.getByRole('dialog', { name: 'Crear nuevo proyecto' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).toBeHidden();
  await expect(crear).toBeFocused();
});

test('la lista conserva la tarjeta de creación compacta', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await abrirPanel(page);
  await page.getByRole('button', { name: 'Vista de lista' }).click();

  const crear = await page.locator('.crear-tarjeta').boundingBox();
  const tarjeta = await page.locator('.tarjeta-proyecto').first().boundingBox();
  expect(crear).not.toBeNull();
  expect(tarjeta).not.toBeNull();
  expect(crear!.height).toBeLessThan(120);
  expect(tarjeta!.height).toBeLessThan(120);
});
