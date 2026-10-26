# Iteración 35 — Árbol jerárquico, menús e iconos

- **Fecha:** 2026-10-25
- **Estado:** implementado en Linux x86_64.
- **Capturas:** [`../revisiones/arbol-jerarquico-menu-2026-10-25.png`](../revisiones/arbol-jerarquico-menu-2026-10-25.png),
  [`../revisiones/vista-archivo-2026-10-25.png`](../revisiones/vista-archivo-2026-10-25.png),
  [`../revisiones/vista-plantillas-2026-10-25.png`](../revisiones/vista-plantillas-2026-10-25.png),
  [`../revisiones/cabecera-personajes-2026-10-25.png`](../revisiones/cabecera-personajes-2026-10-25.png),
  [`../revisiones/personajes-grupo-2026-10-25.png`](../revisiones/personajes-grupo-2026-10-25.png),
  [`../revisiones/cuaderno-carpeta-2026-10-25.png`](../revisiones/cuaderno-carpeta-2026-10-25.png),
  [`../revisiones/personajes-seccion-menu-2026-10-25.png`](../revisiones/personajes-seccion-menu-2026-10-25.png),
  [`../revisiones/cuaderno-menu-2026-10-25.png`](../revisiones/cuaderno-menu-2026-10-25.png).

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
- Personajes agrupados por rol para que la ruta superior coincida con el árbol.

## 3. Cabecera sincronizada

- El primer libro con carpeta genérica (`01-libro`) se muestra con el **título
  del proyecto**, no como «Libro 1».
- La cabecera muestra una miga de pan según el nodo activo:
  - escena: libro → capítulo → sección → escena;
  - personaje: grupo de rol → nombre;
  - trama: documento;
  - cuaderno: carpeta → documento;
  - Archivo/Papelera/Plantillas: proyecto → vista.
- Los personajes se agrupan en principales, secundarios y sin clasificar.
- La última miga devuelve al editor y queda resaltada.

## 4. Reglas de creación

- **Manuscrito:** un único libro. El libro solo crea capítulos, secciones y
  escenas; no crea tramas, personajes ni notas.
- **Tramas:** solo tramas.
- **Personajes:** solo personajes y nuevas secciones de personajes (grupos por
  rol o grupos personalizados). Cada sección tiene su menú `⋯` para crear un
  personaje; las secciones **personalizadas** también se pueden renombrar y
  eliminar. Al eliminar una sección personalizada, sus personajes pasan a
  «Sin clasificar». Las secciones predeterminadas (principales, secundarios y
  sin clasificar) son estructurales y no se renombran ni eliminan.
- **Cuaderno:** carpetas y notas dentro de ellas. La raíz y cada carpeta tienen
  su menú `⋯` para crear una nota o una carpeta. Las carpetas se pueden
  renombrar y eliminar; al eliminarlas, sus notas pasan a la Papelera.
- Las secciones de personajes se guardan por proyecto en `localStorage` y los
  personajes se asignan con su `rol`.

## 5. Páginas de contenido

- Al pulsar un grupo de personajes se muestra una cuadrícula de tarjetas con
  portada, nombre y vista previa, y el botón **Crear un personaje**.
- Al pulsar una carpeta de Cuaderno se muestra la cuadrícula de notas y los
  botones **Crear una nota** y **Crear una carpeta**.
- El panel derecho se oculta en estas páginas para dejar sitio al contenido.

## 6. Menús por nodo

Cada nodo tiene un menú `⋯` adaptado a su nivel:

| Nodo | Agregar | Otras acciones |
|---|---|---|
| Manuscritos | Libro (solo si no existe ninguno) | — |
| Libro | Capítulo, Sección, Escena | Renombrar, Exportar, Eliminar |
| Capítulo | Capítulo, Sección | Renombrar, Exportar, Eliminar |
| Sección | Sección, Escena | Renombrar, Exportar, Eliminar |
| Escena/nota | — | Renombrar, Exportar, Eliminar |
| Tramas | Trama | — |
| Personajes | Personaje, nueva sección | Renombrar/eliminar sección personalizada |
| Cuaderno (raíz) | Nota, carpeta | — |
| Cuaderno (carpeta) | Nota, carpeta | Renombrar, eliminar |

- **Renombrar** carpeta actualiza las rutas y reindexa; renombrar documento usa
  la API existente.
- **Eliminar** mueve los documentos a la Papelera lógica; nunca borra archivos
  del disco. Si la carpeta queda vacía, se retira del árbol.
- **Exportar** genera Markdown con los documentos activos del nodo y usa el
  diálogo nativo de guardado.
- Los portales de notas se crean como documentos de su clase.

## 7. Iconos

Se incorporó **Lucide React** (`lucide-react@0.468.0`) para el árbol, menús,
utilidades y cabecera. Ya no se usan glifos `▤`, `◈`, `♙`, `⌂` o `♲` como
iconos funcionales.

## 8. Páginas de estado

- **Archivo** y **Papelera** se muestran en el área principal, con el árbol
  siempre visible y el panel derecho oculto, siguiendo la referencia.
- Estado vacío centrado y listado con acción **Restaurar**.
- **Plantillas** abre una página del proyecto con acceso al gestor de
  plantillas de Proyectos.
- El botón **+** abre el formulario de creación de escena en Plan.

## 9. Backend

Nuevos comandos Tauri:

- `listar_carpetas`
- `crear_carpeta`
- `renombrar_carpeta`
- `eliminar_carpeta_vacia`
- `crear_escena_en`
- `crear_nota_en`
- `guardar_texto_exportado`

Validaciones: rutas relativas bajo `manuscrito`/`notas`, sin `..`, sin symlinks,
sin nombres vacíos, conflicto si el destino existe y reindexado tras renombrar.

## 10. Verificación

```text
Rust:       91 unitarias + 50 integración + 25 CLI + 1 rendimiento
Desktop:    10/10
UI:         88/88
E2E:        61/61
calidad     OK
interfaz    OK
producto    OK
seguridad   OK
empaquetado OK
```

## 11. Límites

- El borrado de carpetas es lógico para sus documentos: los archivos permanecen
  en disco hasta que se restauren o se gestione el proyecto desde fuera.
- La exportación de carpeta incluye solo documentos activos.
- Las categorías Tramas/Personajes/Cuaderno no se renombran ni eliminan como
  carpetas. Personajes y Cuaderno tienen páginas propias para crear sus
  elementos.
- Los grupos de personajes personalizados viven en `localStorage` por proyecto;
  no se sincronizan entre equipos.
- El gestor de plantillas sigue viviendo en Proyectos.
