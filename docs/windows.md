# Windows — build, pruebas e instaladores

- **Estado:** build, tests y E2E nativo en GitHub Actions (`windows-latest`).
  La prueba en una máquina Windows limpia y la firma siguen pendientes.
- **Objetivo:** paridad funcional con Linux x86_64 para la app de escritorio.

## 1. Requisitos

- Windows 10/11 x64.
- Rust estable con el toolchain MSVC y Visual Studio Build Tools con el
  workload **Desarrollo para el escritorio con C++**.
- Node 22+ y pnpm 12 (`npm`/`npx` no se usan).
- WebView2 Evergreen. Windows 10/11 lo incluye; el instalador puede descargar
  el bootstrapper si falta.
- Para el E2E nativo: Playwright (ya instalado por pnpm) y WebView2. La build
  de prueba habilita CDP en `tauri.e2e.conf.json`; no hacen falta
  `tauri-driver` ni `msedgedriver`.

## 2. Instaladores

```powershell
pnpm install --frozen-lockfile
pnpm build
```

Tauri genera, según `tauri.windows.conf.json`:

```text
apps/desktop/src-tauri/target/release/bundle/msi/writer_0.2.0_x64_en-US.msi
apps/desktop/src-tauri/target/release/bundle/nsis/writer_0.2.0_x64-setup.exe
```

Los paquetes no están firmados. El `.exe` de NSIS es el instalador recomendado
para pruebas; el `.msi` sirve para despliegues gestionados.

## 3. Pruebas

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --locked
pnpm typecheck
pnpm ui:test
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
```

El E2E de navegador se ejecuta con `pnpm e2e`. El E2E nativo de la app Tauri
usa CDP directo sobre WebView2 y Playwright:

```powershell
pwsh -File scripts/e2e-tauri.ps1
```

El script compila la app debug con `tauri.e2e.conf.json`, arranca WebView2 con
`--remote-debugging-port=9222`, conecta Playwright y prueba dos escenarios
reales: `Ctrl+S` con verificación del archivo en disco y cierre de ventana con
cambios pendientes (`WM_CLOSE`, guardado y salida del proceso).

## 4. Paridad cubierta

- Arrastrar y soltar escenas dentro del árbol (HTML5, no el drop nativo de
  WebView2).
- Importación de Markdown con BOM/CRLF y frontmatter.
- Nombres de archivo/carpeta reservados o inválidos de Windows (`CON`, `NUL`,
  `CLOCK$`, `?`, `*`, `:`, puntos/espacios finales, componentes de más de 255
  unidades UTF-16).
- Extensiones `.MD`, `.JSON` y `.JSONL` sin depender del caso.
- Rutas canónicas de la papelera sin el prefijo verbatim `\\?\`.
- Exportaciones con nombres de archivo seguros.
- Instaladores MSI/NSIS y tests del motor y de la UI en Windows.

## 5. Límites conocidos

- Enlaces duros en rutas operativas: la defensa específica sigue siendo
  `#[cfg(unix)]`; en Windows quedan los symlinks/junctions, pero no se detecta
  `nlink`. Es una deuda de seguridad, no un fallo de uso normal.
- `fsync` del directorio es un no-op en Windows; la escritura del archivo sí se
  sincroniza. La durabilidad ante corte de energía es menor que en POSIX.
- Rutas totales >260 caracteres no están garantizadas sin long-path opt-in.
- El instalador no firma binarios ni configura updater.
- La app no implementa asociaciones de archivo ni instancia única; tampoco lo
  hace Linux, así que no es una diferencia de paridad actual.
- La prueba en una máquina limpia (sin Rust/Node/WebView2) y la instalación
  real desde el MSI/NSIS quedan para la validación final.
- La lectura TTS y el portapapeles dependen de WebView2 y no se prueban en el
  E2E nativo.
