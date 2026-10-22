# Fase 2 — Acta de congelación r14 (product owner)

- **Fecha:** 2026-10-21
- **Commit de código:** `c39dcae53fbee53f6340ec4a8ad09a660d41a150`
- **HEAD del pipeline:** `d255179b3fcddc0ab3294a0620b511b7c32b0f09`
- **Decisión de los roles previos:**
  - QA r3: aprobado con observaciones (0 P1/P2).
  - Seguridad r3: aprobado con observaciones (0 P1/P2).
  - Release r1: aprobado con observaciones; release r2 revalidado sobre la UI nueva.
  - UX r5: aprobado (0 P1/P2).
- **Artefactos de release r2:**
  - `writer_0.2.0_amd64.deb` — 5.171.448 B —
    `5c6502d987bc73daae848f83ac0993f7514da0ac16464e1182ba2c5774f05e83`
  - `writer-0.2.0-1.x86_64.rpm` — 5.171.439 B —
    `15a907b738dba01e18ac56021786d27ce96b0bd5a312ad4c2292650de05a29f9`
- **Estado del árbol:** limpio al congelar.

## Observaciones no bloqueantes registradas

- P3 de UX: 15 residuales (tabs ARIA completos, historial largo, cuarentena
  plegable, búsqueda truncada, latencia de 1.000 marcas, etc.).
- P3 de release: binario sin strip/rpaths, timestamps no reproducibles,
  toolchain flotante, versión sin distinguir el release.
- P3 de seguridad: FIFO en blobs/journal, enlaces duros en Windows, límites de
  tamaño, TOCTOU residual.
- P4: metadatos de paquete, CSP/capacidades, onboarding, copy, idioma del
  proyecto.
- Carencias externas: AppImage, firma/notarización, updater, Windows/macOS, CI
  remoto y app instalada en máquina limpia.

## Alcance de la aceptación

El product owner debe contrastar el resultado con los objetivos de la Fase 2 y
los criterios 1, 5, 6 y 14 del plan, decidir si la fase queda aprobada para
iniciar la Fase 3 y dejar por escrito las condiciones de las carencias externas.
