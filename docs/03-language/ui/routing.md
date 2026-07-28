# Routing y Navegación

**Status:** Draft v3.0
**Date:** 2026-07-12
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [composition.md](composition.md) — Layouts y slots
- [animation.md](animation.md) — Transiciones

---

## 0. Target tipo

```kyle
enum Target:
    web
    macos
    windows
    linux
    ios
    android
```

---

## 1. Arquitectura General

> **`<layout>` es un tipo específico**, no un `<view>` genérico. Es un componente nativo con semántica de wrapper persistente. Debe tener exactamente un `<slot />` (verificado en compilación). El router reconoce `<layout>` y sabe que sus hijos (navbar, sidebar, footer) NO se refrescan al navegar.

```
<app>                                    ← 1 ventana nativa (UIWindowScene / NSWindow / HWND / Activity)
  └── <router>                           ← NavigationStack / History API / NavHost
        ├── <route path="/" ...>         ← home renderizado DENTRO de main
        │     └── <layout>               ← PERSISTE: navbar, sidebar NO se refrescan
        │           └── <slot />         ← SOLO ESTO cambia
        │
        ├── <route path="/login" ...>    ← login DENTRO de blank
        │     └── <layout>
        │           └── <slot />
        │
        ├── <route path="/users/{id}" ...>
        │     └── <layout>
        │           └── <slot />
        │
        └── <route path="*" ...>         ← 404 / catch-all
              └── <layout>
                    └── <slot />
```

### 1.1 Comportamiento por plataforma

Cada backend traduce los tipos nativos según la plataforma. El lenguaje NO expone estas diferencias — `app`, `router`, `view` son el mismo tipo siempre.

| Plataforma | Cómo funciona router |
|-----------|---------------------|
| **web** | SPA — History API + swap innerHTML del slot |
| **macos** | SPA — cambia el contentView de la ventana |
| **windows** | SPA — cambia el panel hijo |
| **linux** | SPA — cambia el widget contenido |
| **ios** | Push/Pop nativo — NavigationStack con animaciones reales |
| **android** | Push/Pop nativo — NavHost con animaciones reales |

---

## 2. Routes Centralizadas en `<router>`

### 2.1 Sintaxis básica

Las rutas se declaran en el `<router>` con `<route>`. NO hay `view()` en los archivos `.kyx`.

```kyx
# app.kyx — Entry point con rutas centralizadas
use views.home
use views.login
use views.user_profile
use views.not_found
use layouts.main

<app title="Mi App">
    <router>
        <route path="/" component=home layout=main title="Inicio" />
        <route path="/login" component=login layout=main title="Login" />
        <route path="/users/{id}" component=user_profile layout=main
            title=@"'Perfil de ' + params.get('id')"
        />
        <route path="*" component=not_found layout=main title="404" />
    </router>
</app>
```

### 2.2 Archivos de vista — sin `view()`

Cada `.kyx` es solo un componente. Sin declaración de ruta:

```kyx
# views/home.kyx — NO tiene view("/")
<view>
    <text value="Bienvenido a Kyle!" />
    <button text="Ir a Login" click=@fn (): navigate("/login") />
</view>
```

```kyx
# views/login.kyx
<view>
    <text_field bind=@email />
    <password_field bind=@password />
    <button text="Ingresar" click=@login />
</view>
```

### 2.3 `<route>` — struct tipado

```kyle
struct RouteConfig:
    path: str                          # "/users/{id}"
    component: ^Component              # qué renderizar (obligatorio)
    layout: ^LayoutComponent           # layout opcional
    title: str                         # título de la vista
    guard: ^&(fn (ctx: NavCtx): bool)? # guard opcional
    icon: str?                         # icono opcional
    lazy: bool = false                 # carga diferida

    target(Target.ios):
        large_title = true
        hides_bottom_bar = false
        transition = TransitionType.slide

    target(Target.macos):
        titlebar_style = TitlebarStyle.unified
        window_size = (1024, 768)

    target(Target.web):
        meta_description = "Descripción SEO"
```

Uso en `.kyx`:

```kyx
<route path="/admin" component=admin_panel layout=admin
    guard=@auth_guard title="Admin Panel"
    target(Target.web):
        meta_description = "Panel de administración"
    target(Target.ios):
        large_title = false
    target(Target.macos):
        window_size = (800, 600)
/>
```

