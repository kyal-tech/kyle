# Kyle UI Syntax Specification

**Version:** 2.0
**Status:** Specification

> **Sistema de estilos:** Ver [`docs/03-language/ui/style-system.md`](../../03-language/ui/style-system.md) para el sistema de estilos tipado (sin CSS).
> **Estado y eventos:** Ver [`docs/03-language/ui/state-events.md`](../../03-language/ui/state-events.md) para estado, binding y eventos.
> **Routing:** Ver [`docs/03-language/ui/routing.md`](../../03-language/ui/routing.md) para navegación y rutas.
> **Animaciones:** Ver [`docs/03-language/ui/animation.md`](../../03-language/ui/animation.md) para animaciones y transiciones.
> **Composición:** Ver [`docs/03-language/ui/composition.md`](../../03-language/ui/composition.md) para layouts, slots y composición.

---

## 1. Philosophy

Kyle UI es un sistema de UI declarativo construido sobre el lenguaje Kyle.

Un archivo `.kyx` representa un **componente**. Se importa explícitamente con `use X.Y` y se usa como `<Y />` en templates.

No existe JavaScript, HTML ni CSS. Todo es Kyle tipado.

### 1.1 Estructura de proyecto

```
mi-app/
├── app.kyx              ← Entry point (solo esto en raíz)
├── src/
│   ├── views/           ← Vistas (páginas con ruta)
│   │   ├── home.kyx
│   │   └── not_found.kyx
│   ├── layouts/         ← Layouts persistentes
│   │   └── main.kyx
│   ├── components/      ← Componentes reutilizables
│   │   └── counter.kyx
│   └── lib.ky           ← Lógica de negocio
└── ky.toml
```

---

## 2. Componentes nativos = Tipos

Cada componente nativo es un **tipo** (`final class`). Se usa tanto en markup (`.kyx`) como programáticamente (`.ky`). El backend traduce cada tipo a su equivalente nativo — eso es interno, no parte del lenguaje.

### 2.1 Tipos nativos (built-in)

```kyle
# Contenedores
final class app        # raíz de la aplicación (1 por app)
final class router     # navegador de rutas
final class route      # definición de ruta (path, component, layout)
final class layout     # wrapper persistente con <slot />
final class view       # contenedor genérico
final class group      # agrupación lógica (invisible)
final class scroll     # contenedor scrollable (con direction: vertical/horizontal/both)
enum direction: vertical, horizontal, both

# Layout
final class vstack     # columna flex (vertical)
final class hstack     # fila flex (horizontal)
final class zstack     # superposición (eje Z)
final class spacer     # espaciado flexible
final class divider    # línea separadora

# Elementos
final class text       # texto
final class button     # botón
final class image      # imagen
final class link       # navegación declarativa
final class input      # entrada de texto (genérico)
final class text_field       # campo de texto
final class password_field   # campo de contraseña
final class checkbox    # casilla de verificación
final class radio      # botón de opción
final class switch     # interruptor
final class slider     # deslizador
final class progress   # barra de progreso
final class spinner    # indicador de carga

# Overlays
final class modal      # ventana modal
final class sheet      # hoja (bottom sheet)
final class alert      # alerta / diálogo
final class tooltip    # tooltip

# Navegación
final class navbar     # barra de navegación
final class sidebar    # barra lateral
final class tab_bar    # barra de pestañas
final class footer     # pie de página
```

### 2.2 Imports: `use X.Y`

Los componentes se importan explícitamente con la misma sintaxis que Kyle:

```kyx
# app.kyx
use views.home
use layouts.main
use components.header
use components.footer
```

La resolución busca en este orden:
1. `./views/home.kyx` (relativo al archivo actual)
2. `src/views/home.kyx` (proyecto estándar)
3. `views/home.kyx` (raíz del proyecto, legacy)

Una vez importado, se usa como tag nativo:

```kyx
<app>
    <header />
    <router>
        <route path="/" component=home layout=main />
    </router>
    <footer />
</app>
```

Los componentes nativos (`view`, `vstack`, `text`, etc.) NO requieren import — son built-in.

### 2.2.1 Importar solo estilos

Un archivo con solo declaraciones `style<>` se importa igual y sus estilos se mezclan automáticamente:

```kyx
# src/components/theme.kyx — solo estilos, sin markup
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    border_radius = 8

style<text> Title:
    font_size = 24
    font_weight = font_weight.bold
```

```kyx
# app.kyx
use components.theme

<app title="App">
    <router>
        <route path="/" component=home layout=main />
    </router>
</app>
```

El nombre `theme` queda disponible como referencia (aunque no tenga body). Los estilos `Primary` y `Title` se pueden usar en cualquier vista del proyecto.

### 2.3 Uso declarativo (`.kyx`)

