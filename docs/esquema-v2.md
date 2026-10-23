# Esquema v2 — extensiones locales

- **Fecha:** 2026-10-23
- **Estado:** propuesta normativa para la Fase 2.0. Pendiente de implementación
  en 2.1–2.6.
- **Base:** [`esquema-v1.md`](esquema-v1.md).
- **Principio:** el manuscrito y las notas siguen siendo Markdown; las
  extensiones locales viven en `.escritor/` y nunca sustituyen a los archivos.

## 1. Compatibilidad

`proyecto.yml` conserva `esquema: 1`. El esquema base no cambia en la Fase 2:
libro → capítulo → escena y `notas/<tipo>/<archivo>.md`.

Las extensiones son **aditivas y opcionales**. Un proyecto v1 sin ellas sigue
abriéndose y editándose igual. Un proyecto con extensiones declara capacidades
en `proyecto.yml`:

```yaml
esquema: 1
markdown: 2
comentarios: 2
metas: 1
planificacion: 1
```

- `markdown: 2` activa la sintaxis de comentarios con ULID y las @menciones de
  notas. Sin este campo, el editor usa el contrato v1.
- `comentarios`, `metas` y `planificacion` declaran la versión de cada extensión.
- Un campo ausente significa “extensión no usada”. No se crean directorios
  vacíos.
- Un campo desconocido se conserva y `verify` lo marca como aviso, igual que en
  v1.
- El valor `esquema: 2` queda reservado para un cambio estructural futuro
  (por ejemplo, el nivel de partes). No se usa en la Fase 2.

La compatibilidad es de lectura y escritura:

- un proyecto v1 se abre sin migración;
- al usar una extensión por primera vez, la app añade solo su campo a
  `proyecto.yml`, conserva el resto y registra una revisión;
- una versión antigua de writer que ignore los campos nuevos sigue pudiendo
  abrir el proyecto, pero no interpretará la sintaxis `markdown: 2`; por eso el
  proyecto debe abrirse con la versión actual.

## 2. Estructura

```text
mi-novela/
  proyecto.yml
  manuscrito/
  notas/
  .escritor/
    indice.sqlite
    objetos/
    operaciones.log
    bloqueo
    cuarentena/
    comentarios/
      <ULID>.json
    metas/
      metas.json
    sesiones/
      2026-10.jsonl
    planificacion/
      plot-grid.json
```

Nada de esto es binario en el manuscrito. Los sidecars son JSON UTF-8, con
claves ordenadas por bytes y sin BOM, para que el diff sea estable.

## 3. Comentarios v2

### 3.1 Ancla en Markdown

```markdown
[[comentario:01K...|revisar el ritmo|—No puede ser— dijo.]]
```

- El primer campo después del tipo es un ULID del comentario.
- El segundo es el texto inicial del hilo.
- El tercero es el texto marcado.
- Los comentarios v1 de dos campos
  (`[[comentario:texto|texto marcado]]`) se conservan como legado de solo
  lectura. No se reescriben automáticamente.
- El texto del comentario no puede contener `|` ni `]]`; el texto marcado no
  puede contener `]]`.
- No se admiten comentarios anidados.

### 3.2 Sidecar

` .escritor/comentarios/<ULID>.json`:

```json
{
  "version": 2,
  "id": "01K...",
  "documento": "manuscrito/01-libro/01-capitulo/001-escena.md",
  "ancla": {
    "texto": "—No puede ser— dijo.",
    "hash": "blake3:...",
    "inicio": 120,
    "fin": 140
  },
  "estado": "abierto",
  "creado": "2026-10-23T10:00:00.000Z",
  "actualizado": "2026-10-23T10:00:00.000Z",
  "autor": "local",
  "huerfano": false,
  "mensajes": [
    {
      "id": "01K...",
      "tipo": "comentario",
      "autor": "local",
      "ts": "2026-10-23T10:00:00.000Z",
      "texto": "revisar el ritmo"
    }
  ]
}
```

Reglas:

- `id` coincide con el ULID del ancla y con el nombre del archivo.
- `ancla.hash` es el hash del texto marcado; los offsets son orientativos.
- `estado`: `abierto` o `resuelto`.
- `autor`: `local` o el autor del proyecto; no hay cuentas.
- `mensajes` es append-only. Editar, responder, resolver o borrar añade un
  mensaje con `tipo` `edicion`, `respuesta`, `resolucion` o `borrado` y
  `reemplaza` cuando corresponda. No se destruye el historial.
