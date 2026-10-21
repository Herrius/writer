# Iteración 04 — Fase 2 (corpus Markdown y autosave)

- **Fecha:** 2026-10-20
- **Alcance:** corpus golden de Markdown, editor editable con autosave, guardado
  explícito, conflictos por hash y estados de solo lectura.
- **Estado:** completado en local. Motor, CLI y cáscara Tauri siguen en verde.

## 1. Entregables

### 1.1 Corpus golden

- `ui/src/markdown/corpus/` con 16 parejas `entrada` / `esperado`.
- `ui/src/markdown/roundtrip.test.ts` comprueba ida y vuelta e idempotencia.
- Contrato en [`../markdown-v1.md`](../markdown-v1.md):
  - esquema aceptado;
  - tabla de normalizaciones;
  - construcciones bloqueadas.
- Se añadió Vitest + jsdom a `ui/` y el gate `interfaz` ejecuta `pnpm ui:test`.

### 1.2 Compatibilidad

- `ui/src/markdown/compatibilidad.ts` detecta HTML, imágenes, tablas, listas de
  tareas y notas al pie fuera de bloques de código.
- Si hay hallazgos, el documento se abre en solo lectura con un diagnóstico.
- La comprobación es conservadora: prefiere bloquear a sobrescribir.

### 1.3 Editor editable y autosave

- Tiptap editable con el mismo esquema que el corpus.
- Autosave con debounce de 900 ms y guardado inmediato con `Ctrl+S` / `Cmd+S`.
- Guardado serializado: si llegan cambios mientras se guarda, se drena la cola
  antes de continuar.
- Estados visibles: `guardado`, `cambios sin guardar`, `guardando…`, `conflicto`,
  `modo recuperación`, `proyecto ocupado`, `error al guardar` y `solo lectura`.
- Cambiar de documento o abrir otro proyecto espera al guardado pendiente.
- Aviso del navegador al cerrar con cambios sin guardar.

### 1.4 Conflictos reales

- `DocumentoDto` incluye `hashArchivo`.
- Nuevo `Project::escribir_cuerpo_con_base(id, cuerpo, hash_base)` en el núcleo:
  compara el hash del archivo actual dentro de la sesión de escritura y devuelve
  `Error::Conflict` sin tocar nada si cambió por fuera.
- `GuardadoDto` devuelve la revisión y el hash nuevo; la UI actualiza ambos.
- El comando `escribir` acepta `hashBase` y la CLI conserva
  `escribir_cuerpo` sin comprobación.

### 1.5 Errores estructurados

- `ErrorDto { tipo, mensaje }` y `TipoError` cruzan el puente Tauri.
- La UI distingue conflicto, recuperación, proyecto ocupado, E/S, esquema, etc.,
  sin analizar cadenas.

### 1.6 Pruebas de interfaz

- 26 pruebas de UI:
  - 16 del corpus;
  - 5 de compatibilidad;
  - 5 de autosave, `Ctrl+S`, conflicto, proyecto con errores y documento
    bloqueado.

## 2. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | Corpus antes de habilitar edición | El ADR exigía saber qué se conserva antes de escribir. |
| 2 | Bloquear construcciones con pérdida | Evita destruir HTML, imágenes, tablas, tareas o notas al pie al autoguardar. |
| 3 | Hash de archivo como base optimista | Detecta cambios externos entre la lectura y el guardado; el CAS del motor sigue cubriendo la ventana final. |
| 4 | `ErrorDto` en vez de mensajes | La UI decide por familia de error, no por prefijo de texto. |
| 5 | Guardado en cola | Evita carreras entre autosave, `Ctrl+S` y cambio de documento. |
| 6 | Un solo esquema de extensiones | Producción y corpus usan `ui/src/editor/extensions.ts`. |

## 3. Evidencia

```text
$ cargo test --workspace --all-targets --locked
67/67                                                     # 29 + 16 + 22

$ cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
2/2                                                       # tipos + comandos

$ pnpm ui:test
Test Files  3 passed (3)
Tests       26 passed (26)

$ pnpm typecheck                                          # OK
$ pnpm ui:build                                           # OK

$ python scripts/ci.py interfaz
Interfaz: tipos, build de UI, cáscara Tauri y comandos OK # OK
```

La prueba nueva del núcleo
`escritura_con_hash_base_detecta_cambios_externos` verifica que un hash viejo
produce `Conflict` y que el cuerpo anterior queda intacto. La prueba de la
cáscara verifica lo mismo a través del comando Tauri.
`escribir_cuerpo_preserva_frontmatter` comprueba que guardar el cuerpo no toca la
cabecera YAML.

## 4. Limitaciones

- El autosave compara el hash del archivo, pero la ventana entre la comprobación
  y el reemplazo sigue siendo *last-writer-wins* frente a un proceso que escriba
  exactamente en ese intervalo (documentado en `protocolo-commit.md`).
- Las construcciones bloqueadas solo se pueden editar por CLI o convirtiéndolas.
- El serializador de Tiptap escapa literales (`_`, `[`, `]`, `*`, `\`) y
  entidades (`<`, `>`, `&`); el significado se conserva, la representación no.
- No hay toolbar de formato: se usan los atajos de Markdown de StarterKit.
- No hay E2E real de escritorio; las pruebas de autosave usan API simulada y un
  editor de prueba.
- `pnpm dev` compila y arranca. En este entorno Wayland el `WebProcess` de
  WebKit falla; con `GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1` el
  proceso se mantiene vivo, pero no se hizo inspección visual.

## 5. Pendiente

- Búsqueda FTS5 y línea de tiempo/historial en la UI.
- Cuarentena visible.
- Exportación DOCX y diálogo de destino.
- Marcas propias de Tiptap (personajes, tramas, comentarios).
- Toolbar de formato y recuento de palabras.
- E2E con Playwright/Tauri.
- `bundle.active=true`, iconos completos e instaladores.
- Integrar `interfaz` en un CI remoto (sigue sin `.github/`).
- Probar en Windows/macOS.
