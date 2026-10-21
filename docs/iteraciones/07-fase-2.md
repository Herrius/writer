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

## 3. Correcciones de la revisión

- **Cuarentena por enlaces (P1):** `.escritor`, `.escritor/cuarentena` e
  `informe.jsonl` deben ser reales; los enlaces se rechazan antes de leer,
  escribir o borrar. Las copias enlazadas no se listan y su descarte se rechaza.
  Regresión: `cuarentena_rechaza_enlaces_en_directorio_informe_y_copias`.
- **Marcas anidadas (P1):** las marcas se excluyen entre sí en el esquema y la
  toolbar quita las tres antes de aplicar una nueva. Regresión: superposición
  personaje → trama conserva solo trama.
- **Comentarios con `|` (P1):** se valida el comentario y la selección antes de
  aplicar la marca; el tokenizer exige un único `|` y admite corchetes simples
  (`ver [capítulo 2]`). Regresiones en `marcas.test.ts`.
- **DOCX con código o escapado (P2):** `quitar_marcas` usa los rangos de texto
  del parser CommonMark para limpiar solo prosa. Respeta código en listas y
  citas, backticks sin pareja, destinos de enlaces y construcciones escapadas.
  Regresiones unitarias y una exportación DOCX completa.
- **ULID incompleto (P2):** TypeScript y DOCX reutilizan la regla del esquema
  (primer carácter `0`–`7`). Regresiones en Rust y TypeScript.

La comprobación de cuarentena es común a listar, copiar y descartar: valida
también el ancestro `.escritor` y parte de la raíz canónica del proyecto. El
informe se actualiza por reemplazo atómico, de modo que ni un enlace duro al
informe modifica otro archivo. Las pruebas de la toolbar usan Tiptap real para
aplicar y reabrir marcas, rechazar `|`/`]]` sin cambiar el manuscrito y conservar
los comentarios con corchetes simples.

## 4. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | Cuarentena como copia, no movimiento | El original sigue siendo la fuente de verdad y `verify` lo reporta. |
| 2 | Descartar solo la copia | Evita borrar datos del proyecto por accidente. |
| 3 | Marcas como sintaxis Markdown reservada | Mantiene el archivo legible y la ida y vuelta verificable. |
| 4 | Tokenizer propio con ULID validado | Evita interpretar texto normal como marca. |
| 5 | DOCX solo con el texto marcado | El comentario es una anotación de revisión, no prosa del manuscrito. |
| 6 | Comentario inline en la toolbar | Evita depender de `window.prompt`, que no es fiable en WebView. |

## 5. Evidencia

```text
$ cargo test --workspace --all-targets --locked
84/84                                                     # 33 + 26 + 25

$ cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
2/2                                                       # tipos + comandos

$ pnpm ui:test
Test Files  5 passed (5)
Tests       50 passed (50)

$ pnpm typecheck                                          # OK
$ pnpm ui:build                                           # OK
$ pnpm check                                              # OK

$ python scripts/ci.py calidad                            # OK
$ python scripts/ci.py interfaz                           # bloqueado al instalar: almacén pnpm de solo lectura
$ python scripts/ci.py producto                           # OK
$ cargo deny check                                        # OK
```

En la comprobación posterior, `typecheck`, las 50 pruebas de UI, el build de
Tauri sin bundle y `fmt`/`clippy`/`check`/las 2 pruebas de la cáscara se ejecutan
por separado. La instalación reproducible del gate `interfaz` no se puede
repetir mientras el almacén de pnpm sea de solo lectura.

## 6. Limitaciones

- Sin marcas anidadas.
- El texto de un comentario no puede contener `|`; el texto marcado no puede
  contener `]]`.
- Los comentarios no tienen hilo, autor ni estado resuelto.
- La UI no crea archivos de cuarentena; eso sigue siendo `verify --cuarentena`.
- Descartar una copia no repara el original; `verify` sigue mostrando el error.

## 7. Pendiente

- E2E con Playwright/Tauri.
- `bundle.active=true`, iconos e instaladores.
- CI remoto y prueba en Windows/macOS.
- Probar la toolbar y los colores con inspección visual real.
