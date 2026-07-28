# RFC-0003: Traducción Multi-Target de Kyle UI

**Status:** Implementado (IR+Backend layer) — Actualizado 2026-07-11
**Date:** 2026-07-10 (creado)
**Documentaci&oacute;n relacionada:**
- [`0002-ui-architecture.md`](0002-ui-architecture.md) — Arquitectura general del framework UI
- [`0005-ui-rearchitecture-plan.md`](0005-ui-rearchitecture-plan.md) — Plan de readaptaci&oacute;n
- [`../../03-language/syntax/ui-syntax.md`](../../03-language/syntax/ui-syntax.md) — Sintaxis .kyx
- [`../../06-compiler/wasm.md`](../../06-compiler/wasm.md) — Target WASM

---

## 1. Visión General

El framework Kyle UI debe traducir el código del usuario a **múltiples targets**
sin que el usuario tenga que escribir HTML, CSS o JavaScript. Un solo código fuente
(`.ky` + `.kyx`) se compila a diferentes plataformas.

### 1.1 Principios

| Principio | Implicación |
|-----------|-------------|
| **Zero JS/HTML/CSS** | El usuario nunca escribe JS, HTML o CSS. Todo es Kyle. |
| **Traducción, no wrapper** | El framework traduce código Kyle a código nativo de cada plataforma, no solo envuelve APIs. |
| **Un código, N targets** | La misma app se compila para web, desktop y mobile sin cambios. |
| **Tipado en todas las capas** | Los estilos, layouts y temas son tipos Kyle, no strings. |

### 1.2 Qué se traduce

| Qué | Origen | Web | Desktop | Mobile |
|-----|--------|:---:|:-------:|:------:|
| Lógica de negocio | `.ky` | WASM | Nativo | Nativo |
| Declaración UI | `.kyx` | JS/DOM | Skia | Native UI |
| Estilos | `style<>` | Inline styles | Skia paint | Native style |
| Eventos | `click=@fn` | addEventListener | Event loop | Touch/click |
| Estado | `bind=@var` | Reactive JS | State mgr | State mgr |

---

## 2. Arquitectura del Traductor

### 2.1 Pipeline de compilación web (actualizado con UI-IR)

Para el target web, el pipeline ahora tiene una **capa intermedia agnóstica (UI-IR)**:

```
┌──────────────────────────────────────────────────────┐
│                    source.kyx                        │
│  <view>                                              │
│    @(count: i32)                                     │
│    <button text=@count.to_str() click=@incr />      │
│  </view>                                             │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│              kyc_ui (parser .kyx)                    │
│  Transforma XML a UI-IR (ir.rs)                     │
└──────────────────────┬───────────────────────────────┘
                       │
                       ▼
┌──────────────────────────────────────────────────────┐
│              UI-IR (UiProgram)                       │
│  Agnóstico, tipado, reutilizable por todos backends │
└──────────────────────┬───────────────────────────────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
┌───────────────────┐  ┌───────────────────┐
│  VÍA 1: LÓGICA   │  │  VÍA 2: UI       │
│  (código Kyle)   │  │  (UI-IR)          │
├───────────────────┤  ├───────────────────┤
│  Semantic + HIR   │  │  Web Backend     │
│  + MIR + SSA      │  │  (backend/web)   │
├───────────────────┤  ├───────────────────┤
│  LLVM Backend     │  │  UI-IR → JS (ESM)│
│  (wasm32 target)  │  │  + HTML auto-gen │
├───────────────────┤  ├───────────────────┤
│  app.wasm         │  │  ui-runtime.js   │
│  (lógica + FFI)   │  │  (ES modules)    │
└────────┬──────────┘  └────────┬──────────┘
         │                      │
         └──────────┬───────────┘
                    ▼
┌──────────────────────────────────────┐
│         Navegador (index.html)       │  ← auto-generado
│                                      │
│  <script type="module">              │
│    import { render } from            │
│      './target/debug/ui-runtime.js'  │
│  </script>                           │
└──────────────────────────────────────┘
```

