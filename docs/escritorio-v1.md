# Interfaz de escritorio — v1 (Bloque 2.2)

- **Fecha:** 2026-10-21
- **Estado:** implementada; pendiente pipeline.
- **Referencias:** `referencias-interfaz/dabble-editor-2026-10-20.png` y
  `referencias-interfaz/dabble-historial-versiones-2026-10-20.png`, adaptadas a
  writer sin copiar marca ni datos.

## 1. Estructura

- **Cabecera:** volver a Proyectos, pestañas de proyecto/documento, estado de
  guardado, exportación, Markdown, historial, ayuda y avatar.
- **Árbol lateral:** filtro, manuscritos agrupados por libro y capítulo, tramas,
  personajes, cuaderno (lugares/cronología/objetos) y utilidades.
- **Centro:** página blanca con título del documento, metadatos, toolbar de
  formato, editor, recuento y estado.
- **Panel derecho:** Metas (gráfico y temporizador visuales) y Notas
  (metadatos del documento y del proyecto).
- **Historial:** lista de versiones a la derecha, contenido de la revisión en el
  centro y línea de tiempo inferior con Recuperar / Salir de la máquina del
  tiempo.

## 2. Funciones conservadas

- Autosave, `Ctrl+S`, conflicto por hash y cierre controlado de Tauri.
- Búsqueda FTS5, historial y reversión con confirmación.
- Marcas de personaje/trama/comentario, toolbar y recuento.
- Exportación DOCX, cuarentena, diagnóstico y solo lectura.
- Navegación por teclado, salto al contenido, foco visible y zoom.

## 3. Límites

- El gráfico de metas y el temporizador son visuales; todavía no guardan metas
  ni ejecutan sesiones.
- El historial muestra contenido por archivo, no una máquina del tiempo con
  tarjetas de escenas y cambios resaltados.
- “Construcción del mundo” selecciona la primera nota de cuaderno disponible.
- Plantillas, Archivo, Papelera y Configuración están deshabilitados.
- No se copian ilustraciones, portadas ni la marca de Dabble.
