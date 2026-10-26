# HANDOFF — estado para retomar

- **Fecha:** 2026-10-25
- **Repositorio local:** `writer/` (dentro de una carpeta sincronizada por MEGA)
- **Fase actual:** **Fase 3 — Local avanzado, cerrada funcionalmente en Linux
  x86_64**. La Fase 2 local también está cerrada; Windows sigue pendiente y pasa
  a la Fase 4.
- **Plan activo:** [`docs/plan-ejecucion-local-v1.md`](docs/plan-ejecucion-local-v1.md).
  Etapa local/offline: sin cuentas, servidor, sincronización ni servicios online.
- **Objetivo de plataforma:** Windows y Linux. Estado validado: Linux x86_64.
  Windows requiere una máquina real (MSVC, WebView2, instalador y E2E nativo).
- **Rama:** `main`, sin remoto ni CI remoto. Repositorio inicial `1c9f3c6`.
- **Fase 3 congelada:** `347adcb`, tag local `fase-3-congelado`; QA/seguridad r4 y UX r3 aprobaron.
- **Corrección visual 2026-10-24:** portadas ilustradas, tarjetas de libro y
  cabecera limpia en el panel de Proyectos; validada con `e2e/fase-1-panel`
  12/12 y la captura de [`docs/iteraciones/31-rediseno-portadas.md`](docs/iteraciones/31-rediseno-portadas.md).
- **Organización 2026-10-24:** filtros, colecciones, pestaña de plantillas y
  papelera local de proyectos; ver
  [`docs/iteraciones/32-organizacion-proyectos.md`](docs/iteraciones/32-organizacion-proyectos.md).
- **Barra lateral 2026-10-25:** vistas de Archivo y Papelera con contador,
  restauración por fila y estados vacíos; ver
  [`docs/iteraciones/34-archivo-papelera-sidebar.md`](docs/iteraciones/34-archivo-papelera-sidebar.md).
- **Árbol jerárquico 2026-10-25:** iconos Lucide, reglas de un solo libro,
  grupos de personajes, carpetas de Cuaderno, menús por nodo, miga de pan
  sincronizada y páginas de Archivo/Papelera/Plantillas; ver
  [`docs/iteraciones/35-arbol-jerarquico.md`](docs/iteraciones/35-arbol-jerarquico.md).

## 1. Resumen

La Fase 3 añade sobre el motor y la app existentes:

1. **Control de cambios local:** propuestas en sidecar, aceptar/rechazar con CAS,
   diff lado a lado.
2. **Comparación de versiones:** diff por líneas entre una revisión y el
   documento actual.
3. **Formato y preferencias:** temas claro/sepia/oscuro, tipografía, tamaño,
   interlineado, espacio entre párrafos, Quick Open, paleta de comandos, atajos
   y modo enfoque.
4. **Lectura en voz alta:** voces del sistema, documento o selección, velocidad,
   pausa/reanudación/parada.
5. **Herramientas:** palabras, caracteres, párrafos, tiempo de lectura y
   selección.
6. **Rendimiento:** pruebas permanentes con 1.000 escenas y 1.000 marcas.
7. **Organización:** filtros, colecciones locales, gestión de plantillas y
   papelera local de proyectos.
8. **Meta diaria global:** objetivo y actividad compartidos por todos los
   libros, con días libres, opciones de conteo y recordatorio local.
9. **Archivo y papelera de documentos:** páginas propias en el área principal,
   con árbol visible y restauración directa.
10. **Árbol jerárquico:** libros, capítulos, secciones y escenas con menús de
    crear, renombrar, eliminar y exportar, e iconos profesionales Lucide.

La Fase 3 sigue siendo **local/offline**. El corrector es el nativo del WebView;
no se empaqueta diccionario, tesauro ni gramática propia. Las voces dependen del
WebView y del sistema operativo.

## 2. Regla obligatoria: pnpm

