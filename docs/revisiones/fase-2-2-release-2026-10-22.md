# Bloque 2.2 — Release

- **Commit revisado:** `76b139f`
- **Fecha:** 2026-10-22
- **Ámbito:** empaquetado local Linux x86_64 del Bloque 2.2. **No es un release
  público** y no se autoriza publicar.
- **Decisión:** **aprobado con observaciones** para el hito local; release
  público **bloqueado**.

## 1. Build y paquetes

`python3 scripts/ci.py empaquetado` compila el binario release y genera `.deb` y
`.rpm`; el script verifica estructura, permisos, lanzador, iconos y digests.

| Artefacto | Tamaño | SHA-256 |
|---|---|---|
| `writer_0.2.0_amd64.deb` | 5.258.988 B | `e69855c65fcaa25e71e61310e25438a5a563a3b497085da9d61db3a248605191` |
| `writer-0.2.0-1.x86_64.rpm` | 5.259.577 B | `1c845bbdf3bd7d8d09653bca4292dfa32c57032ae194038e1baef042dd45e362` |

Assets de UI embebidos: `index-D_xucQPF.js` (754.140 B), `index-B14ouRl3.css` (42.449 B) y `window-CT2gOcdX.js` (15.163 B).

## 2. Comprobaciones

- `cargo fmt --check`, `clippy -D warnings` y 94 pruebas Rust: OK.
- 59 pruebas UI y 3 pruebas Tauri: OK.
- 12 flujos E2E de navegador: OK.
- 2 escenarios E2E de escritorio: OK.
- `seguridad`: cargo-deny, gitleaks y árbol versionado sin fugas: OK.
- Los paquetes abren el binario y conservan los assets; no se probó la app
  instalada en una máquina limpia.

## 3. Bloqueos de release público

Siguen abiertos, sin cambios respecto a la Fase 2.1:

1. **Versión:** `0.2.0` se reutiliza con contenido distinto; hace falta `0.2.1`
   o metadata de build.
2. **Firma/updater/AppImage:** no hay firma de paquetes, updater ni AppImage.
3. **Plataformas:** solo Linux x86_64; Windows/macOS sin construir ni probar.
4. **App instalada:** no se instaló el `.deb`/`.rpm` en una máquina limpia.
5. **DOCX en Word/LibreOffice real:** no validado.
6. **CI remoto:** los gates son locales.
7. **SEC21-02:** TOCTOU residual aceptado para el hito local.
8. **E2E de creación con diálogo nativo:** no ejecutado.

## 4. Hallazgos

| ID | Severidad | Hallazgo | Tratamiento |
|---|---|---|---|
| REL22-01 | P3 | Versión `0.2.0` repetida con hashes distintos. | Bloquea release público; no el hito local. |
| REL22-02 | P3 | Paquetes no reproducibles (mtimes/`BUILDTIME`). | Deuda de release. |
| REL22-03 | P3 | Sin firma, updater, AppImage, Windows/macOS ni CI remoto. | Deuda de release. |
| REL22-04 | P4 | Binario sin `strip`/remap documentado. | Deuda heredada. |

No hay P1/P2 para el hito local: los paquetes se generan y verifican, y el código
congelado pasa los gates.

## 5. Decisión

**Aprobado con observaciones para el hito local Linux x86_64.** El release
público sigue **no autorizado** hasta cerrar los bloqueos de §3 y obtener una
nueva firma de seguridad/release sobre el commit exacto que se publique.

**Firma:** Release engineer del proyecto `writer` — 2026-10-22.
