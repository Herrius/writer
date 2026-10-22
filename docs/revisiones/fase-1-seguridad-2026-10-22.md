# Fase 1 — Seguridad del rediseño del panel de Proyectos

- **Commit revisado:** `744aaa5` (`test: completar la QA visual y de teclado del panel`)
- **Tag:** `fase-1-congelado` (resuelve a `744aaa5`)
- **HEAD al revisar:** `522eccf` (`docs: añadir informe QA de la fase 1`). Solo
  añade documentación; no toca `ui/src` ni `e2e`.
- **Baseline:** `fca190f`.
- **Fecha:** 2026-10-22.
- **Entorno:** Linux x86_64 (kernel 7.2.9); Node 26.11.1; pnpm 12.14.0;
  Rust/cargo 1.99.1; Python 3.14.8; cargo-deny 0.20.2; gitleaks local;
  zizmor 1.31.1; Chromium de Playwright 1.64.1. Servidor Vite ya activo en
  `http://127.0.0.1:1420`.
- **Árbol:** limpio al empezar. Al terminar, `git status --porcelain` solo
  muestra este informe sin versionar; no hay cambios en código ni tests.
- **Decisión:** **aprobado con observaciones** — P1: 0 · P2: 0 · P3: 0 ·
  P4: 2. Ningún hallazgo bloquea la fase. Los dos P4 son deuda heredada de
  `SEC21-04`, no una regresión introducida por Fase 1.

No edité código, tests, contratos ni documentación de producto. Las sondas
viven en `/tmp/fase-1-seguridad/` y no se versionan.

## 1. Inmutabilidad del artefacto

```text
$ git diff --stat 744aaa5 -- ui/src e2e
(vacío)

$ git diff --stat 744aaa5
 HANDOFF.md                                   |  63 ++++--
 docs/iteraciones/17-fase-1-rediseno-panel.md |  26 +++
 docs/revisiones/README.md                    |   1 +
 docs/revisiones/fase-1-00-congelado.md       |  76 ++++++++
 docs/revisiones/fase-1-qa-2026-10-22.md      | 276 +++++++++++++++++++++++++++
 5 files changed, 425 insertions(+), 17 deletions(-)
```

El diff posterior al hash congelado es exclusivamente documental. El diff
funcional revisado es:

```text
$ git diff --stat fca190f..744aaa5 -- ui/src e2e docs HANDOFF.md
 docs/README.md                               |   2 +
 docs/inicio-v1.md                            |  14 +-
 docs/iteraciones/17-fase-1-rediseno-panel.md | 235 +++++++++++++++++++++++++++
 e2e/fase-1-panel.spec.ts                     | 137 ++++++++++++++++
 ui/src/App.test.tsx                          |  18 ++
 ui/src/Inicio.tsx                            |  62 ++-----
 ui/src/styles.css                            | 147 ++++++++---------
 7 files changed, 483 insertions(+), 132 deletions(-)
```

`HANDOFF.md` no cambia en `fca190f..744aaa5`; sus cambios son posteriores al
freeze y documentales. No hay cambios en `apps/desktop/src-tauri`, manifiestos,
`pnpm-lock.yaml`, `Cargo.lock` ni `deny.toml`.

## 2. Diff funcional revisado

| Archivo | Cambio | Relevancia de seguridad |
|---|---|---|
| `ui/src/Inicio.tsx` | Retirada de meta diaria, «Proyectos personales», «Ideas» y «Ayuda»; nueva medida oculta de la tarjeta; reserva lateral | Sin HTML crudo, sin nuevas acciones, sin nuevas rutas |
| `ui/src/styles.css` | Tarjeta de creación con `::after`, medida oculta y reserva lateral; retirada de CSS de los bloques eliminados | Sin `url()`, `@import`, `expression()` ni contenido generado con datos |
| `ui/src/App.test.tsx` | Prueba de regresión de retiradas/controles conservados | Solo DOM de test |
| `e2e/fase-1-panel.spec.ts` | 7 pruebas de geometría, overflow, teclado y lista | Usa `localStorage` y datos fijos; no abre URLs externas |
| `docs/inicio-v1.md`, `docs/iteraciones/17…`, `docs/README.md` | Contrato visual y registro de la fase | No hay cambios de contrato de seguridad |

La documentación mantiene la excepción aprobada de controles deshabilitados y
no declara nuevas capacidades, permisos ni superficies de red.

## 3. Superficies de inyección

