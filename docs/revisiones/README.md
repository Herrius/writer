# Revisiones por roles

Registro del pipeline secuencial de la §9 del plan. Cada fase se revisa sobre un
**commit congelado** y no se considera aprobada hasta la firma del product owner.

## Reglas

1. **Separación de funciones:** el builder implementa y corrige; los revisores
   informan. Ningún revisor edita código ni tests del producto.
2. **Commit congelado:** cada revisor recibe un hash concreto. El informe indica
   el hash y comprueba que el código revisado no cambió durante la revisión.
3. **Contexto independiente:** el revisor recibe el artefacto, los contratos y
   los criterios; no recibe la justificación del builder.
4. **Pruebas propias:** el revisor ejecuta los gates y añade sondas, casos
   negativos y escenarios propios fuera del árbol de fuentes (por ejemplo, en
   `/tmp`). Si propone una prueba permanente, queda como recomendación para el
   builder.
5. **Sin edición de código:** un hallazgo se corrige por el builder y se vuelve
   a revisar. El revisor no arregla lo que encuentra.
6. **Secuencialidad:** QA → seguridad → release → UX/accesibilidad/rendimiento →
   product owner. Un hallazgo P1/P2 bloquea el avance; tras la corrección se
   repite el rol afectado y los posteriores.
7. **Trazabilidad:** cada informe sigue la plantilla de §9 del plan: commit,
   entorno, comandos, hallazgos con severidad y reproducción, decisión.
8. **Cierre:** sin firma del product owner la fase está “completada por el
   builder”, no “aprobada”.

## Nombres

```text
fase-2-qa-2026-10-21.md
fase-2-seguridad-2026-10-21.md
fase-2-release-2026-10-21.md
fase-2-ux-2026-10-21.md
fase-2-product-owner-2026-10-21.md
```

Si un rol se repite tras una corrección, se añade `-r2`, `-r3`, etc. El acta de
congelación es `fase-2-00-congelado.md`.

## Severidades

| Nivel | Significado | Efecto |
|---|---|---|
| P1 | Pérdida de datos, seguridad explotable o bloqueo total | Bloquea |
| P2 | Comportamiento incorrecto relevante o garantía incumplida | Bloquea |
| P3 | Defecto menor, documentación imprecisa o deuda acotada | No bloquea; se registra |
| P4 | Observación o mejora | No bloquea |

## Decisiones

- `aprobado`
- `aprobado con observaciones` (solo P3/P4)
- `bloqueado` (algún P1/P2 abierto)

## Estado

