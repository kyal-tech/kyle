# slider — Deslizador de Valor

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class slider:
    # Valor
    bind: ^f32?                     # Binding al valor actual
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    min: f32 = 0.0                  # Valor mínimo
    max: f32 = 100.0                # Valor máximo
    step: f32 = 1.0                 # Incremento
    label: str?                     # Etiqueta
    show_value: bool = true         # Mostrar valor actual
    disabled: bool = false          # Deshabilitado
    
    # Eventos
    change: fn(event: change_event)?
    input: fn(event: input_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<view>
    @(
        volume: ^f32 = 50.0
    )
    
    <slider bind=@volume min=0.0 max=100.0 label="Volumen" />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^f32?` | `none` | Binding al valor actual |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `min` | `f32` | `0.0` | Valor mínimo |
| `max` | `f32` | `100.0` | Valor máximo |
| `step` | `f32` | `1.0` | Incremento |
| `label` | `str?` | `none` | Etiqueta |
| `show_value` | `bool` | `true` | Mostrar valor actual |
| `disabled` | `bool` | `false` | Deshabilitado |
| `change` | `fn(event: change_event)?` | `none` | Al cambiar valor |
| `input` | `fn(event: input_event)?` | `none` | En tiempo real |

---

## Ejemplos Avanzados

### Rango de precio

```kyx
<view>
    @(
        min_price: ^f32 = 0.0
        max_price: ^f32 = 1000.0
    )
    
    <vstack spacing=16>
        <slider 
            bind=@min_price 
            min=0.0 
            max=1000.0 
            step=50.0
            label="Precio mínimo"
        />
        <slider 
            bind=@max_price 
            min=0.0 
            max=1000.0 
            step=50.0
            label="Precio máximo"
        />
    </vstack>
</view>
```

### Con formato personalizado

```kyx
<view>
    @(
        rating: ^f32 = 3.0
    )
    
    <slider 
        bind=@rating 
        min=0.0 
        max=5.0 
        step=0.5
        label="Calificación"
    />
    
    <text value=@"Rating: " + rating.to_str() + " ★" />
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div>
    <label>Volumen: 50</label>
    <input type="range" min="0" max="100" value="50" />
</div>
```

### iOS (SwiftUI)

```swift
VStack {
    Text("Volumen: \(Int(volume))")
    Slider(value: $volume, in: 0...100, step: 1)
}
```

### Android (Jetpack Compose)

```kotlin
Column {
    Text("Volumen: ${volume.toInt()}")
    Slider(
        value = volume,
        onValueChange = { volume = it },
        valueRange = 0f..100f,
        steps = 99
    )
}
```

---

## Referencias

- [text_field](text_field.md)
- [Componentes nativos](README.md)
