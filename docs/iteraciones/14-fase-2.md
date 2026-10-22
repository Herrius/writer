# Iteración 14 — Aceptación y cierre de la Fase 2

- **Fecha:** 2026-10-21
- **Decisión:** **aprobada con observaciones** como hito de producto en Linux
  x86_64.
- **Release público:** **no autorizado**.
- **Fase 3:** autorizado su inicio con condiciones.
- **Informe:** [`../revisiones/fase-2-product-owner-2026-10-21.md`](../revisiones/fase-2-product-owner-2026-10-21.md)

## 1. Cierre

El product owner firmó la Fase 2 tras QA r3, seguridad r3, release r2 y UX r5,
sin P1/P2 abiertos. Se cumplen los criterios 1, 5, 6 y 14 en el alcance
declarado:

- app usable para escribir con árbol, editor, autosave y conflictos;
- búsqueda FTS5 e historial con reversión;
- estados, cuarentena, marcas y contratos de ida y vuelta;
- DOCX básico válido;
- E2E de navegador y escritorio Linux;
- deb/rpm verificados.

## 2. Artefactos

```text
writer_0.2.0_amd64.deb  5.171.448 B  5c6502d9…f05e83
writer-0.2.0-1.x86_64.rpm  5.171.439 B  15a907b7…5a29f9
```

## 3. Condiciones de publicación

- Subir la versión (`0.2.1` o metadata de release) y asociar hashes al commit.
- Revisión de seguridad del commit exacto, incluido zoom/CSP/FIFO/límites.
- Checksums y firma/notarización; decidir AppImage y updater.
- Matriz Windows/macOS o declarar Linux como único soporte; probar la app
  instalada y los diálogos nativos.
- Validar DOCX en Word/LibreOffice.
- Publicar CI remoto y corregir el desglose de pruebas/documentación.

## 4. Condiciones de Fase 3

- Cerrar SEC-04-B/C y centralizar la lectura de archivos regulares con límites.
- AI safety/privacy: BYOK, prompts, claves, inyección, costes y permisos MCP.
- Políticas `auto`/`proponer`/`bloquear` en motor y servidor MCP.
- Gate de Fase 3 (criterios 7, 8, 9, 12 y 13) y pipeline completo.

## 5. Estado

**Fase 2 cerrada.** El release público queda pendiente de las condiciones
anteriores; la Fase 3 puede comenzar sin reabrir el alcance de la interfaz.