| Rol | Informe | Decisión |
|---|---|---|
| Congelación | [`fase-2-00-congelado.md`](fase-2-00-congelado.md) | — |
| QA independiente (r1) | [`fase-2-qa-2026-10-21.md`](fase-2-qa-2026-10-21.md) | bloqueado |
| Congelación r2 | [`fase-2-00-congelado-r2.md`](fase-2-00-congelado-r2.md) | — |
| QA independiente (r2) | [`fase-2-qa-2026-10-21-r2.md`](fase-2-qa-2026-10-21-r2.md) | bloqueado |
| Congelación r3 | [`fase-2-00-congelado-r3.md`](fase-2-00-congelado-r3.md) | — |
| QA independiente (r3) | [`fase-2-qa-2026-10-21-r3.md`](fase-2-qa-2026-10-21-r3.md) | aprobado con observaciones |
| Congelación r4 (seguridad) | [`fase-2-00-congelado-r4.md`](fase-2-00-congelado-r4.md) | — |
| Security reviewer (r1) | [`fase-2-seguridad-2026-10-21.md`](fase-2-seguridad-2026-10-21.md) | bloqueado |
| Congelación r5 (seguridad) | [`fase-2-00-congelado-r5.md`](fase-2-00-congelado-r5.md) | — |
| Security reviewer (r2) | [`fase-2-seguridad-2026-10-21-r2.md`](fase-2-seguridad-2026-10-21-r2.md) | bloqueado |
| Congelación r6 (seguridad) | [`fase-2-00-congelado-r6.md`](fase-2-00-congelado-r6.md) | — |
| Security reviewer (r3) | [`fase-2-seguridad-2026-10-21-r3.md`](fase-2-seguridad-2026-10-21-r3.md) | aprobado con observaciones |
| Congelación r7 (release) | [`fase-2-00-congelado-r7.md`](fase-2-00-congelado-r7.md) | — |
| Release engineer | [`fase-2-release-2026-10-21.md`](fase-2-release-2026-10-21.md) | aprobado con observaciones |
| Congelación r8 (UX) | [`fase-2-00-congelado-r8.md`](fase-2-00-congelado-r8.md) | — |
| UX / accesibilidad / rendimiento (r1) | [`fase-2-ux-2026-10-21.md`](fase-2-ux-2026-10-21.md) | bloqueado |
| Congelación r9 (UX) | [`fase-2-00-congelado-r9.md`](fase-2-00-congelado-r9.md) | — |
| UX / accesibilidad / rendimiento (r2) | [`fase-2-ux-2026-10-21-r2.md`](fase-2-ux-2026-10-21-r2.md) | bloqueado por UX-06 |
| Congelación r10 (UX) | [`fase-2-00-congelado-r10.md`](fase-2-00-congelado-r10.md) | — |
| UX / accesibilidad / rendimiento (r3) | [`fase-2-ux-2026-10-21-r3.md`](fase-2-ux-2026-10-21-r3.md) | bloqueado por UX-28 |
| Congelación r11 (UX) | [`fase-2-00-congelado-r11.md`](fase-2-00-congelado-r11.md) | — |
| UX / accesibilidad / rendimiento (r4) | [`fase-2-ux-2026-10-21-r4.md`](fase-2-ux-2026-10-21-r4.md) | bloqueado por UX-28 |
| Congelación r12 (UX) | [`fase-2-00-congelado-r12.md`](fase-2-00-congelado-r12.md) | — |
| UX / accesibilidad / rendimiento (r5) | [`fase-2-ux-2026-10-21-r5.md`](fase-2-ux-2026-10-21-r5.md) | aprobado |
| Congelación r13 (release UX) | [`fase-2-00-congelado-r13.md`](fase-2-00-congelado-r13.md) | — |
| Release engineer (r2) | [`fase-2-release-2026-10-21-r2.md`](fase-2-release-2026-10-21-r2.md) | aprobado con observaciones |
| Congelación r14 (product owner) | [`fase-2-00-congelado-r14.md`](fase-2-00-congelado-r14.md) | — |
| Product owner | [`fase-2-product-owner-2026-10-21.md`](fase-2-product-owner-2026-10-21.md) | aprobado con observaciones (Fase 2 cerrada; release público no autorizado) |
| **Fase 2.1 — congelación** | [`fase-2-1-00-congelado.md`](fase-2-1-00-congelado.md) | — |
| **Fase 2.1 — QA** | [`fase-2-1-qa-2026-10-21.md`](fase-2-1-qa-2026-10-21.md) | aprobado con observaciones |
| **Fase 2.1 — congelación seguridad** | [`fase-2-1-10-congelado-seguridad.md`](fase-2-1-10-congelado-seguridad.md) | — |
| **Fase 2.1 — seguridad (r1)** | [`fase-2-1-seguridad-2026-10-21.md`](fase-2-1-seguridad-2026-10-21.md) | bloqueado |
| **Fase 2.1 — congelación seguridad r2** | [`fase-2-1-11-congelado-seguridad-r2.md`](fase-2-1-11-congelado-seguridad-r2.md) | — |
| **Fase 2.1 — seguridad (r2)** | [`fase-2-1-seguridad-2026-10-21-r2.md`](fase-2-1-seguridad-2026-10-21-r2.md) | aprobado con observaciones |
| **Fase 2.1 — congelación seguridad r3** | [`fase-2-1-12-congelado-seguridad-r3.md`](fase-2-1-12-congelado-seguridad-r3.md) | — |
| **Fase 2.1 — seguridad (r3)** | [`fase-2-1-seguridad-2026-10-21-r3.md`](fase-2-1-seguridad-2026-10-21-r3.md) | aprobado con observaciones |
| **Fase 2.1 — congelación release** | [`fase-2-1-20-congelado-release.md`](fase-2-1-20-congelado-release.md) | — |
| **Fase 2.1 — release** | [`fase-2-1-release-2026-10-21.md`](fase-2-1-release-2026-10-21.md) | aprobado con observaciones |
| **Fase 2.1 — congelación UX** | [`fase-2-1-30-congelado-ux.md`](fase-2-1-30-congelado-ux.md) | — |
| **Fase 2.1 — UX** | [`fase-2-1-ux-2026-10-21.md`](fase-2-1-ux-2026-10-21.md) | aprobado con observaciones |
| **Fase 2.1 — congelación product owner** | [`fase-2-1-40-congelado-product-owner.md`](fase-2-1-40-congelado-product-owner.md) | — |
| **Fase 2.1 — product owner** | [`fase-2-1-product-owner-2026-10-21.md`](fase-2-1-product-owner-2026-10-21.md) | aprobado con observaciones (Fase 2.1 cerrada; Bloque 2.2 autorizado) |
