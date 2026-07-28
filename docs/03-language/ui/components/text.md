# text — Texto

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class text:
    # Contenido
    value: str                      # Texto a mostrar
    
    # Tipografía
    style: style_class?             # Clase de estilo
    font_size: f32?                 # Tamaño de fuente
    font_weight: font_weight?       # Peso de fuente
    color: color?                   # Color del texto
    align: text_align?              # Alineación
    line_height: f32?               # Altura de línea
    
    # Comportamiento
    max_lines: i32?                 # Máximo de líneas
    overflow: text_overflow?        # Qué hacer si no cabe
    selectable: bool = false        # Permitir selección de texto
    
    # Accesibilidad
    aria_label: str?
    aria_hidden: bool = false
```

---

## Uso Básico

```kyx
<text value="Hola Mundo" />
```

### Con estilo

```kyx
<text value="Título" style=Title />
<text value="Subtítulo" style=Subtitle />
```

### Reactivo

```kyx
<view>
    @(
        count: ^i32 = 0
    )
    
    <text value=@"Contador: " + count.to_str() />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `value` | `str` | — | **Requerido.** Texto a mostrar |
| `style` | `style_class?` | `none` | Clase de estilo |
| `font_size` | `f32?` | `none` | Tamaño de fuente |
| `font_weight` | `font_weight?` | `none` | Peso de fuente |
| `color` | `color?` | `none` | Color del texto |
| `align` | `text_align?` | `none` | Alineación |
| `line_height` | `f32?` | `none` | Altura de línea |
| `max_lines` | `i32?` | `none` | Máximo de líneas |
| `overflow` | `text_overflow?` | `none` | Comportamiento de desbordamiento |
| `selectable` | `bool` | `false` | Permitir selección |

---

## Tipos Relacionados

### text_overflow

```kyle
enum text_overflow:
    clip            # Cortar texto
    ellipsis        # Mostrar "..."
    visible         # Mostrar todo (puede desbordar)
```

---

## Ejemplos Avanzados

### Texto con límite de líneas

```kyx
<text 
    value="Este es un texto muy largo que puede no caber en una sola línea..."
    max_lines=2
    overflow=text_overflow.ellipsis
/>
```

### Texto seleccionable

```kyx
<text value="Código: ABC123" selectable=true />
```

### Texto con interpolación

```kyx
<view>
    @(
        user: User = get_user()
    )
    
    <text value=@"Hola, " + user.name + ". Tienes " + user.messages.len().to_str() + " mensajes." />
</view>
```

### Múltiples textos con formato

```kyx
<hstack spacing=4>
    <text value="Precio:" font_weight=font_weight.bold />
    <text value="$99.99" color=color("#10B981") />
</hstack>
```

---

## Estilos de texto predefinidos

```kyle
style<text> Title:
    font_size = 24
    font_weight = font_weight.bold
    color = color("#1A1A1A")

style<text> Subtitle:
    font_size = 18
    font_weight = font_weight.semi_bold
    color = color("#333333")

style<text> Body:
    font_size = 14
    font_weight = font_weight.normal
    color = color("#1A1A1A")

style<text> Caption:
    font_size = 12
    font_weight = font_weight.normal
    color = color("#666666")

style<text> Error:
    font_size = 12
    color = color("#DC3545")
```

---

## Traducción Multiplataforma

### Web

```html
<span style="font-size: 24px; font-weight: bold;">Título</span>
```

### iOS (SwiftUI)

```swift
Text("Título")
    .font(.title.bold())
    .foregroundColor(Color(hex: "#1A1A1A"))
```

### Android (Jetpack Compose)

```kotlin
Text(
    text = "Título",
    fontSize = 24.sp,
    fontWeight = FontWeight.Bold,
    color = Color.parseColor("#1A1A1A")
)
```

---

## Referencias

- [Sistema de estilos](../style-system.md)
- [Componentes nativos](README.md)
