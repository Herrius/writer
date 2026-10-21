# Iteración 07 — Fase 2 (cuarentena visible y marcas propias)

- **Fecha:** 2026-10-21
- **Alcance:** listar y descartar copias de cuarentena desde CLI y UI; marcas de
  personaje, trama y comentario en Tiptap, con ida y vuelta Markdown.
- **Estado:** completado en local. Motor, CLI, UI y cáscara Tauri en verde.

## 1. Cuarentena visible

### Motor y CLI

- `Project::cuarentena()` cruza `.escritor/cuarentena/` con `informe.jsonl` y
  devuelve archivo, ruta original, motivo, fecha y tamaño.
- `Project::descartar_cuarentena(archivo)` elimina solo la copia; valida el
  nombre y nunca toca el original.
- `verify::cuarentenar` añade el campo `archivo` al informe para que la lista
  pueda vincular cada copia con su motivo.
- Comandos nuevos:

```bash
writer quarantine list ~/novela
writer quarantine list ~/novela --json
writer quarantine discard <archivo> ~/novela
```

### Tauri y UI

- Comandos `cuarentena` y `descartar_cuarentena`.
- Panel **Cuarentena** cuando hay copias, con archivo, motivo, ruta original y
  tamaño.
- **Descartar** en dos pasos; los originales no se tocan.
- La lista se carga al abrir el proyecto.

## 2. Marcas propias

### Sintaxis

```text
[[personaje:ULID|texto]]
[[trama:ULID|texto]]
[[comentario:texto|texto]]
```

Contrato completo: [`../marcas-v1.md`](../marcas-v1.md).

### Editor

- `ui/src/editor/marcas.ts` define:
  - un tokenizer inline para la sintaxis;
  - las marcas `personaje`, `trama` y `comentario`;
  - serialización y parseo Markdown.
- `extensionesEditor` incluye las marcas.
- Toolbar sobre el editor:
  - selector de personaje;
  - selector de trama;
  - comentario con campo inline;
  - quitar marcas.
- Las marcas se pintan con colores/subrayados distintos.
- `comentario` no se borra con “quitar formato” (`clearable: false`).

### Exportación

- Markdown conserva la sintaxis.
- DOCX elimina los delimitadores y muestra solo el texto marcado; los
  comentarios no se imprimen.
- Las construcciones con ULID inválido o sintaxis ajena se dejan intactas.

### Corpus

- Nuevo caso `17-marcas.md` y su esperado.
- Pruebas específicas en `ui/src/editor/marcas.test.ts`: ida y vuelta,
  ULID inválido, sintaxis ajena y formato inline dentro de una marca.

## 3. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | Cuarentena como copia, no movimiento | El original sigue siendo la fuente de verdad y `verify` lo reporta. |
| 2 | Descartar solo la copia | Evita borrar datos del proyecto por accidente. |
| 3 | Marcas como sintaxis Markdown reservada | Mantiene el archivo legible y la ida y vuelta verificable. |
| 4 | Tokenizer propio con ULID validado | Evita interpretar texto normal como marca. |
| 5 | DOCX solo con el texto marcado | El comentario es una anotación de revisión, no prosa del manuscrito. |
| 6 | Comentario inline en la toolbar | Evita depender de `window.prompt`, que no es fiable en WebView. |

## 4. Evidencia

```text
$ cargo test --workspace --all-targets --locked
76/76                                                     # 29 + 22 + 25

$ cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
2/2                                                       # tipos + comandos

$ pnpm ui:test
Test Files  4 passed (4)
Tests       41 passed (41)

$ pnpm typecheck                                          # OK
$ pnpm ui:build                                           # OK
$ pnpm check                                              # OK

$ python scripts/ci.py calidad                            # OK
$ python scripts/ci.py interfaz                           # OK
$ python scripts/ci.py producto                           # OK
$ cargo deny check                                        # OK
```

## 5. Limitaciones

- Sin marcas anidadas.
- El texto de un comentario no puede contener `|`; el texto marcado no puede
  contener `]]`.
- Los comentarios no tienen hilo, autor ni estado resuelto.
- La UI no crea archivos de cuarentena; eso sigue siendo `verify --cuarentena`.
- Descartar una copia no repara el original; `verify` sigue mostrando el error.

## 6. Pendiente

- E2E con Playwright/Tauri.
- `bundle.active=true`, iconos e instaladores.
- CI remoto y prueba en Windows/macOS.
- Probar la toolbar y los colores con inspección visual real.