Búsquedas ejecutadas sobre `ui/src`, `e2e`, `crates` y `apps` (sin
`node_modules` ni `target`):

```text
rg -n "dangerouslySetInnerHTML|innerHTML|outerHTML|eval\(|new Function|window\.open|document\.write|insertAdjacentHTML|srcdoc|javascript:" ui/src e2e apps crates
→ sin coincidencias

rg -n "href=|url\(|fetch\(|WebSocket|EventSource|sendBeacon|XMLHttpRequest|http://|https://" ui/src e2e
→ solo el enlace de salto `href="#panel-principal"`, URLs de corpus de Markdown y la URL del servidor de desarrollo en Playwright
```

- **HTML crudo:** no se usa `dangerouslySetInnerHTML` ni `innerHTML`. `marked`
  se emplea solo como lexer (`marked.lexer`/`walkTokens`), no como parser a
  HTML; el editor es Tiptap y los títulos/autores se renderizan como nodos de
  texto de React.
- **JavaScript dinámico:** no hay `eval`, `new Function`, `document.write`,
  `window.open` ni `javascript:`. El único `href` del panel es el enlace de
  salto interno; no recibe datos.
- **Red:** no hay `fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource` ni
  `sendBeacon` en la UI. La sonda de navegador no registró ninguna petición
  fuera de `http://127.0.0.1:1420`.
- **Estilos inline:** solo hay dos en toda la UI:
  - `Inicio.tsx:69`, `background: linear-gradient(155deg, ${color}, #2f2732)`;
  - `Inicio.tsx:565`, `background: tono` con la paleta fija.
  El único valor dinámico es `color`, validado como `#rrggbb` en
  `recientes.ts:58`; la paleta es una constante. No hay CSS construido con
  títulos, autores ni rutas.
- **CSS:** no hay `url()`, `@import`, `expression()` ni `-moz-binding` en
  `styles.css`. El `::after` nuevo tiene `content: ""` y `pointer-events: none`.

## 4. `localStorage` y uso de títulos, autores, colores y rutas

### 4.1 Validación de lectura

`ui/src/recientes.ts:34-71` valida, antes de exponer los datos a React:

- JSON malformado, no-array y excepciones → `[]`;
- `ruta`: string no vacía y ≤ 4096 caracteres;
- `titulo`: string no vacío y ≤ 200 caracteres;
- `autor`: `undefined` o string ≤ 200 caracteres;
- `palabras`: `undefined` o número finito ≥ 0;
- `color`: `undefined` o `#rrggbb` (regex anclada, mayúsculas aceptadas);
- deduplicación por `ruta`;
- límite de 8 entradas.

La escritura (`guardarRecientes`) captura errores de cuota/almacenamiento
deshabilitado. El color que escribe `recordarProyecto` procede de la paleta
fija o de `colorDesdeTitulo`, no de un campo libre.

### 4.2 Sondas de inyección

Sonda `/tmp/fase-1-seguridad/probe.mjs` (Chromium, `addInitScript` con
`writer.recientes.v1`):

| Caso | Resultado |
|---|---|
| `titulo: '<img src=x onerror="window.__xssTitulo=1">'` | Se muestra como texto; 0 nodos `img`, 0 `svg`, `__xssTitulo = 0` |
| `autor: '<svg onload="window.__xssAutor=1">'` | Texto escapado; `__xssAutor = 0` |
| `ruta: 'javascript:window.__xssRuta=1'` | Tarjeta aceptada; al pulsarla no hay navegación, diálogo ni ejecución (`__xssRuta = 0`) |
| `color: 'red; background-image: url(http://127.0.0.1:9/pixel)'` | Entrada rechazada por la regex; no se crea tarjeta ni petición |
| `color: '#AABBCC'` | Aceptado y aplicado como color de portada |
| `titulo` de 201 caracteres / `ruta` de 4097 / `autor` de 201 | Entrada rechazada |
| JSON inválido, objeto, `null`, número, array anidado | Panel vacío sin excepción de página |
| 12 entradas | Solo 8 tarjetas; duplicados por ruta deduplicados |

La sonda `/tmp/fase-1-seguridad/probe-limites.mjs` repitió los límites y no
produjo `pageerror`. La sonda `/tmp/fase-1-seguridad/probe-rutas-bidi.mjs`
comprobó títulos con `U+202E` (RLO) y autores con `U+0000`: se renderizan como
texto, sin HTML ni ejecución; tampoco hay peticiones externas.

