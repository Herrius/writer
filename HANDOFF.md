# HANDOFF — estado para retomar

- **Fecha:** 2026-10-21
- **Repositorio local:** `writer/` (dentro de una carpeta sincronizada por MEGA)
- **Fase actual:** Fase 2 con QA, seguridad y release aprobados; correcciones
  de UX aplicadas; pendiente repetir UX y aceptación del product owner.
- **Siguiente tarea:** UX/accesibilidad/rendimiento sobre el nuevo commit;
  después product owner.
- **Repositorio Git local:** rama `main`, commit inicial `1c9f3c6` (sin remoto).

## 1. Resumen

La Fase 1 está cerrada en local Linux:

- motor + CLI completos;
- escritura atómica, blobs, op log, recuperación, lock, CAS, índice SQLite/FTS5;
- 92 pruebas de núcleo en verde (34 unitarias + 33 de integración + 25 de CLI);
- MSRV del núcleo verificada con Rust 1.89.0;
- `cargo-deny` en verde para el núcleo.

La Fase 2 tiene seis rebanadas verticales:

1. cáscara Tauri, workspace pnpm, tipos generados, árbol y lectura con Tiptap;
2. corpus golden Markdown, editor editable, autosave y conflictos por hash;
3. búsqueda FTS5, línea de tiempo e historial con reversión;
4. exportación DOCX básica desde CLI y UI;
5. cuarentena visible y marcas propias (personajes, tramas, comentarios);
6. **E2E de navegador y escritorio, toolbar/recuento y empaquetado deb/rpm**.

**La Fase 2 tiene las correcciones de QA r1 y r2, pero no está aprobada.**
Debe repetirse QA sobre el nuevo commit y continuar el pipeline secuencial de §9
del plan: seguridad, release, UX/accesibilidad/rendimiento y aceptación del
product owner. La validación externa (AppImage, firma, Windows/macOS y CI
remoto) queda para el cierre.

Estado actual de la UI:

- workspace **pnpm** con `ui/` y `apps/desktop/`;
- cáscara **Tauri 2.12.4** con comandos que llaman a `writer-core`;
- UI **React 19 + Vite 7 + Tiptap 3**;
- árbol de escenas por libro/capítulo y notas por tipo;
- panel con pestañas **Editor** (editable) / **Markdown** (solo lectura) /
  **Historial**;
- búsqueda FTS5 en la barra lateral con `Ctrl+F`, fragmento resaltado y salto al
  documento;
- exportación DOCX con casilla de notas y diálogo nativo de destino;
- panel de **Cuarentena** con motivo, ruta original y descarte en dos pasos; la
  ruta, el informe y las copias se validan contra enlaces simbólicos;
- toolbar de formato y marcas, con recuento de palabras; aplicar una marca nueva
  sustituye la anterior y el comentario valida `|`/`]]`;
- historial por revisión: metadatos, contenido de cada archivo y reversión con
  confirmación;
- el borrador sobrevive al cambio de pestaña y la reversión bloquea la edición
  durante toda la transición;
- respuestas antiguas del historial se descartan y la confirmación queda ligada
  a proyecto, revisión y `opId`;
- si el proyecto queda sin documentos, la pestaña Historial sigue disponible;
- la búsqueda se repite al cambiar la revisión y el fallback sin `.escritor`
  usa un índice FTS5 en memoria, con la misma tokenización (Unicode descompuesto
  y guiones incluidos);
- autosave a los 900 ms y guardado explícito con `Ctrl+S` / `Cmd+S`;
- estados: `guardado`, `cambios sin guardar`, `guardando…`, `conflicto`,
  `modo recuperación`, `proyecto ocupado`, `error al guardar`, `solo lectura`;
- conflicto real por hash de archivo (`hashBase` → `Error::Conflict`);
- documentos con HTML, imágenes, tablas, tareas o notas al pie se abren en solo
  lectura para no perder contenido;
- 57 pruebas de UI (18 de corpus, 9 de compatibilidad y 30 de autosave, búsqueda,
  historial, DOCX, cuarentena, marcas, accesibilidad y regresiones);
- 9 flujos E2E de navegador con Playwright y 2 escenarios E2E de escritorio con
  Tauri/WebDriver (`Ctrl+S` y cierre de ventana);
- empaquetado deb/rpm generado y verificado en Linux;
- 2 pruebas de la cáscara Tauri;
- gate `python scripts/ci.py interfaz` en verde, además de `e2e` y
  `empaquetado`.

## 2. Regla obligatoria: pnpm, nunca npm/npx

`npm` y `npx` están **prohibidos** en este proyecto. No regenerar
`package-lock.json`.

