# CI y gates

El CI de `writer` cubre, además de la calidad del código, la interfaz, el E2E de
navegador, el empaquetado, el producto y la seguridad:

1. **Interfaz**: que el workspace pnpm instale reproduciblemente, que TypeScript
   compile, que Vite genere la UI y que la cáscara Tauri compile.
2. **E2E**: que los flujos principales de la demo funcionen en Chromium.
3. **Empaquetado**: que Tauri genere el binario y los instaladores de la
   plataforma.
4. **Producto**: que se cumpla el contrato del esquema y que las características
   prometidas sigan funcionando (fixtures, determinismo, hallazgos esperados,
   rendimiento y documentación).
5. **Seguridad**: dependencias, licencias, orígenes, secretos y los propios
   workflows de GitHub Actions.

La regla es la misma que en el resto del proyecto: **si no se puede ejecutar en
local, no es un gate**. Todo se orquesta desde `scripts/ci.py`.

```bash
python scripts/ci.py calidad
python scripts/ci.py interfaz
python scripts/ci.py e2e
python scripts/ci.py e2e-tauri
python scripts/ci.py empaquetado
python scripts/ci.py producto
python scripts/ci.py seguridad
python scripts/ci.py todo
```

## 1. Grupos

### Calidad (`calidad`)

| Comprobación | Qué detecta |
|---|---|
| `cargo fmt --all --check` | Formato divergente. |
| `cargo clippy --workspace --all-targets --all-features` con `-D warnings` | Lints de corrección, estilo, complejidad y rendimiento; incluye `clippy::unwrap_used` y `dbg_macro` por política del workspace. |
| `cargo test --workspace --all-targets --locked` | Tests unitarios y de contrato de la CLI, sin resolver dependencias nuevas. Incluye secuencias fallo → recuperación → reapertura, `ABORT` terminal, corrupción de checksum/blob/página SQLite, fallo de `fsync` después del reemplazo/borrado y reversión reparadora. |

`unsafe_code = "forbid"` en el workspace: la única forma de introducir `unsafe`
es cambiar la política, y eso se revisa.

La matriz prevista es **Windows, Linux y macOS**; el código manipula rutas,
locks y `fsync`, así que la matriz no es decorativa. Hoy el pipeline se ejecuta
en local sobre Linux y el CI remoto aún no está publicado.

### Interfaz (`interfaz`)

