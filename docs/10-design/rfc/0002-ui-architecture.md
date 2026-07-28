# RFC-0002: Kyle UI Framework

**Status:** Implementado (FASIS A-C) — Actualizado 2026-07-11
**Date:** 2026-07-09 (creado)
**Documentaci&oacute;n relacionada:**
- [`docs/10-design/rfc/0003-ui-translation.md`](0003-ui-translation.md) — Traducci&oacute;n multi-target (JS/WASM/nativo)
- [`docs/10-design/rfc/0005-ui-rearchitecture-plan.md`](0005-ui-rearchitecture-plan.md) — Plan de readaptaci&oacute;n multi-plataforma
- [`docs/03-language/syntax/ui-syntax.md`](../../03-language/syntax/ui-syntax.md) — sintaxis .kyx
- [`docs/07-tools/distribution.md`](../../07-tools/distribution.md) — distribuci&oacute;n multi-plataforma (Fase 0)

---

## 1. Visi&oacute;n General

Kyle UI es un sistema de UI declarativo que permite escribir una sola vez la interfaz
en `.kyx` y desplegarla en **cualquier plataforma**:

| Plataforma | Target | Pipeline |
|------------|--------|----------|
| **Web** | `wasm32-unknown-unknown` | Kyle → LLVM → WASM → DOM |
| **Windows** | `x86_64-pc-windows-gnu` | Kyle → LLVM → Win32/Skia |
| **macOS** | `aarch64-apple-darwin` | Kyle → LLVM → Cocoa/Skia |
| **Linux** | `x86_64-unknown-linux-gnu` | Kyle → LLVM → X11/Wayland/Skia |
| **Android** | `aarch64-linux-android` | Kyle → LLVM → Android NDK |
| **iOS** | `aarch64-apple-ios` | Kyle → LLVM → UIKit/SwiftUI |
| **Custom OS** | `x86_64-unknown-kyleos` | Kyle → LLVM → Kyle Graphics Stack |

El principio fundamental: **un solo lenguaje, un solo compilador, m&uacute;ltiples backends**.

### 1.1 Filosof&iacute;a de Dise&ntilde;o

| Principio | Significado |
|-----------|-------------|
| **Declarativo** | El qu&eacute;, no el c&oacute;mo. El framework optimiza el renderizado. |
| **Sin JS** | Todo el c&oacute;digo es Kyle. No hay transpilaci&oacute;n a JavaScript. |
| **FFI nativo** | Cada plataforma expone sus APIs v&iacute;a `extern fn` + `@link`. |
| **Un solo parser** | .kyx se transforma a AST de Kyle. El mismo pipeline sem&aacute;ntico + MIR + LLVM. |
| **Runtime en Kyle** | Los componentes UI se escriben en Kyle puro (como http, json, sqlite). |

---

## 2. C&oacute;mo se compila .kyx

El archivo `.kyx` no es un lenguaje nuevo. Es **az&uacute;car sint&aacute;ctico** que el compilador
transforma a Kyle est&aacute;ndar antes de pasarlo al pipeline existente.

### 2.1 Pipeline de compilaci&oacute;n

```
.kyx (XML-like)
  │
  ▼
┌──────────────────────┐
│  .kyx Parser         │  ← NUEVO: transforma XML a AST Kyle
│  (frontend/kyx.rs)   │
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│  AST Kyle            │  ← Existente: mismo AST
│  (kyc_core/ast.rs)   │
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│  Semantic + HIR      │  ← Existente
│  + MIR + SSA         │
└──────────┬───────────┘
           ▼
┌──────────────────────┐
│  LLVM Codegen        │  ← Existente: set target triple
│  (kyc_backend)       │
└──────────┬───────────┘
           ▼
    .wasm / .exe / app  ← Seg&uacute;n target
```

### 2.2 Transformaci&oacute;n XML → AST

```kyx
<button text="Login" click=@login />
```

se transforma a:

```kyle
button(text: "Login", click: login)
```

El parser .kyx debe soportar:

