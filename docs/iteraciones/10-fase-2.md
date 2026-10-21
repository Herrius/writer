# Iteración 10 — Correcciones de la QA independiente

- **Fecha:** 2026-10-21
- **Alcance:** corrección de los hallazgos de la primera revisión QA de la Fase 2.
- **Estado:** correcciones aplicadas; pendiente repetir QA sobre el nuevo commit
  y continuar el pipeline de roles.
- **Informe de origen:** [`../revisiones/fase-2-qa-2026-10-21.md`](../revisiones/fase-2-qa-2026-10-21.md)

## 1. Resultado de la revisión

La QA independiente bloqueó la fase con 5 P1, 4 P2, 6 P3 y 2 P4. Los P1 eran
pérdida o corrupción de datos reproducibles:

1. cierre de la ventana Tauri sin guardar ni avisar;
2. tabla dentro de una cita borrada al editar;
3. imagen por referencia borrada al editar;
4. bloque de código con valla de cuatro backticks corrompido;
5. carrera de autosave que dejaba disco y editor desincronizados.

## 2. Correcciones

### Cierre de ventana

`ui/src/App.tsx` registra `onCloseRequested` en Tauri: cancela el cierre,
ejecuta el guardado pendiente y solo destruye la ventana si termina bien. Si hay
conflicto, error o una reversión en curso, la ventana sigue abierta con el aviso.
El E2E de escritorio cierra con `Alt+F4` (X11) y comprueba el archivo.

### Compatibilidad Markdown

`ui/src/markdown/compatibilidad.ts` usa el lexer de CommonMark/GFM de `marked`
en lugar de expresiones regulares. Detecta tablas, imágenes y tareas dentro de
citas y listas, imágenes de referencia y con alt anidado, y no confunde HTML
dentro de bloques o fragmentos de código.

### Vallas de código

`ui/src/editor/code-block.ts` sustituye el serializador de `CodeBlock` y elige
una valla más larga que cualquier secuencia de backticks del contenido. Se añadió
la pareja 18 del corpus con cuatro backticks y tres dentro.

### Autosave

`App.tsx` conserva el cuerpo actual del editor en un `ref` monotónico. Un
guardado en vuelo ya no limpia el pendiente si el editor cambió: al terminar,
compara el cuerpo persistido con el actual y vuelve a drenar si difieren. La
prueba de regresión usa una API diferida y comprueba que la última edición se
persiste.

### Índice y duplicados

- `Escaneo::filas_indice` y `ordenados` excluyen los documentos marcados como
  inválidos, incluidos los IDs duplicados. `buscar`, `reindexar` y `revertir`
  siguen operativos.
- `Indice::esta_vigente` compara revisión, rutas, hashes, `mtime_ns` y tamaño.
  Una edición externa hace que la siguiente búsqueda reconstruya el índice.

### E2E de escritorio

`scripts/e2e-tauri.sh` usa un puerto libre, un grupo de procesos propio,
comprueba que el driver siga vivo, limpia la app y requiere `xdotool`. El test
separa los escenarios `guardar` y `cerrar`, reintenta la escritura con una
referencia fresca y no oculta fallos de teardown. Se ejecutó 5/5 veces sin
procesos huérfanos.

### P3 y P4

- recuento de palabras inicializado desde el Markdown y con la misma definición
  alfanumérica que el núcleo;
- títulos de personaje/trama resueltos contra las notas y aviso de referencia
  desconocida;
- fecha en el panel de cuarentena;
- avisos de verificación desplegables;
- resaltado FTS con delimitadores de control, sin confundir corchetes literales;
- mensajes de demo que declaran la simulación de reversión y exportación;
- documentación de CI, HANDOFF, Markdown, E2E, marcas y cuarentena ajustada al
  estado real.

### Endurecimiento previo a seguridad

- CSP restrictiva en producción y `devCsp` para Vite.
- El hook `WRITER_E2E_PROJECT` queda detrás de la feature `e2e`; la app de
  producción no autoabre proyectos.

## 3. Evidencia

```text
cargo test --workspace --all-targets --locked   86/86
pnpm ui:test                                    55/55
pnpm e2e                                        7/7
e2e-tauri guardar + cerrar                      5/5 repeticiones
python scripts/ci.py interfaz                   OK
python scripts/ci.py producto                   OK
```

## 4. Pendiente

- Repetir QA independiente sobre el commit congelado de esta iteración.
- Security reviewer, release engineer, UX/accesibilidad/rendimiento y product
  owner.
- Validación externa: AppImage, firma, Windows/macOS, CI remoto y app instalada.