### 4.3 Rutas

`Inicio.tsx:421` entrega `reciente.ruta` a `onAbrirReciente`; `App.tsx:1131`
lo pasa a `cargar`; `api.ts:174-179` invoca `abrir_proyecto` con `{ ruta }` en
Tauri. El backend (`apps/desktop/src-tauri/src/lib.rs:273-307`) construye
`Project::new(&ruta)` y llama a `verificar`; no interpreta la cadena como URL ni
la ejecuta.

En la demo web, pulsar una tarjeta con `ruta: 'javascript:…'` no navega ni abre
diálogos. En Tauri, la cadena llega al comando IPC como ruta de sistema; no hay
`eval` ni apertura de URL. La validación de forma de la ruta (absoluta, local,
sin UNC/dispositivo/ADS) sigue siendo la deuda `SEC21-04`; ver SEC-F1-01.

## 5. DOM nuevo de medición (`crear-medida`)

El nuevo bloque de `Inicio.tsx:401-416` es estático:

```html
<span class="crear-medida" aria-hidden="true">
  <span class="crear-medida-portada"></span>
  <span class="tarjeta-datos"><strong>&nbsp;</strong><span>&nbsp;</span><small>&nbsp;</small></span>
</span>
<span class="crear-contenido">
  <span class="crear-mas" aria-hidden="true">+</span>
  <span>Crear nuevo proyecto</span>
</span>
```

Comprobado en `/tmp/fase-1-seguridad/probe-dom.mjs`:

- `aria-hidden="true"` y `visibility: hidden` en la medida;
- 0 elementos enfocables dentro de la medida;
- ningún dato de usuario ni HTML crudo;
- el nombre accesible exacto del botón es `Crear nuevo proyecto` (el `+` está
  oculto a tecnologías de asistencia);
- la medida no recibe eventos de puntero ni introduce contenido ejecutable.

La tarjeta sigue siendo un `<button>` sin descendientes interactivos. El
`::after` es decorativo y no intercepta clics (`pointer-events: none`).

## 6. Retirada de «Ayuda» y código muerto

- Se eliminaron el estado `aviso`, su `setAviso`, el banner asociado y el botón
  «Ayuda» del panel. `rg` no encuentra `setAviso` ni las clases
  `.tarjeta-meta`, `.meta-punto`, `.proyectos-personales`, `.cuaderno-ideas`,
  `.cuaderno-espiral` o `.cuaderno-bombillas` en la UI actual.
- No había listeners, temporizadores, peticiones ni escrituras asociadas al
  aviso; solo mostraba un mensaje al pulsar «Ideas». Su retirada no deja efectos
  secundarios.
- El banner `exito` y `.inicio-exito` siguen en uso para el resultado de crear
  un proyecto; no son código muerto.
- El botón «Ayuda» del editor (`ui/src/App.tsx:1244`) sigue existiendo, sin
  `onClick`, fuera del alcance de Fase 1. Es un control inerte preexistente, no
  una regresión del panel.

## 7. CSP, aislamiento, permisos, IPC y dependencias

Comparación de hashes entre `fca190f` y `744aaa5`:

| Archivo | Resultado |
|---|---|
| `apps/desktop/src-tauri/tauri.conf.json` | idéntico |
| `apps/desktop/src-tauri/tauri.linux.conf.json` | idéntico |
| `apps/desktop/src-tauri/capabilities/default.json` | idéntico |
| `apps/desktop/src-tauri/src/lib.rs` | idéntico |
| `ui/index.html` | idéntico |
| `ui/package.json`, `package.json`, `pnpm-lock.yaml` | idénticos |
| `Cargo.toml`, `Cargo.lock`, `deny.toml` | idénticos |

- **CSP de producción:** sin cambios. Sigue siendo
  `default-src 'self'; connect-src ipc: http://ipc.localhost; img-src 'self'
  asset: http://asset.localhost data:; style-src 'self' 'unsafe-inline';
  script-src 'self'`. No se añadió `unsafe-eval`, `unsafe-inline` a scripts ni
  orígenes remotos.
- **CSP de desarrollo:** `devCsp` mantiene `unsafe-inline`/`unsafe-eval` y
  `localhost:1420`, como en el baseline. Es una configuración de desarrollo, no
  la de producción, y no cambió en la fase.
