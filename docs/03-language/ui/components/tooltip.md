# tooltip — Tooltip

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class tooltip:
    # Contenido
    text: str                       # Texto del tooltip
    
    # Configuración
    position: tooltip_position = tooltip_position.top  # Posición
    delay: i32 = 300                # Delay antes de mostrar (ms)
    duration: i32 = 0               # Duración (0 = hasta mouse_leave)
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<button text="Hover me">
    <tooltip text="Este es un tooltip" />
</button>
```

### Con posición

```kyx
<button text="Info">
    <tooltip text="Información adicional" position=tooltip_position.bottom />
</button>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `text` | `str` | — | **Requerido.** Texto del tooltip |
| `position` | `tooltip_position` | `top` | Posición |
| `delay` | `i32` | `300` | Delay antes de mostrar (ms) |
| `duration` | `i32` | `0` | Duración (0 = hasta mouse_leave) |

---

## Tipos Relacionados

### tooltip_position

```kyle
enum tooltip_position:
    top
    bottom
    left
    right
```

---

## Ejemplos Avanzados

### Tooltip en ícono

```kyx
<button icon="info" variant=button_variant.icon>
    <tooltip text="Más información sobre esta función" />
</button>
```

### Tooltip con delay

```kyx
<button text="Guardar">
    <tooltip text="Guarda los cambios actuales" delay=500 />
</button>
```

### Tooltip en tabla

```kyx
<table data=@users>
    <col field="name" label="Nombre">
        <text value=@item.name>
            <tooltip text=@"Email: " + item.email />
        </text>
    </col>
</table>
```

---

## Traducción Multiplataforma

### Web

```html
<button data-tooltip="Este es un tooltip">Hover me</button>
```

```css
[data-tooltip] {
    position: relative;
}
[data-tooltip]:hover::after {
    content: attr(data-tooltip);
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);
    padding: 4px 8px;
    background: #333;
    color: white;
    border-radius: 4px;
    white-space: nowrap;
}
```

### iOS (SwiftUI)

```swift
Button("Hover me") { }
    .help("Este es un tooltip")
```

### Android (Jetpack Compose)

```kotlin
var showTooltip by remember { mutableStateOf(false) }

Button(
    onClick = { },
    modifier = Modifier.onLongHover(
        onEnter = { showTooltip = true },
        onExit = { showTooltip = false }
    )
) {
    Text("Hover me")
}

if (showTooltip) {
    Tooltip(text = "Este es un tooltip")
}
```

---

## Referencias

- [button](button.md)
- [Componentes nativos](README.md)
