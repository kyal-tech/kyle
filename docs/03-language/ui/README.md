# Kyle UI Framework — Documentación

**Status:** Specification v2.0
**Date:** 2026-07-28

Kyle UI es un framework UI declarativo multiplataforma que compila `.kyx` a web (HTML/CSS/JS), desktop (SDL2/Skia), iOS (SwiftUI), y Android (Jetpack Compose).

---

## Filosofia

- **35 componentes nativos**, no 200 como Flutter
- **Tipado fuerte** — Todo verificado en compilación
- **snake_case** — Consistente en todo
- **Composición** — Componentes pequeños combinables
- **Multiplataforma real** — Misma sintaxis, rendering nativo

---

## Core

| Documento | Descripción |
|-----------|-------------|
| [architecture.md](architecture.md) | Arquitectura multiplataforma, anti-patrones, traducción por target |
| [style-system.md](style-system.md) | Sistema de estilos tipado (color, spacing, layout, theme, responsive) |
| [state-events.md](state-events.md) | Estado, eventos, binding, formularios, validación, modelos |
| [events.md](events.md) | **Eventos completos** — click, hover, touch, keyboard, etc. |
| [lifecycle.md](lifecycle.md) | **Ciclo de vida** — on_created, on_mounted, on_updated, on_unmounted |
| [animation.md](animation.md) | Animaciones y transiciones tipadas |
| [routing.md](routing.md) | Routing, `<route>`, `<layout>`, `<slot>`, guards, navegación |
| [accessibility.md](accessibility.md) | Accesibilidad WCAG 2.1 AA, ARIA, teclado, screen readers |
| [anti-patterns.md](anti-patterns.md) | Anti-patrones de otros frameworks y cómo Kyle los evita |
| [framework-comparison.md](framework-comparison.md) | **Comparativa** con React, Vue, SwiftUI, Jetpack Compose, Flutter |

## Componentes Nativos

| Documento | Descripción |
|-----------|-------------|
| [components/README.md](components/README.md) | **Índice completo** de todos los componentes nativos |

### Layout & Contenedores

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `view` | Contenedor genérico | [view.md](components/view.md) |
| `card` | Tarjeta con sombra | [card.md](components/card.md) |

### Texto & Enlaces

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `text` | Texto con estilos | [text.md](components/text.md) |
| `link` | Enlace de navegación | [link.md](components/link.md) |

### Inputs & Formularios

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `button` | Botón (filled, outlined, text, icon) | [button.md](components/button.md) |
| `text_field` | Campo de texto | [text_field.md](components/text_field.md) |
| `text_area` | Campo multilínea | [text_area.md](components/text_area.md) |
| `checkbox` | Casilla de verificación | [checkbox.md](components/checkbox.md) |
| `radio` | Botón de opción | [radio.md](components/radio.md) |
| `switch` | Interruptor on/off | [switch.md](components/switch.md) |
| `slider` | Deslizador de valor | [slider.md](components/slider.md) |
| `select` | Selector desplegable | [select.md](components/select.md) |
| `file_picker` | Selector de archivos | [file_picker.md](components/file_picker.md) |
| `form` | Formulario con modelos | [form.md](components/form.md) |

### Media

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `img` | Imágenes (URL, local, lazy) | [img.md](components/img.md) |

### Feedback

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `progress` | Barra de progreso | [progress.md](components/progress.md) |
| `spinner` | Indicador de carga | [spinner.md](components/spinner.md) |

### Overlays

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `modal` | Ventana modal | [modal.md](components/modal.md) |
| `alert` | Diálogo de alerta | [alert.md](components/alert.md) |
| `tooltip` | Tooltip | [tooltip.md](components/tooltip.md) |
| `toast` | Notificación temporal | [toast.md](components/toast.md) |

### Navegación

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `app_bar` | Barra superior | [app_bar.md](components/app_bar.md) |
| `sidebar` | Barra lateral | [sidebar.md](components/sidebar.md) |
| `tab_bar` | Barra de pestañas | [tab_bar.md](components/tab_bar.md) |

### Datos

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `list` | Lista con virtualización | [list.md](components/list.md) |
| `table` | Tabla con sort/pagination | [table.md](components/table.md) |

## Patrones Avanzados

| Documento | Descripción |
|-----------|-------------|
| [composition.md](composition.md) | Slots, render props, compound components |
| [context-patterns.md](context-patterns.md) | Context avanzado: selectores, reducers, multi-context |
| [portals.md](portals.md) | Portales: modals, tooltips fuera del árbol |
| [error-boundaries.md](error-boundaries.md) | Captura de errores, fallback UI, recovery |

## Infraestructura

| Documento | Descripción |
|-----------|-------------|
| [ssr.md](ssr.md) | Server-Side Rendering, streaming, hidratación |
| [i18n.md](i18n.md) | Internacionalización, plurales, fechas, RTL |
| [testing.md](testing.md) | Testing: unit, interacción, snapshots, E2E |
| [file-picker.md](file-picker.md) | File picker nativo multiplataforma |

## Documentos Relacionados

- [Sintaxis .kyx](../syntax/ui-syntax.md) — Sintaxis completa de archivos .kyx
- [RFC-0002: Arquitectura UI](../../10-design/rfc/0002-ui-architecture.md)
- [RFC-0003: Traducción multi-target](../../10-design/rfc/0003-ui-translation.md)
- [RFC-0005: UI v2](../../10-design/rfc/0005-ui-rearchitecture-plan.md)

---

## Quick Start

```bash
ky new kyui mi-app
cd mi-app
ky run web
```

```kyx
# app.kyx
from views.home import home

<app title="Mi App">
    <router>
        <route path="/" component=home />
    </router>
</app>
```

```kyx
# src/views/home.kyx
<view>
    @(
        count: ^i32 = 0
        fn increment():
            count += 1
    )
    
    <vstack spacing=16 alignment=alignment.center>
        <text value=@"Contador: " + count.to_str() />
        <button text="+" style=Primary click=@increment />
    </vstack>
</view>
```

---

## Eventos Multiplataforma

Todos los componentes soportan eventos completos:

```kyx
<button 
    text="Click me"
    click=@handle_click
    mouse_enter=@handle_hover
    mouse_leave=@handle_hover_end
    touch_start=@handle_touch_start
    long_press=@handle_long_press
    keydown=@handle_keydown
/>
```

Ver [events.md](events.md) para la lista completa de eventos y tipos.

---

## Visión a Largo Plazo

Kyle UI está diseñado para:

1. **Web** — HTML/CSS/JS nativo (actual)
2. **Desktop** — SDL2/Skia para macOS, Linux, Windows
3. **Mobile** — SwiftUI para iOS, Jetpack Compose para Android
4. **KYOS OS** — Sistema operativo nativo con kernel propio (futuro)

Toda la sintaxis y lógica es **idéntica** en todas las plataformas. Solo cambia el rendering backend.