| Sintaxis .kyx | Equivalente Kyle |
|---------------|------------------|
| `<Componente />` | `componente()` |
| `<Comp attr=@val />` | `comp(attr: val)` |
| `<Comp attr="str" />` | `comp(attr: "str")` |
| `<Comp>hijo</Comp>` | `comp(hijo)` |
| `@expr` | `expr` |
| `@(bloque)` | Bloque Kyle embebido |
| `@if:` / `@for:` | Control flow |
| `@match:` | Pattern matching |
| `style<X> Name:` | Struct de estilo |
| `theme Name:` | Struct de tema |

### 2.3 Extensi&oacute;n de archivo

- `.kyx` = vista (componente o p&aacute;gina)
- `.ky` = l&oacute;gica opcional (el mismo componente puede tener ambos)

El module resolver debe buscar `.kyx` adem&aacute;s de `.ky`:

```
use Login.view
  → busca Login.ky (no existe)
  → busca Login.kyx (existe) → lo parsea
```

---

## 3. Roadmap de Implementaci&oacute;n

### Fase 0 — Base (ahora)

**Objetivo:** Que Kyle se instale en cualquier plataforma y el compilador soporte target triples.

| # | Tarea | Dependencia |
|---|-------|-------------|
| 0.1 | Instalador multi-plataforma (install.sh + install.ps1) | — |
| 0.2 | Flag `--target` en `ky build` | — |
| 0.3 | Linker seleccionable por target (no por host) | 0.2 |
| 0.4 | Runtime cross-compilado para cada target | 0.1 |
| 0.5 | Directorio de runtimes: `~/.ky/lib/targets/<triple>/` | 0.4 |

Documento asociado: [`docs/07-tools/distribution.md`](../../07-tools/distribution.md)

---

### Fase 1 — Parser .kyx

**Objetivo:** Poder escribir `.kyx` y compilarlo a nativo.

| # | Tarea | Dependencia |
|---|-------|-------------|
| 1.1 | Registrar `.kyx` como extensi&oacute;n conocida | — |
| 1.2 | Parser XML → AST de Kyle (nuevo crate: `kyc_ui`) | — |
| 1.3 | Module resolver busca `.kyx` adem&aacute;s de `.ky` | 1.1 |
| 1.4 | `ky build login.kyx` produce ejecutable nativo | 1.2, 1.3 |
| 1.5 | Soporte de fragmentos `@(...)` con c&oacute;digo Kyle | 1.2 |
| 1.6 | Soporte de `@if`, `@for`, `@match` en .kyx | 1.5 |
| 1.7 | Soporte de `style<>`, `layout<>`, `tpl<>`, `theme` | 1.5 |
| 1.8 | Soporte de slots y componentes hijos | 1.4 |

---

### Fase 2 — WebAssembly

**Objetivo:** Compilar .kyx a WASM y correr en el navegador.

| # | Tarea | Dependencia |
|---|-------|-------------|
| 2.1 | Soportar target `wasm32-unknown-unknown` en codegen | 0.2 |
| 2.2 | Compilar `kyc_runtime` a WASM | — |
| 2.3 | Linker WASM: `wasm-ld` en linker.rs | 0.3 |
| 2.4 | Runtime DOM: bindings Kyle → JS (`extern fn`) | — |
| 2.5 | `ky new webapp` — template con HTML shell | 2.1–2.4 |
| 2.6 | Eventos del DOM (click, input, change) en Kyle | 2.4 |
| 2.7 | Two-way binding en WASM | 2.6 |
| 2.8 | Optimizaci&oacute;n: tama&ntilde;o del .wasm, lazy loading | 2.1 |

**Detalle t&eacute;cnico WASM:**

```
LLVM genera .wasm directamente (wasm32-unknown-unknown)
  │
  ▼
Runtime Kyle compilado a WASM (memoria, strings, lists)
  │
  ▼
DOM bindings via imports WASM:
  (import "dom" "createElement" (func ...))
  (import "dom" "addEventListener" (func ...))
  │
  ▼
Browser carga el WASM. Kyle llama a funciones JS
para manipular el DOM. Sin JS engorroso, todo desde Kyle.
```

---

### Fase 3 — UI Runtime Nativo (Skia)

