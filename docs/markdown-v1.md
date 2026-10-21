# Markdown v1 — contrato del editor

- **Fecha:** 2026-10-20
- **Estado:** vigente para la Fase 2
- **Decisiones relacionadas:** [`adr/0001-editor-tiptap.md`](adr/0001-editor-tiptap.md)

El editor no es la autoridad del proyecto: el cuerpo Markdown va a
`writer-core`, que normaliza el salto final y conserva el frontmatter. Este
documento fija qué construcciones acepta el editor, cómo las normaliza y cuáles
se bloquean para no perder contenido.

## 1. Esquema aceptado

El editor usa `StarterKit` + `@tiptap/markdown` y acepta:

- párrafos y líneas en blanco;
- títulos ATX (`#`, `##`, `###`…);
- énfasis, negrita, negrita y cursiva, tachado;
- código en línea;
- listas de viñetas y listas ordenadas, con anidamiento;
- citas;
- bloques de código cercados, con lenguaje;
- enlaces inline y autolinks;
- separadores (`---`);
- saltos de línea duros;
- marcas propias: `[[personaje:ULID|texto]]`, `[[trama:ULID|texto]]` y
  `[[comentario:texto|texto]]` (ver [`marcas-v1.md`](marcas-v1.md)).

Las imágenes, tablas, listas de tareas, HTML embebido y notas al pie **no** están
en el esquema de la v0.2.

## 2. Corpus golden

El corpus vive en `ui/src/markdown/corpus/`. Cada caso tiene dos archivos:

```text
NN-caso.md           # entrada
NN-caso.esperado.md  # salida canónica del editor
```

La prueba `ui/src/markdown/roundtrip.test.ts` comprueba dos propiedades por
pareja:

1. `markdown → Tiptap → markdown` produce exactamente el archivo esperado;
2. volver a serializar el esperado no cambia nada (idempotencia).

Casos cubiertos: párrafos, énfasis, títulos, listas, citas, código, enlaces,
separadores, saltos, literales escapados, Unicode, documento mixto, setext,
referencias, código indentado, entidades HTML y marcas propias.

```bash
pnpm ui:test
```

Para añadir una construcción: se agrega la pareja, se ejecuta la prueba y, si la
salida no es la deseada, se decide explícitamente entre aceptar la
normalización, documentarla o bloquearla.

## 3. Marcas propias

El esquema incluye tres marcas que se serializan al Markdown:

```text
[[personaje:ULID|texto marcado]]
[[trama:ULID|texto marcado]]
[[comentario:texto del comentario|texto marcado]]
```

- `personaje` y `trama` exigen un ULID válido (primer carácter `0`–`7`).
- El texto marcado admite formato inline, pero no marcas anidadas ni `]]`.
- El comentario no puede contener `|` ni `]]`; admite corchetes simples.
- La UI permite aplicar y quitar marcas sobre la selección; aplicar una nueva
  sustituye la anterior.
- En DOCX se exporta solo el texto marcado en prosa; código y construcciones
  escapadas se conservan literales.

El contrato completo está en [`marcas-v1.md`](marcas-v1.md).

## 4. Normalizaciones aceptadas

| Entrada | Salida | ¿Se conserva el significado? |
|---|---|---|
| Título setext (`Título` + `====`) | Título ATX (`# Título`) | Sí |
| Enlace de referencia (texto y definición) | Enlace inline con la URL resuelta | Sí; desaparece la definición |
| Salto duro con barra final (`\`) | Salto con dos espacios | Sí |
| Bloque de código indentado | Bloque cercado | Sí |
| `*` / `+` como viñeta | `-` | Sí |
| Varias líneas en blanco | Una línea en blanco | Sí |
| `_`, `[`, `]`, `*`, `\` en texto normal | Escapados con `\` | Sí |
| `<`, `>`, `&` en texto normal | `&lt;`, `&gt;`, `&amp;` | Sí en Markdown |
| Autolink (`<https://…>`) | Enlace inline | Sí |
| Sin salto final | `writer-core` añade `\n` | Sí |

El escapado de literales y las entidades son una decisión del serializador de
Tiptap: el archivo sigue siendo Markdown válido y vuelve al mismo texto, pero la
representación no es byte a byte la original. Es la normalización más visible
para quien abra el `.md` fuera del editor.

## 5. Construcciones bloqueadas

`ui/src/markdown/compatibilidad.ts` detecta las construcciones que pierden
contenido y la interfaz abre el documento en **solo lectura**:

| Construcción | Qué pasaría si se editara |
|---|---|
| HTML embebido | Se descartan las etiquetas y queda solo el texto. |
| Imágenes (texto alternativo y ruta) | Se conserva el texto alternativo y se pierde la imagen. |
| Tablas | Se convierten en un párrafo y se pierden las celdas. |
| Listas de tareas | Se conserva el texto y se pierden las casillas. |
| Notas al pie | La referencia queda escapada como texto literal. |

La comprobación ignora el contenido de bloques y fragmentos de código, donde
esas marcas son literales. Un falso positivo solo abre el documento en solo
lectura; nunca lo sobrescribe.

Para editar uno de estos documentos hoy hay que usar la CLI o convertirlo a las
construcciones aceptadas. Añadir una extensión (imágenes, tablas) exige ampliar
el esquema, el corpus y este documento.

## 6. Autosave y conflictos

- El editor guarda 900 ms después de la última pulsación y con `Ctrl+S` /
  `Cmd+S`.
- Antes de guardar se envía el hash del archivo leído al abrir.
- `writer-core` compara el hash con el archivo actual dentro de la sesión de
  escritura; si cambió por fuera, devuelve `Error::Conflict` y no escribe.
- La UI muestra `conflicto`, bloquea la edición y ofrece **Reintentar**,
  **Recargar desde disco** y **Copiar mi texto**.
- Si el proyecto tiene errores de verificación, o el documento usa una
  construcción bloqueada, el editor queda en solo lectura.
- Un fallo de E/S deja los cambios pendientes y permite reintentar; un modo
  recuperación bloquea la escritura.

La ventana entre la comprobación del hash y el reemplazo sigue siendo la
documentada en [`protocolo-commit.md`](protocolo-commit.md): es
*last-writer-wins* frente a un proceso que escriba exactamente en ese intervalo.

## 7. Pruebas

```bash
pnpm ui:test        # corpus + compatibilidad + autosave/conflictos
pnpm typecheck
python scripts/ci.py interfaz
```

Las pruebas de autosave usan un editor de prueba y la API simulada; no sustituyen
al E2E de escritorio, que sigue pendiente.
