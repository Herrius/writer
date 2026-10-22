# Fase 2 — Acta de congelación r8 (UX)

- **Fecha:** 2026-10-21
- **Commit de código congelado:** `b336467726373ea863eb960796f817ae7b8d0b48`
- **HEAD del pipeline:** `0215293df85b617df58473c52444ab5efcadf1f4`
- **QA:** r3 aprobada con observaciones.
- **Seguridad:** r3 aprobada con observaciones.
- **Release:** aprobado con observaciones (0 P1/P2).
- **Estado del árbol:** limpio al congelar.

## Artefactos del release

| Artefacto | SHA-256 |
|---|---|
| `writer_0.2.0_amd64.deb` | `2fc369947e9310f11b0b87780fc7cf3e99a0599431f4d60aacacf838ff060cc7` |
| `writer-0.2.0-1.x86_64.rpm` | `c28d21cf1226a8acc4a6d323b4e18af908267d3636766268169dfa3bc856520f` |

## Alcance de UX

- Uso real de la interfaz, árbol, editor, búsqueda, historial, cuarentena,
  marcas, exportación y estados de guardado.
- Teclado, foco, lector de pantalla, IME, contraste, textos y estados vacíos.
- Proyectos grandes y latencia; la QA r3 midió 203–225 ms con 1.000 marcas
  frente a 95 ms sin marcas (P3).
- El revisor no debe abrir la app de escritorio ni ventanas: puede usar
  Playwright headless sobre la demo, análisis estático y la evidencia de QA r3.

## Observaciones que UX debe considerar

- Latencia con 1.000 marcas.
- Toolbar y editor sin diálogo de enlace.
- Diálogos nativos y app instalada no probados.
- Documentos con construcciones no soportadas abren en solo lectura.
