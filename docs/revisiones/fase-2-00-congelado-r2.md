# Fase 2 — Acta de congelación r2 (correcciones de QA)

- **Fecha:** 2026-10-21
- **Commit congelado:** `06a158105919cdc394a47a5acb827443bdda25f1`
- **Commit anterior revisado por QA:** `0cfe2af7e064ec7cfec0c83ab976109be8302f61`
- **Informe que motivó las correcciones:**
  [`fase-2-qa-2026-10-21.md`](fase-2-qa-2026-10-21.md)
- **Estado del árbol:** limpio al congelar.
- **Alcance:** corrección de los 5 P1, 4 P2 y los P3/P4 accionables de la
  primera revisión QA; endurecimiento del E2E y de la cáscara Tauri.

## Cambios principales

| Área | Corrección |
|---|---|
| Cierre de ventana | `onCloseRequested` guarda el pendiente y solo destruye la ventana si termina bien |
| Compatibilidad | Lexer CommonMark/GFM: tablas e imágenes anidadas, tareas en citas/listas, imágenes de referencia; código ignorado |
| Vallas de código | Valla más larga que cualquier secuencia de backticks del contenido; caso 18 del corpus |
| Autosave | El cuerpo actual del editor se conserva y se vuelve a drenar tras un guardado en vuelo |
| Índice | Frescura por hash/`mtime`/tamaño y exclusión de documentos inválidos/duplicados |
| E2E escritorio | Puerto libre, grupo de procesos, limpieza, `xdotool`, reintentos y dos escenarios separados |
| P3/P4 | Recuento, referencias de marcas, fecha de cuarentena, avisos, resaltado FTS y mensajes de demo |
| Seguridad | CSP restrictiva y hook E2E detrás de la feature `e2e` |

## Evidencia del builder sobre el commit congelado

| Gate | Resultado |
|---|---|
| `python3 scripts/ci.py calidad` | OK — 86 pruebas Rust (33 unitarias + 28 integración + 25 CLI) |
| `python3 scripts/ci.py interfaz` | OK — typecheck, 55 pruebas de UI, build Vite, 2 pruebas Tauri |
| `python3 scripts/ci.py producto` | OK — autopruebas, esquema, determinismo, 12 hallazgos, 1.000 escenas, enlaces, CLI |
| `python3 scripts/ci.py e2e` | OK — 7/7 flujos Chromium |
| `python3 scripts/ci.py e2e-tauri` | OK — escenarios `guardar` y `cerrar`; 5/5 repeticiones sin huérfanos |
| `python3 scripts/ci.py empaquetado` | OK — deb y rpm con estructura, permisos y checksums |
| `cargo-deny` núcleo y desktop | OK |

## Pendiente

- Repetir QA independiente sobre este commit.
- Security reviewer, release engineer, UX/accesibilidad/rendimiento y product
  owner.
- Validación externa: AppImage, firma, Windows/macOS, CI remoto y app instalada.
