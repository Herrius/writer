# Iteración 12 — Correcciones de seguridad

- **Fecha:** 2026-10-21
- **Alcance:** SEC-01, SEC-02 y SEC-03 de la revisión de seguridad.
- **Estado:** correcciones aplicadas; pendiente repetir la revisión de seguridad.
- **Informe de origen:** [`../revisiones/fase-2-seguridad-2026-10-21.md`](../revisiones/fase-2-seguridad-2026-10-21.md)

## 1. Resultado de la revisión

Security bloqueó la fase con:

- **SEC-01 (P1):** un `PREPARE` con un enlace simbólico en un subdirectorio de
  `manuscrito/` hacía que la recuperación escribiera fuera del proyecto.
- **SEC-02 (P2):** `.escritor` enlazado se aceptaba; el lock, los blobs o el
  índice podían truncar, escribir o borrar fuera del proyecto.
- **SEC-03 (P3):** el gate `seguridad` no podía quedar verde por falsos
  positivos de `gitleaks` y por `zizmor` sin workflows.

## 2. Correcciones

### Rutas de proyecto

`fsutil::validar_ruta_relativa` rechaza rutas vacías, absolutas, con `\`, `:`,
UNC, `.`/`..`, controles, espacios o puntos finales y nombres reservados de
Windows. `ruta_del_proyecto` exige que todos los componentes existentes sean
reales y rechaza enlaces simbólicos. Los efectos se limitan además a
`manuscrito/` o `notas/` al cargar el log y al preparar una transacción.

### `.escritor` y rutas operativas

`fsutil::validar_escritor` comprueba que `.escritor` sea un directorio real y que
`operaciones.log`, `bloqueo`, `objetos`, `cuarentena`, `indice.sqlite` y sus
sidecars no sean enlaces. Se aplica antes del lock, del log, de los blobs y del
índice, tanto en lectura como en escritura. `Lock`, `BlobStore`, `Log` e
`Indice` repiten la validación como defensa en profundidad.

### Gate de seguridad

- `.gitleaks.toml` documenta la allowlist del falso positivo del fixture E2E.
- `gitleaks dir` escanea una copia temporal de `git ls-files`, sin `target/` ni
  `node_modules/`.
- `zizmor` se omite con un mensaje explícito si no existe `.github/`.

## 3. Pruebas nuevas

- `validar_ruta_relativa` rechaza formas de Windows, controles y nombres
  reservados.
- Un enlace en un ancestro de `manuscrito/` hace fallar la recuperación sin crear
  el archivo externo.
- `.escritor` enlazado hace fallar `lectura` y `reindexar`.
- El gate `seguridad` completo termina en verde.

## 4. Evidencia

```text
cargo test --workspace --all-targets --locked   88/88
python scripts/ci.py calidad                    OK
python scripts/ci.py seguridad                  OK
```

## 5. Pendiente

- Repetir security review sobre el commit congelado.
- Release engineer, UX/accesibilidad/rendimiento y product owner.
- Probar las rutas de Windows en Windows antes del release.
