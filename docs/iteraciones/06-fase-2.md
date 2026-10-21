# Iteración 06 — Fase 2 (exportación DOCX)

- **Fecha:** 2026-10-21
- **Alcance:** exportación básica a DOCX desde la CLI y la UI, con notas
  opcionales y destino elegido por diálogo.
- **Estado:** completado en local. Motor, CLI, UI y cáscara Tauri en verde.

## 1. Entregables

### 1.1 Motor

- `Project::exportar_docx(destino, incluir_notas, forzar)`.
- Generación directa de OOXML: `[Content_Types].xml`, `_rels/.rels`,
  `word/document.xml`, `word/_rels/document.xml.rels`, `docProps/core.xml` y
  `docProps/app.xml`, comprimidos con `zip` 9.
- Renderizador Markdown propio con `pulldown-cmark`:
  - títulos, párrafos, negrita, cursiva, tachado, código inline, enlaces,
    listas, citas, bloques de código, separadores y saltos;
  - imágenes como `[imagen: alt]`;
  - notas agrupadas por tipo.
- `preparar_destino` unifica las comprobaciones de Markdown y DOCX: destino
  fuera del proyecto y `--forzar`.
- Escritura atómica con `fsutil::escribir_atomico`.

### 1.2 CLI

```bash
writer export docx ~/salida.docx ~/novela
writer export docx ~/salida.docx ~/novela --notas
writer export docx ~/salida.docx ~/novela --forzar
```

### 1.3 Tauri y UI

- Comando `exportar_docx(ruta, destino, incluirNotas, forzar)`.
- Botón **Exportar DOCX** y casilla **notas** en la barra superior.
- Diálogo nativo `save` con filtro `.docx` y nombre por defecto basado en el
  título del proyecto.
- La UI guarda cambios pendientes antes de exportar y sobrescribe después de la
  confirmación del diálogo.
- Modo demo: devuelve el destino simulado.

### 1.4 Decisión de dependencia

Se empezó con `docx-rs`, pero arrastraba `zip` 8.6 con
`RUSTSEC-2026-0368` y `RUSTSEC-2026-0369`. Aunque writer solo escribe DOCX y no
extrae archivos, se evitó añadir una excepción de seguridad: se genera el OOXML
directamente con `zip` 9.0.5, cuyas advisories quedan en verde. El documento
sigue siendo válido y se comprobó con `pandoc`.

### 1.5 Correcciones de la revisión

- **Destino por enlace simbólico (P1):** `preparar_destino` ahora normaliza la
  ruta y resuelve enlaces simbólicos del destino y de sus ancestros existentes
  antes de comprobar que queda fuera del proyecto. La comprobación es compartida
  por Markdown y DOCX y usa `symlink_metadata` para detectar enlaces rotos.
  Regresión: `exportar_rechaza_destino_dentro_del_proyecto_por_symlink`.
- **XML inválido con caracteres de control (P2):** `validar_xml` recorre
  `word/document.xml`, `word/_rels/document.xml.rels` y `docProps/core.xml` y
  rechaza cualquier carácter fuera del rango permitido por XML 1.0 antes de
  escribir el ZIP. El error es `Error::Input` con el código `U+XXXX` y el
  destino existente queda intacto. Regresión:
  `exportar_docx_rechaza_caracteres_de_control_sin_tocar_destino`.

## 2. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | CLI antes que UI | Regla CLI-first. |
| 2 | OOXML directo + `zip` 9 | Evita una dependencia vulnerable y da control sobre el XML. |
| 3 | `pulldown-cmark` | Parser CommonMark probado, sin reimplementar Markdown. |
| 4 | Listas con prefijo y sangría | Word nativo exige numeración/estilos; la v1 prioriza texto completo y legible. |
| 5 | `--forzar` y diálogo nativo | La CLI es explícita; la UI confía en la confirmación del diálogo. |
| 6 | Notas opcionales | Igual que la exportación Markdown. |

## 3. Evidencia

```text
$ cargo test --workspace --all-targets --locked
74/74                                                     # 29 + 21 + 24

$ cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
2/2                                                       # tipos + comandos + DOCX

$ pnpm ui:test
Test Files  3 passed (3)
Tests       34 passed (34)

$ pnpm typecheck                                          # OK
$ pnpm ui:build                                           # OK
$ pnpm check                                              # OK

$ python scripts/ci.py calidad                            # OK
$ python scripts/ci.py interfaz                           # OK
$ python scripts/ci.py producto                           # OK
$ cargo deny check                                        # OK (núcleo y desktop)
```

Comprobación manual adicional: `pandoc salida.docx -t plain` muestra la portada,
los títulos, el texto con énfasis, las listas, la cita, el código y el enlace; y
`unzip -t` no detecta errores en el ZIP.

## 4. Limitaciones

- Las listas no son listas nativas de Word: llevan prefijo y sangría.
- No hay tablas, HTML, TOC, números de página ni estilos de Word.
- Las imágenes solo aportan texto alternativo.
- El DOCX es una salida, no un formato de ida y vuelta.
- No se probó visualmente en Microsoft Word; sí con `pandoc`, `unzip` y lectura
  directa de `document.xml`.

## 5. Pendiente

- Cuarentena visible.
- Marcas propias de Tiptap (personajes, tramas, comentarios).
- Toolbar de formato y recuento de palabras.
- E2E con Playwright/Tauri.
- `bundle.active=true`, iconos e instaladores.
- CI remoto y prueba en Windows/macOS.
