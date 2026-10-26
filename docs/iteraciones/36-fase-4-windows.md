# Iteración 36 — Fase 4: Windows en CI

- **Fecha:** 2026-10-26
- **Estado:** build, tests, E2E nativo e instaladores en GitHub Actions;
  pendiente la prueba en máquina limpia y la firma.
- **Objetivo:** que Windows funcione igual que Linux x86_64 para la app de
  escritorio, no solo que compile.

## 1. Qué se corrigió

### Compatibilidad funcional

- `dragDropEnabled: false` en la ventana Tauri: sin esto, WebView2 intercepta
  el drag-and-drop nativo y el reordenamiento HTML5 de escenas no funciona en
  Windows.
- `devUrl` y la CSP de desarrollo usan `127.0.0.1` en vez de `localhost`, que
  en Windows puede resolver a IPv6 y no conectar con Vite.
- `tauri.windows.conf.json` genera MSI y NSIS; Linux mantiene deb/rpm.
- Importación Markdown y plantillas normalizan BOM/CRLF antes de quitar el
  frontmatter. El parser de documentos también tolera BOM/CRLF en blobs
  históricos.
- Validación única de componentes para Linux y Windows: caracteres `?*"<>|`,
  nombres de dispositivo (`CON`, `CLOCK$`, `COM¹`…), puntos/espacios finales y
  componentes de más de 255 unidades UTF-16.
- Extensiones `.md`, `.json` y `.jsonl` se aceptan sin depender del caso.
- La papelera guarda y devuelve rutas canónicas sin el prefijo `\\?\` mediante
  `dunce`; los tests comparan rutas canónicas.
- Los nombres de exportación y los slugs de proyecto/nota evitan nombres
  reservados de Windows.
- La lectura en voz alta comprueba también `SpeechSynthesisUtterance`, no solo
  `speechSynthesis`.
- La UI empaqueta Inter (Fontsource) para que las métricas de las tarjetas no
  dependan de la fuente instalada en el sistema; la fecha de las tarjetas usa
  un tamaño que no desborda en Linux ni Windows.

### CI y pruebas

- Matriz `ubuntu-22.04` + `windows-latest` para `fmt`, `clippy`, tests Rust,
  tipos/tests de UI, tests de la cáscara Tauri e instaladores.
- Gate `producto` en ambos sistemas.
- E2E de navegador Playwright en ambos sistemas.
- E2E nativo de Windows con `tauri-driver` + `msedgedriver`: `Ctrl+S` y cierre
  de ventana con `WM_CLOSE`.
- Workflow de release por tag `v*` que adjunta deb/rpm/MSI/NSIS a un borrador.
- Acciones fijadas por SHA, permisos mínimos y `persist-credentials: false`.

## 2. Evidencia

La ejecución de CI debe quedar verde en Linux y Windows antes de considerar
cerrada la iteración. Los gates locales siguen siendo la referencia rápida:

```bash
cargo test --workspace --all-targets --locked
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
pnpm typecheck
pnpm ui:test
pnpm e2e
python3 scripts/ci.py producto
```

## 3. Límites que siguen abiertos

- Enlaces duros en rutas operativas: la defensa específica es `#[cfg(unix)]`;
  en Windows no se lee `nlink`.
- `fsync` del directorio es un no-op en Windows; la durabilidad ante corte de
  energía es menor que en POSIX.
- Rutas totales >260 caracteres no están garantizadas.
- Sin firma de código, updater ni asociaciones de archivo.
- Falta instalar el MSI/NSIS en una máquina Windows 10/11 limpia y validar
  Word/LibreOffice con el DOCX exportado.
- La app instalada, no solo el binario de `target/`, sigue pendiente de prueba.
