# Iteración 23 — Fase 2.5: importación y backup

- **Fecha:** 2026-10-23
- **Estado:** Markdown, DOCX, HTML, RTF, backup/restore, impresión, PDF nativo
  con formato configurable, importación como escena o nota, selección múltiple
  y previsualización antes de importar; falta la validación en Word/LibreOffice.
- **Contrato:** [`../importacion-v1.md`](../importacion-v1.md) y
  [`../exportacion-v2.md`](../exportacion-v2.md).

## 1. Alcance

- `crates/core/src/import.rs`:
  - importación de Markdown con frontmatter eliminado;
  - importación de DOCX básico desde `word/document.xml`;
  - límites de 20 MB por archivo, 200 MB descomprimido y 1 GB de backup;
  - backup ZIP sin lock, índice, cuarentena ni sidecars reconstruibles;
  - restauración con validación de rutas, destinos vacíos y verificación final.
- Comandos Tauri `importar_markdown`, `importar_docx`, `crear_backup` y
  `restaurar_backup`.
- UI: botones «Importar Markdown o DOCX» y «Crear backup ZIP» en Plan.
- Pruebas de núcleo de importación, backup y restauración.

## 2. Verificación

```text
cargo test -p writer-core import                          4/4
pnpm typecheck                                            OK
pnpm ui:test                                              63/63
```

## 3. Pendiente

- Validación de DOCX en Word/LibreOffice.
- Pruebas con zip bombs y DOCX malformados reales.

## 4. Límites

- El DOCX solo extrae texto; tablas, imágenes y macros se ignoran.
- No se descargan recursos remotos.
- El backup no incluye `indice.sqlite`, `bloqueo` ni `cuarentena/`.
- Un backup con rutas inválidas o destino ocupado se rechaza.
