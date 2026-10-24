# Lectura en voz alta y herramientas de escritura — v1

- **Fecha:** 2026-10-24
- **Estado:** implementado en la Fase 3.4.
- **Ámbito:** local/offline; usa las voces instaladas en el sistema operativo.

## 1. Lectura en voz alta

La UI usa la Web Speech API del WebView (`speechSynthesis` y
`SpeechSynthesisUtterance`). No hay servicios de voz, cuentas ni envío de texto.

- **Leer documento:** convierte el Markdown visible en texto plano y lo lee.
- **Leer selección:** lee el texto seleccionado en el editor.
- **Pausar / Reanudar / Detener:** controlan la síntesis.
- **Voz:** se elige entre las voces expuestas por el sistema; se prefiere una
  voz `es-*` cuando existe.
- **Velocidad:** 0,5×–2,0×.
- La lectura se detiene al cambiar de documento.

Si el WebView no expone `speechSynthesis`, el panel muestra un aviso y desactiva
los controles. Si no hay voces instaladas, también se informa; no se descarga
ninguna voz.

### Texto de lectura

El texto se obtiene de `textoSinMarcas` y se limpia de:

- bloques y fragmentos de código;
- imágenes y destinos de enlaces;
- encabezados, citas y énfasis;
- etiquetas HTML.

Las marcas propias se conservan como su texto visible. La limpieza no modifica
el documento.

## 2. Estadísticas de escritura

En el panel **Herramientas**:

- palabras del documento;
- caracteres del texto visible;
- párrafos separados por línea en blanco;
- tiempo estimado de lectura a 200 palabras/minuto;
- palabras de la selección actual.

El recuento de palabras usa la misma definición que el motor
(`contarPalabras`): secuencias maximales de letras o números Unicode.

## 3. Modo enfoque

Se puede activar desde **Herramientas** o con `Alt+Z`. Oculta las barras
laterales y centra el editor. `Escape` sale del enfoque si no hay un diálogo
abierto.

## 4. Corrector, diccionario y gramática

- El corrector nativo del WebView se activa o desactiva desde **Preferencias**.
- No se empaqueta un diccionario, tesauro ni motor de gramática propio en v1.
- No se envía texto a ningún servicio de corrección.
- La disponibilidad y calidad del corrector dependen de WebViewGTK/WebView2 y de
  los diccionarios instalados en el sistema.

Esta decisión evita presentar como «offline» un motor que en realidad requeriría
una API externa o un binario no auditado. Queda como línea de trabajo futura
incorporar un diccionario Hunspell local y un tesauro con licencia compatible.

## 5. Accesibilidad

- Controles con etiquetas explícitas y estados `aria-pressed`/`aria-selected`
  cuando aplica.
- El panel conserva el foco y se puede recorrer con teclado.
- Los avisos de voz no disponible son texto visible, no solo un icono.
- La lectura no depende de arrastrar ni de gestos.

## 6. Límites

- La voz y la pausa dependen de la implementación del WebView; no se garantiza
  que WebKitGTK y WebView2 expongan las mismas voces.
- No se guarda el audio, no se genera un archivo y no hay lectura en segundo
  plano al cerrar la app.
- El tiempo de lectura es una estimación, no una promesa.
- El corrector nativo puede no estar disponible en un WebView sin diccionarios.
