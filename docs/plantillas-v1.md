# Plantillas locales v1 — contrato

- **Fecha:** 2026-10-23; actualizado 2026-10-24.
- **Estado:** implementado. Incluye creación y eliminación desde la UI.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §6.

## 1. Alcance

Plantillas locales para crear proyectos nuevos. Sin biblioteca pública, sin
descarga y sin red.

## 2. Ubicación

```text
<app_data_dir>/plantillas/<slug>/
  plantilla.json
  manuscrito/
  notas/
```

- `<app_data_dir>` es el directorio de datos de la app en cada plataforma.
- El nombre de la carpeta es el `slug` de la plantilla; se valida como nombre de
  archivo seguro.
- Las plantillas no viven dentro del proyecto y no se incluyen en su backup.

## 3. Manifiesto

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

- `nombre` no vacío y ≤ 200 caracteres.
- `idioma` es un código corto (`es`, `en`).
- `estructura` usa enteros entre 1 y 999 y el producto total no puede superar
  500 escenas.
- `notas_iniciales` solo admite tipos de nota válidos.
- Un manifiesto inválido no se ofrece en la UI; se puede eliminar por slug desde la API aunque no aparezca listado.

## 4. Creación desde plantilla

1. La UI pide título, autor, carpeta padre y nombre de carpeta.
2. Se valida el destino con las mismas reglas que `Project::init`.
3. Se copia la estructura y las notas de la plantilla.
4. Se generan ULID, fechas y nombres de archivo nuevos.
5. No se copian historial, `.escritor/`, comentarios, metas ni índices.
6. Si falla un paso, se limpia el proyecto parcial.
7. El proyecto se abre o se guarda en recientes según la acción elegida.

## 5. Seguridad

- La plantilla no puede contener rutas absolutas, `..`, enlaces simbólicos ni
  nombres reservados de Windows. Se validan también las raíces `manuscrito/` y
  `notas/`; los FIFOs y entradas que no son archivos regulares se ignoran.
- Se copian como máximo 500 archivos Markdown, 10 MiB por archivo y 64 MiB en
  total. Las notas iniciales se deduplican y están limitadas a 100.
- No se ejecutan scripts ni macros.
- No se aceptan imágenes remotas.
- El contenido se copia como texto Markdown; no se interpreta ni se ejecuta
  como HTML crudo.
- Una plantilla corrupta se ignora en el listado; no bloquea el resto. La API permite eliminarla.

## 6. UI

- La pestaña **Plantillas** muestra nombre, descripción, estructura y notas.
- **Nueva plantilla** abre un formulario para nombre, descripción, idioma,
  estructura y notas iniciales.
- **Usar plantilla** abre el asistente con la plantilla preseleccionada.
- **Eliminar** pide confirmación y borra solo la carpeta de la plantilla, nunca
  un proyecto.
- «Crear desde plantilla» abre el asistente; si no hay plantillas, la pestaña
  ofrece crear la primera.

## 7. Pruebas

- Crear proyecto desde plantilla con estructura y notas.
- IDs nuevos y sin historial.
- Plantilla con `..` o enlace simbólico se rechaza.
- Plantilla inválida no aparece en la UI.
- Proyecto parcial se limpia si falla la creación.
- Guardar, listar y eliminar una plantilla local.
- Rechazar un directorio que no tenga `plantilla.json` al intentar borrarlo.
- Rechazar una plantilla que generaría más de 500 escenas, con archivos de
  manuscrito extra o con demasiados archivos.
- Rechazar raíces enlazadas y no bloquearse con un FIFO.

## 8. Gestión y API

Núcleo:

- `templates::guardar_plantilla(directorio, slug, plantilla)`.
- `templates::eliminar_plantilla(directorio, slug)`.
- `templates::listar_plantillas`.

Tauri:

- `guardar_plantilla(plantilla)`
- `eliminar_plantilla(slug)`
- `listar_plantillas`
- `crear_desde_plantilla(slug, ruta, titulo, autor, idioma)`
