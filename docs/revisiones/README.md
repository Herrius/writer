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
| QA independiente (r2) | pendiente | — |
| Security reviewer | pendiente | — |
| Release engineer | pendiente | — |
| UX / accesibilidad / rendimiento | pendiente | — |
| Product owner | pendiente | — |
