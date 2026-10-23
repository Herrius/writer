# Iteración 25 — Fase 2.7: integración y cierre funcional

- **Fecha:** 2026-10-23
- **Estado:** funcionalidad principal implementada y gates en verde; los P2 de
  la QA independiente se corrigieron y queda pendiente la re-revisión formal y
  la deuda registrada.
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
| 2.7 integración | Gates completos, correcciones QA y este cierre | 25 |

## 2. Verificación global

```text
cargo fmt --all --check                                   OK
cargo clippy --workspace --all-targets --all-features     OK
cargo test --workspace --all-targets --locked             69 + 38 + 25
cargo test desktop                                        7/7
pnpm typecheck                                            OK
pnpm ui:test                                              63/63
pnpm e2e                                                  37/37
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

- Planificación: la vista es una línea de puntos, no un lienzo con arrastre;
  la edición de nombre/color y de puntos está implementada.
- Plantillas: la gestión desde la UI (importar, borrar, duplicar) queda fuera
  del contrato mínimo.
- Comentarios: salto al ancla implementado; queda pulir el resaltado visual.

### Técnica

- `project.rs`, `App.tsx` y `styles.css` siguen siendo grandes; la separación
  por feature existe en el núcleo (`goals`, `comments`, `planning`, `templates`,
  `import`, `pdf`) pero falta dividir la UI y `styles.css` por feature.
- Reordenar escenas dentro del mismo capítulo por drag & drop.
- Cobertura de accesibilidad de los paneles nuevos.
- Windows sigue sin build, instalador ni pruebas; release público no
  autorizado.
- Deuda de release y seguridad de la etapa anterior (SEC-04-B/C, límites,
  TOCTOU, strip, reproducibilidad, AppImage, CI remoto).

## 5. Decisión

La funcionalidad principal de la Fase 2 está implementada y los gates están en
verde en Linux x86_64. La QA independiente detectó ocho P2; todos se corrigieron
con pruebas (backup, sidecars, DOCX/RTF, exportación v2, temporizador y
plantillas). No se declara “cerrada con firma” hasta repetir la revisión formal
y aceptar la deuda de §4.

Próximo paso: pipeline de revisión de Fase 2 y, después, Fase 3.
