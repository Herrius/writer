# Metas y temporizador v1 — contrato local

- **Fecha:** 2026-10-23
- **Estado:** propuesta normativa para la Fase 2.4.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §4.

## 1. Alcance

Metas diarias, de libro y desafíos con fecha; estadísticas, rachas y
temporizador. Todo local, por proyecto, sin cuenta ni red.

## 2. Metas

` .escritor/metas/metas.json` con el esquema de `esquema-v2.md` §4.1.

- **Diaria:** palabras objetivo por día y fecha de inicio.
- **Libro:** palabras objetivo para el proyecto.
- **Desafío:** palabras objetivo y fecha límite.
- **Archivadas:** metas terminadas o canceladas, con motivo.

Una meta puede estar ausente. No se muestran tarjetas de metas en el panel de
Proyectos; las metas viven dentro del proyecto.

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

- Palabras por día, por semana y por proyecto.
- Racha actual y mejor racha, calculadas desde las sesiones y el historial de
  palabras.
- Gráfico de progreso de los últimos 30 días.
- Una racha se rompe si un día no alcanza la meta diaria; la definición exacta
  se fija en la UI y se prueba con fechas controladas.
- No se guardan agregados como verdad: se derivan de sesiones y manuscrito.

## 5. Persistencia

- `metas.json` se escribe con reemplazo atómico y `fsync`.
- Las sesiones se append con `fsync`; el archivo se rota por mes.
- Si un archivo está corrupto, se aparta a `.escritor/cuarentena/` y se avisa.
- El backup incluye `metas/` y `sesiones/`.
- Las metas no se guardan en `proyecto.yml`.

## 6. UI

- Panel «Metas» del editor con pestañas o secciones: diaria, libro, desafío,
  estadísticas.
- Temporizador visible con presets, tiempo restante, iniciar/pausar/finalizar.
- Estado vacío con «Configurar meta» y sin controles falsos.
- Los cambios de meta se guardan al confirmar, no en cada tecla.
- El temporizador sigue funcionando al cambiar de documento; la sesión queda
  asociada al documento activo al iniciar.

## 7. Límites

- Sin metas compartidas, ligas, cuentas ni notificaciones.
- Sin sincronización entre dispositivos.
- Sin sonidos remotos; el chime usa el sistema o un tono local.
- Sin estadísticas de otros proyectos fuera de los abiertos.

## 8. Pruebas

- Crear, editar y archivar metas.
- Iniciar/pausar/finalizar una sesión y verificar palabras/tiempo.
- Limpiar una sesión no deja registro.
- Rachas con fechas controladas y cambio de mes.
- Sidecar corrupto se aparta sin tocar el manuscrito.
- Backup/restore conserva metas y sesiones.