### 2.2 Comunicación WASM ↔ JS

El WASM contiene la lógica de negocio. El JS contiene la UI. Se comunican vía FFI:

```kyle
# Generado automáticamente por el compilador
@link "js"
extern fn dom_create_element(tag: ptr) ptr
extern fn dom_set_text(el: ptr, text: ptr)
extern fn dom_add_event(el: ptr, event: ptr, callback: ptr)
extern fn dom_set_style(el: ptr, prop: ptr, value: ptr)
extern fn dom_append_child(parent: ptr, child: ptr)
extern fn dom_remove_child(parent: ptr, child: ptr)

@link "js"
extern fn js_console_log(msg: ptr)
```

El JS glue runtime (generado automáticamente) implementa estas funciones:

```javascript
// ui-runtime.js (generado por el compilador)
const env = {
  dom_createElement: (tagPtr) => {
    const tag = readStr(wasmMemory, tagPtr);
    return createElement(tag);
  },
  dom_setText: (elPtr, textPtr) => {
    const el = resolvePtr(elPtr);
    el.textContent = readStr(wasmMemory, textPtr);
  },
  dom_addEvent: (elPtr, eventPtr, callbackPtr) => {
    const el = resolvePtr(elPtr);
    const event = readStr(wasmMemory, eventPtr);
    el.addEventListener(event, () => {
      // Callback al WASM
      wasm.exports.handleEvent(callbackPtr);
    });
  }
};
```

### 2.3 Traductor .kyx → JS (no WASM)

Para el target web, el compilador genera un **segundo archivo** además del `.wasm`:
`ui-runtime.js`. Este archivo contiene:

1. **Declaración de componentes** — Cada componente .kyx se traduce a una función JS
2. **Sistema de estilos** — Los `style<>` se compilan a objetos de estilo inline
3. **Event handlers** — Los eventos se conectan a funciones exportadas por WASM
4. **Reactivity** — El binding bidireccional se implementa con setters/getters JS

```javascript
// ui-runtime.js generado automáticamente
// Componente: button.kyx
function Button(props) {
  const el = document.createElement('button');
  el.textContent = props.text;
  el.className = props.style || '';
  
  if (props.click) {
    el.addEventListener('click', () => {
      // Llama a la función en WASM
      wasm.exports[props.click]();
    });
  }
  
  // Aplicar estilos del theme
  applyStyle(el, props.theme);
  
  return el;
}
```

---

## 3. Sistema de Estilos Tipado (sin CSS)

### 3.1 Tipos de estilo en Kyle

Los estilos NO son CSS. Son **tipos Kyle** con propiedades fuertemente tipadas:

```kyle
final class Style:
    background: Color
    color: Color
    font_size: i32       # en píxeles
    font_weight: FontWeight  # enum: Light, Normal, Bold, Black
    padding: Spacing     # (top, right, bottom, left)
    margin: Spacing
    border: Border
    border_radius: f32
    opacity: f32         # 0.0 a 1.0
    shadow: Shadow
    cursor: Cursor       # enum: Pointer, Default, Text, Wait
    display: Display     # enum: Flex, Grid, None
    flex_direction: FlexDirection
    align_items: Alignment
    justify_content: Alignment
    gap: i32
    width: Length        # enum: Px(i32), Percent(f32), Auto
    height: Length
    min_width: Length
    max_width: Length
    overflow: Overflow   # enum: Visible, Hidden, Scroll
    position: Position   # enum: Static, Relative, Absolute, Fixed
    top: Length
    left: Length
    right: Length
    bottom: Length
    z_index: i32
    transform: Transform
    transition: Transition
```

### 3.2 Declaración de estilos

