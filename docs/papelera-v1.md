# Papelera local de proyectos — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado en Linux; la validación Windows queda para Fase 4.
- **Ámbito:** retirar, restaurar y purgar proyectos completos sin perder el
  manuscrito.

## 1. Principio

La papelera **no borra** al mover. El proyecto se mueve con `rename` a una
carpeta oculta en el mismo directorio padre:

```text
<padre>/.writer-papelera/<ULID>/
  proyecto/             # el proyecto completo
  papelera.json         # metadata de recuperación
```

Al estar en el mismo sistema de archivos, el movimiento es atómico y no hay
copia intermedia. El proyecto conserva todo su contenido: manuscrito, notas,
`.escritor/`, comentarios, metas e historial.

## 2. Metadata

```json
{
  "version": 1,
  "id": "01K...",
  "titulo": "El umbral",
  "autor": "Marta Ruiz",
  "color": "#6b5747",
  "palabras": 12345,
  "ruta_original": "/home/.../el-umbral",
  "ruta_papelera": "/home/.../.writer-papelera/01K...",
  "eliminado": "2026-10-24T10:00:00Z"
}
```

- El `id` es ULID y coincide con el nombre de la carpeta.
- Las rutas son absolutas.
- El color es `#rrggbb` si existe.
- La ruta de papelera debe estar dentro de un directorio llamado
  `.writer-papelera`; `ruta_original` debe ser hermana de ese directorio. Se
  rechazan rutas que apunten dentro del slot o fuera del padre.
- La metadata está limitada a 64 KiB; el autor a 200 caracteres.

## 3. Operaciones

### Mover

1. Se comprueba que la ruta sea un directorio real, no un symlink, y que tenga
   `proyecto.yml`.
2. Se crea `.writer-papelera/<ULID>/`.
3. Se escribe y valida `papelera.json` antes de tocar el proyecto.
4. Se renombra el proyecto a `proyecto/`.
5. Si falla la metadata, el slot solo contiene el manifiesto y se elimina; el
   proyecto sigue intacto. Si falla el renombrado, el proyecto tampoco se toca.

### Restaurar

1. Se valida la metadata y que `proyecto/` sea un directorio real.
2. Se exige que la carpeta padre original exista y que el destino no esté
   ocupado.
3. Se renombra `proyecto/` a la ruta original.
4. Se elimina el contenedor de papelera.

Si el destino está ocupado, no se sobrescribe: el usuario debe resolverlo.

### Purgar

Se valida la metadata y el nombre de la carpeta y se ejecuta `remove_dir_all`.
Es la única operación que borra contenido del disco.

## 4. UI

- Pestaña **Papelera** con título, ruta original, fecha, **Restaurar** y
  **Eliminar definitivamente**.
- La eliminación definitiva pide confirmación.
- Al restaurar, el proyecto vuelve a recientes con la fecha actual.
- El índice de entradas se guarda en `localStorage`
  (`writer.papelera.v1`) para poder listar la papelera; los archivos y la
  metadata son la fuente de verdad para restaurar.
- Se admiten como máximo 100 entradas para añadir. Al llegar al límite, mover
  muestra un error; las entradas heredadas que ya existan no se ocultan al
  recargar.
- Las colecciones que contenían el proyecto pierden su ruta al moverlo.

## 5. Seguridad y límites

- Se rechazan symlinks en el proyecto, en el contenedor de papelera y en la
  metadata.
- No se permite restaurar sobre un destino existente.
- No se purga un directorio que no tenga `papelera.json` válido.
- La papelera es hermana del proyecto; no usa la papelera del sistema operativo
  ni la red.
- Si el usuario borra `.writer-papelera` manualmente, no hay restauración.
- Si se pierde `localStorage`, los archivos siguen en `.writer-papelera`, pero
  la UI no los lista automáticamente.
- No hay purga automática por antigüedad.
- La validación Windows de `rename`, rutas largas y permisos queda en Fase 4.

## 6. Pruebas

- Mover, comprobar ausencia del proyecto y presencia del contenedor.
- Restaurar y comprobar `proyecto.yml`.
- Purgar y comprobar que desaparece.
- Rechazar directorios que no son proyectos, symlinks y destinos ocupados.
- Rechazar metadata o rutas de papelera ajenas.
- UI: mover, restaurar, purgar y confirmar; E2E de navegador.