---

## 3. Layouts Persistentes

### 3.1 Definición de layout

```kyx
# layouts/main.kyx
<layout>
    <navbar title="Mi App" />
    <hstack>
        <sidebar />
        <main>
            <slot />    ← contenido de la ruta activa
        </main>
    </hstack>
    <footer />
</layout>
```

```kyx
# layouts/blank.kyx — sin navbar, sidebar, ni footer
<layout>
    <slot />
</layout>
```

### 3.2 Cómo funciona la persistencia

Cuando el usuario navega de `/` a `/login`:

```
Antes:                      Después:
<main>               <main>        ← PERSISTE (no se recrea)
  <navbar />                  <navbar />         ← PERSISTE
  <vstack>                    <vstack>            ← PERSISTE
    <sidebar />                 <sidebar />       ← PERSISTE
    <main>                      <main>            ← PERSISTE
      <slot>                      <slot>
        <home />    →         <login />  ← SOLO ESTO CAMBIA
      </slot>                     </slot>
    </main>                     </main>
  </vstack>                   </vstack>
  <footer />                  <footer />          ← PERSISTE
</main>              </main>
```

El layout se monta UNA VEZ. Solo el `<slot />` se re-renderiza al navegar.

### 3.3 Multi-ruta con layouts diferentes

```kyx
<router>
    <route path="/" component=home layout=main title="Home" />
    <route path="/login" component=login layout=blank title="Login" />
    <route path="/dashboard" component=dashboard layout=main
        guard=@auth_guard title="Dashboard"
    />
    <route path="*" component=not_found layout=main title="404" />
</router>
```

- `/` → main con navbar + sidebar
- `/login` → blank (solo formulario, sin navbar)
- `/dashboard` → main (requiere auth)
- `*` → main (página no encontrada)

---

## 4. Navegación

### 4.1 Programática

```kyle
fn go_to_login():
    navigate("/login")

fn go_to_user(id: i32):
    navigate("/users/" + id.to_str())

fn go_back():
    navigate_back()

fn replace_current(path: str):
    navigate_replace(path)  # sin agregar al historial
```

### 4.2 `<link>` (declarativa)

```kyx
<link to="/dashboard">Ir al Dashboard</link>

<link to=@"'/users/' + user.id.to_str()">Ver Usuario</link>

<link to="/settings" style=Style.nav_link>Configuración</link>
```

### 4.3 Título dinámico

```kyx
# Desde el código
set_title("Login - Mi App")
set_meta("description", "Página de inicio de sesión")
```

```kyx
# Desde la ruta (parámetro title del <route>)
<route path="/profile" component=profile title="Mi Perfil" />
```

```kyx
# Desde la vista (prop title dinámico)
<view title=@"'Bienvenido, ' + user.name">
    <text value="Contenido" />
</view>
```

---

## 5. Parámetros de Ruta

### 5.1 Sintaxis `{param}`

```kyx
<route path="/users/{id}" component=user layout=main
    title=@"'Perfil de ' + params.get('id')"
/>
```

```kyx
# user.kyx
<view>
    params: {str: str} = route_params()
    id: str = params.get("id") ?? ""

    <text value=@"Usuario: " + id />
</view>
```

`route_params()` es una función built-in del router.

### 5.2 Múltiples parámetros

```kyx
<route path="/posts/{year}/{slug}" component=post />
```

```kyx
<view>
    params = route_params()
    year: str = params.get("year") ?? ""
    slug: str = params.get("slug") ?? ""
</view>
```

### 5.3 Parámetros opcionales

```kyx
<route path="/search/{query?}" component=search />
# /search/foo → params.query = "foo"
# /search     → params.query = None
```

---

## 6. Guardias

### 6.1 Protección de rutas

```kyx
<router>
    <route path="/dashboard" component=dashboard layout=main
        guard=@auth_guard title="Dashboard"
    />
    <route path="/admin" component=admin_panel layout=admin
        guard=@admin_guard title="Admin"
    />
</router>
```

### 6.2 Definición de guard

