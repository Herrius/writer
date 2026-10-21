# Iteración 05 — Fase 2 (búsqueda e historial)

- **Fecha:** 2026-10-21
- **Alcance:** búsqueda FTS5 en CLI y UI, línea de tiempo del proyecto y
  reversión de revisiones desde la interfaz.
- **Estado:** completado en local. Motor, CLI, UI y cáscara Tauri en verde.

## 1. Entregables

### 1.1 Búsqueda en el motor y la CLI

- Nuevo `Project::buscar(consulta, limite)` en `writer-core`.
- Usa FTS5 cuando el proyecto ya tiene `.escritor`; si no lo tiene (fixture de
  solo lectura), construye un índice FTS5 efímero en memoria y **no crea** el
  directorio operativo.
- La consulta se convierte en términos citados para que `"`, `*` o `-` no
  alteren el significado.
- `Project::buscar` no reconstruye el índice si la revisión y los recuentos ya
  coinciden; tras una escritura el índice ya está fresco.
- Nuevo comando CLI:

```bash
writer buscar puerto ~/novela --limite 10
writer buscar puerto ~/novela --json
```

- El JSON devuelve `id`, `ruta`, `tipo`, `titulo` y `fragmento`.

### 1.2 Comandos Tauri

- `buscar(ruta, consulta, limite)`.
- `historial(ruta)`.
- `mostrar(ruta, revision)`.
- `revertir(ruta, revision)`.
- DTOs exportados con `ts-rs`: `BusquedaDto`, `RevisionDto`,
  `ContenidoRevisionDto`, `RevisionDetalleDto`, `ReversionDto`.

### 1.3 Búsqueda en la UI

- Campo en la barra lateral y atajo `Ctrl+F` / `Cmd+F`.
- Debounce de 300 ms; mínimo dos caracteres.
- Mientras hay consulta, el árbol se sustituye por resultados con título, tipo,
  ruta y fragmento.
- El fragmento marca la coincidencia entre `[` y `]`; la UI la muestra con
  `mark`.
- Al pulsar un resultado se abre el documento.
- Estados: buscando, sin resultados y error.

### 1.4 Línea de tiempo e historial

- Nueva pestaña **Historial** en el panel del documento.
- Lista de revisiones de más reciente a más antigua, con fecha, autor, alcance,
  número de archivos y rutas.
- Marca visual de la revisión actual.
- Al seleccionar una revisión se carga su detalle con el contenido completo de
  cada archivo afectado (o “archivo eliminado”).
- Reversión en dos pasos: el primer clic arma la confirmación y el segundo
  ejecuta. Se avisa de que se crea una revisión nueva.
- Antes de revertir se guarda cualquier cambio pendiente.
- Tras revertir se recarga el proyecto conservando el documento activo si sigue
  existiendo; si no, se abre la primera escena.
- Los errores de conflicto o E/S se muestran en el panel de detalle.

### 1.5 Correcciones de robustez

- **Borrador al cambiar de pestaña:** el editor se remonta con el cuerpo
  pendiente (`cuerpoVisible`), no con la última versión guardada. Regresión:
  escribir → Historial → Editor → seguir escribiendo → guardar conserva todo.
- **Reversión sin escritura concurrente:** `revirtiendoRef` bloquea la edición
  desde antes del guardado previo hasta que termina la recarga. La UI muestra un
  aviso y deshabilita las revisiones mientras dura.
- **Respuestas fuera de orden:** `detalleSeq` descarta respuestas antiguas y la
  confirmación de reversión se vincula a `{raiz, revision, opId}`. Una respuesta
  vieja no puede cambiar la revisión que se confirma.
- **Historial sin documentos activos:** el panel con pestañas ya no depende de
  que exista una escena. Si una reversión deja el proyecto vacío, Historial sigue
  disponible para deshacerla.
- **Búsqueda tras guardar:** el efecto depende también de `proyecto.revision`, así
  que repite la consulta y actualiza resultados y fragmentos.
- **Fallback sin `.escritor` con la misma semántica:** se construye un índice
  FTS5 efímero en memoria con los documentos escaneados. Así la tokenización,
  los términos AND, los diacríticos (incluido Unicode descompuesto) y los guiones
  coinciden con el índice persistente. Regresiones: `cancion` encuentra
  `cancio\u{301}n` y `Puerto-Viejo` no encuentra `puertoviejo`.

## 2. Decisiones

| # | Decisión | Motivo |
|---|---|---|
| 1 | Buscar primero en CLI | Regla CLI-first: la UI no estrena una capacidad que la CLI no tiene. |
| 2 | Fallback en memoria sin `.escritor` | Buscar no debe crear el directorio operativo en un proyecto de solo lectura. |
| 3 | Reutilizar el índice si revisión y recuentos coinciden | Evita reconstruir 1.000 escenas en cada consulta. |
| 4 | Reversión en dos pasos y revisión nueva | Evita pérdidas accidentales y conserva el historial. |
| 5 | Recargar el proyecto tras revertir | Una reversión puede afectar varios archivos y cambiar los errores de `verify`. |

## 3. Evidencia

```text
$ cargo test --workspace --all-targets --locked
70/70                                                     # 29 + 18 + 23

$ cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --locked
2/2                                                       # tipos + comandos

$ pnpm ui:test
Test Files  3 passed (3)
Tests       33 passed (33)

$ pnpm typecheck                                          # OK
$ pnpm ui:build                                           # OK
$ pnpm check                                              # OK

$ python scripts/ci.py interfaz
Interfaz: tipos, build de UI, cáscara Tauri y comandos OK # OK
```

La prueba de la CLI `buscar_encuentra_escenas_y_notas` verifica el comando y el
fragmento resaltado. Las pruebas del núcleo
`buscar_en_fixture_sin_escritor_no_crea_directorio` y
`buscar_fallback_coincide_con_fts_en_unicode_y_guiones` protegen el modo de solo
lectura, la paridad con FTS5, el Unicode descompuesto y los guiones. Las pruebas
de UI cubren búsqueda, historial, borrador entre pestañas, bloqueo durante la
reversión, respuestas fuera de orden, proyecto vacío y refresco de búsqueda tras
guardar.

## 4. Limitaciones

- La búsqueda usa el índice FTS5; si se editan archivos por fuera del motor hay
  que ejecutar `reindex` para ver los cambios. El tokenizador `unicode61` ignora
  diacríticos: `cancion` encuentra `canción`.
- El fallback sin `.escritor` reconstruye el índice FTS5 en memoria en cada
  consulta. Es correcto para fixtures o proyectos de solo lectura; un proyecto
  real con miles de escenas conviene que tenga `.escritor` e índice persistente.
- El historial es del proyecto completo, no por documento. El detalle muestra
  los archivos afectados por cada revisión.
- La reversión falla con conflicto si alguno de los archivos afectados cambió
  después de la revisión; el historial no se toca.
- Sin E2E de escritorio; la UI se prueba con Testing Library y API simulada.

## 5. Pendiente

- Exportación DOCX y diálogo de destino (criterio 14).
- Cuarentena visible.
- Marcas propias de Tiptap (personajes, tramas, comentarios).
- Toolbar de formato y recuento de palabras.
- E2E con Playwright/Tauri.
- Empaquetado e instaladores.
- CI remoto y prueba en Windows/macOS.
