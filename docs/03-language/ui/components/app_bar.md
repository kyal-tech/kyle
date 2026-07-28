# app_bar — Barra Superior

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class app_bar:
    # Contenido
    title: str?                     # Título de la app/vista
    
    # Configuración
    style: style_class?             # Clase de estilo
    elevation: f32 = 4.0            # Sombra
    transparent: bool = false       # Fondo transparente
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<app_bar title="Mi Aplicación">
</app_bar>
```

### Con acciones

```kyx
<app_bar title="Mi Aplicación">
    <app_bar_actions>
        <button icon="search" click=@handle_search />
        <button icon="settings" click=@handle_settings />
    </app_bar_actions>
</app_bar>
```

### Con navegación (back button)

```kyx
<app_bar title="Detalle">
    <app_bar_navigation>
        <button icon="arrow_back" click=@go_back />
    </app_bar_navigation>
    <app_bar_actions>
        <button icon="more_vert" click=@show_menu />
    </app_bar_actions>
</app_bar>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `title` | `str?` | `none` | Título |
| `style` | `style_class?` | `none` | Clase de estilo |
| `elevation` | `f32` | `4.0` | Sombra |
| `transparent` | `bool` | `false` | Fondo transparente |

---

## Slots

| Slot | Descripción |
|------|-------------|
| `app_bar_navigation` | Botón de navegación (back, menu) |
| (default) | Título personalizado |
| `app_bar_actions` | Botones de acción (derecha) |

---

## Ejemplos Avanzados

### Con búsqueda

```kyx
<view>
    @(
        searching: ^bool = false
        search_query: ^str = ""
    )
    
    <app_bar>
        @if searching:
            <text_field 
                bind=@search_query 
                placeholder="Buscar..."
                focus=@() => none
            />
            <button icon="close" click=@() => searching = false />
        @else:
            <text value="Mi App" />
            <app_bar_actions>
                <button icon="search" click=@() => searching = true />
            </app_bar_actions>
    </app_bar>
</view>
```

### Con perfil de usuario

```kyx
<app_bar title="Dashboard">
    <app_bar_actions>
        <button icon="notifications" click=@show_notifications />
        <button click=@show_profile>
            <img src=@user.avatar width=32 height=32 style=style(border_radius: 16) />
        </button>
    </app_bar_actions>
</app_bar>
```

---

## Traducción Multiplataforma

### Web

```html
<header class="app-bar" style="height: 56px; background: #0066FF; color: white; display: flex; align-items: center; padding: 0 16px;">
    <h1>Mi Aplicación</h1>
    <div class="actions" style="margin-left: auto;">
        <button>🔍</button>
        <button>⚙️</button>
    </div>
</header>
```

### iOS (SwiftUI)

```swift
NavigationView {
    ContentView()
        .navigationTitle("Mi Aplicación")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(action: handle_search) { Image(systemName: "magnifyingglass") }
            }
        }
}
```

### Android (Jetpack Compose)

```kotlin
TopAppBar(
    title = { Text("Mi Aplicación") },
    actions = {
        IconButton(onClick = { handle_search() }) {
            Icon(Icons.Default.Search, contentDescription = "Buscar")
        }
    }
)
```

---

## Diferencia con navbar

| Característica | app_bar | navbar (legacy) |
|----------------|---------|-----------------|
| Nombre | ✅ Actual, multiplataforma | ❌ Específico de web |
| Mobile | ✅ Nativo | ❌ No optimizado |
| Desktop | ✅ Adaptable | ❌ Solo web |

**Usar `app_bar`** en todos los casos nuevos. `navbar` es legacy.

---

## Referencias

- [sidebar](sidebar.md)
- [tab_bar](tab_bar.md)
- [Componentes nativos](README.md)
