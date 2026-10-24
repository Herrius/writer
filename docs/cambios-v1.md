# Control de cambios local — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado en la Fase 3.1.
- **Ámbito:** local/offline. No hay cuentas, servidores ni sincronización.

## 1. Principio

El documento Markdown sigue siendo la fuente de verdad. Un cambio propuesto **no
modifica el manuscrito**: se guarda en un sidecar y solo se aplica cuando el
usuario lo acepta. Aceptar usa la misma transacción, CAS y revisión que un
guardado normal; rechazar solo elimina el sidecar.

Esto permite revisar un cambio completo sin introducir HTML de sugerencias en el
Markdown ni romper el corpus golden del editor.

## 2. Almacenamiento

Ruta:

```text
.escritor/cambios/<ULID>.json
```

Un cambio por documento. Si se vuelve a guardar una propuesta para el mismo
documento, se actualiza el sidecar existente mientras su `baseHash` siga
coincidiendo. Si el documento cambió por fuera, la propuesta anterior se
descarta y se crea una nueva con la base actual.

```json
{
  "version": 1,
  "id": "01K...",
  "documento": "01K...",
  "ruta": "manuscrito/01-libro/01-capitulo/001-escena.md",
  "autor": "local",
  "ts": "2026-10-24T10:00:00Z",
  "baseRevision": 12,
  "baseHash": "blake3:...",
  "base": "Cuerpo actual sin frontmatter.\n",
  "propuesta": "Cuerpo propuesto sin frontmatter.\n"
}
```

Reglas:

- `id` y `documento` son ULID válidos.
- `ruta` cumple las mismas reglas que una ruta de documento del proyecto.
- `version` debe ser `1`.
- `autor` no vacío, máximo 200 caracteres.
- `ts` es RFC 3339.
- `baseHash` es un hash del esquema (`blake3:` + 64 hex) del archivo completo,
  no solo del cuerpo.
- Cada cuerpo está limitado a 4 MiB.
- `.escritor/cambios/` debe ser un directorio real; se rechazan enlaces.
- Un sidecar ilegible o inválido se aparta a `.escritor/cuarentena/` y no
  bloquea el proyecto.

## 3. Flujo

1. El usuario activa **Proponer cambios**.
2. Edita el documento.
3. El autosave llama a `registrar_cambio` en vez de escribir el archivo.
4. El panel **Cambios** muestra el diff entre `base` y `propuesta`.
5. **Aceptar** verifica que el archivo siga teniendo `baseHash`, escribe
   `propuesta` mediante una transacción y elimina el sidecar.
6. **Rechazar** elimina el sidecar y deja el documento intacto.

Si el editor cambia mientras una propuesta está en vuelo, se conserva la versión
más reciente y se vuelve a registrar; nunca se declara guardado un texto que no
sea el último del editor.

## 4. Concurrencia y conflictos

- Registrar un cambio toma el lock del proyecto y lee la base con el mismo
  escaneo que un guardado.
- Aplicar compara `baseHash` con el hash actual del archivo. Si no coincide,
  devuelve conflicto y no escribe.
- No hay fusión automática entre una propuesta y una edición externa. El usuario
  debe rechazar/recargar y volver a proponer.
- La ventana CAS conocida de v1 sigue aplicando entre la comprobación y el
  reemplazo; los procesos cooperativos quedan protegidos por el lock.

## 5. Diff

El detalle incluye líneas con tipo `igual`, `insertada` o `borrada`, número de
línea en cada lado y texto. El contrato del algoritmo está en
[`diff-v1.md`](diff-v1.md).

La vista es lado a lado. Aceptar y rechazar operan sobre el cambio completo; no
hay aceptación por fragmento en v1.

## 6. API

Núcleo:

- `Project::listar_cambios`
- `Project::leer_cambio`
- `Project::registrar_cambio`
- `Project::aplicar_cambio`
- `Project::descartar_cambio`
- `Project::comparar_documento`

Tauri:

- `listar_cambios(ruta)`
- `leer_cambio(ruta, id)`
- `registrar_cambio(ruta, objetivo, cuerpo, autor)`
- `aplicar_cambio(ruta, id)`
- `descartar_cambio(ruta, id)`
- `comparar_documento(ruta, objetivo, revision)`

## 7. Límites de v1

- No hay sugerencias inline ni aceptación por hunk: el cambio es el documento
  completo.
- Un solo cambio pendiente por documento; el más reciente sustituye al anterior.
- No hay autoría multiusuario ni firmas; `autor` es el nombre local del proyecto
  o `local`.
- No se registra un cambio si la propuesta coincide con la base; se elimina la
  propuesta anterior.
- El sidecar se elimina después del commit. Si el borrado fallara, el cambio
  quedaría visible pero su `baseHash` ya no coincidiría y no podría aplicarse
  otra vez.