| Comprobación | Qué detecta |
|---|---|
| `pnpm install --frozen-lockfile` | Que el lockfile pnpm esté completo y no se resuelvan versiones nuevas. |
| `pnpm typecheck` | Errores de TypeScript en la UI. |
| `pnpm ui:test` | Corpus golden de Markdown, compatibilidad, autosave, búsqueda, historial, DOCX, cuarentena y marcas. |
| `pnpm ui:build` | Que Vite y Tiptap generen la UI de producción. |
| `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | Que la cáscara Tauri y los comandos compilen con su `Cargo.lock`. |
| `cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked` | Exportación de tipos y comandos básicos contra un proyecto temporal. |

Requiere **pnpm 12** y, en Linux, las bibliotecas de Tauri (`webkit2gtk-4.1`,
`gtk3`, `libappindicator-gtk3`, `librsvg`, `patchelf`). `npm` y `npx` no se usan.

### E2E (`e2e` y `e2e-tauri`)

| Comprobación | Qué detecta |
|---|---|
| `pnpm e2e` | 48 flujos de la demo en Chromium: autosave, búsqueda, historial, cambios, preferencias, paleta, TTS simulado, cuarentena, DOCX y responsive/zoom. |
| `pnpm e2e:tauri` | 2 escenarios reales de escritorio en Linux: `Ctrl+S` con verificación en disco y cierre de ventana con cambios pendientes. |

Requiere Chromium de Playwright (`pnpm e2e:install`) y, para Tauri,
`tauri-driver` 2.0.6 + `WebKitWebDriver` + `xdotool` en Linux. Detalle:
[`e2e.md`](e2e.md).

### Empaquetado (`empaquetado`)

| Comprobación | Qué detecta |
|---|---|
| `pnpm build` y verificación | Que Tauri compile y que el `.deb`/`.rpm` contenga binario ejecutable, lanzador, iconos y checksums/digests correctos. |

En Linux genera `.deb` y `.rpm`; AppImage queda pendiente por `linuxdeploy`.
Windows/macOS requieren sus toolchains. Detalle:
[`empaquetado.md`](empaquetado.md).

### Producto (`producto`)

| Comprobación | Qué detecta |
|---|---|
| `--autoprueba` | Casos negativos, positivos y de tolerancia del verificador (BOM, CRLF, comillas, `#`, líneas en blanco, IDs, nota → escena). |
| `--verificar-docs docs/esquema-v1.md` | Que los ejemplos del contrato sigan siendo válidos: ULIDs, hashes `blake3:`, bloques YAML/Markdown/JSON. |
| `--verificar fixtures/proyecto-pequeno` | Integridad del fixture versionado. |
| **Determinismo** | Regenera el fixture pequeño en un directorio temporal y exige árbol y bytes idénticos a los versionados. Detecta cambios accidentales del generador o ediciones manuales del fixture. |
| **Hallazgos esperados** | `fixtures/proyecto-roto/` contiene 12 defectos deliberados y `esperados.txt` fija el contrato: `--json` debe devolver exactamente esos errores, ni uno más ni uno menos. Es la prueba de regresión negativa de `verify`. |
| **Rendimiento** | Genera y verifica 1.000 escenas; presupuesto por defecto 60 s (`--presupuesto`). Mide el generador/verificador de referencia en Python; las pruebas Rust de rendimiento con `criterion` siguen pendientes. |
| **CLI Rust** | `writer verify` sobre el fixture pequeño (debe salir 0) y sobre el roto (debe devolver exactamente los 12 hallazgos). Es la paridad entre el verificador Python y el motor Rust. |
| **Enlaces** | Los enlaces relativos de todos los `.md` del repo apuntan a archivos existentes. |

Se ejecuta en **Windows y Linux** (macOS se cubre en calidad): los separadores de
ruta y la codificación son parte del producto.

### Seguridad (`seguridad`)

| Comprobación | Qué detecta | Herramienta |
|---|---|---|
| Dependencias | Advisories (RustSec), versiones yanked, licencias fuera de la lista blanca, dependencias con comodín, registros y repos Git no permitidos. Se revisan el workspace Rust y la cáscara Tauri (`apps/desktop/src-tauri`). | `cargo-deny` (`deny.toml`) |
| Secretos | Claves, tokens y credenciales en el historial de Git y en el árbol versionado. El escaneo de árbol copia solo `git ls-files` a un temporal, sin `target/` ni `node_modules/`. | `gitleaks` |
| Workflows | Acciones sin fijar por hash, permisos excesivos, credenciales persistidas en artefactos, buenas prácticas de Dependabot. Si no existe `.github/`, el gate lo indica y omite `zizmor` en vez de fallar por ausencia de entradas. | `zizmor` |

Política de `deny.toml`:

- **Advisories**: `yanked = "deny"`, sin excepciones salvo la documentada en
  `deny.toml`: `RUSTSEC-2024-0370` (`proc-macro-error` unmaintained) llega por
  gtk-rs/Tauri en Linux y no tiene reemplazo compatible en Tauri 2.12. Añadir
  otra excepción exige motivo escrito en el propio archivo.
- **Licencias**: lista blanca permisiva (MIT, Apache-2.0, BSD, ISC, MPL-2.0,
  Unicode-3.0, Zlib, CC0). Una dependencia con otra licencia rompe el gate.
