# tab_bar — Barra de Pestañas

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class tab_bar:
    # Valor
    bind: ^str?                     # Binding a la pestaña activa
    
    # Configuración
    style: style_class?             # Clase de estilo
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

```kyx
<view>
    @(
        active_tab: ^str = "general"
    )
    
    <tab_bar bind=@active_tab>
        <tab name="general" label="General">
            <text value="Contenido de General" />
        </tab>
        <tab name="security" label="Seguridad">
            <text value="Contenido de Seguridad" />
        </tab>
        <tab name="notifications" label="Notificaciones">
            <text value="Contenido de Notificaciones" />
        </tab>
    </tab_bar>
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^str?` | `none` | Binding a pestaña activa |
| `style` | `style_class?` | `none` | Clase de estilo |

---

## Tipos Relacionados

### tab

```kyle
final class tab:
    name: str                       # Identificador único
    label: str                      # Etiqueta mostrada
    icon: str?                      # Ícono (opcional)
    disabled: bool = false          # Pestaña deshabilitada
```

---

## Ejemplos Avanzados

### Tabs con íconos

```kyx
<tab_bar bind=@active_tab>
    <tab name="home" label="Inicio" icon="home">
        <home_view />
    </tab>
    <tab name="search" label="Buscar" icon="search">
        <search_view />
    </tab>
    <tab name="profile" label="Perfil" icon="person">
        <profile_view />
    </tab>
</tab_bar>
```

### Tabs con badge

```kyx
<tab_bar bind=@active_tab>
    <tab name="inbox" label="Bandeja">
        <hstack spacing=4>
            <text value="Bandeja" />
            <badge value=@unread_count style=Badge />
        </hstack>
        <inbox_view />
    </tab>
    <tab name="sent" label="Enviados">
        <sent_view />
    </tab>
</tab_bar>
```

### Tabs scrollables (muchas pestañas)

```kyx
<scroll horizontal=true>
    <tab_bar bind=@active_tab>
        @for(category in categories):
            <tab name=@category.id label=@category.name>
                <category_view category=@category />
            </tab>
    </tab_bar>
</scroll>
```

---

## Traducción Multiplataforma

### Web

```html
<div class="tab-bar">
    <button class="tab active" onclick="selectTab('general')">General</button>
    <button class="tab" onclick="selectTab('security')">Seguridad</button>
</div>
<div class="tab-content">
    <!-- Contenido de la pestaña activa -->
</div>
```

### Mobile (Bottom Navigation)

En mobile, las tabs pueden convertirse en bottom navigation:

```kyx
<view>
    @if current_breakpoint() == breakpoint.mobile:
        <bottom_nav bind=@active_tab>
            <bottom_nav_item name="home" label="Inicio" icon="home" />
            <bottom_nav_item name="search" label="Buscar" icon="search" />
            <bottom_nav_item name="profile" label="Perfil" icon="person" />
        </bottom_nav>
    @else:
        <tab_bar bind=@active_tab>
            <tab name="home" label="Inicio" icon="home">
                <home_view />
            </tab>
            <!-- ... -->
        </tab_bar>
</view>
```

### iOS (SwiftUI)

```swift
TabView(selection: $active_tab) {
    HomeView()
        .tabItem {
            Label("Inicio", systemImage: "house")
        }
        .tag("home")
    
    SearchView()
        .tabItem {
            Label("Buscar", systemImage: "magnifyingglass")
        }
        .tag("search")
}
```

### Android (Jetpack Compose)

```kotlin
var selectedTab by remember { mutableStateOf(0) }
val tabs = listOf("Inicio", "Buscar", "Perfil")

Column {
    TabRow(selectedTabIndex = selectedTab) {
        tabs.forEachIndexed { index, title ->
            Tab(
                selected = selectedTab == index,
                onClick = { selectedTab = index },
                text = { Text(title) }
            )
        }
    }
    
    // Contenido de la pestaña
    when (selectedTab) {
        0 -> HomeView()
        1 -> SearchView()
        2 -> ProfileView()
    }
}
```

---

## Referencias

- [bottom_nav](bottom_nav.md)
- [sidebar](sidebar.md)
- [Componentes nativos](README.md)
