# table — Tabla

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class table:
    # Datos
    data: ^{any}                      # Colección de filas
    
    # Columnas
    columns: {column_def}             # Definición de columnas
    
    # Ordenamiento
    sortable: bool = false            # Permitir ordenamiento
    sort_column: str?                 # Columna actual de orden
    sort_direction: sort_dir?         # Dirección (asc/desc)
    on_sort: fn(column: str)?         # Callback al ordenar
    
    # Paginación
    per_page: i32?                    # Filas por página
    page: ^i32?                       # Página actual
    on_page_change: fn(page: i32)?    # Callback al cambiar página
    
    # Selección
    selectable: bool = false          # Permitir selección
    selected_rows: ^{i32}?            # Índices de filas seleccionadas
    on_select: fn(rows: {i32})?       # Callback al seleccionar
    
    # Configuración
    striped: bool = false             # Filas alternadas
    hoverable: bool = true            # Resaltar al pasar mouse
    empty_message: str = "Sin datos"  # Mensaje cuando está vacía
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Tabla simple

```kyx
<view>
    @(
        users: ^{User} = {}
    )
    
    <table data=@users>
        <col field="name" label="Nombre" />
        <col field="email" label="Email" />
        <col field="age" label="Edad" />
    </table>
</view>
```

### Tabla con ordenamiento

```kyx
<table data=@users sortable=true on_sort=@handle_sort>
    <col field="name" label="Nombre" sortable=true />
    <col field="email" label="Email" sortable=true />
    <col field="age" label="Edad" sortable=true />
</table>
```

### Tabla con paginación

```kyx
<table data=@users per_page=20 page=@current_page on_page_change=@change_page>
    <col field="name" label="Nombre" />
    <col field="email" label="Email" />
</table>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `data` | `^{any}` | — | **Requerido.** Colección de filas |
| `sortable` | `bool` | `false` | Permitir ordenamiento |
| `sort_column` | `str?` | `none` | Columna actual de orden |
| `sort_direction` | `sort_dir?` | `none` | Dirección (asc/desc) |
| `on_sort` | `fn(column: str)?` | `none` | Callback al ordenar |
| `per_page` | `i32?` | `none` | Filas por página |
| `page` | `^i32?` | `none` | Página actual |
| `on_page_change` | `fn(page: i32)?` | `none` | Callback al cambiar página |
| `selectable` | `bool` | `false` | Permitir selección |
| `striped` | `bool` | `false` | Filas alternadas |
| `hoverable` | `bool` | `true` | Resaltar al pasar mouse |
| `empty_message` | `str` | `"Sin datos"` | Mensaje cuando está vacía |

---

## Tipos Relacionados

### column_def

```kyle
final class column_def:
    field: str                        # Nombre del campo en los datos
    label: str                        # Etiqueta de la columna
    width: length?                    # Ancho de la columna
    sortable: bool = false            # Si esta columna es ordenable
    align: text_align = text_align.left  # Alineación
    formatter: fn(any) -> str?        # Formateador personalizado
```

### sort_dir

```kyle
enum sort_dir:
    asc                             # Ascendente
    desc                            # Descendente
```

---

## Ejemplos Avanzados

### Tabla completa con todas las features

```kyx
<view>
    @(
        users: ^{User} = {}
        sort_col: ^str? = none
        sort_dir: ^sort_dir = sort_dir.asc
        page: ^i32 = 1
        selected: ^{i32} = {}
        
        fn handle_sort(column: str):
            if sort_col == column:
                sort_dir = if sort_dir == sort_dir.asc: sort_dir.desc else: sort_dir.asc
            else:
                sort_col = column
                sort_dir = sort_dir.asc
        
        fn handle_delete_selected():
            for idx in selected.sort_desc():
                users.remove_at(idx)
            selected = {}
    )
    
    <table 
        data=@users 
        sortable=true 
        sort_column=@sort_col 
        sort_direction=@sort_dir
        on_sort=@handle_sort
        per_page=20 
        page=@page
        selectable=true
        selected_rows=@selected
        striped=true
    >
        <col field="name" label="Nombre" sortable=true />
        <col field="email" label="Email" sortable=true />
        <col field="age" label="Edad" sortable=true align=text_align.center />
        <col field="actions" label="Acciones" width=length.px(200)>
            <hstack spacing=4>
                <button text="Editar" style=Secondary click=@(row) => edit(row) />
                <button text="Eliminar" style=Danger click=@(row) => delete(row) />
            </hstack>
        </col>
    </table>
    
    @if selected.len() > 0:
        <button text=@"Eliminar (" + selected.len().to_str() + ")" style=Danger click=@handle_delete_selected />