- **Bans**: `wildcards = "deny"` (por eso las dependencias internas llevan
  `version` además de `path`).
- **Sources**: solo crates.io. Nada de Git arbitrario ni registros alternativos.

Endurecimiento previsto del CI remoto (aún no hay `.github/`; hoy estas reglas
son un objetivo documentado):

- Acciones fijadas por **SHA** con el tag en comentario.
- `permissions: contents: read` y `persist-credentials: false`.
- `concurrency` para cancelar ejecuciones obsoletas.
- Dependabot semanal con **cooldown de 7 días** (evita adoptar una versión
  comprometida en la primera hora) para Cargo y GitHub Actions.
- `RUSTFLAGS=-D warnings` en todos los jobs.

## 2. Equivalencia local ↔ CI

> El repositorio aún no tiene `.github/` ni remoto. La columna «CI» describe los
> jobs previstos; la evidencia actual es la ejecución local en Linux.

| Local (`scripts/ci.py`) | CI |
|---|---|
| `calidad`: fmt, clippy, test | Job `calidad` en 3 sistemas |
| `interfaz`: pnpm, tipos, corpus, autosave, búsqueda, historial, Vite y Tauri | Job `interfaz` en Linux (y build de escritorio en la matriz) |
| `e2e`: Playwright sobre la demo | Job `e2e` en Linux con Chromium |
| `e2e-tauri`: WebDriver sobre la app real | Job `e2e-tauri` en Linux con WebKitWebDriver |
| `empaquetado`: Tauri release + instaladores | Job `empaquetado` en la matriz |
| `producto`: esquema y características | Job `producto` en Windows y Linux |
| `seguridad`: cargo-deny, gitleaks, zizmor | Job `seguridad` con `cargo-deny-action`, `gitleaks-action` y `zizmor-action` |
| — | Job `msrv`: `cargo check` con Rust 1.89 para el núcleo; la cáscara Tauri declara 1.90 |

El job de seguridad usa las acciones oficiales porque instalan y cachean las
herramientas con su propia verificación; `ci.py seguridad` ejecuta los mismos
binarios y la misma configuración (`deny.toml`).

Instalación local de las herramientas:

```bash
cargo install cargo-deny --locked
winget install Gitleaks.Gitleaks   # Windows
winget install zizmor.zizmor       # Windows
brew install gitleaks zizmor       # macOS
```

## 3. Qué se añadirá en fases siguientes

- **Fuzzing** del parser Rust de frontmatter y del log con `cargo-fuzz`; la
  referencia Python sigue siendo la especificación ejecutable.
- **Mutation testing** (`cargo-mutants`) sobre el motor, ya existe.
- **Rendimiento del motor** con `criterion`: apertura, escritura y búsqueda de
  1.000 escenas. Hoy el presupuesto mide el generador Python.
- **CodeQL**: útil y gratuito en repos públicos; en privados requiere GitHub
  Advanced Security. Se activará al publicar el repositorio.
- **SBOM** (`cargo cyclonedx`) y firma de artefactos cuando haya releases.
- **Dependency review** en pull requests: requiere GHAS en repos privados.

La inyección de fallos (`WRITER_FAULT=prepare|after_write:N|commit|before_replace|before_check`)
ya forma parte de `cargo test`: las pruebas de CLI lanzan procesos hijos, cortan
con código 70 y comprueban la recuperación en la siguiente apertura.

## 4. Cómo añadir un gate

1. Añadir la comprobación a `scripts/ci.py` en el grupo que corresponda.
2. Ejecutarla en local y dejarla en verde.
3. Si necesita una herramienta nueva, añadirla a `HERRAMIENTAS` con instrucciones
   de instalación y al job de CI con su acción fijada por SHA.
4. Documentarla en la tabla de este archivo.

Un gate que no falla cuando debe no es un gate: cada comprobación nueva debería
tener su caso negativo (como `fixtures/proyecto-roto/`).
