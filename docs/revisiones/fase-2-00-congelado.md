# Fase 2 — Acta de congelación para el pipeline

- **Fecha:** 2026-10-21
- **Commit congelado:** `0cfe2af7e064ec7cfec0c83ab976109be8302f61`
- **Árbol:** `76a423903310d43c68e4ce8f388a82dbb9f1687a`
- **Rama:** `main`
- **Estado del árbol:** limpio (`git status --porcelain` vacío)
- **Alcance:** Fase 2 completa del builder (interfaz Tauri v0.2), pendiente solo
  el pipeline de §9 y la validación externa.

## Entorno de referencia

```text
Linux enrique-ms7d43 7.2.9-arch1-1 x86_64
Rust 1.99.1 (2026-10-13)
pnpm 12.14.0
Node 26.11.1
Python 3.14.8
tauri-driver 2.0.6
WebKitWebDriver presente
Chromium de Playwright presente
rpm, ar y cargo-deny presentes
gitleaks y zizmor ausentes en el momento de la congelación
```

## Evidencia del builder sobre el commit congelado

| Gate | Resultado |
|---|---|
| `python3 scripts/ci.py calidad` | OK — 84 pruebas Rust (33 unitarias + 26 integración + 25 CLI) |
| `python3 scripts/ci.py interfaz` | OK — typecheck, 50 pruebas de UI, build Vite, 2 pruebas Tauri |
| `python3 scripts/ci.py producto` | OK — autopruebas, esquema, determinismo, 12 hallazgos, 1.000 escenas, enlaces, CLI |
| `python3 scripts/ci.py e2e` | OK — 7/7 flujos Chromium |
| `python3 scripts/ci.py e2e-tauri` | OK — 1/1 flujo de escritorio Linux |
| `python3 scripts/ci.py empaquetado` | OK — deb y rpm con estructura, permisos y checksums |
| `python3 scripts/ci.py seguridad` | No ejecutado: faltan `gitleaks` y `zizmor`; `cargo-deny` presente |

Artefactos del empaquetado:

```text
apps/desktop/src-tauri/target/release/bundle/deb/writer_0.2.0_amd64.deb
apps/desktop/src-tauri/target/release/bundle/rpm/writer-0.2.0-1.x86_64.rpm
```

## Criterios de aceptación de la fase

Según el plan: criterios 1, 5, 6 y 14 (DOCX) de la definición, más la aprobación
secuencial de QA, seguridad, release, UX/accesibilidad/rendimiento y product
owner.

## Limitaciones declaradas que los revisores deben contrastar

- AppImage, firma/notarización, updater, Windows y macOS no están validados.
- El E2E de escritorio es Linux-only y no cubre diálogos nativos ni la app
  instalada.
- No hay CI remoto ni `.github/`.
- `gitleaks` y `zizmor` no se ejecutaron en la congelación.
- La ventana CAS no es atómica y el historial no se compacta.
- El DOCX es básico y no incluye tablas nativas, TOC ni imágenes.
