# Inicio y creación de proyectos — v1 (Fase 2.1)

- **Fecha:** 2026-10-21
- **Estado:** implementado; pendiente pipeline de revisión.

## 1. Panel de Proyectos

- Cabecera con marca, buscador de proyectos, filtros/colecciones (deshabilitados
  hasta una fase posterior) y pestañas Activo / Plantillas / Papelera.
- “Mis Proyectos” muestra tarjetas de los proyectos recientes (máximo 8,
  `localStorage`) y una tarjeta “Crear nuevo proyecto”.
- Estado vacío con “No proyectos”, texto de bienvenida y “Abrir proyecto
  existente”.
- Barra lateral con bienvenida, número de proyectos, palabras registradas, meta
  diaria (deshabilitada) y cuaderno “Ideas”.
- El proyecto se abre al pulsar una tarjeta; los recientes se actualizan al
  abrir o crear.

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
| Color de portada | No | preferencia visual guardada en recientes, no en el manifiesto |

Al confirmar:

1. Valida título, carpeta y nombre en la UI.
2. Llama a `crear_proyecto` en Tauri.
3. `Project::init` **vuelve a validar en el backend**: normaliza la ruta,
   rechaza vacío/`.`/`..`, exige que el padre exista, rechaza destinos enlazados
   o no vacíos y valida el nombre (incluidos reservados de Windows).
4. `Project::init` crea el manifiesto, `.escritor/`, Libro 1 / Capítulo 1 /
   Primera escena.
4. Si están activas, crea las tres notas iniciales.
5. “Guardar” deja el proyecto en recientes; “Guardar y abrir” lo abre.
6. Cualquier error del motor se muestra en el modal sin cerrar el formulario.

## 3. Límites conocidos

- No hay importación de DOCX/Markdown ni plantillas.
- No hay renombrar, borrar ni mover proyectos desde la GUI.
- El color de portada es una preferencia local de recientes, no un dato del
  proyecto; no hay subida de imágenes.
- La creación real requiere la app de escritorio; la demo solo muestra el
  formulario y explica el límite.