**Objetivo:** Componentes UI nativos que renderizan igual en Windows, macOS y Linux.

| # | Tarea | Dependencia |
|---|-------|-------------|
| 3.1 | Crear `packages/ui/` con componentes base en Kyle | — |
| 3.2 | FFI a Skia (`@link "libskia"`) para canvas 2D | — |
| 3.3 | Ventana nativa: Win32, Cocoa, X11 v&iacute;a FFI | — |
| 3.4 | Sistema de layout (flexbox-like) | 3.1 |
| 3.5 | Componentes: View, Text, Button, Image | 3.2, 3.4 |
| 3.6 | Componentes: TextField, Scroll, List, Form | 3.5 |
| 3.7 | Componentes: Navigation, Tab, Drawer, Dialog | 3.6 |
| 3.8 | Sistema de estilos y temas (from `style<>`/`theme`) | 3.1 |
| 3.9 | Animaciones y transiciones | 3.8 |
| 3.10 | `ky new app` — template multiplataforma | 3.5 |

---

### Fase 4 — Mobile (Android + iOS)

**Objetivo:** Compilar .kyx a apps nativas de m&oacute;vil.

| # | Tarea | Dependencia |
|---|-------|-------------|
| 4.1 | Target `aarch64-linux-android` (Android NDK) | 0.2 |
| 4.2 | Target `aarch64-apple-ios` (Xcode toolchain) | 0.2 |
| 4.3 | Runtime Android: SurfaceView + touch events | 3.2 |
| 4.4 | Runtime iOS: Metal + UIKit bridge | 3.2 |
| 4.5 | Componentes nativos mobile (BottomBar, Swipe, etc.) | 3.5 |
| 4.6 | `ky new mobile` — template Android + iOS | 4.1–4.5 |

---

### Fase 5 — Custom OS

**Objetivo:** Kyle como lenguaje de sistema para un OS basado en Linux.

| # | Tarea | Dependencia |
|---|-------|-------------|
| 5.1 | Kyle runtime sin libc (freestanding) | — |
| 5.2 | Kyle graphics stack en lugar de X11/Wayland | 3.2 |
| 5.3 | Kernel Linux + init en Kyle | 5.1 |
| 5.4 | Sistema de ventanas en Kyle | 5.2 |
| 5.5 | Drivers de dispositivo en Kyle (FFI al kernel) | 5.1 |
| 5.6 | `ky build --target kyle-os` | 0.2 |

---

## 4. Arquitectura del Runtime UI

El runtime UI sigue el mismo patr&oacute;n que los paquetes existentes (http, json, sqlite):

```
packages/ui/
  src/
    lib.ky             # Entry point, re-exports
    components/
      view.ky          # <view>
      text.ky          # <text>
      button.ky        # <button>
      text_field.ky    # <text_field>
      ...
    layout/
      flex.ky          # Flexbox layout engine
      grid.ky          # Grid layout
    style/
      theme.ky         # Sistema de temas
      animation.ky     # Animaciones
    platform/
      web.ky           # DOM bindings (Fase 2)
      desktop.ky       # Skia bindings (Fase 3)
      mobile.ky        # Mobile bindings (Fase 4)
    extern/
      skia.ky          # @link "libskia"
      dom.ky           # @link a funciones JS (WASM)
      win32.ky         # @link "user32"
      cocoa.ky         # @link a frameworks macOS
      x11.ky           # @link "libX11"
```

Cada componente UI en Kyle:

```kyle
# packages/ui/src/components/button.ky

extern fn skia_draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32)
extern fn skia_draw_text(x: f32, y: f32, text: &str)

final class Button:
    text: str
    on_click: ^&(fn ())

    fn render(this, x: f32, y: f32, w: f32, h: f32):
        skia_draw_rounded_rect(x, y, w, h, 4.0)
        skia_draw_text(x + 10, y + 20, this.text)
```

---

## 5. Decisiones T&eacute;cnicas Vinculantes

### 5.1 Flag `--target`

`ky build` debe aceptar `--target` para especificar la plataforma de salida:

