# Iteración 34 — Archivo y papelera en la barra lateral

- **Fecha:** 2026-10-25
- **Estado:** implementado en Linux x86_64.
- **Capturas:** [`../revisiones/archivo-papelera-vacia-2026-10-25.png`](../revisiones/archivo-papelera-vacia-2026-10-25.png),
  [`../revisiones/archivo-papelera-lista-2026-10-25.png`](../revisiones/archivo-papelera-lista-2026-10-25.png).

## 1. Problema

Al entrar en **Archivo** o **Papelera**, la barra lateral seguía mostrando
Manuscritos, Tramas, Personajes, Cuaderno e Ideas vacíos. No era evidente qué
documentos había en ese estado ni cómo devolverlos a activos.

## 2. Cambios

- Vista dedicada para **Archivo** y **Papelera**:
  - cabecera con nombre, explicación y contador;
  - listado único de documentos, con tipo y etiquetas;
  - acción **Restaurar** en cada fila;
  - estado vacío específico («El archivo está vacío» / «La papelera está vacía»);
  - filtro por etiqueta y opción para limpiarlo.
- Contadores en las utilidades **Archivo** y **Papelera**.
- El botón **+** de la barra lateral ya no está deshabilitado: abre el panel Plan
  y enfoca **Título de la nueva escena**.
- El botón **Plantillas** vuelve al panel de Proyectos y abre su pestaña de
  plantillas.
- Si el estado contrario tiene documentos, el estado vacío ofrece un enlace para
  ir a él.

## 3. Flujo

1. Desde un documento activo: **Notas → Archivar** o **A papelera**.
2. En Utilidades se actualiza el contador.
3. Al abrir el estado se ve una lista compacta con la acción de restaurar.
4. **Restaurar** devuelve el documento a activos sin salir de la barra lateral.
5. **← Volver a activos** restaura la vista normal del manuscrito.

## 4. Verificación

- E2E `fase-2-1-archivo`: archiva, comprueba el listado, restaura desde la fila,
  comprueba el estado vacío y usa el botón **+** para abrir la creación.
- E2E de metas y organización sin cambios.
- UI 88/88; E2E 58/58.

## 5. Límites

- La papelera de documentos es un estado lógico del proyecto; no borra archivos.
- No hay acciones masivas ni purga permanente desde esta barra.
- Las plantillas se gestionan en el panel de Proyectos; la barra lateral solo
  sirve como acceso directo a esa pestaña.
