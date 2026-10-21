# Iteración 03 — Fase 2 (Interfaz, primera rebanada)

> **Nota:** este informe registra la primera rebanada de solo lectura. La segunda
> rebanada ([`04-fase-2.md`](04-fase-2.md)) añadió corpus golden, autosave,
> conflictos por hash y estados de solo lectura; lo que aquí figura como
> pendiente ya está resuelto o reubicado en la iteración 04.

- **Fecha:** 2026-10-20
- **Alcance:** primera rebanada vertical de la Fase 2: cáscara Tauri, workspace
  pnpm, tipos TypeScript generados y lectura de proyectos con Tiptap.
- **Estado:** en curso. Motor y CLI intactos; la UI abre, lista y lee, pero
  todavía no guarda.

## 1. Entregables

### 1.1 Workspace pnpm

- `package.json` raíz con `packageManager: pnpm@12.14.0`.
- `pnpm-workspace.yaml` con `ui` y `apps/desktop`.
- `pnpm-lock.yaml` único; se eliminaron `package-lock.json`.
- `npm` y `npx` quedan prohibidos en la documentación y en los comandos.
- `pnpm-workspace.yaml` autoriza únicamente el build de `esbuild`.

### 1.2 Cáscara Tauri

- `apps/desktop/src-tauri` con Tauri 2.12, `tauri-plugin-dialog`, ts-rs y
  `writer-core` por path.
- Comandos: `abrir_proyecto`, `listar`, `leer`, `verificar`, `escribir`.
- La UI no implementa reglas: los comandos llaman a `writer-core`.
- El crate queda **fuera del workspace Rust** para que `cargo test --workspace`
  siga sin depender de WebKit/WebView2. Tiene su propio `Cargo.lock`.
- MSRV de la cáscara: Rust 1.90 (Tauri 2.12). El núcleo mantiene 1.89.

### 1.3 UI React + Vite + Tiptap

- `ui/`: React 19, Vite 7, TypeScript 5.9.
- Editor de solo lectura con Tiptap 3 + `@tiptap/markdown`.
- Árbol de manuscrito por libro/capítulo y notas por tipo.
- Panel de documento con metadatos, pestañas **Editor** / **Markdown** y cuerpo
  renderizado.
- Barra de estado con revisión, escenas, notas, errores y avisos.
- Panel de diagnóstico si `verify` devuelve errores.
- Modo demo en navegador (`pnpm ui:dev`) para trabajar sin Tauri.
- Atajo `Ctrl+O` / `Cmd+O` para abrir proyecto.

### 1.4 Tipos generados

- DTOs Rust con `ts-rs`.
- Exportación a `ui/src/types/`:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml exportar_tipos_typescript
```

- `i64`/`u64` se exportan como `number` (revisiones y órdenes caben en el rango
  seguro de JavaScript).

### 1.5 Gate de interfaz

`python scripts/ci.py interfaz` ejecuta:

1. `pnpm install --frozen-lockfile`
2. `pnpm typecheck`
3. `pnpm ui:build`
4. `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml --locked`

## 2. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | Tiptap 3 + `@tiptap/markdown` | Ver `docs/adr/0001-editor-tiptap.md`. |
| 2 | pnpm 12 | Un solo lockfile, instalación reproducible y prohibición explícita de npm/npx. |
| 3 | Tauri fuera del workspace Rust | Evita que los gates del motor dependan de WebKit/WebView2 y acelera `cargo test --workspace`. |
| 4 | Primera rebanada de solo lectura | Valida Tauri, tipos, árbol, Tiptap y estados antes de conectar autosave. |
| 5 | DTOs camelCase | Contrato JSON y TypeScript coherentes. |

## 3. Evidencia

```text
$ pnpm install --frozen-lockfile
Done using pnpm v12.14.0

$ pnpm typecheck
tsc --noEmit                                              # OK

$ pnpm ui:build
vite v7.3.1 · 95 módulos · dist generado                  # OK

$ cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
Finished dev profile                                      # OK

$ cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
2/2                                                       # tipos + comandos

$ pnpm check
Built application at apps/desktop/src-tauri/target/release/writer-desktop

$ python scripts/ci.py interfaz
Interfaz: tipos, build de UI, cáscara Tauri y comandos OK # OK

$ cargo test --workspace --all-targets --locked
65/65                                                     # núcleo intacto
```

## 4. Pendiente

- **Autosave** con debounce, guardado explícito y estados visibles
  (`guardado`, `guardando`, `conflicto`, `recuperación`).
- Editor editable y extensión de marcas propias (personajes, tramas).
- Corpus golden de ida y vuelta Markdown antes de habilitar autosave.
- Búsqueda FTS5 y línea de tiempo en la UI.
- Cuarentena y conflictos en la UI.
- Exportación DOCX y diálogo de destino.
- Iconos/instaladores (`bundle.active` está en `false` en esta rebanada).
- E2E con Playwright/Tauri.
- Integrar `interfaz` en el workflow remoto (`.github` sigue ausente en esta
  copia).
- Criterios 1, 5, 6 y 14 del gate de Fase 2: todavía no cumplidos.

## 5. Cómo probar

```bash
pnpm install
pnpm dev          # Tauri
pnpm ui:dev       # navegador, con botón "Ver demo"
pnpm typecheck
pnpm check
```
