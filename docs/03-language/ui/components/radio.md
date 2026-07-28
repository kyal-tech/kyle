# radio — Botón de Opción

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class radio:
    # Valor
    bind: ^str?                     # Binding al valor seleccionado
    field: str?                     # Nombre del campo (con modelo)
    value: str                      # Valor de este radio
    
    # Configuración
    label: str?                     # Etiqueta al lado
    disabled: bool = false          # Deshabilitado
    group: str?                     # Nombre del grupo (opcional)
    
    # Eventos
    change: fn(event: change_event)?
    click: fn(event: click_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Grupo de radios

```kyx
<view>
    @(
        selected_color: ^str = "red"
    )
    
    <vstack spacing=8>
        <radio bind=@selected_color value="red" label="Rojo" />
        <radio bind=@selected_color value="green" label="Verde" />
        <radio bind=@selected_color value="blue" label="Azul" />
    </vstack>
    
    <text value=@"Color seleccionado: " + selected_color />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^str?` | `none` | Binding al valor seleccionado |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `value` | `str` | — | **Requerido.** Valor de este radio |
| `label` | `str?` | `none` | Etiqueta al lado |
| `disabled` | `bool` | `false` | Deshabilitado |
| `group` | `str?` | `none` | Nombre del grupo |
| `change` | `fn(event: change_event)?` | `none` | Al cambiar selección |
| `click` | `fn(event: click_event)?` | `none` | Al hacer click |

---

## Ejemplos Avanzados

### Con modelo

```kyx
<form model=@form>
    <radio field="gender" value="male" label="Masculino" />
    <radio field="gender" value="female" label="Femenino" />
    <radio field="gender" value="other" label="Otro" />
</form>
```

### Radio group con layout horizontal

```kyx
<view>
    @(
        size: ^str = "md"
    )
    
    <hstack spacing=16>
        <radio bind=@size value="sm" label="Small" />
        <radio bind=@size value="md" label="Medium" />
        <radio bind=@size value="lg" label="Large" />
    </hstack>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<label>
    <input type="radio" name="color" value="red" checked />
    <span>Rojo</span>
</label>
```

### iOS (SwiftUI)

```swift
Picker("Color", selection: $selected_color) {
    Text("Rojo").tag("red")
    Text("Verde").tag("green")
    Text("Azul").tag("blue")
}
.pickerStyle(.inline)
```

### Android (Jetpack Compose)

```kotlin
Column {
    RadioButton(selected = selected_color == "red", onClick = { selected_color = "red" })
    Text("Rojo")
}
```

---

## Referencias

- [checkbox](checkbox.md)
- [select](select.md)
- [Componentes nativos](README.md)
