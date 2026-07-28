# select — Selector Desplegable

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class select:
    # Valor
    bind: ^str?                     # Binding al valor seleccionado
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    label: str?                     # Etiqueta
    placeholder: str = "Seleccionar..."  # Texto placeholder
    options: {select_option}         # Lista de opciones
    disabled: bool = false          # Deshabilitado
    multiple: bool = false          # Selección múltiple
    
    # Eventos
    change: fn(event: change_event)?
    focus: fn(event: focus_event)?
    blur: fn(event: focus_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Con opciones estáticas

```kyx
<view>
    @(
        country: ^str = ""
        countries = {"Perú", "Chile", "Argentina", "Colombia"}
    )
    
    <select 
        bind=@country 
        label="País"
        placeholder="Selecciona un país"
        options=@countries
    />
</view>
```

### Con option explícitos

```kyx
<select bind=@country label="País">
    <option value="pe" label="Perú" />
    <option value="cl" label="Chile" />
    <option value="ar" label="Argentina" />
</select>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^str?` | `none` | Binding al valor seleccionado |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `label` | `str?` | `none` | Etiqueta |
| `placeholder` | `str` | `"Seleccionar..."` | Texto placeholder |
| `options` | `{select_option}` | `none` | Lista de opciones |
| `disabled` | `bool` | `false` | Deshabilitado |
| `multiple` | `bool` | `false` | Selección múltiple |
| `change` | `fn(event: change_event)?` | `none` | Al cambiar selección |
| `focus` | `fn(event: focus_event)?` | `none` | Al enfocar |
| `blur` | `fn(event: focus_event)?` | `none` | Al perder foco |

---

## Tipos Relacionados

### select_option

```kyle
final class select_option:
    value: str                      # Valor
    label: str                      # Etiqueta mostrada
    disabled: bool = false          # Opción deshabilitada
    group: str?                     # Grupo (optgroup)
```

---

## Ejemplos Avanzados

### Con grupos

```kyx
<select bind=@product label="Producto">
    <option_group label="Frutas">
        <option value="apple" label="Manzana" />
        <option value="banana" label="Banana" />
    </option_group>
    <option_group label="Verduras">
        <option value="carrot" label="Zanahoria" />
        <option value="lettuce" label="Lechuga" />
    </option_group>
</select>
```

### Selección múltiple

```kyx
<view>
    @(
        skills: ^{str} = {}
        skill_options = {"Kyle", "Rust", "Python", "JavaScript"}
    )
    
    <select 
        bind=@skills 
        label="Habilidades"
        options=@skill_options
        multiple=true
    />
</view>
```

### Con búsqueda

```kyx
<select 
    bind=@user_id 
    label="Usuario"
    searchable=true
    options=@user_options
/>
```

---

## Traducción Multiplataforma

### Web

```html
<div>
    <label>País</label>
    <select>
        <option value="">Selecciona un país</option>
        <option value="pe">Perú</option>
        <option value="cl">Chile</option>
    </select>
</div>
```

### iOS (SwiftUI)

```swift
Picker("País", selection: $country) {
    Text("Perú").tag("pe")
    Text("Chile").tag("cl")
}
```

### Android (Jetpack Compose)

```kotlin
var expanded by remember { mutableStateOf(false) }
var selected by remember { mutableStateOf("") }

ExposedDropdownMenuBox(
    expanded = expanded,
    onExpandedChange = { expanded = !expanded }
) {
    OutlinedTextField(
        value = selected,
        onValueChange = {},
        readOnly = true,
        label = { Text("País") },
        trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded) }
    )
    ExposedDropdownMenu(
        expanded = expanded,
        onDismissRequest = { expanded = false }
    ) {
        DropdownMenuItem(onClick = { selected = "pe"; expanded = false }) {
            Text("Perú")
        }
    }
}
```

---

## Referencias

- [radio](radio.md)
- [checkbox](checkbox.md)
- [Componentes nativos](README.md)
