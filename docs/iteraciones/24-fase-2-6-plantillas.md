# Iteración 24 — Fase 2.6: plantillas locales

- **Fecha:** 2026-10-23
- **Estado:** plantillas locales y creación desde plantilla implementadas.
- **Contrato:** [`../plantillas-v1.md`](../plantillas-v1.md).

## 1. Alcance

- `crates/core/src/templates.rs`: manifiesto, listado, validación y creación
  desde plantilla con estructura y notas iniciales.
- Directorio de datos de la app: `<app_data_dir>/plantillas/<slug>/`.
- Comandos Tauri `listar_plantillas` y `crear_desde_plantilla`.
- UI: selector de plantilla en el asistente de creación.
- Pruebas de plantilla inválida, listado y creación real.

## 2. Verificación

```text
cargo test -p writer-core templates                       4/4
pnpm typecheck                                            OK
pnpm ui:test                                              63/63
```

## 3. Pendiente

- Copiar contenido personalizado de `manuscrito/` y `notas/` de la plantilla;
  hoy se genera estructura y notas iniciales.
- Gestión de plantillas desde la UI (importar, borrar, duplicar).
- Prueba de escritorio con el directorio de datos real.

## 4. Límites

- Las plantillas no viven en el proyecto ni se incluyen en su backup.
- No se copian IDs, historial, comentarios, metas ni índices.
- No hay biblioteca pública ni descarga.
