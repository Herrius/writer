# Bloque 2.2 — Aceptación del product owner

- **Commit de código:** `76b139f` (`style: conservar el panel lateral a 960 px`)
- **Base:** `cd9683e` (Fase 2.1 cerrada)
- **Fecha:** 2026-10-22
- **Decisión:** **aprobado con observaciones**
- **Bloque 2.2:** cerrado como continuación de la Fase 2 en Linux x86_64.
- **Release público:** **no autorizado**.
- **Fases 3, 4 y 5:** **pospuestas**; no se reactivan con esta firma.

## 1. Qué se entrega

El Bloque 2.2 sustituye la interfaz antigua de escritura por una adaptación de
las dos referencias de Dabble y cierra la deuda P3 prioritaria de la Fase 2.1:

- Editor con cabecera de proyecto, árbol por manuscritos/tramas/personajes/
  cuaderno/utilidades, página blanca, toolbar, panel Metas/Notas y pie de
  recuento.
- Historial con lista de versiones, contenido de la revisión y línea de tiempo
  inferior con `Recuperar` y `Salir de la máquina del tiempo`.
- Lista de proyectos compacta; modal con pie visible a 960×640 y 200 %, trampa
  de foco, `Escape` y retorno de foco.
- Vuelta al panel desde el editor guardando antes de salir.
- `Abrir proyecto` siempre visible, recientes etiquetados, confirmación de
  `Guardar`, limpieza de filtro/error y palabras a 0.
- Creación: validación antes de crear, rechazo de proyectos anidados y rollback
  de la carpeta nueva y del comando si falla una nota.
- Documentación de escritorio, inicio, E2E, iteración 16 y HANDOFF.

## 2. Criterios

| Criterio | Resultado | Evidencia |
|---|---|---|
| **1. Réplica del editor** | **Cumplido con observaciones** | Cabecera, árbol, página, toolbar, panel derecho y pie; capturas a 1440/960/200 %. Diferencias cosméticas y metas/temporizador visuales declarados. |
| **2. Réplica del historial** | **Cumplido con observaciones** | Versiones, contenido y timeline operables; el detalle es por archivo, no tarjetas visuales de escenas. |
| **3. Deuda P3 prioritaria** | **Cumplido** | QA21-01 a QA21-08, QA21-10 a QA21-14 y UX21-15 cerrados o con controles deshabilitados. QA22-01/02 quedan como condiciones. |
| **4. Robustez de creación** | **Cumplido** | `Project::init` valida antes de crear; pruebas de título inválido, anidado y rollback del comando. SEC21-03 cerrado. |
| **5. Pruebas y gates** | **Cumplido con observaciones** | 94 Rust, 3 Tauri, 59 UI, 12 E2E de navegador y 2 E2E de escritorio. El diálogo nativo de creación no se ejecutó en la app real. |
| **6. Documentación** | **Cumplido** | README, `inicio-v1`, `escritorio-v1`, `e2e`, iteración 15/16, HANDOFF y plan actualizados. |
| **7. Release local** | **Cumplido con observaciones** | `.deb` y `.rpm` verificados; release público bloqueado por versión, firma, updater, AppImage, Windows/macOS, app instalada, DOCX en Word y CI remoto. |

## 3. Observaciones aceptadas

- **QA22-01 / condición de validación:** el E2E real de creación con el diálogo
  nativo GTK sigue pendiente. El modal, el comando Tauri y la CLI están cubiertos;
  no se reprodujo un defecto del motor.
- **SEC21-02:** TOCTOU de `.escritor` aceptado como riesgo residual P3 hasta la
  preparación de release; no hay P1/P2.
- **QA22-02:** el historial no es una máquina del tiempo visual con tarjetas de
  escenas; es un límite de réplica declarado.
- **QA22-03 / UX22-03:** “Hoy día”, metas y temporizador son visuales y sus
  controles están deshabilitados.
- **REL22-01/02/03:** versión `0.2.0` repetida, paquetes no reproducibles y
  ausencia de firma/updater/AppImage/Windows/macOS/CI remoto. Bloquean el release
  público, no el hito local.

No hay P1 ni P2 abiertos. El defecto visual del párrafo oculto del modal se
corrigió antes del congelado y quedó cubierto por el E2E del modal.

## 4. Decisión y siguiente paso

**Decisión: aprobado con observaciones.**

El Bloque 2.2 cumple el objetivo de producto: la escritura y el historial siguen
las referencias de Dabble de forma reconocible, la deuda P3 prioritaria está
cerrada y la creación es más robusta. Se acepta como hito local Linux x86_64,
no como release público.

**Release público:** no autorizado. Cualquier publicación debe abrir un bloque de
preparación de release, resolver o aceptar formalmente SEC21-02, añadir el E2E
del diálogo nativo, probar la app instalada y validar DOCX en Word/LibreOffice.

**Fases 3, 4 y 5:** siguen pospuestas por decisión del usuario.

**Firma:** Product owner del proyecto `writer` — 2026-10-22.
