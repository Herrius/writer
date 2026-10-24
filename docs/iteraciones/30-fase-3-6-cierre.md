# Iteración 30 — Fase 3.6: integración y cierre funcional

- **Fecha:** 2026-10-24
- **Estado:** Fase 3 cerrada funcionalmente en Linux x86_64; Windows pasa a la
  Fase 4.
- **Plan:** [`../plan-ejecucion-local-v1.md`](../plan-ejecucion-local-v1.md).

## 1. Bloques

| Bloque | Estado | Contrato |
|---|---|---|
| 3.1 Control de cambios | Implementado | [`../cambios-v1.md`](../cambios-v1.md) |
| 3.2 Comparación de versiones | Implementado | [`../diff-v1.md`](../diff-v1.md) |
| 3.3 Formato y preferencias | Implementado | [`../formato-v1.md`](../formato-v1.md) |
| 3.4 Lectura y escritura | Implementado | [`../tts-v1.md`](../tts-v1.md) |
| 3.5 Rendimiento | Implementado | [`../rendimiento-v1.md`](../rendimiento-v1.md) |
| 3.6 Integración | Este documento | — |

## 2. Cambios de mayor riesgo

- **Cambios locales:** el sidecar no modifica el Markdown; aceptar usa la misma
  transacción y CAS que un guardado. Un sidecar corrupto se aparta y no bloquea
  el proyecto.
- **Diff:** algoritmo local, sin dependencias de red; el fallback para documentos
  desproporcionados pierde granularidad pero no corrección.
- **Preferencias:** `localStorage` normalizado; no se escriben en el proyecto.
- **TTS:** Web Speech API; las voces son del sistema. El E2E inyecta una
  implementación simulada, no valida las voces reales de WebKitGTK/WebView2.
- **Rendimiento:** presupuestos amplios en build debug; no es un benchmark
  estadístico.

## 3. Verificación

```text
cargo fmt --all --check                              OK
cargo clippy --workspace --all-targets --all-features  OK
cargo test --workspace --all-targets --locked        74 + 48 + 25 + 1 perf
cargo test desktop                                   8/8
pnpm typecheck                                       OK
pnpm ui:test                                         77/77
pnpm e2e                                             48/48
ci.py calidad                                        OK
ci.py interfaz                                       OK
ci.py producto                                       OK
ci.py seguridad                                      OK
ci.py e2e-tauri                                      OK
ci.py empaquetado                                    OK
```

## 4. Deuda y límites

- No hay aceptación por hunk ni sugerencias inline; el cambio es el documento
  completo.
- La comparación es por líneas y solo contra revisiones que tocaron el documento.
- No se incluye diccionario, tesauro ni gramática propia; el corrector es nativo.
- Las preferencias son por perfil, no por proyecto.
- La lectura TTS real depende del WebView y de las voces instaladas.
- La latencia con 1.000 marcas en una escena sigue siendo una limitación de
  experiencia; la prueba solo detecta degradación extrema.
- Windows, DOCX en Word/LibreOffice y la app instalada quedan en Fase 4/5.
- `App.tsx`, `styles.css` y `project.rs` siguen siendo grandes.

## 5. Decisión

La Fase 3 está cerrada funcionalmente en Linux x86_64. No queda deuda funcional
local del alcance 3.1–3.5; los límites anteriores son decisiones de v1 o
validaciones externas. El siguiente paso es la Fase 4 en una máquina Windows.

## 6. Correcciones de la revisión QA/UX r1

La primera revisión independiente bloqueó la fase. Se corrigieron:

- **P1 pérdida de texto al aceptar/rechazar:** el editor se bloquea mientras
  `procesandoCambio` está activo; aceptar guarda antes la propuesta más reciente
  y relee el sidecar antes de aplicar.
- **P1 panel lateral inaccesible a 720 px/zoom:** el panel derecho se abre como
  drawer con botón, fondo y cierre; las pestañas usan truncado accesible.
- **P1 diff ilegible en 250 px:** el diff se revisa en un modal de ancho
  completo; el panel solo conserva el resumen y las acciones.
- **P1 voz remota:** solo se listan y usan voces `localService = true`; sin voz
  local no se habla y los botones se desactivan.
- **P2 propuestas huérfanas:** mover, eliminar, metadatos, reordenar, escribir
  por la ruta normal y revertir se bloquean mientras haya una propuesta.
- **P2 sidecar no regular:** FIFO, directorios, symlinks y JSON gigantes se
  apartan a cuarentena; la carga del proyecto no espera al panel de cambios.
- **P2 CAS:** el hash base esperado se comprueba también en la preparación y
  antes del reemplazo, no solo en el escaneo inicial.
- **P2 modo propuesto:** se reinicia al cambiar de proyecto; el interruptor
  queda bloqueado mientras hay una propuesta pendiente.
- **P2 modales:** foco inicial, trampa de Tab/Shift+Tab, Escape y bloqueo de
  atajos globales mientras hay un diálogo.
- **P2 foco y anuncios:** foco al editor tras aceptar/rechazar o Quick Open;
  banner de éxito con `role=status`.
- **P2 contraste:** estados vacíos y textos secundarios oscurecidos en claro y
  sepia.
- **P2 demo:** snapshots por revisión para que la comparación no sea ficticia;
  el E2E comprueba una revisión anterior y texto concreto.
- **P3:** `Alt+E`, marcas `nota` en el texto leído y recuento de caracteres
  visible.

## 7. Correcciones de la re-revisión r2

- **P1 texto durante el guardado previo:** aceptar/rechazar activa
  `procesandoCambio` antes del primer `await`; el editor queda deshabilitado y
  `limpiarPendienteDe` solo borra el pendiente del documento aplicado. Prueba
  de regresión en `ui/src/App.test.tsx`.
- **P2 `.escritor/cambios` inválido:** `exigir_sin_cambio_pendiente` y
  `revertir` tratan un almacén de propuestas ilegible como “sin propuesta
  verificable”; el manuscrito se sigue pudiendo guardar. Prueba con symlink en
  `crates/core/tests/proyecto.rs`.
- **P1 instaladores:** `ci.py empaquetado` reconstruyó `.deb` y `.rpm` desde el
  código corregido; el binario embebe el asset JS actual.
- **P2 foco del drawer:** al abrir se enfoca el cierre; al cerrar vuelve al
  toggle; el historial usa el mismo fondo/cierre; `Escape` en un modal no cierra
  el drawer. E2E ampliados.
- **P3 responsive:** tabs con salto de línea en vez de elipsis, diff con ancho
  mínimo y pruebas de foco/geometría a 720/960 px.
- **P3 documentación:** README, HANDOFF, e2e, empaquetado y conteos
  actualizados a 74 + 48 + 25 + 1, 77 UI y 48 E2E.
