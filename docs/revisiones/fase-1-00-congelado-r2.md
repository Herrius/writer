# Fase 1 — Acta de congelación r2

- **Fecha:** 2026-10-22
- **Commit de código y tests:** `1857431`
- **Tag:** `fase-1-congelado-r2`
- **Commit anterior:** `744aaa5` (tag `fase-1-congelado`)
- **Motivo:** corrección del P2 UX-F1-01 (foco del buscador) y de los P3
  UX-F1-02 a UX-F1-06 detectados por la revisión UX.
- **Entorno:** Linux x86_64; Node 26.11.1; pnpm 12.14.0; Playwright 1.64.1;
  Tauri 2.12.4; WebKitWebDriver.

## 1. Cambios respecto a r1

| Hallazgo | Corrección |
|---|---|
| UX-F1-01 P2 | `.buscador-inicio:focus-within` con anillo de 2 px `#9b4a2f` y borde de acento |
| UX-F1-02 P3 | `.crear-tarjeta { align-self: start }` y container query de 600 px para pasar el vacío a fila completa a 200 % |
| UX-F1-03 P3 | `.proyectos-lista .proyectos-vacio { grid-column: 1 / -1 }` |
| UX-F1-04 P3 | Recuento de lista a `#756b5f` (5,22:1) |
| UX-F1-05 P3 | Token `--borde-control: #94897a` para controles (≥3:1) |
| UX-F1-06 P3 | Portada con `aria-hidden`; nombre accesible único por tarjeta |
| UX-F1-10 P4 | Topbar con `min-height`; «Ver demo» ya no se corta a 200 % |
| UX-F1-07 P4 | «Crear desde plantilla» con opacidad 0,55 |
| UX-F1-09 P4 | Avatares decorativos con `aria-hidden` |

No se tocaron dependencias, Tauri, CSP, permisos, IPC, rutas ni lockfiles.

## 2. Verificación

```text
pnpm typecheck                         OK
pnpm ui:test                           60/60
pnpm e2e                               25/25
pnpm exec playwright test e2e/fase-1-panel.spec.ts   13/13
python scripts/ci.py interfaz          OK
python scripts/ci.py e2e               OK
python scripts/ci.py e2e-tauri         OK
python scripts/ci.py empaquetado       OK
python scripts/ci.py producto          OK
python scripts/ci.py seguridad         OK
python scripts/ci.py calidad           OK (94/94)
```

Medidas clave tras la corrección:

- tarjeta de creación a 1440: 155,27 × 297,08; proyecto: 155,27 × 297,08;
- vacío a 960/200: tarjeta 237,22 × 481,20; mensaje en fila completa de
  506,44 px; sin solapamiento;
- lista vacía a 1440: tarjeta 1011,63 × 64,00; mensaje debajo;
- buscador enfocado: `outline: 2px solid #9b4a2f`, borde `#9b4a2f`;
- bordes de controles: `#94897a` (3,17:1 sobre blanco);
- tarjeta accesible: «El umbral Marta Ruiz 12.345 palabras», sin duplicar.

## 3. Estado de los informes

- QA r1 revisó `744aaa5` y aprobó con observaciones; el P3 QA-F1-01 y el P4
  QA-F1-02 quedan corregidos en r2.
- Seguridad r1 revisó `744aaa5`; el diff r1→r2 es presentacional y el gate de
  seguridad se repitió en r2.
- Release r1 revisó `744aaa5`; el empaquetado y el E2E de escritorio se
  repitieron en r2.
- UX r1 bloqueó `744aaa5`; debe repetirse sobre `1857431`.

## 4. Deuda aceptada

- `tablist` sin `tabpanel`/`aria-controls`/flechas (P4).
- Recorte de la portada del modal a 200 % (P4).
- Deuda de release: binario sin strip, rutas absolutas, reproducibilidad de
  paquetes, bundle AppImage obsoleto y cobertura del gate de empaquetado.
- Windows sigue sin build ni pruebas. Release público no autorizado.

## 5. Decisión

Se congela `1857431` para repetir UX/accesibilidad/rendimiento y, si queda
aprobado, pasar al product owner. No se declara aprobada la fase hasta la firma.