```kyx
fn auth_guard(ctx: NavCtx) bool:
    if !is_authenticated():
        navigate("/login")
        false
    else:
        true

fn admin_guard(ctx: NavCtx) bool:
    if current_user.role != Role.admin:
        navigate("/")
        false
    else:
        true
```

### 6.3 Tipos de guardia

| Guardia | Momento | Uso |
|---------|---------|-----|
| `guard` | Antes de entrar | Auth checks |
| `before_leave` | Antes de salir | Confirmar cambios no guardados |

```kyx
<route path="/edit" component=edit_form
    before_leave=@fn (): has_unsaved_changes() ? confirm("¿Descartar?") : true
/>
```

---

## 7. 404 / Catch-all

```kyx
<route path="*" component=not_found layout=main title="No Encontrado" />
```

```kyx
# not_found.kyx
<view>
    <text value="404 — Página no encontrada" />
    <link to="/">Volver al inicio</link>
</view>
```

---

## 8. Lazy Loading

```kyx
<router>
    <route path="/" component=home />         # eager
    <route path="/heavy" component=heavy
        lazy=true                                   # carga diferida
    />
</router>
```

Web target → code splitting automático:
```javascript
const routes = {
    '/': render_home,
    '/heavy': () => import('./pages/heavy.wasm'),
};
```

---

## 9. Transiciones

```kyx
<router transition=PageSlide transition_duration=300>
    <route path="/" component=home />
    <route path="/login" component=login />
</router>
```

Ver [animation.md](animation.md) para tipos de transición.

---

## 10. Config por Plataforma

### 10.1 Por ruta

```kyx
<route path="/profile" component=profile layout=main
    target(Target.ios):
        large_title = true
        hides_bottom_bar = false
    target(Target.macos):
        titlebar_style = TitlebarStyle.unified
    target(Target.android):
        transition = TransitionType.fade
/>
```

### 10.2 Desde adentro de una vista (dinámico)

Se puede modificar title, icon y configuración nativa desde el `@(...)` de cualquier vista:

```kyx
# views/profile.kyx
<view>
    @(
        set_title(@"Perfil de " + user.name)
        set_icon("person.circle")

        target(Target.ios):
            large_title = true
            hides_bottom_bar = true
            tab_bar_item = TabItem(icon: "person", title: "Perfil")

        target(Target.web):
            meta_description = "Página de perfil de usuario"
            meta_og_image = "/og/profile.png"

        target(Target.macos):
            titlebar_style = TitlebarStyle.unified
            window_size = (800, 600)
    )

    <text value=@"Bienvenido, " + user.name />
</view>
```

Funciones built-in disponibles:

| Función | Descripción |
|---------|-------------|
| `set_title(str)` | Cambia el título de la vista/página |
| `set_icon(str)` | Cambia el icono (SF Symbol, icon name, favicon) |
| `set_meta(key, value)` | Setea meta tags (web: `<meta>`, iOS: info.plist) |
| `target(Target.X):` | Config específica por plataforma |

### 10.3 Global en app.kyx

```kyx
<app title=@"Mi App">
    config:
        target(Target.web):
            port = 8080
        target(Target.macos):
            window_size = (1280, 720)
            menu_bar = true
        target(Target.windows):
            window_size = (1280, 720)
        target(Target.ios):
            bundle_id = "com.miapp"
            deployment_target = "16.0"
        target(Target.android):
            package = "com.miapp"
            min_sdk = 24

    <router>
        ...
    </router>
</app>
```



## 12. Resumen

| Concepto | Sintaxis |
|----------|----------|
| Ruta fija | `<route path="/" component=comp />` |
| Ruta con params | `<route path="/users/{id}" ... />` |
| Layout persistente | `<layout><slot /></layout>` |
| Link | `<link to="/path">texto</link>` |
| Navegación | `navigate("/path")` |
| Atrás | `navigate_back()` |
| Reemplazar | `navigate_replace("/path")` |
| Título dinámico | `set_title("Nuevo Título")` |
| Guard | `<route guard=@fn />` |
| 404 | `<route path="*" component=not_found />` |
| Config plataforma | `target(Target.web): port = 8080` |
| Route struct | `RouteConfig { path, component, layout, title, guard, icon, lazy }` |

---

## 13. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [composition.md](composition.md) — Layouts y slots
- [animation.md](animation.md) — Page transitions
