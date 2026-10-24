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
cargo test --workspace --all-targets --locked        74 + 41 + 25 + 1 perf
cargo test desktop                                   8/8
pnpm typecheck                                       OK
pnpm ui:test                                         75/75
pnpm e2e                                             43/43
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