- **Aislamiento de Tauri:** no se activó ni se relajó el patrón `isolation`; la
  configuración es idéntica al baseline. La ventana sigue con las mismas
  capacidades: `core:default`, destruir ventana, zoom de webview y
  `dialog:default`. No hay permisos de `fs`, `http`, `shell` ni nuevos comandos.
- **IPC:** el diff no toca `lib.rs` ni el `generate_handler!`; no hay comandos
  nuevos. La UI solo usa los comandos existentes.
- **Dependencias:** no hay cambios en manifiestos ni lockfiles; no se añadió
  ninguna dependencia de red, filesystem o ejecución.
- **Escritura de archivos:** el nuevo código de UI no escribe archivos. La
  creación de proyectos sigue siendo el flujo existente del modal y su comando
  Tauri; la tarjeta nueva solo abre ese modal.
- **Red:** sin nuevas URLs, fetch ni plugins de red. La única superficie de
  conexión de la CSP sigue siendo IPC.

## 8. Gate de seguridad

```text
$ python scripts/ci.py seguridad
== SEGURIDAD ==
$ cargo deny --all-features --manifest-path .../Cargo.toml check
advisories ok, bans ok, licenses ok, sources ok
$ cargo deny --manifest-path .../apps/desktop/src-tauri/Cargo.toml check
advisories ok, bans ok, licenses ok, sources ok
$ gitleaks git . --redact --no-banner
77 commits scanned, no leaks found
$ gitleaks dir <árbol versionado> --redact --no-banner
no leaks found
zizmor: sin .github/ ni workflows; se omite
== seguridad: OK en 2.9 s ==
```

- **cargo-deny:** verde en las dos invocaciones. Los avisos son
  `multiple-versions = "warn"` (duplicados de `hashbrown`, `syn`, `thiserror`,
  etc.), no errores. La excepción `RUSTSEC-2024-0370` de `deny.toml` no se
  encuentra en el grafo raíz, pero `proc-macro-error` sí está en el lock de
  Tauri (`apps/desktop/src-tauri/Cargo.lock:2455`) y la segunda invocación
  cubre ese árbol; no hay una vulnerabilidad detectada.
- **gitleaks:** verde sobre los 77 commits y sobre la copia del árbol
  versionado; no hay secretos.
- **zizmor:** no se ejecutó porque el repositorio no tiene `.github/` ni
  workflows. No es un fallo del gate: `scripts/ci.py` lo omite expresamente en
  ese caso. Si se añade CI, debe ejecutarse.
- **Complementarios:** `pnpm typecheck` OK y `pnpm ui:test` 60/60. No son el
  gate de seguridad, pero confirman que el artefacto congelado es coherente.

## 9. Hallazgos

| ID | Severidad | Hallazgo | Origen |
|---|---|---|---|
| SEC-F1-01 | **P4** | La ruta de `localStorage` solo se valida por tipo/longitud; puede ser una ruta señuelo, UNC o de dispositivo | Heredado de `SEC21-04`; no lo introduce Fase 1 |
| SEC-F1-02 | **P4** | Títulos y autores admiten caracteres de control y bidi dentro del límite de 200; pueden alterar la presentación | Heredado de `SEC21-04`; no lo introduce Fase 1 |

No hay P1, P2 ni P3 de seguridad en el diff.

### SEC-F1-01 — P4: ruta de recientes sin validación semántica

**Descripción.** `recientes.ts:46-48` acepta cualquier string no vacío de hasta
4096 caracteres como `ruta`. `Inicio.tsx:421` la usa como identificador y la
entrega a `onAbrirReciente`; en Tauri, `App.tsx:1131` → `cargar` →
`api.ts:174-179` la envía a `abrir_proyecto`. El backend no exige que sea una
ruta absoluta local ni rechaza UNC, rutas de dispositivo o ADS.

**Reproducción.**

```text
1. Abrir la app (o la demo) con DevTools.
2. localStorage['writer.recientes.v1'] = JSON.stringify([{
     ruta: '\\\\atacante\\share\\proyecto',
     titulo: 'Señuelo', palabras: 1, color: '#4d6b8a'
   }])
3. Recargar: aparece una tarjeta con ese título.
4. En Tauri, pulsarla invoca abrir_proyecto con esa ruta.
```

En la sonda web `/tmp/fase-1-seguridad/probe.mjs`, una entrada con
`ruta: 'javascript:…'` se muestra como tarjeta y al pulsarla no navega, no abre
diálogos y no ejecuta código. En Tauri la cadena se trata como ruta de sistema,
no como URL; no hay ejecución de comandos. El riesgo residual es una tarjeta
señuelo o un acceso a una ruta UNC/recurso local no previsto, no una ejecución
remota.

