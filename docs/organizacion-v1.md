# Organización de proyectos — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado.
- **Ámbito:** filtros, orden y colecciones locales del panel de Proyectos.

## 1. Filtros

Se guardan en `localStorage` con la clave `writer.filtros.v1`:

```json
{
  "version": 1,
  "autor": "",
  "periodo": "todos",
  "orden": "recientes",
  "minimoPalabras": 0
}
```

- `autor`: coincidencia parcial sin distinguir mayúsculas; vacío = cualquiera.
- `periodo`: `todos`, `7d`, `30d`, `ano`.
- `minimoPalabras`: entero ≥ 0.
- `orden`: `recientes` (respeta el orden ya guardado en recientes),
  `titulo` (A–Z) o `palabras` (descendente).
- La búsqueda por título/ruta se combina con los filtros.
- Los filtros y la colección activa se aplican juntos.
- Un valor inválido en `localStorage` se normaliza a los valores por defecto.

## 2. Colecciones

Se guardan en `localStorage` con la clave `writer.colecciones.v1`. No viven en
el proyecto ni se sincronizan.

```json
{
  "id": "uuid-local",
  "nombre": "Prioridad",
  "color": "#6b5747",
  "rutas": ["/ruta/al/proyecto"],
  "creada": "2026-10-24T10:00:00Z"
}
```

- Hasta 50 colecciones; cada una hasta 1.000 rutas. El límite se comprueba
  al crear/asignar y se muestra un error; no se recorta al leer.
- Nombre no vacío, ≤ 120 caracteres.
- Color `#rrggbb`.
- Las rutas son las mismas rutas absolutas de `writer.recientes.v1`.
- Crear o borrar una colección no mueve ni modifica proyectos.
- Al mover un proyecto a la papelera se elimina su ruta de todas las
  colecciones.
- La barra de colecciones solo aparece cuando hay alguna.
- El menú `⋯` de una tarjeta permite asignarla a varias colecciones.

## 3. UI

- **Filtros**: botón en la cabecera; muestra `Filtros (n)` cuando hay filtros
  activos.
- **+ Agregar una colección**: crea una colección con nombre y color.
- **Barra de colecciones**: «Todas», chips de colecciones y «Gestionar».
- **Gestionar**: permite renombrar y eliminar colecciones con confirmación.
- **Tarjeta `⋯`**: abrir, asignar colecciones o mover a la papelera.

## 4. Límites

- No hay colecciones anidadas ni reglas automáticas.
- Las colecciones son por perfil/equipo, no por proyecto.
- Si se pierde `localStorage`, se pierden los filtros y las colecciones; los
  proyectos y sus archivos no se tocan. Las entradas que superen los límites no
  se eliminan al cargar: la UI impide seguir creciendo.
- Las rutas movidas o renombradas fuera de writer no se actualizan solas; el
  filtro de colección simplemente deja de encontrar esa ruta.

## 5. Pruebas

- Filtro combinado de autor, periodo, palabras y búsqueda.
- Orden por título y palabras.
- Crear/asignar/renombrar/eliminar colecciones y persistir al recargar.
- Validación de `localStorage` y de colecciones corruptas.
