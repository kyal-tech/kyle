# view — Contenedor Genérico

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class view:
    # Layout
    style: style_class?             # Clase de estilo
    width: length?                  # Ancho
    height: length?                 # Alto
    padding: spacing?               # Padding interno
    margin: spacing?                # Margen externo
    
    # Comportamiento
    focusable: bool = false         # Puede recibir foco
    clickable: bool = false         # Puede recibir clicks
    
    # Eventos
    click: fn(event: click_event)?
    mouse_enter: fn(event: mouse_event)?
    mouse_leave: fn(event: mouse_event)?
    keydown: fn(event: key_event)?
    
    # Accesibilidad
    aria_label: str?
    aria_role: str?
```

---

## Uso Básico

### Contenedor simple

```kyx
<view>
    <text value="Contenido" />
</view>
```

### Con estilo

```kyx
<view style=style(
    padding: spacing.all(16),
    background: color("#F5F5F5"),
    border_radius: 8
)>
    <text value="Contenido con padding y fondo" />
</view>
```

### Clickable

```kyx
<view 
    clickable=true 
    click=@handle_click
    style=style(cursor: cursor.pointer)
>
    <text value="Click aquí" />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `style` | `style_class?` | `none` | Clase de estilo |
| `width` | `length?` | `none` | Ancho |
| `height` | `length?` | `none` | Alto |
| `padding` | `spacing?` | `none` | Padding interno |
| `margin` | `spacing?` | `none` | Margen externo |
| `focusable` | `bool` | `false` | Puede recibir foco |
| `clickable` | `bool` | `false` | Puede recibir clicks |
| `click` | `fn(event: click_event)?` | `none` | Al hacer click |
| `mouse_enter` | `fn(event: mouse_event)?` | `none` | Al entrar cursor |
| `mouse_leave` | `fn(event: mouse_event)?` | `none` | Al salir cursor |
| `keydown` | `fn(event: key_event)?` | `none` | Al presionar tecla |

---

## View vs Layouts

| Componente | Propósito | Uso |
|------------|-----------|-----|
| `view` | Contenedor genérico | Agrupar elementos sin dirección específica |
| `vstack` | Layout vertical | Organizar elementos en columna |
| `hstack` | Layout horizontal | Organizar elementos en fila |
| `zstack` | Layout en profundidad | Superponer elementos |

---

## Ejemplos Avanzados

### Card personalizada

```kyx
<view style=style(
    padding: spacing.all(16),
    background: color("#FFFFFF"),
    border_radius: 8,
    shadow: shadow(x: 0, y: 2, blur: 8, color: color("#000000").with_alpha(0.1))
)>
    <vstack spacing=12>
        <text value="Título" style=Title />
        <text value="Descripción del contenido" />
        <button text="Acción" style=Primary />
    </vstack>
</view>
```

### Contenedor con hover

```kyx
<view>
    @(
        is_hovered: ^bool = false
    )
    
    <view 
        mouse_enter=@() => is_hovered = true
        mouse_leave=@() => is_hovered = false
        style=@if is_hovered:
            style(background: color("#E3F2FD"))
        @else:
            style(background: color("#FFFFFF"))
    >
        <text value="Pasa el cursor" />
    </view>
</view>
```

### Contenedor scrollable

```kyx
<view style=style(
    height: length.px(300),
    overflow: overflow.scroll
)>
    @for(item in long_list):
        <text value=@item.name />
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div style="padding: 16px; background: #F5F5F5; border-radius: 8px;">
    <span>Contenido</span>
</div>
```

### Desktop (SDL2/Skia)

```kyle
fn render_view(x: f32, y: f32, w: f32, h: f32):
    skia_fill_rounded_rect(x, y, w, h, 8.0, color("#F5F5F5"))
    render_children(x + 16, y + 16, w - 32, h - 32)
```

### iOS (SwiftUI)

```swift
VStack {
    Text("Contenido")
}
.padding(16)
.background(Color(hex: "#F5F5F5"))
.cornerRadius(8)
```

### Android (Jetpack Compose)

```kotlin
Box(
    modifier = Modifier
        .padding(16.dp)
        .background(Color.parseColor("#F5F5F5"))
        .clip(RoundedCornerShape(8.dp))
) {
    Text("Contenido")
}
```

---

## Referencias

- [vstack](#vstack)
- [hstack](#hstack)
- [zstack](#zstack)
- [Sistema de estilos](../style-system.md)
- [Componentes nativos](README.md)