- Gestor: **pnpm 12.14.0** (`packageManager` en `package.json`).
- Lockfile: `pnpm-lock.yaml` (raíz).
- Workspace: `pnpm-workspace.yaml` con `ui` y `apps/desktop`.
- `pnpm-workspace.yaml` autoriza únicamente el build de `esbuild`.

```bash
curl -fsSL https://get.pnpm.io/install.sh | sh -
# o instalar el binario oficial de pnpm; no usar npm
```

Comandos:

```bash
pnpm install --frozen-lockfile
pnpm typecheck
pnpm ui:test      # corpus + compatibilidad + autosave
pnpm ui:dev       # UI en navegador con demo
pnpm dev          # Tauri + Vite
pnpm ui:build
pnpm check        # tauri build --no-bundle
```

## 3. Estructura relevante

```text
writer/
  Cargo.toml                 # workspace Rust: core + CLI
  package.json               # workspace pnpm y scripts
  pnpm-workspace.yaml
  pnpm-lock.yaml
  crates/core/               # motor
  crates/cli/                # binario writer
  apps/desktop/              # Tauri
    src-tauri/
      Cargo.toml             # workspace Rust propio (fuera del raíz)
      Cargo.lock
      tauri.conf.json
      capabilities/default.json
      icons/
      src/lib.rs             # comandos + DTOs ts-rs
      src/main.rs
  ui/
    src/App.tsx              # árbol, autosave, estados, conflictos
    src/EditorMarkdown.tsx   # Tiptap editable
    src/api.ts               # puente Tauri + demo
    src/editor/extensions.ts # esquema único del editor
    src/markdown/
      compatibilidad.ts      # construcciones bloqueadas
      roundtrip.test.ts      # corpus golden
      corpus/                # 18 parejas entrada/esperado
    src/types/               # generado por ts-rs
    src/styles.css
  docs/
    markdown-v1.md           # contrato del editor
    adr/0001-editor-tiptap.md
    iteraciones/03-fase-2.md
    iteraciones/04-fase-2.md
```

`apps/desktop/src-tauri` está **excluido** del workspace Rust raíz para que
`cargo test --workspace` no necesite WebKit/WebView2. Tiene su propio
`Cargo.lock`. El núcleo mantiene MSRV 1.89; la cáscara Tauri requiere Rust 1.90.

## 4. Qué está implementado

### Núcleo

- `Project::escribir_cuerpo_con_base(id, cuerpo, hash_base)`:
  - compara `hash_base` con el hash del archivo actual dentro de la sesión de
    escritura;
  - devuelve `Error::Conflict` y no escribe si cambió;
  - devuelve `ResultadoGuardado { revision, hash_archivo }`.
- `Project::escribir_cuerpo` sigue existiendo para la CLI sin comprobación.
- `Project::exportar_docx(destino, incluir_notas, forzar)` genera OOXML con
  `pulldown-cmark` y `zip` 9, sin dependencias de extracción de DOCX.
- `preparar_destino` resuelve enlaces simbólicos y `..` antes de comprobar que el
  destino queda fuera del proyecto; lo comparten Markdown y DOCX.
- La exportación DOCX valida los caracteres permitidos por XML 1.0 antes de
  escribir el destino.
- `verify::Documento::hash()` expone el hash del archivo escaneado.

### Comandos Tauri

En `apps/desktop/src-tauri/src/lib.rs`:

- `abrir_proyecto(ruta)`: resumen, revisión, conteos, errores y avisos.
- `listar(ruta, notas)`: escenas y notas, con `hashArchivo`.
- `leer(ruta, objetivo)`: cuerpo Markdown, metadatos y `hashArchivo`.
- `verificar(ruta)`: recuento y diagnósticos.
- `buscar(ruta, consulta, limite)`: resultados FTS5 con fragmento resaltado.
- `historial(ruta)`: revisiones con metadatos y rutas afectadas.
- `mostrar(ruta, revision)`: contenido de los archivos de una revisión.
- `revertir(ruta, revision)`: crea una revisión nueva y devuelve los errores
  pendientes.
- `exportar_docx(ruta, destino, incluirNotas, forzar)`: escribe el DOCX y
  devuelve la ruta final.
- `cuarentena(ruta)`: lista copias con motivo, ruta original, fecha y tamaño.
- `descartar_cuarentena(ruta, archivo)`: elimina solo la copia.
- `escribir(ruta, objetivo, cuerpo, hashBase)`: devuelve
  `GuardadoDto { revision, hashArchivo }`.
- Errores estructurados `ErrorDto { tipo, mensaje }`; `TipoError` distingue
  `conflicto`, `recuperacion`, `ocupado`, `io`, `esquema`, etc.

### UI

