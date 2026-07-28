# text_area — Área de Texto Multilínea

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class text_area:
    # Valor
    bind: ^str?                     # Binding bidireccional
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    label: str?                     # Etiqueta
    placeholder: str = ""           # Texto placeholder
    rows: i32 = 4                   # Número de filas
    max_length: i32?                # Longitud máxima
    required: bool = false          # Campo requerido
    disabled: bool = false          # Deshabilitado
    readonly: bool = false          # Solo lectura
    error: str?                     # Mensaje de error
    
    # Eventos
    input: fn(event: input_event)?
    change: fn(event: change_event)?
    focus: fn(event: focus_event)?
    blur: fn(event: focus_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<view>
    @(
        description: ^str = ""
    )
    
    <text_area 
        bind=@description 
        label="Descripción"
        placeholder="Escribe una descripción..."
        rows=6
    />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^str?` | `none` | Binding bidireccional |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `label` | `str?` | `none` | Etiqueta |
| `placeholder` | `str` | `""` | Texto placeholder |
| `rows` | `i32` | `4` | Número de filas |
| `max_length` | `i32?` | `none` | Longitud máxima |
| `required` | `bool` | `false` | Campo requerido |
| `disabled` | `bool` | `false` | Deshabilitado |
| `readonly` | `bool` | `false` | Solo lectura |
| `error` | `str?` | `none` | Mensaje de error |

---

## Ejemplos Avanzados

### Con contador de caracteres

```kyx
<view>
    @(
        comment: ^str = ""
        max_chars = 500
    )
    
    <text_area 
        bind=@comment 
        label="Comentario"
        max_length=max_chars
        rows=4
    />
    
    <text value=@comment.len().to_str() + " / " + max_chars.to_str() />
</view>
```

### Editor de markdown

```kyx
<view>
    @(
        content: ^str = ""
    )
    
    <hstack spacing=16>
        <text_area 
            bind=@content 
            label="Markdown"
            rows=20
            style=style(font_family: "monospace")
        />
        
        <view style=style(width: length.percent(50))>
            <text value=@render_markdown(content) />
        </view>
    </hstack>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div>
    <label>Descripción</label>
    <textarea rows="6" placeholder="Escribe..."></textarea>
</div>
```

### iOS (SwiftUI)

```swift
VStack(alignment: .leading) {
    Text("Descripción")
    TextEditor(text: $description)
        .frame(height: 150)
        .border(Color.gray, width: 1)
}
```

### Android (Jetpack Compose)

```kotlin
OutlinedTextField(
    value = description,
    onValueChange = { description = it },
    label = { Text("Descripción") },
    minLines = 6,
    maxLines = 10
)
```

---

## Referencias

- [text_field](text_field.md)
- [Componentes nativos](README.md)
