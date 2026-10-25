# Iteración 33 — Meta diaria global

- **Fecha:** 2026-10-24
- **Estado:** implementado en Linux x86_64; Windows pendiente de Fase 4.
- **Capturas:** [`../revisiones/metas-globales-panel-2026-10-24.png`](../revisiones/metas-globales-panel-2026-10-24.png),
  [`../revisiones/metas-globales-modal-2026-10-24.png`](../revisiones/metas-globales-modal-2026-10-24.png).

## 1. Objetivo

La meta diaria deja de ser un dato del proyecto: **se aplica a cualquier libro**
que se abra en writer. El progreso se calcula sobre la actividad global del día,
no sobre las sesiones de un proyecto concreto.

## 2. Cambios

- Nueva tarjeta **METAS DIARIAS** en el panel Metas.
- Modal **Meta diaria** con:
  - palabras por día;
  - contar solo manuscrito, palabras pegadas y eliminaciones (progreso neto);
  - días libres D L M X J V S;
  - recordatorio local (sin correo, porque la app es offline);
  - guardar y eliminar meta.
- Progreso total del día, porcentaje, restante y gráfico de los últimos 14 días.
- El progreso se actualiza al escribir y se conserva en
  `localStorage` (`writer.metas-globales.v1`).
- Migración inicial: si el proyecto abierto tiene una meta diaria antigua y aún
  no hay una global, se adopta esa meta una sola vez.
- Se retiraron la tarjeta «Establece tus metas de escritura» y el bloque
  «Otras metas»: la meta se configura desde el engranaje de la cabecera.
- Los datos antiguos de libro y desafío se conservan en el proyecto por
  compatibilidad, pero ya no se editan desde este panel.
- El temporizador de sesión se conserva debajo del panel.

## 3. Cálculo

- Actividad diaria = delta de palabras del documento activo.
- `contarSoloManuscrito`: ignora notas si está activo.
- `contarPegadas`: si está desactivado, descuenta las palabras pegadas.
- `contarEliminaciones`: si está desactivado, no cuenta deltas negativos.
- Los días libres no rompen la racha.
- El estado se guarda por fecha local `YYYY-MM-DD`.
- No se mezcla con las sesiones del temporizador para no contar dos veces.

## 4. Verificación

```text
pnpm typecheck                         OK
pnpm ui:test                           88/88
pnpm e2e                               57/57 (metas 3/3)
cargo test --workspace                 91 + 49 + 25 + 1
cargo test desktop                     9/9
```

Casos nuevos: meta global, progreso tras escribir, persistencia al recargar,
día libre, opciones de conteo, validación de localStorage y temporizador.

## 5. Límites

- `localStorage` es por perfil del WebView; no se sincroniza entre equipos.
- No se envían correos ni notificaciones del sistema: el recordatorio es un
  aviso local al abrir el proyecto.
- La migración adopta la primera meta diaria de proyecto que encuentre; no
  fusiona metas de varios libros.
- El gráfico es de 14 días y no es interactivo.
- La actividad se calcula del texto editado en writer, no de archivos cambiados
  por fuera.
