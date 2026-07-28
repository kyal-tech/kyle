# list — Lista

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class list:
    # Datos
    data: ^{any}                      # Colección de items
    
    # Renderizado
    item_height: f32?                 # Altura fija (para virtualización)
    virtual: bool = false             # Virtualizar (solo renderiza visibles)
    
    # Paginación
    per_page: i32?                    # Items por página
    page: ^i32?                       # Página actual
    on_load_more: fn()?               # Callback al cargar más
    
    # Eventos
    on_click: fn(item: any)?          # Callback al hacer click en item
    on_select: fn(item: any)?         # Callback al seleccionar
    
    # Configuración
    selectable: bool = false          # Permitir selección
    multi_select: bool = false        # Selección múltiple
    empty_message: str = "Sin datos"  # Mensaje cuando está vacía
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Lista simple

```kyx
<view>
    @(
        items: ^{str} = {"Manzana", "Banana", "Naranja"}
    )
    
    <list data=@items>
        <item>
            <text value=@item />
        </item>
    </list>
</view>
```

### Lista con objetos

```kyx
<view>
    @(
        users: ^{User} = {}
    )
    
    <list data=@users>
        <item>
            <hstack spacing=12>
                <img src=@item.avatar width=40 height=40 />
                <vstack>
                    <text value=@item.name style=font_weight.bold />
                    <text value=@item.email style=font_weight.normal />
                </vstack>
            </hstack>
        </item>
    </list>
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `data` | `^{any}` | — | **Requerido.** Colección de items |
| `item_height` | `f32?` | `none` | Altura fija (para virtualización) |
| `virtual` | `bool` | `false` | Virtualizar lista |
| `per_page` | `i32?` | `none` | Items por página |
| `page` | `^i32?` | `none` | Página actual |
| `on_load_more` | `fn()?` | `none` | Callback al cargar más |
| `on_click` | `fn(item: any)?` | `none` | Callback al hacer click |
| `on_select` | `fn(item: any)?` | `none` | Callback al seleccionar |
| `selectable` | `bool` | `false` | Permitir selección |
| `multi_select` | `bool` | `false` | Selección múltiple |
| `empty_message` | `str` | `"Sin datos"` | Mensaje cuando está vacía |

---

## Ejemplos Avanzados

### Lista virtualizada (miles de items)

```kyx
<view>
    @(
        big_list: ^{Item} = load_thousand_items()
    )
    
    <list data=@big_list virtual=true item_height=48>
        <item>
            <text value=@item.name />
        </item>
    </list>
</view>
```

### Lista con paginación

```kyx
<view>
    @(
        items: ^{Item} = {}
        page: ^i32 = 1
        per_page = 20
        
        fn load_more():
            new_items = api.get_items(page: page, per_page: per_page)
            items.extend(new_items)
            page += 1
    )
    
    <list data=@items per_page=per_page page=@page on_load_more=@load_more>
        <item>
            <text value=@item.name />
        </item>
    </list>
</view>
```

### Lista seleccionable

```kyx
<view>
    @(
        items: ^{Item} = {}
        selected: ^Item? = none
        
        fn handle_select(item: Item):
            selected = item
    )
    
    <list data=@items selectable=true on_select=@handle_select>
        <item>
            <hstack spacing=8>
                <checkbox checked=@(selected == item) />
                <text value=@item.name />
            </hstack>
        </item>
    </list>
</view>
```

### Lista con acciones

```kyx
<view>
    @(
        items: ^{Item} = {}
        
        fn handle_edit(item: Item):
            navigate("/edit/" + item.id.to_str())
        
        fn handle_delete(item: Item):
            if confirm("¿Eliminar?"):
                api.delete(item.id)
                items.remove(item)
    )
    
    <list data=@items>
        <item>
            <hstack spacing=8>
                <text value=@item.name style=style(flex_grow: 1.0) />
                <button text="Editar" style=Secondary click=@() => handle_edit(item) />
                <button text="Eliminar" style=Danger click=@() => handle_delete(item) />
            </hstack>
        </item>
    </list>
</view>
```

### Lista vacía

```kyx
<view>
    @(
        items: ^{Item} = {}
    )
    
    <list data=@items empty_message="No hay items. Crea uno nuevo.">
        <item>
            <text value=@item.name />
        </item>
    </list>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div class="list">
    <div class="list-item">Item 1</div>
    <div class="list-item">Item 2</div>
    <div class="list-item">Item 3</div>
</div>
```

```css
.list {
    display: flex;
    flex-direction: column;
}
.list-item {
    padding: 12px;
    border-bottom: 1px solid #eee;
}
```

### Desktop (SDL2/Skia)

```kyle
fn render_list(items: {Item}, scroll_y: f32):
    for i, item in items:
        y = i * item_height - scroll_y
        if y + item_height > 0 and y < viewport_height:
            render_item(item, 0, y, width, item_height)
```

### iOS (SwiftUI)

```swift
List(items) { item in
    Text(item.name)
}
```

### Android (Jetpack Compose)

```kotlin
LazyColumn {
    items(items) { item ->
        Text(item.name)
    }
}
```

---

## Virtualización

### ¿Cuándo usar virtual=true?

| Items | Virtualizar | Razón |
|-------|-------------|-------|
| < 100 | ❌ No | No necesario |
| 100-1000 | ⚠️ Opcional | Mejora rendimiento |
| > 1000 | ✅ Sí | Esencial para rendimiento |

### Requisitos para virtualización

1. `item_height` debe ser fijo (no auto)
2. Todos los items deben tener la misma altura
3. No usar layouts complejos dentro del item

---

## Errores Comunes

### ❌ Malo: Lista sin virtualizar con muchos items

```kyx
<list data=@thousand_items>
    <item>...</item>
</list>
```

**Problema:** Renderiza los 1000 items, lento.

### ✅ Bueno: Virtualizar listas grandes

```kyx
<list data=@thousand_items virtual=true item_height=48>
    <item>...</item>
</list>
```

### ❌ Malo: Lógica compleja en el render

```kyx
<list data=@items>
    <item>
        @if item.type == "special":
            <special_view item=@item />
        @else:
            <normal_view item=@item />
    </item>
</list>
```

**Problema:** Dificulta virtualización y mantenimiento.

### ✅ Bueno: Componente separado

```kyx
<list data=@items>
    <item>
        <item_renderer item=@item />
    </item>
</list>
```

---

## Referencias

- [table](table.md)
- [grid](grid.md)
- [Componentes nativos](README.md)