- `huerfano` lo calcula la app al abrir o verificar: si el ancla no aparece en
  el documento, se marca `true`; no se borra el hilo.
- `verify` comprueba ULID, documento, hash y coherencia del estado.
- El backup incluye `.escritor/comentarios/`.

### 3.3 Sticky notes

Una sticky note es un comentario sin selección:

```json
{
  "version": 2,
  "id": "01K...",
  "tipo": "sticky",
  "documento": "manuscrito/.../001-escena.md",
  "ancla": null,
  "estado": "abierto",
  "creado": "...",
  "actualizado": "...",
  "autor": "local",
  "huerfano": false,
  "mensajes": []
}
```

Vive en el mismo directorio. No se imprime en DOCX.

## 4. Metas y sesiones v1

### 4.1 Metas

`.escritor/metas/metas.json`:

```json
{
  "version": 1,
  "diaria": {
    "objetivo": 500,
    "desde": "2026-10-23"
  },
  "libro": {
    "objetivo": 80000
  },
  "desafio": {
    "objetivo": 50000,
    "limite": "2026-12-31"
  },
  "archivadas": []
}
```

- Todos los campos son opcionales; un archivo ausente equivale a “sin metas”.
- `objetivo` es un entero ≥ 0; `desde` y `limite` son fechas ISO (`YYYY-MM-DD`).
- Las metas archivadas se guardan en `archivadas` con `tipo`, `objetivo`,
  `desde`, `hasta` y `motivo`.
- El archivo se escribe con reemplazo atómico y `fsync`.
- No se guarda en `proyecto.yml` para no ensuciar el manifiesto.

### 4.2 Sesiones

`.escritor/sesiones/<YYYY-MM>.jsonl`, una línea JSON por sesión:

```json
{"id":"01K...","inicio":"2026-10-23T10:00:00.000Z","fin":"2026-10-23T10:25:00.000Z","preset":25,"pausas":[],"palabras":412,"documento":"manuscrito/.../001-escena.md","autor":"local"}
```

- `preset` ∈ `5`, `10`, `15`, `25`, `30`.
- `pausas` es una lista de `{ "inicio": "...", "fin": "..." }`.
- `palabras` es el delta medido al finalizar; puede ser 0.
- Se hace append con `fsync`; el archivo se rota por mes.
- Las rachas y estadísticas se derivan de las sesiones; no se guardan como
  verdad.
- El backup incluye `.escritor/sesiones/`.

## 5. Planificación v1

### 5.1 Plot Grid

`.escritor/planificacion/plot-grid.json`:

```json
{
  "version": 1,
  "tramas": [
    {
      "id": "01K...",
      "nombre": "El faro",
      "color": "#4d6b8a",
      "puntos": [
        {
          "escena": "01K...",
          "orden": 10,
          "etiqueta": "giro"
        }
      ]
    }
  ]
}
```

- `id` es el ULID de una nota de tipo `tramas`.
- `escena` es el ULID de una escena existente; una referencia desconocida se
  muestra como tal y no se borra.
- `color` es `#rrggbb`.
- `orden` es un entero; sirve para ordenar puntos dentro de la trama.

### 5.2 Notas de planificación

Se añaden dos tipos opcionales a `notas/`:

```text
notas/cuaderno/<archivo>.md   # Story Notes / construcción del mundo
notas/ideas/<archivo>.md      # Ideas locales
```

- El campo `tipo` debe coincidir con la carpeta, como en v1.
- Las notas de personajes admiten campos opcionales:
  `rol: principal|secundario`, `cast: [ULID]` y `alias: [texto]`.
- Las @menciones entre notas usan `[[nota:ULID|texto]]`. Si el ULID no existe,
  la referencia se muestra como desconocida; no se borra.
- Las Ideas pueden convertirse en proyecto local; la conversión copia el
  contenido y genera IDs nuevos, sin historial.

## 6. Plantillas locales v1

Las plantillas viven en el directorio de datos de la app, no en el proyecto:

```text
<app_data_dir>/plantillas/<slug>/
  plantilla.json
  manuscrito/
  notas/
```

`plantilla.json`:

```json
{
  "version": 1,
  "nombre": "Novela en tres actos",
  "descripcion": "Estructura base con tres actos y fichas iniciales.",
  "idioma": "es",
  "estructura": {
    "libros": 1,
    "capitulos_por_libro": 3,
    "escenas_por_capitulo": 1
  },
  "notas_iniciales": ["personajes", "tramas", "lugares"]
}
```