```kyle
style<button> Primary:
    background = Color("#0066FF")
    color = Color("#FFFFFF")
    font_size = 14
    font_weight = FontWeight.Bold
    padding = Spacing(12, 24, 12, 24)
    border_radius = 8
    cursor = Cursor.Pointer

style<button> PrimaryHover: Primary:
    background = Color("#0052CC")

style<button> PrimaryDisabled: Primary:
    opacity = 0.5
    cursor = Cursor.Default
```

### 3.3 Compilación de estilos por target

```kyle
style<button> Primary:
    background = Color("#0066FF")
    border_radius = 8
    padding = Spacing(12, 24, 12, 24)
```

| Target | Salida |
|--------|--------|
| **Web** | `style="background:#0066FF;border-radius:8px;padding:12px 24px"` |
| **Desktop (Skia)** | `skia_draw_rounded_rect(x, y, w, h, 8)` + `skia_draw_text(x+12, y+24, text)` |
| **Mobile** | `android: background="#0066FF"` o `ios: view.layer.cornerRadius = 8` |

### 3.4 Theme system

```kyle
theme AppTheme:
    # Colores semánticos
    primary = Color("#0066FF")
    primary_dark = Color("#0044CC")
    accent = Color("#FF6600")
    background = Color("#FFFFFF")
    surface = Color("#F5F5F5")
    text_primary = Color("#000000")
    text_secondary = Color("#666666")
    error = Color("#FF0000")
    success = Color("#00CC66")

    # Tipografía
    font_family = "Inter, sans-serif"
    font_size_small = 12
    font_size_body = 14
    font_size_title = 20
    font_size_header = 28

    # Spacing
    spacing_xs = 4
    spacing_sm = 8
    spacing_md = 16
    spacing_lg = 24
    spacing_xl = 32

    # Breakpoints
    breakpoint_sm = 640    # móvil
    breakpoint_md = 1024   # tablet
    breakpoint_lg = 1280   # desktop
```

### 3.5 Responsive

```kyle
style<view> ResponsiveCard:
    # Mobile first: columna
    flex_direction = FlexDirection.Column
    width = Length.Percent(100)
    padding = Spacing(8)

    # Tablet: row con más padding
    @media(min_width: 640):
        flex_direction = FlexDirection.Row
        padding = Spacing(16)

    # Desktop: row ancho con max-width
    @media(min_width: 1024):
        max_width = Length.Px(800)
        margin = Spacing.AUTO  # centrado horizontal
```

---

## 4. Ciclo de Vida del Componente

### 4.1 Estados

```
  ┌──────────┐
  │ CREATED  │  ← constructor / new()
  └────┬─────┘
       │
  ┌────▼─────┐
  │ MOUNTED  │  ← appended to DOM / parent
  └────┬─────┘
       │
  ┌────▼─────┐
  │ UPDATED  │  ← props changed (re-render)
  └────┬─────┘
       │ (repite)
       │
  ┌────▼─────┐
  │ UNMOUNTED│  ← removed from DOM
  └──────────┘
```

### 4.2 Hooks

```kyle
final class Counter:
    count: i32 = 0

    fn on_created(this):
        # Se llama una vez, después del constructor
        print("componente creado")

    fn on_mounted(this):
        # Se llama cuando el componente se monta en el DOM/padre
        print("componente montado")

    fn on_updated(this, changed: {str}):
        # Se llama cuando cambian las props
        # changed = conjunto de props que cambiaron
        if changed.contains("count"):
            print("count cambió a " + this.count.to_str())

    fn on_unmounted(this):
        # Se llama cuando el componente se desmonta
        print("componente desmontado")
        # cleanup: timers, listeners, etc.

    fn on_error(this, error: str):
        # Error boundary: captura errores en hijos
        print("error: " + error)
```

---

## 5. Componentes Built-in

### 5.1 Contenedores

