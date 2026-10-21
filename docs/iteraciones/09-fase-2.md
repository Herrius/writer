# Iteración 09 — Cierre de la Fase 2

- **Fecha:** 2026-10-21
- **Alcance:** toolbar de formato, recuento de palabras, E2E real de escritorio y
  cierre local de la Fase 2.
- **Estado:** Fase 2 cerrada en local Linux. Validación externa pendiente.

## 1. Toolbar y recuento

- Toolbar sobre el editor con:
  - negrita, cursiva y tachado;
  - título de sección (H2);
  - cita;
  - lista de viñetas y lista ordenada;
  - bloque de código;
  - marcas de personaje, trama y comentario;
  - quitar marca.
- Cada botón refleja el estado activo de la selección.
- Recuento de palabras en la propia toolbar, actualizado con el documento.
- Los controles se ocultan en solo lectura.

## 2. E2E de escritorio

- `e2e-tauri/tauri.test.mjs` usa `tauri-driver` 2.0.6 + `WebKitWebDriver`.
- `scripts/e2e-tauri.sh` crea un proyecto temporal, arranca la app real, escribe,
  guarda con `Ctrl+S` y verifica el Markdown en disco.
- Hook de prueba `WRITER_E2E_PROJECT`: solo abre un proyecto automáticamente si
  la variable de entorno existe.
- Gate `python scripts/ci.py e2e-tauri`, Linux-only.
- El E2E de navegador subió a 7 flujos (se añadió formato/recuento).

## 3. Cierre de la Fase 2

La Fase 2 queda cerrada en local Linux con:

- motor + CLI;
- Tauri + React/Tiptap;
- autosave, conflictos y recuperación;
- búsqueda FTS5, historial y reversión;
- DOCX;
- cuarentena;
- marcas propias;
- toolbar y recuento;
- 7 E2E de navegador + 1 E2E de escritorio;
- empaquetado deb/rpm con verificación estructural.

## 4. Evidencia

```text
cargo test --workspace --all-targets --locked   84/84
pnpm ui:test                                    50/50
pnpm e2e                                        7/7
pnpm e2e:tauri                                  1/1
python scripts/ci.py e2e-tauri                  OK
python scripts/ci.py empaquetado                OK
cargo deny                                      OK
```

## 5. Pendiente de validación externa

- AppImage (`linuxdeploy`).
- Firma de código y notarización.
- Windows (WebView2) y macOS.
- CI remoto con todos los gates.
- Prueba visual/uso real de la app instalada.
