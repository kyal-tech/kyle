# checkbox — Casilla de Verificación

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class checkbox:
    # Valor
    bind: ^bool?                    # Binding bidireccional
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    label: str?                     # Etiqueta al lado
    disabled: bool = false          # Deshabilitado
    indeterminate: bool = false     # Estado intermedio
    
    # Eventos
    change: fn(event: change_event)?  # Al cambiar valor
    click: fn(event: click_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Checkbox simple

```kyx
<view>
    @(
        accept_terms: ^bool = false
    )
    
    <checkbox bind=@accept_terms label="Acepto los términos y condiciones" />
</view>
```

### Con modelo

```kyx
<form model=@form>
    <checkbox field="is_active" label="Usuario activo" />
    <checkbox field="receive_emails" label="Recibir emails" />
</form>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^bool?` | `none` | Binding bidireccional |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `label` | `str?` | `none` | Etiqueta al lado |
| `disabled` | `bool` | `false` | Deshabilitado |
| `indeterminate` | `bool` | `false` | Estado intermedio |
| `change` | `fn(event: change_event)?` | `none` | Al cambiar valor |
| `click` | `fn(event: click_event)?` | `none` | Al hacer click |

---

## Ejemplos Avanzados

### Grupo de checkboxes

```kyx
<view>
    @(
        selected_options: ^{str} = {}
        options = {"Option 1", "Option 2", "Option 3"}
        
        fn toggle_option(option: str):
            if selected_options.contains(option):
                selected_options.remove(option)
            else:
                selected_options.add(option)
    )
    
    <vstack spacing=8>
        @for(option in options):
            <checkbox 
                bind=@(selected_options.contains(option))
                label=@option
                change=@() => toggle_option(option)
            />
    </vstack>
</view>
```

### Select all / Deselect all

```kyx
<view>
    @(
        items: ^{Item} = {}
        selected: ^{i32} = {}
        all_selected: ^bool = false
        
        fn toggle_all():
            if all_selected:
                selected = {}
            else:
                selected = {0..items.len()}
            all_selected = !all_selected
    )
    
    <checkbox 
        bind=@all_selected 
        label="Seleccionar todos"
        indeterminate=@(selected.len() > 0 and selected.len() < items.len())
        change=@toggle_all
    />
    
    <list data=@items>
        <item>
            <checkbox 
                bind=@(selected.contains(idx))
                label=@item.name
                change=@() => toggle_item(idx)
            />
        </item>
    </list>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<label>
    <input type="checkbox" checked />
    <span>Acepto los términos</span>
</label>
```

### iOS (SwiftUI)

```swift
Toggle("Acepto los términos", isOn: $accept_terms)
```

### Android (Jetpack Compose)

```kotlin
Row {
    Checkbox(
        checked = accept_terms,
        onCheckedChange = { accept_terms = it }
    )
    Text("Acepto los términos")
}
```

---

## Referencias

- [radio](radio.md)
- [switch](switch.md)
- [form](form.md)
- [Componentes nativos](README.md)
