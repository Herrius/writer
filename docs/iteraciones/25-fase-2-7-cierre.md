# Iteración 25 — Fase 2.7: integración y cierre funcional

- **Fecha:** 2026-10-23
- **Estado:** **funcionalidad de la Fase 2 implementada y verificada**; deuda
  técnica y de revisión registrada.
- **Plan de producto:** `../../Plan - Rediseño del panel de Proyectos.md`.
- **Plan de ejecución:** [`../plan-ejecucion-local-v1.md`](../plan-ejecucion-local-v1.md).

## 1. Bloques de la Fase 2

| Bloque | Estado | Iteración |
|---|---|---|
| 2.0 contratos | Contratos escritos; modularización parcial | 18 |
| 2.1 manuscrito | Renombrar, duplicar, mover, archivo/papelera, estados, etiquetas, filtros, copiar/pegar | 20 |
| 2.2 comentarios | Anclas v2, sticky, hilos, respuestas, resolución, huérfanos y filtros | 21 |
| 2.3 planificación | Plot Grid, tipos de nota, @menciones, rol/alias de personajes | 22 |
| 2.4 metas | Metas, sesiones, temporizador, racha y gráfico de 7 días | 19 |
| 2.5 importación/backup | Markdown, DOCX, HTML, RTF, backup/restore e impresión | 23 |
| 2.6 plantillas | Plantillas locales y creación desde plantilla | 24 |
| 2.7 integración | Gates completos y este cierre | 25 |

## 2. Verificación global

```text
cargo fmt --all --check                                   OK
cargo clippy --workspace --all-targets --all-features     OK
cargo test --workspace --all-targets --locked             61 + 36 + 25
cargo test desktop                                        7/7
pnpm typecheck                                            OK
pnpm ui:test                                              63/63
pnpm e2e                                                  34/34
python scripts/ci.py interfaz                             OK
python scripts/ci.py producto                             OK
python scripts/ci.py seguridad                            OK
python scripts/ci.py e2e-tauri                            OK
python scripts/ci.py empaquetado                          OK
```

## 3. Cambios de mayor riesgo

- **Esquema aditivo:** las extensiones viven en `.escritor/` y no cambian
  `esquema: 1`; los proyectos v1 siguen abriéndose.
- **Comentarios:** el ancla usa ULID v2 y sidecar; los v1 quedan como legado.
- **Importación:** DOCX/HTML/RTF se procesan en local con límites; el DOCX solo
  extrae texto y no ejecuta macros.
- **Backup:** excluye lock, índice y cuarentena; restaura solo en destino
  vacío y valida rutas.
- **Plantillas:** no copian historial ni IDs.

## 4. Deuda que queda registrada

### Funcional

- Comentarios: editar/borrar mensajes con historial visible, copiar/pegar con y
  sin comentarios, comentarios de solo lectura en previsualización.
- Metas: estadísticas históricas fuera del mes actual.
- Planificación: vista gráfica del Plot Grid, edición/borrado de puntos,
  resolución de @menciones a títulos y conversión de Ideas a proyecto.
- Importación: selección parcial, previsualización y PDF nativo con formato de
  manuscrito (hoy se usa el diálogo de impresión).
- Plantillas: copiar contenido personalizado de `manuscrito/` y `notas/`.

### Técnica

- `project.rs` y `App.tsx` siguen siendo grandes; la separación por feature
  existe en el núcleo (`goals`, `comments`, `planning`, `templates`, `import`)
  pero falta dividir la UI y `styles.css` por feature.
- Reordenar escenas dentro del mismo capítulo por drag & drop.
- Cobertura de accesibilidad de los paneles nuevos.
- Windows sigue sin build, instalador ni pruebas; release público no
  autorizado.
- Deuda de release y seguridad de la etapa anterior (SEC-04-B/C, límites,
  TOCTOU, strip, reproducibilidad, AppImage, CI remoto).

## 5. Decisión

La **funcionalidad de la Fase 2 está implementada y los gates están en verde**
en Linux x86_64. No se declara “cerrada con firma” hasta pasar la revisión
formal de QA/UX sobre este commit y aceptar la deuda de §4.

Próximo paso: pipeline de revisión de Fase 2 y, después, Fase 3.