```bash
ky build app.ky                           # nativo (host)
ky build --target wasm32 app.ky           # WebAssembly
ky build --target x86_64-unknown-linux-gnu app.ky  # Linux x64
```

Esto implica:
- `codegen.rs`: usar el triple del flag, no `get_default_triple()`
- `linker.rs`: seleccionar linker seg&uacute;n el triple, no `cfg!(target_os)`

### 5.2 Resoluci&oacute;n de m&oacute;dulos

El resolver debe priorizar `.ky` sobre `.kyx` si ambos existen:

```
use Login.view
  → Login.ky existe? → usarlo (.ky tiene prioridad)
  → Login.kyx existe? → parsearlo como .kyx
  → Error: no encontrado
```

### 5.3 Directorio de runtimes

```
~/.ky/
  bin/ky
  lib/
    libkyc_runtime.a                       # runtime host
    targets/
      aarch64-apple-darwin/                # macOS ARM
      x86_64-apple-darwin/                 # macOS Intel
      x86_64-unknown-linux-gnu/            # Linux x64
      aarch64-unknown-linux-gnu/           # Linux ARM
      x86_64-pc-windows-gnu/               # Windows x64
      wasm32-unknown-unknown/              # WebAssembly
      ...
```

Cada subdirectorio contiene:
```
libkyc_runtime.a              # Runtime base compilado para ese target
libkyc_ui_runtime.a           # Runtime UI (Fase 3+)
```

### 5.4 Componentes nativos del sistema

Los componentes predefinidos (App, View, Text, Button, etc.) se implementan
como paquetes Kyle, no como built-ins del compilador. El compilador solo
provee el pipeline de compilaci&oacute;n y el FFI.

---

## 6. Lo que NO hacer (anti-patrones)

| Anti-patr&oacute;n | Riesgo | Alternativa |
|-------------------|--------|-------------|
| Backend especializado para un solo target | El c&oacute;digo se vuelve insostenible cuando llegan m&aacute;s targets | LLVM ya maneja todos; solo configura el triple |
| Linker hardcodeado por OS | No se puede cross-compilar | Seleccionar linker por target (din&aacute;mico) |
| Parser de .kyx separado del de .ky | Duplicaci&oacute;n de l&oacute;gica sem&aacute;ntica | .kyx → AST Kyle → pipeline existente |
| Runtime UI monol&iacute;tico | Dif&iacute;cil de mantener y portar | Componentes modulares (un archivo por componente) |
| Bindings JS espec&iacute;ficos de un browser | No funciona en todos los browsers | DOM est&aacute;ndar W3C |
| Dependencia de npm/Node.js para web | Contradice la filosof&iacute;a "sin JS" | WASM puro, sin Node.js |
| Compilaci&oacute;n JIT o interpreter para UI | P&eacute;rdida de rendimiento vs nativo | LLVM O3 + AOT compilation |

---

## 7. Glosario

| T&eacute;rmino | Significado |
|---------------|-------------|
| .kyx | Archivo de vista Kyle UI (XML-like) |
| Target triple | Identificador de plataforma LLVM (ej: `wasm32-unknown-unknown`) |
| WASM | WebAssembly, formato binario para navegadores |
| FFI | Foreign Function Interface, mecanismo para llamar C desde Kyle |
| Skia | Biblioteca de gr&aacute;ficos 2D multiplataforma (usada por Chrome, Flutter) |
| Shell HTML | `index.html` que carga un m&oacute;dulo WASM |
| Componente | Unidad de UI reutilizable en .kyx |
| Slot | Lugar reservado para contenido hijo en un componente |
| Binding | Conexi&oacute;n bidireccional entre variable y UI |

---

## 8. Referencias

- [`docs/03-language/syntax/ui-syntax.md`](../../03-language/syntax/ui-syntax.md) — Sintaxis completa de .kyx
- [`docs/07-tools/distribution.md`](../../07-tools/distribution.md) — Distribuci&oacute;n multi-plataforma (Fase 0)
- [`docs/03-language/ffi/abi.md`](../../03-language/ffi/abi.md) — FFI extern fn + @link
- LLVM WebAssembly target: `wasm32-unknown-unknown`
- Skia Graphics Library: https://skia.org