- Crear desde plantilla copia la estructura y las notas, pero genera ULIDs,
  fechas y nombres de archivo nuevos.
- No se copia historial, índice, blobs ni comentarios.
- No hay biblioteca pública, descarga ni plantillas remotas.

## 7. Importación, exportación y backup v2

### 7.1 Importación

- Formatos: DOCX, Markdown, HTML y RTF.
- Prioridad: DOCX y Markdown primero; HTML y RTF después.
- La importación puede ser total o parcial (selección de documentos).
- Todo archivo se valida antes de escribir: tamaño máximo, UTF-8, estructuras
  permitidas y rutas relativas.
- Un archivo inválido va a cuarentena; el proyecto no se toca.
- DOCX/HTML/RTF no pueden introducir HTML crudo, scripts, rutas `..`, enlaces
  simbólicos ni imágenes externas.
- Los límites concretos se fijan en `docs/importacion-v1.md`.

### 7.2 Exportación

- DOCX y Markdown conservan el contrato actual.
- PDF/impresión se genera desde el manuscrito, no desde el editor.
- El formato de manuscrito para editorial se define en `exportacion-v2.md`.
- La exportación no modifica el proyecto.

### 7.3 Backup

- El backup es un ZIP local del proyecto completo:
  `proyecto.yml`, `manuscrito/`, `notas/` y `.escritor/`.
- Se excluyen `bloqueo`, `indice.sqlite` y `cuarentena/` (reconstruibles o
  diagnósticos).
- Se incluyen `objetos/`, `operaciones.log`, `comentarios/`, `metas/`,
  `sesiones/` y `planificacion/`.
- La importación del backup valida el manifiesto, los hashes y las rutas antes
  de escribir; nunca sobrescribe un proyecto abierto.
- El formato y los límites se fijan en `importacion-v1.md`.

## 8. Validación y cuarentena

`verify` debe comprobar:

- que `proyecto.yml` siga siendo válido y que `esquema` sea `1`;
- que los campos de extensión tengan la versión soportada;
- que los ULID de comentarios, metas y planificación sean válidos;
- que los sidecars JSON parseen y tengan las claves obligatorias;
- que las anclas de comentarios apunten a documentos existentes o queden
  marcadas como huérfanas;
- que las referencias de planificación apunten a escenas/notas existentes o se
  reporten como desconocidas;
- que las sesiones tengan fechas, preset y palabras válidos.

Un sidecar inválido no se corrige en silencio: se aparta a
`.escritor/cuarentena/` con un motivo y el proyecto sigue operativo. Los
archivos del manuscrito nunca se tocan por un sidecar inválido.

## 9. Seguridad

- Los sidecars son locales y no se sirven por red.
- Los límites de tamaño se aplican antes de parsear JSON o DOCX.
- Los nombres de archivo se validan con las mismas reglas que v1 (sin `..`,
  rutas absolutas, controles ni nombres reservados de Windows).
- Los enlaces simbólicos se rechazan en `.escritor/` y en las rutas de efecto.
- La importación de backups y documentos se ejecuta sin red y sin ejecutar
  contenido.
- No se aceptan imágenes remotas, scripts, macros ni HTML crudo.

## 10. Fuera de alcance

- Nivel de «partes» en el manuscrito (se diseña como migración estructural
  posterior).
- Sincronización, cuentas, colaboración, review copies y servicios externos.
- Historial de cambios de los sidecars más allá de los mensajes de comentarios.
- Compresión o compactación del log de operaciones.
- Windows: el contrato es multiplataforma, pero la validación real sigue
  pendiente.

## 11. Pruebas de contrato

- Un proyecto v1 se abre, edita y guarda sin crear sidecars.
- Al usar comentarios v2 se añade `markdown: 2` y `comentarios: 2` sin perder
  campos desconocidos.
- Los comentarios v1 siguen siendo legibles y no se reescriben.
- Un comentario con ancla borrada queda huérfano, no desaparece.
- Metas y sesiones sobreviven a cerrar y abrir el proyecto.
- Un sidecar corrupto se aparta y el manuscrito sigue intacto.
- Un backup con `..` o un enlace simbólico se rechaza antes de escribir.
- Las plantillas no copian IDs ni historial.
- La exportación DOCX/Markdown no cambia el proyecto.
