# Importación y backup v1 — contrato

- **Fecha:** 2026-10-23
- **Estado:** propuesta normativa para la Fase 2.5.
- **Base:** [`esquema-v2.md`](esquema-v2.md) §7.

## 1. Alcance

Importar documentos y backups locales; exportar y restaurar sin red. No hay
Google Docs, email ni servicios externos.

## 2. Formatos de importación

| Formato | Prioridad | Destino |
|---|---|---|
| DOCX | Alta | Escenas o notas |
| Markdown | Alta | Escenas o notas |
| HTML | Media | Escenas o notas, sin HTML crudo |
| RTF | Media | Texto plano / Markdown permitido |

La importación puede ser total o parcial:

- total: crea o reemplaza un proyecto;
- parcial: añade documentos a un proyecto existente.

## 3. Límites

- Tamaño máximo por archivo: 20 MB; total por importación: 200 MB.
- DOCX: máximo 5.000 párrafos y 1.000 imágenes (las imágenes no se importan en
  v1; se avisa).
- HTML/RTF: máximo 2 millones de caracteres.
- Profundidad de listas y citas: 20 niveles.
- Se rechazan archivos vacíos, binarios inesperados y comprimidos anidados.

Los límites son configurables solo por el product owner y se prueban con
fixtures.

## 4. Flujo

1. Elegir archivo(s) con diálogo nativo.
2. Detectar formato por extensión y contenido; no confiar solo en la extensión.
3. Validar tamaño, codificación y estructura.
4. Convertir al subconjunto Markdown permitido.
5. Mostrar previsualización y destino (proyecto nuevo o existente).
6. Confirmar.
7. Escribir mediante las transacciones del motor.
8. Si algo falla, no se toca el proyecto; el archivo se rechaza o se pone en
   cuarentena.

## 5. DOCX

- Se lee OOXML con un parser local.
- Se conservan párrafos, encabezados, énfasis, listas, citas y enlaces.
- Tablas, imágenes, notas al pie, comentarios y macros se convierten en texto o
  se rechazan con aviso; nunca se ejecutan.
- Un DOCX corrupto no se abre parcialmente.
- Se rechazan zip bombs, rutas `..` y relaciones externas.

## 6. HTML y RTF

- HTML se parsea y se convierte al subconjunto permitido; `<script>`, estilos,
  iframes y URLs externas se descartan con aviso.
- RTF se convierte a texto/Markdown básico; no se admiten objetos incrustados.
- No se descargan recursos remotos.

## 7. Backup

- El backup es un ZIP local del proyecto.
- Incluye `proyecto.yml`, `manuscrito/`, `notas/` y `.escritor/` con
  `objetos/`, `operaciones.log`, `comentarios/`, `metas/`, `sesiones/` y
  `planificacion/`.
- Excluye `bloqueo`, `indice.sqlite` y `cuarentena/`.
- La restauración valida manifiesto, hashes, rutas y versión de extensiones.
- Nunca sobrescribe un proyecto abierto; restaura en una carpeta nueva o con
  confirmación explícita.
- Un backup con `..`, enlaces simbólicos o rutas absolutas se rechaza antes de
  escribir.

## 8. UI

- Diálogo de importación con formato detectado, destino y avisos.
- Progreso para archivos grandes; cancelación segura.
- Resultado con documentos creados, rechazados y motivos.
- No hay controles de nube, email ni Google Docs.

## 9. Pruebas

- DOCX/Markdown/HTML/RTF válidos.
- DOCX corrupto, zip bomb, ruta `..`, HTML con script y RTF con objeto.
- Importación total y parcial.
- Backup/restore round-trip con comentarios, metas y planificación.
- Restauración no sobrescribe un proyecto abierto.
- Límites de tamaño y cancelación.
