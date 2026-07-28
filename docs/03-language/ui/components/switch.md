# switch — Interruptor On/Off

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class switch:
    # Valor
    bind: ^bool?                    # Binding bidireccional
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    label: str?                     # Etiqueta al lado
    disabled: bool = false          # Deshabilitado
    
    # Eventos
    change: fn(event: change_event)?
    click: fn(event: click_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<view>
    @(
        dark_mode: ^bool = false
    )
    
    <switch bind=@dark_mode label="Modo oscuro" />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^bool?` | `none` | Binding bidireccional |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `label` | `str?` | `none` | Etiqueta al lado |
| `disabled` | `bool` | `false` | Deshabilitado |
| `change` | `fn(event: change_event)?` | `none` | Al cambiar valor |
| `click` | `fn(event: click_event)?` | `none` | Al hacer click |

---

## Ejemplos Avanzados

### Configuración de notificaciones

```kyx
<view>
    @(
        notifications: ^bool = true
        email_notifications: ^bool = true
        push_notifications: ^bool = false
    )
    
    <vstack spacing=16>
        <switch bind=@notifications label="Notificaciones activadas" />
        
        @if notifications:
            <vstack spacing=8>
                <switch bind=@email_notifications label="Email" />
                <switch bind=@push_notifications label="Push" />
            </vstack>
    </vstack>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<label class="switch">
    <input type="checkbox" checked />
    <span class="slider"></span>
    <span>Modo oscuro</span>
</label>
```

### iOS (SwiftUI)

```swift
Toggle("Modo oscuro", isOn: $dark_mode)
```

### Android (Jetpack Compose)

```kotlin
Row {
    Switch(
        checked = dark_mode,
        onCheckedChange = { dark_mode = it }
    )
    Text("Modo oscuro")
}
```

---

## Referencias

- [checkbox](checkbox.md)
- [Componentes nativos](README.md)
