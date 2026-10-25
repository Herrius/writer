# Iteración 35 — Árbol jerárquico, menús e iconos

- **Fecha:** 2026-10-25
- **Estado:** implementado en Linux x86_64.
- **Capturas:** [`../revisiones/arbol-jerarquico-menu-2026-10-25.png`](../revisiones/arbol-jerarquico-menu-2026-10-25.png),
  [`../revisiones/vista-archivo-2026-10-25.png`](../revisiones/vista-archivo-2026-10-25.png),
  [`../revisiones/vista-plantillas-2026-10-25.png`](../revisiones/vista-plantillas-2026-10-25.png).

## 1. Objetivo

Sustituir el árbol plano de la barra lateral por una estructura jerárquica
navegable, con menús contextuales por nodo, iconos profesionales y páginas
propias para Archivo, Papelera y Plantillas.

## 2. Árbol

- Estructura ilimitada de carpetas bajo `manuscrito/`:
  - Manuscritos → Libro → Capítulo → Sección → Escena.
- Las carpetas vacías se conservan y aparecen en el árbol.
- Se eliminó la sección **Ideas**.
- Tramas, Personajes y Cuaderno conservan sus documentos y su creación rápida.
- Los documentos activos se pueden arrastrar a capítulos y secciones.
- Navegación con flechas, `Home` y `End` sobre las escenas.

## 3. Menús por nodo

Cada nodo tiene un menú `⋯` adaptado a su nivel:

| Nodo | Agregar | Otras acciones |
|---|---|---|
| Manuscritos | Libro | — |
| Libro | Capítulo, Sección, Escena, Trama, Personaje, Nota | Renombrar, Exportar, Eliminar |
| Capítulo | Capítulo, Sección | Renombrar, Exportar, Eliminar |
| Sección | Sección, Escena | Renombrar, Exportar, Eliminar |
| Escena/nota | — | Renombrar, Exportar, Eliminar |
| Tramas/Personajes/Cuaderno | Trama/Personaje/Nota | — |

- **Renombrar** carpeta actualiza las rutas y reindexa; renombrar documento usa
  la API existente.
- **Eliminar** mueve los documentos a la Papelera lógica; nunca borra archivos
  del disco. Si la carpeta queda vacía, se retira del árbol.
- **Exportar** genera Markdown con los documentos activos del nodo y usa el
  diálogo nativo de guardado.
- Los portales de notas se crean como documentos de su clase.

## 4. Iconos

Se incorporó **Lucide React** (`lucide-react@0.468.0`) para el árbol, menús,
utilidades y cabecera. Ya no se usan glifos `▤`, `◈`, `♙`, `⌂` o `♲` como
iconos funcionales.

## 5. Páginas de estado

- **Archivo** y **Papelera** se muestran en el área principal, con el árbol
  siempre visible y el panel derecho oculto, siguiendo la referencia.
- Estado vacío centrado y listado con acción **Restaurar**.
- **Plantillas** abre una página del proyecto con acceso al gestor de
  plantillas de Proyectos.
- El botón **+** abre el formulario de creación de escena en Plan.

## 6. Backend

Nuevos comandos Tauri:

- `listar_carpetas`
- `crear_carpeta`
- `renombrar_carpeta`
- `eliminar_carpeta_vacia`
- `crear_escena_en`
- `guardar_texto_exportado`

Validaciones: rutas relativas bajo `manuscrito`/`notas`, sin `..`, sin symlinks,
sin nombres vacíos, conflicto si el destino existe y reindexado tras renombrar.

## 7. Verificación

```text
Rust:       91 unitarias + 50 integración + 25 CLI + 1 rendimiento
Desktop:    10/10
UI:         88/88
E2E:        59/59
calidad     OK
interfaz    OK
producto    OK
seguridad   OK
empaquetado OK
```

## 8. Límites

- El borrado de carpetas es lógico para sus documentos: los archivos permanecen
  en disco hasta que se restauren o se gestione el proyecto desde fuera.
- La exportación de carpeta incluye solo documentos activos.
- Las categorías Tramas/Personajes/Cuaderno no se renombran ni eliminan como
  carpetas; su menú solo crea elementos.
- El gestor de plantillas sigue viviendo en Proyectos.