</view>
```

### Tabla con formateo personalizado

```kyx
<table data=@orders>
    <col field="id" label="ID" />
    <col field="date" label="Fecha" formatter=@(val) => format_date(val) />
    <col field="amount" label="Monto" formatter=@(val) => "$" + val.to_str() align=text_align.right />
    <col field="status" label="Estado">
        <badge value=@item.status style=@get_status_style(item.status) />
    </col>
</table>
```

### Tabla con expansión de filas

```kyx
<view>
    @(
        users: ^{User} = {}
        expanded: ^{i32} = {}
        
        fn toggle_expand(idx: i32):
            if expanded.contains(idx):
                expanded.remove(idx)
            else:
                expanded.add(idx)
    )
    
    <table data=@users>
        <col field="name" label="Nombre">
            <hstack spacing=8>
                <button text=@(if expanded.contains(idx): "▼" else: "▶") click=@() => toggle_expand(idx) />
                <text value=@item.name />
            </hstack>
        </col>
        <col field="email" label="Email" />
        
        @if expanded.contains(idx):
            <row_expansion>
                <view style=style(padding: spacing.all(16), background: color("#F5F5F5"))>
                    <text value=@"Detalles de " + item.name />
                    <text value=@"Teléfono: " + item.phone />
                    <text value=@"Dirección: " + item.address />
                </view>
            </row_expansion>
    </table>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<table>
    <thead>
        <tr>
            <th onclick="sort('name')">Nombre ↑</th>
            <th onclick="sort('email')">Email</th>
        </tr>
    </thead>
    <tbody>
        <tr>
            <td>Juan</td>
            <td>juan@email.com</td>
        </tr>
    </tbody>
</table>
```

### Desktop (SDL2/Skia)

```kyle
fn render_table(data: {User}, columns: {column_def}):
    # Header
    for i, col in columns:
        x = i * col_width
        skia_draw_text(x, 0, col.label, header_color)
    
    # Rows
    for row_idx, row in data:
        y = (row_idx + 1) * row_height
        for col_idx, col in columns:
            x = col_idx * col_width
            value = row.get_field(col.field)
            skia_draw_text(x, y, value, text_color)
```

### iOS (SwiftUI)

```swift
Table(users) {
    TableColumn("Nombre", value: \.name)
    TableColumn("Email", value: \.email)
}
```

### Android (Jetpack Compose)

```kotlin
Table {
    TableRow {
        Text("Nombre")
        Text("Email")
    }
    users.forEach { user ->
        TableRow {
            Text(user.name)
            Text(user.email)
        }
    }
}
```

---

## Errores Comunes

### ❌ Malo: Tabla sin paginación con muchos datos

```kyx
<table data=@thousand_rows>
    <col field="name" label="Nombre" />
</table>
```

**Problema:** Renderiza todas las filas, muy lento.

### ✅ Bueno: Paginar tablas grandes

```kyx
<table data=@thousand_rows per_page=50 page=@page>
    <col field="name" label="Nombre" />
</table>
```

### ❌ Malo: Columnas sin ancho definido

```kyx
<table data=@data>
    <col field="long_field_name" label="Campo muy largo" />
</table>
```

**Problema:** Layout impredecible.

### ✅ Bueno: Definir anchos

```kyx
<table data=@data>
    <col field="name" label="Nombre" width=length.px(200) />
    <col field="email" label="Email" width=length.fill />
</table>
```

---

## Referencias

- [list](list.md)
- [grid](grid.md)
- [Componentes nativos](README.md)
