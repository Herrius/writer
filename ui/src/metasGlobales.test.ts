import { afterEach, describe, expect, it } from 'vitest';
import {
  cargarMetasGlobales,
  diasRecientes,
  esDiaLibre,
  fechaLocal,
  guardarMetasGlobales,
  normalizarMetasGlobales,
  progresoDelDia,
  rachaGlobal,
  registrarActividad,
  type MetaDiariaGlobal,
  type MetasGlobales,
} from './metasGlobales';

const meta: MetaDiariaGlobal = {
  objetivo: 10,
  desde: '2026-10-20',
  diasLibres: [],
  contarSoloManuscrito: true,
  contarPegadas: true,
  contarEliminaciones: true,
  recordatorio: false,
};

function estado(actividad: Record<string, number> = {}): MetasGlobales {
  return { version: 1, diaria: meta, actividad };
}

afterEach(() => {
  window.localStorage.clear();
});

describe('metas globales', () => {
  it('registra actividad y calcula el progreso', () => {
    const hoy = fechaLocal();
    const conActividad = registrarActividad(estado(), hoy, 3);
    expect(conActividad.actividad[hoy]).toBe(3);
    expect(progresoDelDia(conActividad)).toMatchObject({
      hechas: 3,
      objetivo: 10,
      porcentaje: 30,
      restantes: 7,
    });
  });

  it('no deja actividad negativa y valida datos de localStorage', () => {
    const hoy = fechaLocal();
    expect(registrarActividad(estado({ [hoy]: 2 }), hoy, -10).actividad[hoy]).toBe(0);
    window.localStorage.setItem(
      'writer.metas-globales.v1',
      JSON.stringify({
        version: 1,
        diaria: { ...meta, objetivo: 0, diasLibres: [8] },
        actividad: { [hoy]: -4, mala: 10 },
      }),
    );
    const cargadas = cargarMetasGlobales();
    expect(cargadas.diaria).toBeNull();
    expect(cargadas.actividad[hoy]).toBeUndefined();
  });

  it('respeta días libres y calcula la racha', () => {
    const hoy = new Date();
    const ayer = new Date(hoy);
    ayer.setDate(ayer.getDate() - 1);
    const anteayer = new Date(hoy);
    anteayer.setDate(anteayer.getDate() - 2);
    const libreHoy = { ...meta, diasLibres: [hoy.getDay()] };
    const conLibre: MetasGlobales = {
      version: 1,
      diaria: libreHoy,
      actividad: {
        [fechaLocal(hoy)]: 0,
        [fechaLocal(ayer)]: 10,
        [fechaLocal(anteayer)]: 10,
      },
    };
    expect(esDiaLibre(libreHoy, hoy)).toBe(true);
    expect(rachaGlobal(conLibre, hoy)).toBe(3);
  });

  it('devuelve los días recientes con etiquetas', () => {
    const dias = diasRecientes(estado({ [fechaLocal()]: 4 }), 7);
    expect(dias).toHaveLength(7);
    expect(dias.at(-1)?.palabras).toBe(4);
    expect(dias.at(-1)?.etiqueta).toHaveLength(1);
  });

  it('persiste y normaliza la meta', () => {
    guardarMetasGlobales({ ...estado(), diaria: { ...meta, diasLibres: [1, 1, 8] } });
    const cargadas = cargarMetasGlobales();
    expect(cargadas.diaria?.diasLibres).toEqual([1]);
    expect(normalizarMetasGlobales(null).diaria).toBeNull();
  });
});
