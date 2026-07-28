# sidebar — Barra Lateral

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class sidebar:
    # Configuración
    width: f32 = 250.0              # Ancho
    collapsible: bool = false       # Se puede colapsar
    collapsed: ^bool = false        # Estado colapsado
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<layout>
    <app_bar title="Mi App" />
    
    <hstack>
        <sidebar>
            <sidebar_item icon="home" label="Inicio" href="/" />
            <sidebar_item icon="users" label="Usuarios" href="/users" />
            <sidebar_item icon="settings" label="Configuración" href="/settings" />
        </sidebar>
        
        <main>
            <slot />
        </main>
    </hstack>
</layout>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `width` | `f32` | `250.0` | Ancho |
| `collapsible` | `bool` | `false` | Se puede colapsar |
| `collapsed` | `^bool` | `false` | Estado colapsado |

---

## Tipos Relacionados

### sidebar_item

```kyle
final class sidebar_item:
    icon: str?                      # Nombre del ícono
    label: str                      # Etiqueta
    href: str?                      # Ruta de navegación
    active: bool = false            # Item activo
    click: fn(event: click_event)?  # Al hacer click
```

---

## Ejemplos Avanzados

### Sidebar colapsable

```kyx
<view>
    @(
        collapsed: ^bool = false
    )
    
    <hstack>
        <sidebar width=250.0 collapsible=true collapsed=@collapsed>
            @if !collapsed:
                <sidebar_item icon="home" label="Inicio" href="/" />
                <sidebar_item icon="users" label="Usuarios" href="/users" />
            @else:
                <sidebar_item icon="home" href="/" />
                <sidebar_item icon="users" href="/users" />
        </sidebar>
        
        <main>
            <button text=@(if collapsed: "Expandir" else: "Colapsar") click=@() => collapsed = !collapsed />
            <slot />
        </main>
    </hstack>
</view>
```

### Con grupos

```kyx
<sidebar>
    <sidebar_group label="Principal">
        <sidebar_item icon="home" label="Inicio" href="/" />
        <sidebar_item icon="dashboard" label="Dashboard" href="/dashboard" />
    </sidebar_group>
    
    <sidebar_group label="Administración">
        <sidebar_item icon="users" label="Usuarios" href="/users" />
        <sidebar_item icon="settings" label="Configuración" href="/settings" />
    </sidebar_group>
</sidebar>
```

---

## Traducción Multiplataforma

### Web

```html
<aside class="sidebar" style="width: 250px; background: #F5F5F5; padding: 16px;">
    <nav>
        <a href="/" class="sidebar-item">🏠 Inicio</a>
        <a href="/users" class="sidebar-item">👥 Usuarios</a>
    </nav>
</aside>
```

### Mobile (Drawer)

En mobile, el sidebar se convierte automáticamente en un drawer:

```kyx
<view>
    @(
        drawer_open: ^bool = false
    )
    
    <app_bar>
        <app_bar_navigation>
            <button icon="menu" click=@() => drawer_open = true />
        </app_bar_navigation>
    </app_bar>
    
    <drawer open=@drawer_open on_close=@() => drawer_open = false>
        <sidebar>
            <sidebar_item icon="home" label="Inicio" href="/" />
            <sidebar_item icon="users" label="Usuarios" href="/users" />
        </sidebar>
    </drawer>
</view>
```

### iOS (SwiftUI)

```swift
NavigationSplitView {
    List {
        NavigationLink("Inicio", destination: HomeView())
        NavigationLink("Usuarios", destination: UsersView())
    }
    .navigationTitle("Menú")
} detail: {
    ContentView()
}
```

### Android (Jetpack Compose)

```kotlin
val scaffoldState = rememberScaffoldState()

Scaffold(
    scaffoldState = scaffoldState,
    drawerContent = {
        ModalDrawerContent {
            Text("Inicio")
            Text("Usuarios")
        }
    }
) {
    // Contenido principal
}
```

---

## Referencias

- [app_bar](app_bar.md)
- [bottom_nav](bottom_nav.md)
- [Componentes nativos](README.md)
