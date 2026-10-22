# Bloque 2.2 — Seguridad

- **Commit revisado:** `76b139f`
- **Fecha:** 2026-10-22
- **Alcance:** rutas de creación de proyectos, rollback y validación de destinos.
  No se tocaron autenticación, red, MCP ni IA.
- **Decisión:** **aprobado con observaciones** — P1: 0 · P2: 0 · P3: 1 · P4: 2.
- **Release público:** no autorizado.

## 1. Cambios revisados

### Validación antes de escribir

`Project::init` ahora construye y valida el manifiesto (`Proyecto::nuevo`) antes de
`preparar_raiz_inicial`. Un título vacío, un autor con caracteres de control o un
idioma inválido fallan sin crear la carpeta. La prueba
`init_rechaza_rutas_invalidas_y_destinos_no_vacios` comprueba que no queda una
carpeta vacía.

### Proyectos anidados

`preparar_raiz_inicial` recorre los ancestros canónicos del padre y rechaza crear
un proyecto si encuentra `proyecto.yml` o `.escritor`. Esto evita que
`writer init /tmp/padre/manuscrito/hijo` invalide el proyecto padre. La prueba
Rust cubre el rechazo y comprueba que el destino no existe después.

### Rollback

- `Project::init` limpia la carpeta que creó si falla después de crearla. No toca
  una carpeta vacía preexistente.
- El comando Tauri `crear_proyecto` limpia el proyecto si falla una nota inicial
  o `abrir_proyecto` después de `init`. Si la carpeta preexistía vacía, la
  elimina y la recrea vacía; si no existía, la elimina.
- La prueba Tauri comprueba que un título inválido no deja carpeta.

### SEC21-02 — TOCTOU de `.escritor`

Sigue siendo un riesgo residual P3. La creación/validación de `.escritor` no usa
`openat` ni un directorio temporal con rename atómico; un atacante local con
escritura en la raíz del proyecto podría competir entre la comprobación y el uso.
No hay un P1/P2 reproducible y el escenario exige acceso local al proyecto. Se
acepta formalmente para este bloque; debe cerrarse o aceptarse con firma expresa
antes de un release público.

### Rollback y TOCTOU

El rollback usa `remove_dir_all` sobre la raíz canónica después de comprobar que
existe `proyecto.yml`. Un atacante local que sustituya la ruta entre la
canonicalización y el borrado podría influir en el objetivo; el mismo límite de
acceso local aplica. No se encontró una ruta de ejecución remota ni una pérdida
de datos fuera del proyecto. Se registra como P4 residual.

## 2. Gates

| Gate | Resultado |
|---|---|
| `scripts/ci.py seguridad` | OK — cargo-deny, gitleaks y árbol versionado sin fugas |
| `scripts/ci.py calidad` | OK — 94 pruebas Rust |
| `scripts/ci.py interfaz` | OK — 3 pruebas Tauri, 59 UI |
| `scripts/ci.py e2e` | OK — 12/12 |
| `scripts/ci.py e2e-tauri` | OK — 2 escenarios Linux |

No se añadieron dependencias. `gitleaks` no encontró secretos. `cargo-deny`
mantiene la excepción documentada de `proc-macro-error` en el árbol de Tauri
Linux.

## 3. Hallazgos

### P3

| ID | Hallazgo | Tratamiento |
|---|---|---|
| **SEC22-01 / SEC21-02** | TOCTOU de `.escritor` (validación y uso no atómicos). | Aceptado como riesgo residual hasta la preparación de release; no hay P1/P2. |

### P4

| ID | Hallazgo | Tratamiento |
|---|---|---|
| SEC22-02 | Rollback con `remove_dir_all` y canonicalización posteriores a una comprobación; riesgo local de sustitución. | Aceptado; el atacante necesita escritura en el padre. |
| SEC21-04/05/06 | Rutas señuelo en `localStorage`, nombres Windows y `umask` no cubiertos por pruebas específicas. | Deuda heredada; no se tocó en este bloque. |

No se encontraron nuevas rutas de salida del proyecto, enlaces aceptados,
inyección de rutas ni fugas de datos. La creación sigue rechazando `.`/`..`,
enlaces, destinos no vacíos, nombres reservados y proyectos anidados.

## 4. Decisión

**Aprobado con observaciones.** Los cambios de creación reducen la superficie de
fallo y cubren SEC21-03/QA21-14 con pruebas. SEC21-02 permanece como riesgo
residual P3 y el rollback como P4 local; ninguno bloquea el cierre del Bloque 2.2,
pero ambos deben resolverse o aceptarse formalmente antes de publicar.

**Firma:** Seguridad del proyecto `writer` — 2026-10-22.
