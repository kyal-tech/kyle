# Componentes Nativos Kyle UI

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Filosofía

Kyle UI tiene **~35 componentes nativos**, no 200 como Flutter. Cada uno es un **tipo Kyle** con props fuertemente tipadas.

### Principios

1. **Mínimos pero poderosos** — Solo lo esencial, composición libre
2. **Tipados** — Todo verificado en compilación
3. **Multiplataforma** — Misma sintaxis, rendering nativo
4. **snake_case** — Props, eventos, todo consistente

---

## Lista Completa de Componentes

### Layout

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `view` | Contenedor genérico | [view.md](components/view.md) |
| `vstack` | Layout vertical (columna) | [view.md](components/view.md) |
| `hstack` | Layout horizontal (fila) | [view.md](components/view.md) |
| `zstack` | Layout en profundidad (Z-axis) | [view.md](components/view.md) |
| `scroll` | Contenedor scrollable | [view.md](components/view.md) |
| `spacer` | Espacio flexible | — |
| `divider` | Línea separadora | — |

### Texto

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `text` | Texto estático o reactivo | [text.md](components/text.md) |
| `link` | Enlace de navegación | [link.md](components/link.md) |

### Input

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `button` | Botón clickeable | [button.md](components/button.md) |
| `text_field` | Campo de texto | [text_field.md](components/text_field.md) |
| `text_area` | Campo de texto multilínea | [text_area.md](components/text_area.md) |
| `password_field` | Campo de contraseña | [text_field.md](components/text_field.md) |
| `checkbox` | Casilla de verificación | [checkbox.md](components/checkbox.md) |
| `radio` | Botón de opción | [radio.md](components/radio.md) |
| `switch` | Interruptor on/off | [switch.md](components/switch.md) |
| `slider` | Deslizador de valor | [slider.md](components/slider.md) |
| `select` | Selector desplegable | [select.md](components/select.md) |
| `file_picker` | Selector de archivos | [file_picker.md](components/file_picker.md) |

### Media

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `img` | Imagen (URL o local) | [img.md](components/img.md) |
| `video` | Video | — |
| `audio` | Audio | — |

### Feedback

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `progress` | Barra de progreso | [progress.md](components/progress.md) |
| `spinner` | Indicador de carga | [spinner.md](components/spinner.md) |
| `skeleton` | Placeholder de carga | — |

### Overlay

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `modal` | Ventana modal | [modal.md](components/modal.md) |
| `sheet` | Hoja deslizante | [modal.md](components/modal.md) |
| `alert` | Diálogo de alerta | [alert.md](components/alert.md) |
| `tooltip` | Tooltip | [tooltip.md](components/tooltip.md) |
| `toast` | Notificación temporal | [toast.md](components/toast.md) |

### Navegación

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `app_bar` | Barra superior | [app_bar.md](components/app_bar.md) |
| `sidebar` | Barra lateral | [sidebar.md](components/sidebar.md) |
| `tab_bar` | Barra de pestañas | [tab_bar.md](components/tab_bar.md) |
| `bottom_nav` | Navegación inferior (mobile) | — |

### Datos

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `list` | Lista de items | [list.md](components/list.md) |
| `table` | Tabla de datos | [table.md](components/table.md) |
| `grid` | Grid de items | — |
| `card` | Tarjeta | [card.md](components/card.md) |

### Formulario

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `form` | Contenedor de formulario | [form.md](components/form.md) |

### Contenedor

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `app` | Raíz de la aplicación | — |
| `router` | Router de navegación | — |
| `route` | Definición de ruta | — |
| `layout` | Layout persistente | — |
| `slot` | Slot de contenido | — |

### Gráficos

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `chart` | Gráfico (líneas, barras, torta) | — |

---

## Eventos

Todos los componentes soportan eventos multiplataforma. Ver [events.md](events.md) para la lista completa.

### Eventos más comunes

| Evento | Descripción | Componentes |
|--------|-------------|-------------|
| `click` | Click simple | Todos interactivos |
| `dblclick` | Doble click | View, button, etc. |
| `mouse_enter` | Cursor entra | Todos |
| `mouse_leave` | Cursor sale | Todos |
| `touch_start` | Touch inicia | Mobile |
| `touch_end` | Touch termina | Mobile |
| `long_press` | Presión larga | Mobile |
| `keydown` | Tecla presionada | Input, view |
| `keyup` | Tecla liberada | Input, view |
| `focus` | Enfocado | Input, button |
| `blur` | Perdió foco | Input, button |
| `change` | Valor cambió | Input, select, etc. |
| `input` | Valor cambiando | Input en tiempo real |
| `submit` | Form enviado | Form |
| `scroll` | Scroll | Scroll, list, etc. |

---

## Tipos Compartidos

### Eventos

```kyle
final class click_event:
    x: f32
    y: f32
    target: str
    button: i32
    alt_key: bool
    ctrl_key: bool
    shift_key: bool
    meta_key: bool

final class mouse_event:
    x: f32
    y: f32
    client_x: f32
    client_y: f32
    button: i32
    alt_key: bool
    ctrl_key: bool
    shift_key: bool

final class touch_event:
    touches: {touch_point}
    changed_touches: {touch_point}
    target: str

final class key_event:
    key: str
    code: str
    alt_key: bool
    ctrl_key: bool
    shift_key: bool
    meta_key: bool

final class input_event:
    value: str
    target: str

final class change_event:
    value: str
    target: str
    previous_value: str

final class focus_event:
    target: str
    related_target: str?

final class submit_event:
    target: str
    prevent_default: fn()

final class scroll_event:
    scroll_x: f32
    scroll_y: f32
    target: str
```

### Tamaños

```kyle
enum size:
    xs      # extra small
    sm      # small
    md      # medium (default)
    lg      # large
    xl      # extra large
```

### Estados

```kyle
enum state:
    idle
    loading
    success
    error(msg: str)
```

---

## Referencias

- [Arquitectura multiplataforma](architecture.md)
- [Eventos](events.md)
- [Sistema de estilos](style-system.md)
- [Estado y eventos](state-events.md)