```kyx
<view>
    <vstack>
        <text value="Hola Mundo" />
        <button text="Click" click=@handler />
    </vstack>
</view>
```

### 2.4 Uso programático (`.ky`)

Como son tipos, se pueden instanciar desde código Kyle:

```kyle
fn build_dynamic_view(items: {str}) view:
    container = view()
    stack = vstack()
    for item in items:
        label = text(value: item)
        stack.add_child(label)
    container.add_child(stack)
    container
```

```kyle
fn show_loading():
    modal(
        content: vstack(
            children: {spinner(), text(value: "Cargando...")}
        )
    )
```

### 2.4 Layout: `vstack`, `hstack`, `zstack`

Son los tres tipos de layout. Cada uno organiza hijos en una dirección:

```kyle
# vstack: vertical (columna) — hijos de arriba a abajo
final class vstack:
    alignment: alignment           # cómo alinear horizontalmente
    spacing: f32 = 0               # espacio entre hijos
    padding: spacing?              # padding interno
    scroll: bool = false           # ← TRUE = scroll vertical automático

# hstack: horizontal (fila) — hijos de izquierda a derecha
final class hstack:
    alignment: alignment           # cómo alinear verticalmente
    spacing: f32 = 0
    padding: spacing?
    scroll: bool = false           # ← TRUE = scroll horizontal automático

# zstack: superposición — hijos apilados en eje Z
final class zstack:
    alignment: alignment           # alineación de todos los hijos
    padding: spacing?

# scroll: contenedor scrollable genérico (para contenido no-stack)
final class scroll:
    direction: direction           # vertical | horizontal | both
```

Ejemplos:

```kyx
# vstack — columna, centrada, con gap de 12px
<vstack alignment=alignment.center spacing=12>
    <text value="Uno" />
    <text value="Dos" />
    <text value="Tres" />
</vstack>

# hstack — fila, espacio entre elementos
<hstack alignment=alignment.center spacing=8>
    <button text="Aceptar" />
    <button text="Cancelar" />
</hstack>

# zstack — superposición (imagen de fondo + texto encima)
<zstack>
    <image src="fondo.jpg" />
    <text value="Texto sobre imagen" />
</zstack>

# vstack con padding
<vstack spacing=16 padding=spacing.all(24)>
    <text value="Título" typography=Title />
    <text value="Cuerpo del contenido" />
</vstack>
```

Los stacks se adaptan al contenido (fit content). Para scroll:

```kyx
# Opción recomendada: scroll=true en el stack
<vstack scroll=true spacing=8>
    @for(item in items):
        <item_card data=@item />
</vstack>

# scroll horizontal
<hstack scroll=true spacing=12>
    @for(card in cards):
        <card data=@card />
</hstack>

# scroll genérico para contenido que no es stack
<scroll direction=vertical>
    <text value="Mucho contenido que necesita scroll..." />
</scroll>
```

### 2.5 `layout` — reglas del tipo

`layout` es un tipo específico con semántica de wrapper persistente:

1. Debe tener exactamente un `slot` (verificado en compilación)
2. Puede tener otros hijos (navbar, sidebar, footer) que PERSISTEN al navegar
3. El `slot` es donde el router inyecta el contenido de la ruta activa
4. Los hijos del layout NO se re-renderizan al cambiar de ruta

```kyx
<layout>
    <navbar title="Mi App" />
    <hstack>
        <sidebar />
        <main>
            <slot />
        </main>
    </hstack>
</layout>
```

---

## 3. El bloque `@(...)` — Todo el código Kyle

El bloque `@(...)` es donde va TODO el código Kyle: variables, props, funciones, estado, contexto.
Separa el código de la marcación (markup).

### 3.1 Sintaxis

```kyx
<view>
    @(
        # ── Variables / Props ──
        count: ^i32                # ← público = prop
        label: str                 # ← público = prop
        on_click: ^&(fn ())        # ← callback como prop
        color: str = "#333"        # ← prop con default (opcional)

        # ── Internos (NO son props) ──
        _step: i32 = 1             # ← _ = interno, no es prop
        __cache: [i32] = {}        # ← __ = privado, no es prop
        _fn helper(this):          # ← función interna
            count += _step

        # ── Funciones públicas = callbacks ──
        fn increment():
            count += _step
    )

    <text value=@"Valor: " + count.to_str() />
    <button text="+" click=@increment />
</view>
```

Uso:
```kyx
<counter count=@mi_var label="Clicks" on_click=@mi_callback color="#FF0000" />
```

### 3.2 Props via Visibilidad

Dentro de `@(...)`, lo que es prop se determina por **visibilidad** (convención Kyle):

