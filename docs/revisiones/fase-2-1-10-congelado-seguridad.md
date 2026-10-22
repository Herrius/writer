# Fase 2.1 — Congelación para seguridad

- **Fecha:** 2026-10-21
- **Commit de código:** `376c0b8a07c4381f36c56f9a0620ea150580b352`
- **HEAD del pipeline:** `79ee925714a89786bd81b8983c7ce5ff4122c868`
- **QA:** aprobada con observaciones (0 P1/P2).
- **Estado del árbol:** limpio.

## Superficie nueva a revisar

- Comando Tauri `crear_proyecto(ruta, titulo, autor, idioma, notasIniciales)`.
- Creación de carpetas y archivos en una ruta elegida por el usuario.
- Creación opcional de tres notas iniciales.
- Nuevo panel de inicio y modal de creación; `localStorage` de recientes.
- Selección de carpeta con `tauri-plugin-dialog`.
- No se añaden plugins `fs`, `shell`, `http` ni red.

## Evidencia

| Gate | Resultado |
|---|---|
| Rust | 92/92 |
| Tauri | 3/3 |
| UI | 58/58 |
| E2E navegador | 10/10 |
| Calidad / interfaz / producto | OK |
| E2E Tauri | OK en la evidencia del builder |

## Pendiente

- Security review de la Fase 2.1.
- Release, UX y product owner.
