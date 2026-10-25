# Metas y temporizador v1 — contrato local

- **Fecha:** 2026-10-23; actualizado 2026-10-24.
- **Estado:** implementado. La meta diaria es global; libro y desafío siguen
  siendo por proyecto.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §4 y
  [`metas-globales-v1.md`](metas-globales-v1.md).

## 1. Alcance

Meta diaria global, metas de libro y desafíos con fecha; estadísticas, rachas y
temporizador. Todo local, sin cuenta ni red.

## 2. Metas

- **Diaria:** global, en `writer.metas-globales.v1`; ver
  [`metas-globales-v1.md`](metas-globales-v1.md). Se aplica a todos los libros.
- **Libro y desafío:** en `.escritor/metas/metas.json` del proyecto.
- **Archivadas:** metas terminadas o canceladas del proyecto.

El campo `diaria` de `metas.json` se conserva por compatibilidad; si no hay meta
global, se migra una sola vez al abrir el proyecto.

## 3. Sesiones

` .escritor/sesiones/<YYYY-MM>.jsonl`, una línea por sesión.

- Presets: 5, 10, 15, 25 y 30 minutos.
- Iniciar, pausar, reanudar, finalizar y limpiar.
- Una sesión finalizada no se reabre; si se limpia antes de finalizar, no se
  registra.
- `palabras` es el delta de palabras del documento activo entre inicio y fin.
- `pausas` guarda intervalos; el tiempo efectivo excluye pausas.
- El chime es local y opcional; no requiere archivo remoto.

## 4. Estadísticas y rachas

- La meta diaria usa la actividad global por fecha.
- Libro y desafío usan el proyecto activo.
- Racha global: días consecutivos con la meta cumplida, contando los días
  libres como válidos.
- La actividad se actualiza con el delta de palabras del editor; no se guarda
  como verdad duplicada del manuscrito.

## 5. Persistencia

- `metas.json` se escribe con reemplazo atómico y `fsync`.
- Las sesiones se append con `fsync`; el archivo se rota por mes.
- La meta diaria global y su actividad se guardan en `localStorage`.
- Si un archivo está corrupto, se aparta a `.escritor/cuarentena/` y se avisa.
- El backup incluye `metas/` y `sesiones/`; no incluye las preferencias globales
  del WebView.
- Las metas no se guardan en `proyecto.yml`.

## 6. UI

- Tarjeta **METAS DIARIAS** con engranaje, estado de hoy y progreso.
- El engranaje abre el modal de meta diaria con las opciones de conteo y días
  libres; no hay tarjeta inferior de llamada a la acción.
- Temporizador con presets, tiempo restante, iniciar/pausar/finalizar,
  anclado al fondo del panel.
- Sin meta no aparece la cabecera ni el estado del día; se muestra un estado
  vacío con gráfico, título, mensaje y «Agregar una meta» al pasar el ratón.
- Las metas de libro y desafío se conservan en el archivo del proyecto, pero no
  se editan en este panel.
- Los cambios de meta se guardan al confirmar, no en cada tecla.
- El temporizador sigue funcionando al cambiar de documento; la sesión queda
  asociada al documento activo al iniciar.

## 7. Límites

- Sin metas compartidas, ligas, cuentas ni correos.
- Sin sincronización entre dispositivos.
- Sin sonidos remotos; el chime usa el sistema o un tono local.
- La meta diaria es global por perfil; libro y desafío son por proyecto.
- El recordatorio es un aviso local, no una notificación del sistema.

## 8. Pruebas

- Configurar, editar y eliminar la meta diaria global.
- Verificar progreso global al escribir y persistencia al recargar.
- Días libres y opciones de conteo.
- Iniciar/pausar/finalizar una sesión y verificar palabras/tiempo.
- Limpiar una sesión no deja registro.
- Rachas con fechas controladas y cambio de mes.
- Sidecar corrupto se aparta sin tocar el manuscrito.
- Backup/restore conserva metas y sesiones.
