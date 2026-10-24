# Rendimiento local — v1

- **Fecha:** 2026-10-24
- **Estado:** presupuestos automatizados para 1.000 escenas y 1.000 marcas.
- **Ámbito:** Linux x86_64, build de depuración; no es un benchmark de
  producción.

## 1. Escenario de 1.000 escenas

Prueba permanente: `crates/core/tests/rendimiento.rs`.

No depende de un fixture versionado: crea un proyecto temporal, escribe 1.000
escenas válidas y mide:

| Operación | Presupuesto | Medición local (debug) |
|---|---:|---:|
| `verificar` | 20 s | 25 ms |
| `listar` | 10 s | 29 ms |
| `reindexar` | 20 s | 79 ms |
| `buscar("faro", 30)` | 10 s | 31 ms |

Los presupuestos son deliberadamente amplios para detectar regresiones graves en
máquinas lentas, no para comparar hardware. La prueba corre en
`cargo test --workspace`.

## 2. Escenario de 1.000 marcas

Prueba permanente: `ui/src/editor/marcas.test.ts`.

Se construye un párrafo con 1.000 marcas de personaje, se parsea y se serializa
con Tiptap. La prueba exige:

- que el texto final conserve la marca 1.000;
- que el ciclo completo termine en menos de 5 s en jsdom.

La medición sirve como detección de bucles o degradación cuadrática. No mide la
latencia de tecleo en un WebView real; la observación anterior de ~2,2–2,4× con
1.000 marcas en una sola escena sigue siendo una limitación conocida de
experiencia, no un bloqueo funcional.

## 3. Búsqueda e historial

- La búsqueda usa el índice FTS5 y se reconstruye si el proyecto no tiene
  `.escritor` o si el índice está corrupto.
- El historial es append-only y no se compacta en v1; la prueba de 1.000 escenas
  no genera 1.000 revisiones, solo mide el escaneo, el listado, el índice y la
  consulta.
- No se promete un presupuesto de arranque de la ventana Tauri: depende de
  WebKitGTK/WebView2 y del tamaño de `ui/dist`.

## 4. Criterios de regresión

Se considera regresión bloqueante:

- que la prueba de 1.000 escenas supere los presupuestos de la tabla;
- que la prueba de 1.000 marcas no termine o supere 5 s;
- que `buscar` deje de devolver el límite solicitado;
- que `verificar` o `listar` cambien el número de documentos.

## 5. Límites

- No hay `criterion` ni comparación estadística entre commits.
- Las cifras son de una máquina Linux concreta y build debug.
- No se mide consumo de memoria ni latencia de tecleo real.
- Los proyectos en red (SMB/NFS) siguen fuera del alcance de escritura.