- Abrir proyecto con diálogo nativo (`tauri-plugin-dialog`).
- `Ctrl+O` / `Cmd+O`, `Ctrl+S` / `Cmd+S`, `Ctrl+F` / `Cmd+F` para buscar.
- Árbol lateral:
  - manuscrito agrupado por libro/capítulo;
  - notas agrupadas por tipo.
- Buscador FTS5: reemplaza el árbol mientras hay consulta, muestra tipo, ruta y
  fragmento con la coincidencia resaltada; al pulsar un resultado se abre el
  documento.
- Pestaña **Historial**:
  - revisiones del proyecto de más reciente a más antigua;
  - marca de revisión actual;
  - detalle con metadatos y contenido de cada archivo afectado;
  - reversión en dos pasos, con aviso de que crea una revisión nueva.
- **Exportar DOCX** en la barra superior, con casilla **notas** y diálogo nativo.
- **Cuarentena** en el panel: copias de archivos inválidos, con descarte en dos
  pasos; los originales no se tocan.
- Editor Tiptap editable con el esquema de `ui/src/editor/extensions.ts` y
  toolbar de formato, marcas y recuento de palabras.
- Autosave con debounce y cola serializada (si llegan cambios mientras se
  guarda, se drenan antes de continuar).
- Cambiar de documento o abrir otro proyecto espera al guardado pendiente.
- Aviso del navegador al cerrar con cambios sin guardar.
- Banner de error con **Reintentar**, **Recargar desde disco** y **Copiar mi
  texto**.
- Diagnóstico de verificación y de compatibilidad Markdown.
- Modo demo en navegador con persistencia en memoria.

### Tipos

