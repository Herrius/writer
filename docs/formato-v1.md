# Formato, preferencias y paleta — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado en la Fase 3.3.
- **Ámbito:** preferencias de interfaz en el equipo local; el proyecto no las
  sincroniza.

## 1. Preferencias de escritura

Se guardan en `localStorage` con la clave `writer:preferencias:v1`. No se
escriben en el proyecto ni se envían a ningún servicio.

```json
{
  "version": 1,
  "tema": "claro | sepia | oscuro",
  "fuente": "serif | sans | mono",
  "tamano": 18,
  "interlineado": 1.85,
  "espaciadoParrafo": 1.05,
  "corrector": true,
  "modoEnfoque": false
}
```

Límites y normalización:

| Campo | Rango | Defecto |
|---|---|---:|
| `tamano` | 12–32 px | 18 |
| `interlineado` | 1,2–2,6 | 1,85 |
| `espaciadoParrafo` | 0–2,5 em | 1,05 |
| `tema` | claro, sepia, oscuro | claro |
| `fuente` | serif, sans, mono | serif |

Un JSON corrupto, una versión desconocida o un valor fuera de rango se
normaliza a los valores admitidos. Si `localStorage` no está disponible, las
preferencias viven solo en memoria durante la sesión.

Las preferencias se aplican con variables CSS en `document.documentElement`:

```text
--editor-font-family
--editor-font-size
--editor-line-height
--editor-paragraph-gap
data-tema="claro|sepia|oscuro"
```

El formato del PDF no se duplica aquí: sigue en `.escritor/formato.json` y se
edita desde el panel **Notas** (fuente, interlineado y margen).

## 2. Temas y tipografía

- **Claro:** papel blanco y tinta oscura.
- **Sepia:** papel cálido, pensado para lectura prolongada.
- **Oscuro:** superficies oscuras y texto claro; el editor, el diff y los
  paneles mantienen contraste.

La tipografía, el tamaño, el interlineado y el espacio entre párrafos se aplican
al editor y a la vista de escritura. No cambian el Markdown guardado ni el DOCX.

## 3. Modo enfoque

Oculta las barras laterales y deja el editor centrado. Se activa desde
**Preferencias** o con `Alt+Z`; `Escape` también sale cuando no hay un diálogo
abierto. Es una preferencia local y sobrevive a la reapertura de la app.

## 4. Atajos

| Atajo | Acción |
|---|---|
| `Ctrl+S` / `Cmd+S` | Guardar ahora |
| `Ctrl+O` / `Cmd+O` | Abrir proyecto |
| `Ctrl+F` / `Cmd+F` | Buscar en el proyecto |
| `Ctrl+P` / `Cmd+P` | Quick Open: abrir documento |
| `Ctrl+Shift+P` / `Cmd+Shift+P` | Paleta de comandos |
| `Ctrl+Shift+H` / `Cmd+Shift+H` | Historial |
| `Alt+Z` | Alternar modo enfoque |
| `Alt+E` | Volver al editor |
| `Escape` | Cerrar paleta/diálogo o salir del enfoque |

## 5. Quick Open y paleta de comandos

- **Quick Open** lista los documentos del proyecto y filtra por título o ruta.
- **Paleta de comandos** ofrece guardar, buscar, historial, cambios propuestos,
  preferencias, temas, modo enfoque y exportaciones.
- Navegación con `↑`/`↓`, apertura con `Enter` y cierre con `Escape`.
- El foco vuelve al elemento anterior al cerrar. Los diálogos son modales
  (`aria-modal="true"`) y tienen una trampa de foco básica.

## 6. Corrector ortográfico

- El editor activa el corrector nativo del WebView (`spellcheck`) cuando la
  preferencia está activa.
- No hay diccionario propio empaquetado, tesauro ni gramática local en v1.
- El corrector nativo no envía texto a writer; el comportamiento de la
  plataforma depende de WebViewGTK/WebView2 y de los diccionarios instalados.

## 7. Accesibilidad

- Etiquetas explícitas en selects, sliders, switches y diálogos.
- `role="dialog"`, `aria-modal`, `aria-labelledby` y cierre con `Escape`.
- Foco inicial en el diálogo o en el buscador de la paleta.
- El modo oscuro mantiene contraste de texto y foco visible.
- Las preferencias no dependen de arrastrar ni de gestos.

## 8. Límites de v1

- Las preferencias son por perfil del WebView, no por proyecto.
- No hay temas personalizados ni importación/exportación de preferencias.
- No se incluye un motor de gramática o sinónimos offline.
- La paleta no ejecuta comandos que requieran un diálogo nativo sin cerrarse
  primero; las exportaciones abren el diálogo del sistema.
