# Fase 2.1 — Congelación para product owner

- **Fecha:** 2026-10-21
- **Commit de código:** `e0bb3fc3f8f2cbc7a12a7c8ff5f586adac51a002`
- **HEAD:** `1717f0187b8ee4bc1b65b02a44941c8cdea84d7a`
- **QA:** aprobada con observaciones.
- **Seguridad:** aprobada con observaciones.
- **Release:** aprobado con observaciones.
- **UX:** aprobada con observaciones.
- **Estado del árbol:** limpio.

## Resultado a aceptar

- Panel de inicio replicado de las capturas: Proyectos, buscador, pestañas,
  tarjeta de creación, estado vacío, sidebar, cuaderno Ideas.
- Asistente de creación con nombre de carpeta, título, autor, idioma, carpeta
  nativa, portada por color, notas iniciales y `Guardar` / `Guardar y abrir`.
- Comando Tauri `crear_proyecto` con validación de rutas en backend.
- Recientes persistentes con autor, palabras y color.
- Demo explica el límite de creación en navegador.
- 94 pruebas Rust, 3 Tauri, 58 UI, 10 E2E navegador.

## Observaciones no bloqueantes

- Diferencias cosméticas con Dabble: marca, avatar, ilustración, portada con
  imagen, subtítulo y controles de plantillas/colecciones deshabilitados.
- Vista de lista con tarjetas demasiado grandes.
- Foco del modal sin trampa completa y pie fuera de vista a 960×640/200 %.
- Recientes máx. 8; estadísticas solo de recientes.
- Sin vuelta al panel desde el editor; feedback de `Guardar` mejorable.
- P3/P4 de seguridad de Fase 2 y release siguen registrados.

## Decisión solicitada

Aceptar la Fase 2.1 como hito de producto local Linux y autorizar el siguiente
bloque de trabajo, manteniendo las fases 3–5 pospuestas.