Los DTOs se exportan a `ui/src/types/`. Regenerar:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml exportar_tipos_typescript
```

Los `i64`/`u64` se exportan como `number` (`with_large_int("number")`).

## 5. Contrato Markdown

- Esquema aceptado: párrafos, títulos ATX, énfasis, negrita, tachado, código,
  listas, citas, bloques de código, enlaces, separadores, saltos y marcas propias.
- Corpus: `ui/src/markdown/corpus/` (18 parejas).
- Prueba: `ui/src/markdown/roundtrip.test.ts`.
- Normalizaciones aceptadas: setext → ATX, referencias → inline, salto con
  barra → dos espacios, código indentado → cercado, escapado de literales y
  entidades.
- Construcciones bloqueadas: HTML, imágenes, tablas, listas de tareas y notas al
  pie.
- Marcas: [`docs/marcas-v1.md`](docs/marcas-v1.md); detalle del editor:
  [`docs/markdown-v1.md`](docs/markdown-v1.md).
- Cuarentena: [`docs/cuarentena.md`](docs/cuarentena.md).

## 6. Verificación ejecutada al dejar el proyecto

```text
python scripts/ci.py calidad      OK
python scripts/ci.py interfaz     OK
python scripts/ci.py e2e          OK (9/9, Chromium)
python scripts/ci.py e2e-tauri    OK (Linux, WebDriver)
python scripts/ci.py empaquetado  OK (deb/rpm: estructura, permisos, md5/digests)
python scripts/ci.py producto     OK
cargo test --workspace --all-targets --locked   92/92
pnpm ui:test                      57/57
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked   2/2
cargo-deny (núcleo)               OK
cargo-deny (desktop)              OK
pnpm check                        OK
pnpm --version                    12.14.0
```

`cargo-deny` del desktop tiene una excepción documentada en `deny.toml`:
`RUSTSEC-2024-0370` (`proc-macro-error`, unmaintained) llega por gtk-rs/Tauri en
Linux y no tiene reemplazo compatible con Tauri 2.12. No es una vulnerabilidad
explotable en writer; revisar al actualizar Tauri/gtk-rs.

El E2E de navegador usa Chromium real sobre la demo. El E2E de escritorio se
ejecutó en Linux con `tauri-driver`, `WebKitWebDriver`, X11 y `xdotool`; cubre
`Ctrl+S` y el cierre real de la ventana. No se probó la app instalada desde el
`.deb`/`.rpm`, ni los diálogos nativos, ni Windows/macOS.

## 7. Qué falta — orden recomendado

### Siguiente tarea inmediata

1. **Pipeline de revisión por roles** (plan §9):
   - repetir QA sobre el commit con las correcciones;
   - security reviewer;
   - release engineer;
   - UX/accesibilidad/rendimiento;
   - aceptación del product owner.
2. **Validación externa**: AppImage, firma/notarización y Windows/macOS.

### Resto de Fase 2

- Integrar `interfaz`, `e2e`, `e2e-tauri` y `empaquetado` en CI remoto (hoy no
  hay `.github/`).
- Probar visualmente la app instalada y la toolbar de formato.

## 8. Pendientes de publicación / entorno

- **Repositorio Git local inicializado** en `main` (commit inicial `1c9f3c6`);
  no hay remoto ni CI remoto todavía.
- Falta `.github/` (workflows y Dependabot). `.gitignore`, `.editorconfig` y
  `.gitattributes` ya existen.
- `gitleaks` y `zizmor` están instalados en `~/.local/bin`; el gate `seguridad`
  ejecuta ambos (zizmor se omite si no hay `.github/`).
- La carpeta está en MEGA; excluir `target/`, `node_modules/` y `ui/dist/` de la
  sincronización o mover el repo fuera de MEGA.
- `ui/dist/` está ignorado; si se clona en limpio, ejecutar `pnpm ui:build`
  antes de `cargo check` del desktop (Tauri embebe la UI).
- Los instaladores quedan en `apps/desktop/src-tauri/target/release/bundle/`
  (ignorado). Linux genera deb/rpm; AppImage requiere `linuxdeploy`.
- Chromium de Playwright se instala fuera del repo con `pnpm e2e:install`.
- El E2E de Tauri requiere `tauri-driver` 2.0.6, `WebKitWebDriver` y `xdotool`
  en Linux; no se versionan.

## 9. Limitaciones conocidas

- Ventana CAS no atómica: *last-writer-wins* ante un editor que escriba
  exactamente entre la comprobación del hash y el reemplazo.
- Las rutas de efectos se confinan a `manuscrito/` o `notas/` y `.escritor` debe
  ser real; se rechazan enlaces en componentes existentes. Queda una ventana
  TOCTOU si un atacante local cambia el enlace después de la validación.
- El hash base detecta cambios externos anteriores al guardado, no resuelve
  edición simultánea.
- SMB/NFS no soportados para escritura.
- `write` de la CLI cambia el cuerpo, no el frontmatter.
- El historial no se compacta en v1.
- El serializador de Tiptap escapa literales y entidades; el significado se
  conserva, la representación no.
- El DOCX es básico: listas con prefijo y sangría, sin tablas nativas, TOC ni
  imágenes incrustadas; el proyecto sigue siendo la fuente de verdad Markdown.
- Los caracteres de control inválidos para XML 1.0 se rechazan antes de escribir
  el DOCX; el destino existente no se toca.
- La comprobación de destino resuelve enlaces y `..`, pero no elimina una carrera
  TOCTOU con un proceso local que cambie el enlace después de la validación.
- La búsqueda detecta cambios externos por hash/`mtime`/tamaño y reindexa el
  archivo; `reindex` reconstruye todo el índice si hace falta.
- Sin `.escritor`, la búsqueda reconstruye un FTS5 en memoria en cada consulta;
  es correcto para fixtures y proyectos de solo lectura, no para uso intensivo
  con miles de escenas.
- La toolbar cubre formato básico (negrita, cursiva, tachado, H2, cita, listas,
  código) y marcas; no hay diálogo de enlace.
- El E2E de escritorio es Linux-only y no prueba diálogos nativos ni la app
  instalada; AppImage, firma, notarización y updater quedan pendientes.
- Windows/macOS no se probaron en este entorno.
- Las marcas no se anidan, el comentario no admite `|` ni `]]` y el texto marcado
  no admite `]]`; los ULID deben empezar por `0`–`7`. El DOCX limpia solo prosa y
  conserva código y escapado.
- Con 1.000 marcas en una sola escena, la latencia de tecleo es ~2,2–2,4× la de
  un documento sin marcas; no hay bucle ni CPU alta (P3 de la QA r3, pendiente
  de decisión de UX/rendimiento).
- Los fixtures generados incluyen comentarios HTML de relleno; el editor los
  abre en solo lectura. Para probar edición usa la demo o un proyecto real.

Las garantías completas están en `README.md` y `docs/protocolo-commit.md`.

## 10. Cómo retomar después de mover la carpeta

Las rutas de Tauri son relativas (`../../ui`, `../../../ui/dist`), así que no
hay que reescribir configuración por mover el proyecto.

```bash
cd writer
pnpm install --frozen-lockfile
pnpm typecheck
pnpm ui:test
pnpm e2e:install
pnpm e2e
pnpm e2e:tauri
pnpm ui:build
pnpm build

# Núcleo
cargo test --workspace --all-targets --locked

# Interfaz
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
python scripts/ci.py interfaz
python scripts/ci.py e2e
python scripts/ci.py e2e-tauri
python scripts/ci.py empaquetado

# App de escritorio
pnpm dev
```

Si `pnpm` no está en PATH, instalarlo de nuevo con el instalador oficial de
pnpm. **No usar npm ni npx.**
