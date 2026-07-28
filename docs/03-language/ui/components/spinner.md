# spinner — Indicador de Carga

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class spinner:
    # Configuración
    size: f32 = 32.0                # Tamaño en píxeles
    color: color?                   # Color del spinner
    thickness: f32 = 3.0            # Grosor de la línea
    
    # Accesibilidad
    aria_label: str = "Cargando"
```

---

## Uso Básico

```kyx
<spinner />
```

### Con tamaño personalizado

```kyx
<spinner size=48.0 color=color("#0066FF") />
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `size` | `f32` | `32.0` | Tamaño en píxeles |
| `color` | `color?` | `none` | Color del spinner |
| `thickness` | `f32` | `3.0` | Grosor de la línea |

---

## Ejemplos Avanzados

### Loading overlay

```kyx
<view>
    @(
        loading: ^bool = false
    )
    
    <zstack>
        <!-- Contenido principal -->
        <view>
            <text value="Contenido de la página" />
            <button text="Cargar datos" click=@load_data />
        </view>
        
        <!-- Overlay de carga -->
        @if loading:
            <view style=style(
                background: color("#000000").with_alpha(0.5),
                width: length.percent(100),
                height: length.percent(100)
            )>
                <vstack alignment=alignment.center spacing=16>
                    <spinner size=48.0 color=color("#FFFFFF") />
                    <text value="Cargando..." style=style(color: color("#FFFFFF")) />
                </vstack>
            </view>
    </zstack>
</view>
```

### Botón con loading

```kyx
<view>
    @(
        saving: ^bool = false
    )
    
    <button disabled=@saving click=@save>
        @if saving:
            <hstack spacing=8>
                <spinner size=16.0 color=color("#FFFFFF") />
                <text value="Guardando..." />
            </hstack>
        @else:
            <text value="Guardar" />
    </button>
</view>
```

### Lista cargando

```kyx
<view>
    @(
        items: ^{Item}? = none
        loading: ^bool = true
        
        fn on_mounted():
            items = api.get_items()
            loading = false
    )
    
    @if loading:
        <vstack alignment=alignment.center spacing=16>
            <spinner size=48.0 />
            <text value="Cargando items..." />
        </vstack>
    @elif items != none:
        <list data=@items>
            <item>
                <text value=@item.name />
            </item>
        </list>
    @else:
        <text value="No hay items" />
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div class="spinner" style="width: 32px; height: 32px;">
    <svg viewBox="0 0 50 50">
        <circle cx="25" cy="25" r="20" fill="none" stroke="currentColor" stroke-width="3">
            <animateTransform
                attributeName="transform"
                type="rotate"
                from="0 25 25"
                to="360 25 25"
                dur="1s"
                repeatCount="indefinite"
            />
        </circle>
    </svg>
</div>
```

### iOS (SwiftUI)

```swift
ProgressView()
    .scaleEffect(1.5)
```

### Android (Jetpack Compose)

```kotlin
CircularProgressIndicator(
    modifier = Modifier.size(32.dp),
    color = Color.Blue,
    strokeWidth = 3.dp
)
```

---

## Referencias

- [progress](progress.md)
- [skeleton](skeleton.md)
- [Componentes nativos](README.md)
