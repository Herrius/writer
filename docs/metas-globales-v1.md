# Meta diaria global — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado.
- **Ámbito:** hábito diario local compartido por todos los libros abiertos en
  writer.

## 1. Datos

`localStorage`, clave `writer.metas-globales.v1`:

```json
{
  "version": 1,
  "diaria": {
    "objetivo": 500,
    "desde": "2026-10-24",
    "diasLibres": [0, 6],
    "contarSoloManuscrito": true,
    "contarPegadas": true,
    "contarEliminaciones": true,
    "recordatorio": false
  },
  "actividad": {
    "2026-10-24": 3
  }
}
```

- `objetivo`: entero 1–1.000.000.
- `diasLibres`: días de la semana 0 (domingo)–6 (sábado).
- `actividad`: palabras netas por fecha local; nunca negativas.
- Los valores inválidos se normalizan a una meta ausente/actividad vacía.

## 2. Conteo

El editor calcula el delta de palabras de cada documento activo.

- Solo manuscrito: si está activo, ignora las notas.
- Palabras pegadas: si está desactivado, descuenta el texto pegado.
- Progreso neto: si está activo, los borrados restan; si no, solo cuentan los
  incrementos.
- Los días libres no rompen la racha.
- El temporizador no se suma otra vez a la actividad global.

## 3. UI

- **METAS DIARIAS** en el panel Metas.
- «Hoy es un día de escritura» / «Hoy es un día libre».
- Progreso, porcentaje, restante y gráfico de 14 días.
- Modal de configuración con días libres y recordatorio local.
- El engranaje de la cabecera es la única entrada para crear, ajustar o
  eliminar la meta diaria.

## 4. Persistencia y migración

- La meta y la actividad viven en el perfil del WebView, no en el proyecto.
- Si no existe meta global y el proyecto abierto tiene una meta diaria antigua,
  se adopta una vez.
- Borrar la meta diaria no borra la actividad; la actividad se puede reutilizar
  si se crea otra meta.
- El backup del proyecto no incluye estas preferencias globales.

## 5. Límites

- No hay correo ni notificación del sistema: el recordatorio es un aviso local
  al abrir el proyecto.
- No hay sincronización entre equipos.
- La actividad no detecta cambios hechos fuera del editor.
- El gráfico es de 14 días y no es interactivo.