| Declaración dentro de `@(...)` | ¿Es prop? | Visible desde fuera |
|-------------------------------|-----------|-------------------|
| `count: ^i32` | ✅ Sí | Se pasa como atributo |
| `label: str` | ✅ Sí | Se pasa como string |
| `fn handle_click(this)` | ✅ Sí | Se pasa como callback |
| `_internal: str` | ❌ No | Uso interno del componente |
| `__cache: [i32]` | ❌ No | Privado del módulo |
| `_fn helper(this)` | ❌ No | Función interna |

### 3.3 Props con default (opcional)

```kyx
@(
    name: str = "Invitado"         # opcional — default "Invitado"
    age: i32 = 18                  # opcional — default 18
    theme: Theme = Theme.light     # opcional — default light
)
```

### 3.4 Expresiones inline con `@`

Para expresiones pequeñas dentro del markup:

```kyx
<text value=@user.name />
<button click=@fn (): navigate("/login") />
```

---

## 4. Variables y Binding

### 5.1 Variables normales (no reactivas)

```kyx
<view>
    @(
        total = products.count     # variable normal dentro de @(...)
    )
    <text value=@total />
</view>
```

### 5.2 Binding bidireccional

```kyx
<text_field bind=@email />
<checkbox bind=@accept_terms />
```

### 5.3 Binding one-way (estado → UI)

```kyx
<text value=@user.name />          # se actualiza cuando user.name cambia
```

### 5.4 Binding a props

```kyx
<child_component prop=@parent_var />
```

---

## 5. Condicionales

```kyx
@if(user.is_admin):
    <button text="Delete" />

@if(user.is_admin):
    <admin_panel />
@else:
    <user_panel />
```

---

## 6. Match

```kyx
@match(state):
    Loading:
        <spinner />
    Success:
        <dashboard />
    Error:
        <error_view />
```

---

## 7. For

```kyx
@for(product in products):
    <product_card product=@product />
```

---

## 8. Eventos

```kyx
<button click=@login />
<text_field change=@on_change />
<text_field input=@on_input />
<checkbox change=@on_toggle />
<form submit=@handle_submit />
<view mouse_enter=@on_hover />
<view keydown=@on_keypress />
<view scroll=@on_scroll />
```

---

## 9. Ciclo de Vida

```kyle
fn on_created(this):
    # Una vez, después del constructor

fn on_mounted(this):
    # Cuando el componente se monta

fn on_updated(this, changed: {str}):
    # Cuando cambian props (changed = set de props que cambiaron)

fn on_unmounted(this):
    # Cleanup

fn on_error(this, error: str):
    # Error boundary
```

---

## 10. Estilos

### 11.1 Estilos declarados

```kyle
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    border_radius = 8
    padding = spacing.all(12)

style<button> Secondary:
    background = color("transparent")
    color = color("#0066FF")
    border = border(2, color("#0066FF"), BorderStyle.solid)
```

### 11.2 Aplicar estilo

```kyx
<button style=Primary text="Ingresar" />
<button tpl=Elevated>
    <text value="Contenido" />
</button>
```

### 11.3 Inline

```kyx
<text style=style(color: color("#FF0000"), font_size: 16) value="Error" />
```

---

## 11. Recursos (Theme)

### 12.1 Themes

```kyle
theme LightTheme:
    primary = color("#0066FF")
    background = color("#FFFFFF")
    on_background = color("#1A1A1A")

theme DarkTheme: LightTheme:
    primary = color("#4D8EFF")
    background = color("#121212")
```

```kyx
<app theme=@LightTheme>
    ...
</app>
```

---

## 12. Ejemplo completo: Login

```kyx
# app.kyx
use views.home
use views.login
use views.not_found
use layouts.main

<app title="Mi App">
    <router>
        <route path="/" component=home layout=main title="Inicio" />
        <route path="/login" component=login layout=main title="Login" />
        <route path="*" component=not_found layout=main title="404" />
    </router>
</app>
```

```kyx
# src/views/login.kyx
<view>
    @(
        email: ^str = ""
        password: ^str = ""
        loading: ^bool = false

        fn handle_submit():
            loading = true
            navigate("/dashboard")
    )

    <vstack alignment=alignment.center>
        <text value="Login" style=Title />
        <text_field bind=@email placeholder="Email" />
        <password_field bind=@password placeholder="Contraseña" />
        <button style=Primary text=@"Ingresar" disabled=@loading click=@handle_submit />
    </vstack>
</view>
```

```kyx
# src/layouts/main.kyx
<layout>
    <navbar title="Mi App" />
    <hstack>
        <sidebar />
        <main>
            <slot />
        </main>
    </hstack>
</layout>
```

---

## 13. Referencias

- [routing.md](../ui/routing.md) — Routing y navegación
- [state-events.md](../ui/state-events.md) — Estado y eventos
- [style-system.md](../ui/style-system.md) — Sistema de estilos
- [composition.md](../ui/composition.md) — Layouts, slots, composición
- [animation.md](../ui/animation.md) — Animaciones