`npm` y `npx` están prohibidos. Usar siempre `pnpm` (12.14.0) y
`pnpm-lock.yaml`. No regenerar `package-lock.json`.

```bash
pnpm install --frozen-lockfile
pnpm typecheck
pnpm ui:test
pnpm e2e
pnpm e2e:tauri
pnpm ui:build
```

## 3. Estructura relevante

```text
writer/
  crates/core/                 # motor: proyecto, log, blobs, índice, diff, cambios…
  crates/cli/                  # binario writer
  apps/desktop/src-tauri/      # cáscara Tauri, comandos y DTOs
  ui/src/
    App.tsx                    # shell y coordinación
    EditorMarkdown.tsx         # Tiptap
    features/cambios/          # panel y diff de cambios
    features/preferencias/     # preferencias y modal
    features/paleta/           # Quick Open y comandos
    features/lectura/          # TTS y herramientas
    types/                     # DTOs generados por ts-rs
  e2e/                         # Playwright sobre la demo
  docs/                        # contratos, iteraciones y revisiones
```

`apps/desktop/src-tauri` tiene su propio workspace Rust para no exigir
WebKit/WebView2 al ejecutar `cargo test --workspace` del núcleo.

## 4. Fase 3 implementada

### 3.1 Control de cambios

- `crates/core/src/changes.rs`: sidecars
  `.escritor/cambios/<ULID>.json` con base, propuesta, autor, fecha, revisión y
  hash.
- `Project::registrar_cambio`, `aplicar_cambio`, `descartar_cambio`,
  `listar_cambios`, `leer_cambio`.
- La propuesta no toca el Markdown; aceptar crea una transacción normal y
  conserva el CAS. Un sidecar corrupto va a cuarentena.
- UI: pestaña **Cambios**, interruptor **Proponer cambios**, diff lado a lado,
  aceptar/rechazar y badge de pendientes.
- Contrato: [`docs/cambios-v1.md`](docs/cambios-v1.md).

### 3.2 Comparación

- `crates/core/src/diff.rs`: diff por líneas con prefijo/sufijo, LCS y fallback
  para documentos desproporcionados.
- `Project::comparar_documento` compara el contenido posterior a una revisión
  con el documento actual.
- UI: botón **Comparar con el documento actual** en el historial.
- Contrato: [`docs/diff-v1.md`](docs/diff-v1.md).

### 3.3 Preferencias y paleta

- `ui/src/features/preferencias/`: `localStorage`
  `writer:preferencias:v1`, normalización, variables CSS y modal accesible.
- Temas claro/sepia/oscuro, tipografía serif/sans/mono, tamaño 12–32 px,
  interlineado 1,2–2,6 y espacio entre párrafos 0–2,5 em.
- `Ctrl+P`: Quick Open; `Ctrl+Shift+P`: paleta de comandos; `Ctrl+Shift+H`:
  historial; `Alt+Z`: modo enfoque.
- Contrato: [`docs/formato-v1.md`](docs/formato-v1.md).

### 3.4 Lectura y herramientas

- `ui/src/features/lectura/lectura.ts`: Web Speech API; sin red.
- Lee documento o selección, elige voz y velocidad, pausa/reanuda/detiene.
- Si el WebView no expone `speechSynthesis`, muestra un aviso y desactiva los
  controles.
- Panel **Herramientas**: palabras, caracteres, párrafos, minutos de lectura y
  palabras de la selección.
- Contrato: [`docs/tts-v1.md`](docs/tts-v1.md).

### 3.5 Rendimiento

- `crates/core/tests/rendimiento.rs`: 1.000 escenas en proyecto temporal;
  presupuestos de verificación, listado, reindexado y búsqueda.
- `ui/src/editor/marcas.test.ts`: 1.000 marcas parseadas y serializadas.
- Contrato: [`docs/rendimiento-v1.md`](docs/rendimiento-v1.md).

