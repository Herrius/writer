# Fase 1 — Acta de congelación

- **Fecha:** 2026-10-22
- **Commit de código y tests:** `744aaa5`
- **Tag:** `fase-1-congelado`
- **Rama:** `main`
- **Entorno:** Linux x86_64, Chromium de Playwright, WebKitWebDriver y Tauri
  2.12.4. Windows no se ha probado.
- **Alcance:** rediseño del panel de Proyectos.

## 1. Cambios congelados

| Commit | Feature | Archivos principales |
|---|---|---|
| `a5ed191` | F1.0 contrato visual y línea base | `docs/iteraciones/17-fase-1-rediseno-panel.md`, `docs/README.md` |
| `43adf0e` | F1.1 retirada de meta diaria, Proyectos personales, Ideas y Ayuda | `ui/src/Inicio.tsx`, `ui/src/styles.css`, `ui/src/App.test.tsx`, `docs/inicio-v1.md` |
| `aab2c1b` | F1.2 tarjeta de creación y portadas | `ui/src/Inicio.tsx`, `ui/src/styles.css`, `e2e/fase-1-panel.spec.ts` |
| `50be135` | F1.3 barra derecha a toda altura | `ui/src/Inicio.tsx`, `ui/src/styles.css`, `e2e/fase-1-panel.spec.ts` |
| `744aaa5` | F1.4 QA visual, responsive y teclado | `e2e/fase-1-panel.spec.ts`, `docs/iteraciones/17-fase-1-rediseno-panel.md` |

El commit de congelación incluye el código de UI y las pruebas. El acta y la
actualización de `HANDOFF.md` se commitean después y no cambian el código.

## 2. Criterios de aceptación

| Criterio | Resultado |
|---|---|
| La tarjeta de creación iguala la tarjeta completa, no solo la portada | OK: 155,27 × 297,11 = tarjeta de proyecto a 1440 |
| La portada mantiene 2:3 | OK: 155,27 × 232,89; ratio 0,6667 |
| La tarjeta de creación conserva altura en estado vacío | OK: 297,08 px |
| Meta diaria, Proyectos personales e Ideas desaparecen del panel | OK |
| El botón «Ayuda» desaparece del panel | OK |
| El resto de controles deshabilitados se conserva como excepción | OK, documentado en `inicio-v1.md` |
| La barra derecha ocupa toda la altura | OK a 1440 y 960; apilada a ≤900 |
| El espacio liberado queda reservado sin controles | OK: `.inicio-sidebar-reserva`, `aria-hidden` |
| Sin overflow a 1440, 960, 960/200 % y 700 | OK |
| La vista de lista sigue compacta | OK: crear 64 px, proyecto 89,59 px |
| Teclado: abrir con Enter y recuperar foco | OK |

## 3. Gates ejecutados

```text
python scripts/ci.py interfaz       OK  (13,9 s)
python scripts/ci.py e2e            OK  (19/19, Chromium)
python scripts/ci.py e2e-tauri      OK  (Linux, WebDriver)
python scripts/ci.py empaquetado    OK  (deb/rpm)
python scripts/ci.py producto       OK
python scripts/ci.py seguridad      OK  (gitleaks, cargo-deny; zizmor omitido)
python scripts/ci.py calidad        OK  (94/94 Rust)
pnpm typecheck                       OK
pnpm ui:test                        60/60
pnpm ui:build                       OK
```

## 4. Observaciones del builder

- **P3:** a 960 la reserva lateral acompaña la altura del listado y queda muy
  alta. Es el comportamiento pedido; se reevaluará cuando haya contenido.
- **P4:** el ancho de tarjeta a 1440 es 155,27 px, no 160 px exactos. Se acepta
  como referencia aproximada para no romper la cuadrícula de seis columnas.
- **P4:** el aviso de chunk >500 kB de Vite es previo a esta fase y no bloquea.
- **P4:** la vista apilada (≤900 px) oculta la reserva lateral; es deliberado.

No hay P1/P2 conocidos por el builder. La palabra final la tienen los revisores.

## 5. Decisión

Se congela `744aaa5` para el pipeline formal:

1. QA independiente.
2. Seguridad.
3. Release.
4. UX/accesibilidad/rendimiento.
5. Product owner.

El release público sigue sin autorización. Windows sigue pendiente.
