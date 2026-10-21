# Fase 2 — Acta de congelación r7 (release)

- **Fecha:** 2026-10-21
- **Commit de código congelado:** `b336467726373ea863eb960796f817ae7b8d0b48`
- **HEAD del pipeline:** `2297d4ace659d1c3bb2c7a79f4d545c38b8d62e6`
- **QA:** r3 aprobada con observaciones (0 P1/P2).
- **Seguridad:** r3 aprobada con observaciones (0 P1/P2).
- **Estado del árbol:** limpio al congelar.

## Alcance del release

- Linux x86_64, build local; deb/rpm generados por Tauri.
- Validación estructural de paquetes, lanzador, iconos, permisos y checksums.
- Reproducibilidad del lockfile y del build dentro de lo posible en local.
- E2E de escritorio y empaquetado deben ejecutarse en segundo plano.

## Observaciones aceptadas que el release debe registrar

- AppImage, firma/notarización, updater, Windows y macOS no están disponibles en
  este entorno.
- P3 de seguridad: FIFO en blobs/journal de SQLite, enlaces duros en Windows,
  límites de tamaño y TOCTOU residual.
- P3 de UX: latencia con 1.000 marcas.
- No hay CI remoto ni `.github/`.

## Evidencia previa

| Gate | Resultado |
|---|---|
| Rust | 92 pruebas verdes |
| QA r3 | gates calidad, interfaz, producto, e2e, e2e-tauri, empaquetado |
| Seguridad r3 | cargo-deny, gitleaks, zizmor omitido sin workflows |

El release engineer debe ejecutar de nuevo el empaquetado sobre este commit y
verificar los artefactos; no se le pide arreglar código.