**Impacto.** Requiere escritura en `localStorage` (acceso al perfil del WebView
o ejecución de script en el origen local). La app ya permite elegir cualquier
carpeta con «Abrir proyecto», así que no hay elevación de privilegios; el
residual es de defensa en profundidad y de privacidad (la ruta se guarda en
claro). Aplica sobre todo a Windows, donde las rutas UNC pueden provocar
autenticación SMB.

**Tratamiento.** Aceptado como P4 heredado para Fase 1. Para el release:
validar que la ruta sea absoluta y local, rechazar UNC/dispositivo/ADS y
normalizarla antes de persistir o abrir; o pedir confirmación si la ruta ya no
existe. No bloquea esta fase de UI.

### SEC-F1-02 — P4: controles y bidi en títulos/autores

**Descripción.** La validación de `titulo` y `autor` comprueba tipo y longitud,
pero no elimina caracteres de control C0/C1 ni marcas bidireccionales
(`U+202A`–`U+202E`, `U+2066`–`U+2069`). React los escapa como texto, así que no
hay HTML ni JavaScript, pero pueden alterar la lectura visual de una tarjeta.

**Reproducción.**

```text
1. localStorage['writer.recientes.v1'] = JSON.stringify([{
     ruta: '/tmp/x', titulo: 'Señuelo\u202Egnp.exe',
     autor: 'Autor\u0000Oculto', palabras: 1, color: '#4d6b8a'
   }])
2. Recargar: el título conserva el override RLO y el autor el NUL;
   no se crean nodos HTML ni se ejecuta script.
```

Verificado con `/tmp/fase-1-seguridad/probe-rutas-bidi.mjs`: el texto se
renderiza, `innerHTML` no contiene etiquetas inyectadas y no hay peticiones
externas.

**Impacto.** Suplantación visual o glifos extraños en una tarjeta; no hay
ejecución ni cruce de frontera. Requiere escritura en `localStorage`, igual que
SEC-F1-01.

**Tratamiento.** Aceptado como P4 heredado. Sanear controles/bidi al leer o al
mostrar y añadir un caso de prueba si se decide cerrar la deuda. No bloquea.

## 10. Decisiones de configuración no modificadas

- **`isolation` de Tauri:** no está configurado, igual que en el baseline. No
  se relajó nada; el contenido es local y la CSP de producción es estricta.
- **`style-src 'unsafe-inline'`:** necesario para los estilos inline de React y
  presente antes de la fase. No se amplió.
- **`devCsp` con `unsafe-eval`/`unsafe-inline`:** solo desarrollo; no afecta al
  bundle de producción.
- **Controles deshabilitados:** Notificaciones, Filtros, Colección, Plantillas,
  Papelera, Más opciones y Crear desde plantilla siguen presentes y
  deshabilitados. No tienen handlers ni efectos; la excepción está documentada
  en `docs/inicio-v1.md` y no añade superficie.

## 11. Decisión

**Aprobado con observaciones.** No hay P1, P2 ni P3 de seguridad. Los dos P4
(SEC-F1-01 y SEC-F1-02) son deuda heredada de `SEC21-04`, no una regresión de
Fase 1, y no bloquean el pipeline. La fase no añade dependencias, permisos,
comandos IPC, red ni escritura de archivos; no relaja CSP, aislamiento ni
validación de rutas existente.

Para un release público queda pendiente, fuera del alcance de esta fase:
cerrar o aceptar formalmente SEC21-04 (rutas señuelo/UNC y metadatos en claro),
probar Windows y ejecutar zizmor cuando exista CI. El release sigue sin
autorización.

**Firma:** Seguridad del proyecto `writer` — 2026-10-22.

## 12. Evidencia

- Gate: `/tmp/fase-1-seguridad-gate.log`.
- Typecheck: `/tmp/fase-1-seguridad-typecheck.log`.
- UI tests: `/tmp/fase-1-seguridad-uitest.log`.
- Sondas: `/tmp/fase-1-seguridad/probe.mjs`, `probe-limites.mjs`,
  `probe-dom.mjs` y `probe-rutas-bidi.mjs`.
- Diff y hashes: comandos de §1 y §7; `git diff --stat 744aaa5 -- ui/src e2e`
  vacío.
