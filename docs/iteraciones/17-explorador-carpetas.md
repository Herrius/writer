# Iteración 17 — Explorador de carpetas para crear proyectos

- **Fecha:** 2026-10-22
- **Estado:** implementado; Rust, Tauri, UI y E2E de navegador verdes. El
  escenario E2E de escritorio de creación está definido pero falló por una
  sesión WebDriver inválida en la última ejecución; hay que repetirlo.
- **Alcance:** elegir la carpeta padre del proyecto navegando dentro de la app,
  sin depender únicamente del diálogo nativo.

## 1. Comportamiento

- En el asistente, “Carpeta padre” tiene un botón **Examinar…** que abre un
  explorador dentro del mismo diálogo.
- El explorador permite:
  - subir un nivel;
  - entrar en cualquier subcarpeta;
  - escribir una ruta y pulsar `Enter`/`Ir`;
  - ver la ruta actual y los errores de permisos;
  - usar la carpeta actual con **Usar esta carpeta**.
- **Diálogo del sistema…** sigue disponible como alternativa nativa.
- En la demo de navegador, “Examinar…” conserva el prompt de ruta porque no hay
  backend de archivos.

## 2. Implementación

- Comando Tauri `listar_directorios(ruta: Option<String>)`:
  - parte de la carpeta actual, o del home si no se indica ruta;
  - canonicaliza y valida que sea un directorio;
  - devuelve ruta actual, padre y subcarpetas ordenadas;
  - ignora archivos, entradas sin UTF-8 y errores de entradas individuales;
  - limita el listado a 5.000 subcarpetas.
- `DirectorioDto` y `ListadoDirectoriosDto` generados con `ts-rs`.
- `listarDirectorios()` en `ui/src/api.ts`.
- El explorador reutiliza la trampa de foco y el `Escape` del modal: primero
  cierra el explorador y devuelve el foco a **Examinar…**.

## 3. Pruebas

- Rust: `listar_directorios_navega_y_rechaza_archivos` (orden, subida y rechazo
  de archivos).
- UI: `explorador de carpetas` navega a una subcarpeta, usa la ruta actual y
  actualiza “Carpeta padre”.
- E2E de escritorio: tercera ejecución que crea un proyecto completo desde el
  panel usando el explorador y verifica `proyecto.yml`, la estructura inicial y
  la escena.

## 4. Límites

- No crea carpetas nuevas desde el explorador; se elige una existente.
- No oculta carpetas del sistema ni archivos ocultos; muestra solo directorios.
- El diálogo nativo no se automatiza en el E2E; el explorador in-app sí.
- La creación sigue siendo exclusiva de la app de escritorio.
