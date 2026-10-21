# Cuarentena — contrato

- **Fecha:** 2026-10-21
- **Estado:** vigente para la Fase 2
- **Ámbito:** `.escritor/cuarentena/`

La cuarentena es una **copia de diagnóstico**: aparta archivos inválidos sin
modificar ni borrar los originales. El proyecto sigue siendo la fuente de verdad
y `verify` continúa reportando los mismos hallazgos.

## 1. Qué se copia

`verify --cuarentena` (CLI) copia cada archivo inválido del escaneo:

```text
.escritor/cuarentena/<AAAAMMDDTHHMMSSZ>-<slug>-<motivo>.md
.escritor/cuarentena/informe.jsonl
```

Cada línea de `informe.jsonl` tiene:

```json
{"ts":"...","ruta_original":"manuscrito/...","motivo":"...","hash":"blake3:..."}
```

## 2. Comandos

```bash
writer verify --cuarentena          # copia los inválidos (ya existía)
writer quarantine list ~/novela
writer quarantine list ~/novela --json
writer quarantine discard <archivo> ~/novela
```

- `list` cruza el directorio con `informe.jsonl` y muestra archivo, fecha,
  ruta original, motivo y tamaño.
- `discard` elimina **solo la copia** de cuarentena; nunca toca el original.
- Si `informe.jsonl` falta o está incompleto, los archivos se listan con
  metadatos desconocidos; no se inventan.

## 3. UI

- El panel de diagnóstico del proyecto muestra una sección **Cuarentena** cuando
  hay copias.
- Cada entrada muestra archivo, motivo, ruta original y fecha.
- **Descartar** pide confirmación en dos pasos y elimina la copia.
- La UI nunca modifica los archivos originales ni la cuarentena sin acción
  explícita.

## 4. Reglas

- La cuarentena no se indexa ni entra en la búsqueda.
- `reindex` no la toca.
- Descartar una copia no resuelve el error de `verify`; el original sigue
  inválido.
- La cuarentena no crece automáticamente: solo `verify --cuarentena` copia.

## 5. Pruebas

```bash
cargo test -p writer-core --test proyecto cuarentena
cargo test -p writer-cli --test cli quarantine
pnpm ui:test
```