## 5. Verificación de la Fase 3

```text
cargo fmt --all --check                         OK
cargo clippy --workspace --all-targets -D warnings   OK
cargo test --workspace --all-targets --locked   91 + 50 + 25 + 1 perf
cargo test desktop                              10/10
pnpm typecheck                                  OK
pnpm ui:test                                    88/88
pnpm e2e                                        61/61
ci.py calidad                                   OK
ci.py interfaz                                  OK
ci.py producto                                  OK
ci.py seguridad                                 OK
ci.py e2e-tauri                                 OK (Linux)
ci.py empaquetado                               OK (deb/rpm)
```

Los números exactos y el detalle por bloque están en las iteraciones 26–29 y en
[`docs/iteraciones/30-fase-3-6-cierre.md`](docs/iteraciones/30-fase-3-6-cierre.md).

## 6. Pendiente: Fase 4 Windows

No se puede completar en esta máquina:

- toolchain MSVC y target Windows;
- WebView2;
- diálogos nativos, rutas largas/nombres reservados y cierre de ventana en
  Windows real;
- instalador MSI/NSIS, firma y prueba en máquina limpia;
- `tauri-driver` en Windows;
- validación de DOCX en Word/LibreOffice.

La prueba parcial del CLI Windows bajo Wine está en
[`docs/revisiones/fase-2-wine-2026-10-23.md`](docs/revisiones/fase-2-wine-2026-10-23.md);
no autoriza declarar Windows soportado.

## 7. Publicación

El release público de `0.2.0` **no** está autorizado. Siguen pendientes:

- subir versión y revisar seguridad del commit exacto;
- checksums/firma y decisión AppImage/updater;
- probar la app instalada desde `.deb`/`.rpm`;
- validar DOCX en Word/LibreOffice;
- CI remoto (`.github/`) y runner Windows;
- aceptación final de Fase 5.

## 8. Límites conocidos

- La portada por defecto es un arte fijo; el color la tiñe y aún no hay subida de
  imagen ni selector de arte.
- Filtros y colecciones son `localStorage`; no viajan con el proyecto.
- La meta diaria global también vive en `localStorage`
  (`writer.metas-globales.v1`); libro y desafío siguen en el proyecto.
- La papelera es `<padre>/.writer-papelera` y no la papelera del sistema; sin
  purga automática.
- El control de cambios es por documento completo; no hay aceptación por hunk,
  sugerencias inline ni fusión de tres vías.
- La comparación es por líneas y solo contra revisiones que tocaron el documento.
- Las preferencias son del perfil del WebView, no del proyecto.
- El corrector, diccionario de voz y gramática dependen del sistema; no se
  incluye un motor offline propio.
- La lectura TTS no está validada en WebViewGTK/WebView2 real; el E2E usa una
  síntesis simulada.
- La latencia de tecleo con 1.000 marcas en una sola escena sigue siendo una
  limitación de experiencia; la prueba detecta degradación extrema.
- La ventana CAS no es atómica; SMB/NFS no están soportados para escritura.
- El DOCX/PDF son básicos; no hay TOC, tablas nativas ni imágenes incrustadas.
- `App.tsx`, `styles.css` y `project.rs` siguen siendo grandes; la separación
  por features es parcial.

## 9. Cómo retomar

```bash
cd writer
pnpm install --frozen-lockfile
pnpm typecheck
pnpm ui:test
pnpm e2e
cargo test --workspace --all-targets --locked
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
python scripts/ci.py calidad
python scripts/ci.py interfaz
python scripts/ci.py producto
python scripts/ci.py seguridad
python scripts/ci.py e2e-tauri
python scripts/ci.py empaquetado
```

Para desarrollo:

```bash
pnpm ui:dev     # demo en navegador
pnpm dev        # Tauri + Vite
```

Si `pnpm` no está en PATH, instalarlo con el instalador oficial. **No usar npm
ni npx.**
