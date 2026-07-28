# card — Tarjeta

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class card:
    # Estilo
    style: style_class?             # Clase de estilo
    elevation: f32 = 2.0            # Sombra (0 = sin sombra)
    
    # Eventos
    click: fn(event: click_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<card>
    <vstack spacing=12>
        <text value="Título" style=Title />
        <text value="Contenido de la tarjeta" />
    </vstack>
</card>
```

### Con estilo personalizado

```kyx
<card style=style(
    padding: spacing.all(24),
    border_radius: 12,
    background: color("#FFFFFF")
)>
    <text value="Tarjeta personalizada" />
</card>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `style` | `style_class?` | `none` | Clase de estilo |
| `elevation` | `f32` | `2.0` | Sombra (elevación) |
| `click` | `fn(event: click_event)?` | `none` | Al hacer click |

---

## Ejemplos Avanzados

### Tarjeta de producto

```kyx
<card style=style(padding: spacing.all(16))>
    <vstack spacing=12>
        <img src=@product.image width=200 height=150 />
        <text value=@product.name style=Title />
        <text value=@product.description />
        <hstack spacing=8 alignment=alignment.end>
            <text value=@"$" + product.price.to_str() style=style(font_weight: font_weight.bold) />
            <button text="Comprar" style=Primary />
        </hstack>
    </vstack>
</card>
```

### Grid de tarjetas

```kyx
<view>
    @(
        products: ^{Product} = {}
    )
    
    <grid columns=3 spacing=16>
        @for(product in products):
            <card>
                <vstack spacing=8>
                    <img src=@product.image width=200 height=150 />
                    <text value=@product.name />
                    <text value=@"$" + product.price.to_str() />
                </vstack>
            </card>
    </grid>
</view>
```

### Tarjeta clickable

```kyx
<card click=@() => navigate("/product/" + product.id.to_str())>
    <vstack spacing=8>
        <text value=@product.name />
        <text value=@product.description />
    </vstack>
</card>
```

---

## Estilos predefinidos

```kyle
style<card> Elevated:
    background = color("#FFFFFF")
    border_radius = 8
    padding = spacing.all(16)
    shadow = shadow(x: 0, y: 2, blur: 8, color: color("#000000").with_alpha(0.1))

style<card> Outlined:
    background = color("#FFFFFF")
    border_radius = 8
    padding = spacing.all(16)
    border = border(1, color("#E0E0E0"), border_style.solid)

style<card> Filled:
    background = color("#F5F5F5")
    border_radius = 8
    padding = spacing.all(16)
```

---

## Traducción Multiplataforma

### Web

```html
<div class="card" style="padding: 16px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);">
    <div>Contenido</div>
</div>
```

### iOS (SwiftUI)

```swift
VStack {
    Text("Contenido")
}
.padding(16)
.background(Color.white)
.cornerRadius(8)
.shadow(radius: 2)
```

### Android (Jetpack Compose)

```kotlin
Card(
    modifier = Modifier.padding(16.dp),
    elevation = CardDefaults.cardElevation(defaultElevation = 2.dp),
    shape = RoundedCornerShape(8.dp)
) {
    Text("Contenido")
}
```

---

## Referencias

- [view](view.md)
- [Sistema de estilos](../style-system.md)
- [Componentes nativos](README.md)
