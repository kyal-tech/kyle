# Componentes Nativos Kyle UI

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Filosofía

Kyle UI tiene **~30 componentes nativos**, no 200 como Flutter. Cada uno es un **tipo Kyle** con props fuertemente tipadas.

### Principios

1. **Mínimos pero poderosos** — Solo lo esencial, composición libre
2. **Tipados** — Todo verificado en compilación
3. **Multiplataforma** — Misma sintaxis, rendering nativo
4. **snake_case** — Props, eventos, todo consistente

---

## Lista de Componentes

### Layout

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `view` | Contenedor genérico | — |
| `vstack` | Layout vertical (columna) | — |
| `hstack` | Layout horizontal (fila) | — |
| `zstack` | Layout en profundidad (Z-axis) | — |
| `scroll` | Contenedor scrollable | — |
| `spacer` | Espacio flexible | — |
| `divider` | Línea separadora | — |

### Texto

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `text` | Texto estático o reactivo | — |
| `link` | Enlace de navegación | — |

### Input

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `button` | Botón clickeable | [button.md](button.md) |
| `text_field` | Campo de texto | [text_field.md](text_field.md) |
| `password_field` | Campo de contraseña | [text_field.md](text_field.md) |
| `checkbox` | Casilla de verificación | [checkbox.md](checkbox.md) |
| `radio` | Botón de opción | [radio.md](radio.md) |
| `switch` | Interruptor on/off | [switch.md](switch.md) |
| `slider` | Deslizador de valor | [slider.md](slider.md) |
| `select` | Selector desplegable | [select.md](select.md) |
| `file_picker` | Selector de archivos | [file_picker.md](file_picker.md) |

### Media

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `img` | Imagen (URL o local) | [img.md](img.md) |
| `video` | Video | [video.md](video.md) |
| `audio` | Audio | [audio.md](audio.md) |

### Feedback

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `progress` | Barra de progreso | [progress.md](progress.md) |
| `spinner` | Indicador de carga | [spinner.md](spinner.md) |
| `skeleton` | Placeholder de carga | [skeleton.md](skeleton.md) |

### Overlay

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `modal` | Ventana modal | [modal.md](modal.md) |
| `sheet` | Hoja deslizante | [modal.md](modal.md) |
| `alert` | Diálogo de alerta | [alert.md](alert.md) |
| `tooltip` | Tooltip | [tooltip.md](tooltip.md) |
| `toast` | Notificación temporal | [toast.md](toast.md) |

### Navegación

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `app_bar` | Barra superior (reemplaza navbar) | [app_bar.md](app_bar.md) |
| `side_bar` | Barra lateral | [side_bar.md](side_bar.md) |
| `tab_bar` | Barra de pestañas | [tab_bar.md](tab_bar.md) |
| `bottom_nav` | Navegación inferior (mobile) | [bottom_nav.md](bottom_nav.md) |

### Datos

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `list` | Lista de items | [list.md](list.md) |
| `table` | Tabla de datos | [table.md](table.md) |
| `grid` | Grid de items | [grid.md](grid.md) |

### Formulario

| Componente | Descripción | Doc |
|------------|-------------|-----|
| `form` | Contenedor de formulario | [form.md](form.md) |

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
| `chart` | Gráfico (líneas, barras, torta) | [chart.md](chart.md) |

---

## Tipos Compartidos

### Eventos

```kyle
final class click_event:
    x: f32
    y: f32
    target: str

final class input_event:
    value: str
    target: str

final class change_event:
    value: str
    target: str

final class key_event:
    key: str
    code: str
    alt_key: bool
    ctrl_key: bool
    shift_key: bool
    meta_key: bool

final class mouse_event:
    x: f32
    y: f32
    button: i32
    alt_key: bool
    ctrl_key: bool
    shift_key: bool
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

- [Arquitectura multiplataforma](../architecture.md)
- [Sistema de estilos](../style-system.md)
- [Estado y eventos](../state-events.md)
