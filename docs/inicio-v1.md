# Inicio y creación de proyectos — v1 (Fase 2.1; panel revisado en Fase 1)

- **Fecha:** 2026-10-21; actualizado 2026-10-24.
- **Estado:** implementado y revisado visualmente el 2026-10-24 (portadas
  ilustradas y tarjetas de libro).

## 1. Panel de Proyectos

- Cabecera con marca, buscador, **Filtros** y **+ Agregar una colección**.
  Pestañas **Activo**, **Plantillas** y **Papelera**, todas operativas.
- Filtros locales por autor, periodo, palabras mínimas y orden; se combinan con
  la búsqueda y con la colección activa.
- Colecciones locales con nombre, color y lista de proyectos. Se crean,
  asignan desde `⋯`, renombran y eliminan sin mover archivos.
- La pestaña **Plantillas** lista, crea, usa y elimina plantillas locales.
- La pestaña **Papelera** permite restaurar o purgar proyectos movidos a
  `.writer-papelera`.
- “Mis Proyectos” muestra tarjetas de los proyectos recientes (máximo 8,
  `localStorage`) y una tarjeta “Crear nuevo proyecto”.
- La cabecera del panel mantiene el recuento, la vista y “Más opciones”
  ocultos en reposo para no competir con las portadas; aparecen con `hover` o
  al recibir foco de teclado. Siguen siendo accesibles.
- Cada proyecto se representa como un libro: portada ilustrada por defecto
  (`Árbol de invierno`, CC0 de StockSnap), título centrado y autor en la parte
  inferior, con borde interior claro. Debajo aparecen el título, la fecha
  “Modificado el …” y el indicador visual `⋯`.
- La fecha de modificación sale de `Reciente.modificado` y se actualiza al
  abrir o crear el proyecto. El recuento de palabras sigue en la barra de
  estado del editor; no se muestra en la tarjeta.
- La portada usa proporción `0.61` (≈5:8) y la tarjeta de creación mide lo mismo
  que la portada, no que la tarjeta completa. Debajo de la tarjeta de creación
  vive el enlace “Crear desde plantilla”, que abre el asistente.
- El tinte de portada es una preferencia visual local: se aplica con opacidad
  baja sobre el arte por defecto y no se guarda en el manifiesto.
- Estado vacío con “No proyectos”, texto de bienvenida y “Abrir proyecto
  existente”.
- Barra lateral con bienvenida, número de **recientes** (máximo 8) y palabras
  registradas. El espacio restante queda reservado para contenido futuro, sin
  texto ni controles; en la vista apilada (≤900 px) la reserva se oculta.
- El panel no muestra meta diaria, “Proyectos personales” ni el cuaderno
  “Ideas”. La meta diaria y el cuaderno de Ideas pueden existir dentro del
  proyecto.
- La vista de lista conserva filas compactas y una miniatura de la portada.
- El buscador muestra un anillo de foco visible cuando recibe foco.
- El proyecto se abre al pulsar una tarjeta; los recientes se actualizan al
  abrir o crear. “Abrir proyecto” está siempre visible en la cabecera, además
  del estado vacío.

## 2. Asistente de creación

Campos:

| Campo | Obligatorio | Uso |
|---|---|---|
| Nombre del proyecto | Sí | nombre de la carpeta; se deriva del título si no se edita |
| Título | Sí | `proyecto.yml` → `titulo` |
| Autor | No | `proyecto.yml` → `autor` |
| Idioma | No | `proyecto.yml` → `idioma` (`es`/`en`) |
| Carpeta padre | Sí | diálogo nativo de directorio |
| Notas iniciales | No | crea notas de personajes, tramas y lugares |
| Color de portada | No | tinte visual local guardado en recientes; se aplica sobre el arte por defecto, no en el manifiesto |

Al confirmar:

1. Valida título, carpeta y nombre en la UI.
2. Llama a `crear_proyecto` en Tauri.
3. `Project::init` **vuelve a validar en el backend**: normaliza la ruta,
   rechaza vacío y componentes `.`/`..`, exige que el padre exista, rechaza
   destinos enlazados o no vacíos y valida el nombre (controles, `.`/`..`,
   espacios iniciales y reservados habituales de Windows). La paridad completa
   con Windows sigue pendiente.
4. `Project::init` crea el manifiesto, `.escritor/`, Libro 1 / Capítulo 1 /
   Primera escena; valida antes de crear y limpia lo creado si falla.
5. Si están activas, crea las tres notas iniciales; si falla, el comando limpia
   el proyecto parcial.
6. “Guardar” deja el proyecto en recientes, limpia el filtro y muestra una
   confirmación; “Guardar y abrir” lo abre y muestra el mensaje.
7. Cualquier error del motor se muestra en el modal sin cerrar el formulario.

## 3. Límites conocidos

- No hay subida de imágenes ni selector de arte de portada: el arte por
  defecto es fijo y el color solo lo tiñe.
- La papelera es local, hermana del proyecto y sin purga automática; ver
  [`papelera-v1.md`](papelera-v1.md).
- Las colecciones y filtros viven en `localStorage`; no viajan con el proyecto.
- No hay renombrar, borrar ni mover proyectos desde la GUI.
- El color de portada es una preferencia local de recientes, no un dato del
  proyecto.
- La creación real requiere la app de escritorio; la demo solo muestra el
  formulario y explica el límite.
