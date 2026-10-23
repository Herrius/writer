# Exportación v2 — contrato

- **Fecha:** 2026-10-23
- **Estado:** propuesta normativa para la Fase 2.5.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §7.2 y
  [`docx-v1.md`](docx-v1.md).

## 1. Formatos

| Formato | Uso | Estado |
|---|---|---|
| Markdown | Fuente, archivo completo | v1 ampliado |
| DOCX | Editorial, revisión | v1 ampliado |
| PDF/impresión | Lectura, envío | v2 |
| Formato de manuscrito | Editorial | v2 |

La exportación no modifica el proyecto ni el historial.

## 2. Markdown

- Conserva el Manuscrito y las notas en el orden de lectura.
- Conserva las marcas `[[personaje:...]]`, `[[trama:...]]` y
  `[[comentario:...]]` tal cual.
- No incluye `.escritor/`.
- Un proyecto con extensiones no cambia su Markdown al exportar.

## 3. DOCX

- Mantiene el contrato v1: párrafos, encabezados, énfasis, listas, citas,
  enlaces y separadores.
- Los comentarios no se imprimen; el texto marcado sí.
- Las @menciones se imprimen como texto visible, no como ULID.
- Las imágenes incrustadas siguen fuera de v2; se avisa si el manuscrito las
  contiene.
- Tablas, notas al pie y tareas se rechazan o se convierten en texto según
  `docx-v1.md`; nunca se pierde el proyecto.
- El DOCX se valida en Word y LibreOffice antes de declarar la fase cerrada.

## 4. PDF e impresión

- Se generan desde el manuscrito renderizado, no desde el editor.
- Respetan el formato de manuscrito configurado.
- No incluyen comentarios, sticky notes ni metadatos internos.
- El PDF es local; no se envía a ningún servicio.

## 5. Formato de manuscrito

Configuración local por proyecto:

- fuente y tamaño;
- interlineado;
- márgenes;
- sangría de primera línea;
- numeración de páginas;
- encabezado con título/autor;
- formato de capítulos y escenas.

Se aplica a PDF/impresión y, cuando sea posible, a DOCX. No cambia el Markdown.

## 6. UI

- Diálogo de exportación con formato, destino y opciones.
- Casilla «incluir notas» solo cuando aplica.
- Aviso claro de lo que no se puede exportar.
- Sin Google Docs, email ni servicios externos.

## 7. Pruebas

- Markdown/DOCX/PDF con proyecto v1 y con extensiones.
- Comentarios y @menciones no aparecen como ULID en DOCX.
- Formato de manuscrito aplicado a PDF.
- Exportación no modifica el proyecto.
- DOCX validado en Word/LibreOffice antes del cierre.
