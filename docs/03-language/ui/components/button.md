# button — Botón

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class button:
    # Contenido
    text: str?                      # Texto del botón
    icon: str?                      # Nombre del ícono (opcional)
    
    # Estado
    disabled: bool = false          # Deshabilitado
    loading: bool = false           # Mostrando spinner
    
    # Estilo
    style: style_class?             # Clase de estilo (Primary, Secondary, etc.)
    size: size = size.md            # Tamaño (xs, sm, md, lg, xl)
    variant: button_variant = button_variant.filled  # Variante visual
    
    # Eventos
    click: fn(event: click_event)?  # Callback al hacer click
    
    # Accesibilidad
    aria_label: str?                # Label para screen readers
    aria_hidden: bool = false       # Ocultar de screen readers
```

---

## Uso Básico

### Botón simple

```kyx
<button text="Guardar" click=@handle_save />
```

### Con estilo

```kyx
<button text="Eliminar" style=Danger click=@handle_delete />
```

### Con ícono

```kyx
<button icon="plus" text="Nuevo" click=@handle_new />
```

### Deshabilitado

```kyx
<button text="Enviar" disabled=@!form_valid click=@handle_submit />
```

### Loading

```kyx
<button text="Guardar" loading=@is_saving click=@handle_save />
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `text` | `str?` | `none` | Texto del botón |
| `icon` | `str?` | `none` | Nombre del ícono |
| `disabled` | `bool` | `false` | Deshabilitado |
| `loading` | `bool` | `false` | Mostrando spinner |
| `style` | `style_class?` | `none` | Clase de estilo |
| `size` | `size` | `md` | Tamaño |
| `variant` | `button_variant` | `filled` | Variante visual |
| `click` | `fn(event: click_event)?` | `none` | Callback al hacer click |

---

## Tipos Relacionados

### button_variant

```kyle
enum button_variant:
    filled          # Fondo sólido (default)
    outlined        # Solo borde
    text            # Solo texto, sin fondo ni borde
    icon            # Solo ícono circular
```

### size

```kyle
enum size:
    xs              # Extra small (24px height)
    sm              # Small (32px height)
    md              # Medium (40px height, default)
    lg              # Large (48px height)
    xl              # Extra large (56px height)
```

---

## Estilos Predefinidos

```kyle
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    border_radius = 8
    padding = spacing.all(12)

style<button> Secondary:
    background = color("transparent")
    color = color("#0066FF")
    border = border(2, color("#0066FF"), border_style.solid)
    border_radius = 8
    padding = spacing.all(12)

style<button> Danger:
    background = color("#DC3545")
    color = color("#FFFFFF")
    border_radius = 8
    padding = spacing.all(12)

style<button> Ghost:
    background = color("transparent")
    color = color("#1A1A1A")
    border_radius = 8
    padding = spacing.all(12)
```

---

## Ejemplos Avanzados

### Grupo de botones

```kyx
<hstack spacing=8>
    <button text="Cancelar" style=Secondary click=@cancel />
    <button text="Guardar" style=Primary click=@save />
</hstack>
```

### Botón con ícono y texto

```kyx
<button icon="download" text="Descargar" style=Primary click=@download />
```

### Botón de solo ícono

```kyx
<button icon="close" variant=button_variant.icon click=@close />
```

### Botón full width

```kyx
<button text="Ingresar" style=Primary style=style(width: length.fill) click=@login />
```

### Botón con confirmación

```kyx
<view>
    @(
        fn handle_delete():
            if confirm("¿Eliminar este item?"):
                delete_item()
    )
    
    <button text="Eliminar" style=Danger click=@handle_delete />
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<button class="Primary" onclick="handle_save()">Guardar</button>
```

### Desktop (SDL2/Skia)

```kyle
fn render_button(x: f32, y: f32, w: f32, h: f32):
    skia_fill_rounded_rect(x, y, w, h, 8.0, color("#0066FF"))
    skia_draw_text(x + w/2, y + h/2, "Guardar", color("#FFFFFF"))
    
    if mouse_in_rect(x, y, w, h) and mouse_clicked():
        handle_save()
```

### iOS (SwiftUI)

```swift
Button(action: handle_save) {
    Text("Guardar")
}
.padding(.horizontal, 24)
.padding(.vertical, 12)
.background(Color(hex: "#0066FF"))
.foregroundColor(.white)
.cornerRadius(8)
```

### Android (Jetpack Compose)

```kotlin
Button(
    onClick = { handle_save() },
    colors = ButtonDefaults.buttonColors(
        backgroundColor = Color.parseColor("#0066FF"),
        contentColor = Color.WHITE
    ),
    shape = RoundedCornerShape(8.dp)
) {
    Text("Guardar")
}
```

---

## Errores Comunes

### ❌ Malo: Mezclar lógica en el click

```kyx
<button text="Guardar" click=@() => {
    if form_valid:
        save()
        navigate("/success")
} />
```

**Problema:** Lógica inline dificulta testing y reutilización.

### ✅ Bueno: Función separada

```kyx
<view>
    @(
        fn handle_save():
            if form_valid:
                save()
                navigate("/success")
    )
    
    <button text="Guardar" click=@handle_save />
</view>
```

### ❌ Malo: No manejar loading state

```kyx
<button text="Guardar" click=@save />
```

**Problema:** Usuario puede hacer múltiples clicks.

### ✅ Bueno: Mostrar loading

```kyx
<button text="Guardar" loading=@is_saving disabled=@is_saving click=@save />
```

---

## Referencias

- [Arquitectura multiplataforma](../architecture.md)
- [Sistema de estilos](../style-system.md)
- [Componentes nativos](README.md)
