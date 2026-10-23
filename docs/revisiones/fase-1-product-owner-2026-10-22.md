# Fase 1 — Product owner (propuesta de aceptación)

- **Fecha:** 2026-10-22
- **Commit de código y tests:** `1857431`
- **Tag:** `fase-1-congelado-r2`
- **Alcance:** rediseño del panel de Proyectos (local/offline).
- **Estado:** **pendiente de firma del product owner**. La fase está completada
  por el builder y revisada por QA, seguridad, release y UX.

## 1. Resumen

La Fase 1 retiró meta diaria, «Proyectos personales», «Ideas» y el botón
«Ayuda» del panel; conservó el resto de controles deshabilitados como excepción
documentada; alineó la tarjeta de creación con la tarjeta completa; mantuvo la
portada 2:3; amplió la barra lateral a toda la altura con una reserva vacía; y
corrigió foco, estados vacíos y contraste tras la revisión UX.

## 2. Informes

| Rol | Informe | Decisión |
|---|---|---|
| QA | [`fase-1-qa-2026-10-22.md`](fase-1-qa-2026-10-22.md) | aprobado con observaciones |
| Seguridad | [`fase-1-seguridad-2026-10-22.md`](fase-1-seguridad-2026-10-22.md) | aprobado con observaciones |
| Release | [`fase-1-release-2026-10-22.md`](fase-1-release-2026-10-22.md) | aprobado con observaciones |
| UX/accesibilidad/rendimiento r1 | [`fase-1-ux-2026-10-22.md`](fase-1-ux-2026-10-22.md) | bloqueado por UX-F1-01 |
| UX/accesibilidad/rendimiento r2 | [`fase-1-ux-2026-10-22-r2.md`](fase-1-ux-2026-10-22-r2.md) | aprobado con observaciones |
| Congelación r2 | [`fase-1-00-congelado-r2.md`](fase-1-00-congelado-r2.md) | — |

## 3. Evidencia

```text
pnpm typecheck                         OK
pnpm ui:test                           60/60
pnpm e2e                               25/25
python scripts/ci.py interfaz          OK
python scripts/ci.py e2e               OK
python scripts/ci.py e2e-tauri         OK
python scripts/ci.py empaquetado       OK
python scripts/ci.py producto          OK
python scripts/ci.py seguridad         OK
python scripts/ci.py calidad           OK (94/94)
```

## 4. Deuda aceptada

### P3

- **UX-F1-13:** los campos del modal conservan bordes `#d8d0c4` (1,53:1). Es
  deuda previa al rediseño; se propone corregirla en la fase de accesibilidad o
  en un pase breve antes de publicar.

### P4

- **UX-F1-14:** la paleta del modal se solapa 21 px a 960 y 26 px a 200 % con
  las etiquetas de «Carpeta padre»/«Idioma» (UX22-04).
- **UX-F1-15:** el E2E de lista vacía solo cubre 1440 y el de foco usa
  `.focus()` en vez de Tab. Cobertura adicional recomendada.
- **UX-F1-16:** el acta r2 decía 3,17:1; el valor real es 3,43:1. Corregido en
  el acta.
- **UX-F1-08:** `tablist` sin `tabpanel`, `aria-controls` ni flechas.
- **UX-F1-11:** la portada del modal recorta el título a 200 %.
- **SEC-F1-01/02:** validación de rutas de `localStorage` y controles bidi en
  títulos/autores; heredados de SEC21-04.
- **REL-F1-01/02:** binario sin strip, rutas absolutas y paquetes no
  reproducibles; release público bloqueado.
- **REL-F1-03/04:** bundle AppImage obsoleto, warning de chunk y cobertura
  limitada del gate de empaquetado.

## 5. Condiciones

- El release público **no** está autorizado.
- Windows sigue sin build, instalador ni pruebas.
- La deuda P3/P4 queda registrada para las fases siguientes; no bloquea el
  cierre local de la Fase 1.

## 6. Decisión propuesta

**Aprobado con observaciones.** Se cierra la Fase 1 en Linux x86_64 y se
autoriza el arranque de la Fase 2.0 (contratos y modularización) cuando el
product owner firme.

**Firma del product owner:** pendiente.