| Componente | Propósito | Web | Desktop |
|-----------|-----------|:---:|:-------:|
| `<view>` | Contenedor base | `div` | Skia container |
| `<column>` | Columna flex | `div flex-col` | Flex layout |
| `<row>` | Fila flex | `div flex-row` | Flex layout |
| `<grid>` | Grid | `div grid` | Grid layout |
| `<scroll>` | Scroll | `div overflow` | Clip + scroll |
| `<card>` | Tarjeta | `div card` | Rounded rect |
| `<spacer>` | Espacio flexible | `div flex-grow` | Flex grow |

### 5.2 Input

| Componente | Propósito | Web | Desktop |
|-----------|-----------|:---:|:-------:|
| `<text>` | Texto estático | `span` | `draw_text` |
| `<label>` | Etiqueta | `label` | `draw_text` |
| `<button>` | Botón | `button` | Skia button |
| `<text_field>` | Campo texto | `input type=text` | Input field |
| `<password_field>` | Contraseña | `input type=password` | Input field |
| `<checkbox>` | Checkbox | `input type=checkbox` | Checkbox |
| `<radio>` | Radio button | `input type=radio` | Radio |
| `<switch>` | Toggle | `input type=checkbox switch` | Switch |
| `<slider>` | Rango | `input type=range` | Slider |
| `<select>` | Dropdown | `select` | Dropdown |
| `<textarea>` | Texto multi-línea | `textarea` | Text area |

### 5.3 Navegación

| Componente | Propósito |
|-----------|-----------|
| `<tab>` | Pestaña individual |
| `<tabs>` | Contenedor de pestañas |
| `<nav>` | Barra de navegación |
| `<appbar>` | App bar superior |
| `<bottombar>` | Barra inferior |
| `<drawer>` | Menú lateral |
| `<menu>` | Menú desplegable |
| `<menuitem>` | Item de menú |

### 5.4 Feedback

| Componente | Propósito |
|-----------|-----------|
| `<dialog>` | Modal |
| `<popup>` | Popup contextual |
| `<tooltip>` | Tooltip |
| `<snackbar>` | Notificación temporal |
| `<progress>` | Barra de progreso |
| `<spinner>` | Cargando |
| `<skeleton>` | Skeleton loading |

### 5.5 Multimedia

| Componente | Propósito |
|-----------|-----------|
| `<image>` | Imagen |
| `<icon>` | Icono SVG/material |
| `<canvas>` | Canvas 2D |

---

## 6. Seguridad

### 6.1 Prevención de XSS

El traductor JS nunca genera `innerHTML`. Siempre usa métodos seguros del DOM:

```javascript
// ❌ NUNCA: innerHTML
// element.innerHTML = userInput;

// ✅ SIEMPRE: createElement + textContent
const el = document.createElement('div');
el.textContent = userInput;  // automáticamente escapado
```

### 6.2 Validación de tipos

Los estilos y props son tipados en Kyle. El compilador verifica en tiempo de
compilación que los valores sean válidos:

```kyle
# ❌ Error de compilación: "rojo" no es un Color
style<button> Invalid:
    background = "rojo"

# ✅ Correcto:
style<button> Valid:
    background = Color("#FF0000")
```

---

## 7. Glosario

| Término | Significado |
|---------|-------------|
| Traductor JS | Generador de código JavaScript a partir de AST Kyle (solo para target web) |
| JS glue | Código JS que conecta WASM con el DOM |
| ui-runtime.js | Archivo JS generado que contiene los componentes UI traducidos |
| Style type | Tipo Kyle que define propiedades de estilo (no CSS) |
| Reactive binding | Actualización automática de la UI cuando cambia el estado |
| FFI bridge | Comunicación entre WASM (lógica) y JS (UI) |

---

## 8. Referencias

- [RFC-0002: Kyle UI Framework](0002-ui-architecture.md) — Arquitectura general
- [UI Syntax](../03-language/syntax/ui-syntax.md) — Sintaxis .kyx
- [WASM Target](../../06-compiler/wasm.md) — Especificación WASM
- [FFI ABI](../../03-language/ffi/abi.md) — extern fn + @link
