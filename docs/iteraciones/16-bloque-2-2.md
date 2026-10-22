# Iteración 16 — Interfaz de escritura e historial (Bloque 2.2)

- **Fecha:** 2026-10-21
- **Alcance:** sustituir la interfaz antigua de escritura por una adaptación de
  las referencias de Dabble (editor e historial).
- **Estado:** aprobado con observaciones (Linux x86_64); release público no autorizado.

## 1. Interfaz

- Cabecera con volver a Proyectos, pestañas, exportación, Markdown, historial y
  avatar.
- Árbol lateral por manuscritos (libro/capítulo), tramas, personajes, cuaderno y
  utilidades.
- Página blanca central con título, metadatos, toolbar, editor y recuento.
- Panel derecho Metas/Notas con gráfico, temporizador visual y metadatos.
- La cuarentena pasa a un desplegable para no tapar el editor.
- Historial con lista de versiones, contenido de la revisión y línea de tiempo
  inferior sticky con Recuperar / Salir.

## 2. Funciones

- Se conservan autosave, conflictos, búsqueda, reversión, DOCX, cuarentena,
  marcas, accesibilidad y cierre de ventana.
- Se añade botón `Ver Markdown` en la cabecera y vuelta al panel de Proyectos
  guardando antes de salir.

## 3. Evidencia

```text
pnpm typecheck                    OK
pnpm ui:test                      59/59
pnpm e2e                          12/12
cargo test --workspace            94/94
```

## 4. Pendiente y plataformas

- Pipeline del Bloque 2.2 completado; aprobado con observaciones.
- SEC21-02 (TOCTOU de `.escritor`) queda como riesgo residual P3 aceptado hasta
  una revisión de release; el E2E del diálogo nativo sigue siendo una condición
  de validación de la app real.
- Objetivo de producto: **Windows y Linux**. Lo implementado y probado hasta
  ahora es Linux x86_64; Windows está pendiente de build, instalador y pruebas.
