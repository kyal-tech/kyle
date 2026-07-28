# Arquitectura Multiplataforma Kyle UI

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## 1. Filosofía de Diseño

Kyle UI sigue principios que garantizan consistencia en todas las plataformas:

### 1.1 Un solo código, múltiples targets

```kyx
# Este código funciona en web, desktop, iOS, Android
<view>
    @(
        count: ^i32 = 0
        fn increment():
            count += 1
    )
    
    <text value=@"Contador: " + count.to_str() />
    <button text="+" click=@increment />
</view>
```

### 1.2 Principios fundamentales

| Principio | Descripción |
|-----------|-------------|
| **Tipado fuerte** | Todo verificado en compilación, no runtime |
| **snake_case** | Funciones, tipos, propiedades siempre snake_case |
| **Composición** | Componentes pequeños combinables, no widgets monolíticos |
| **Declarativo** | Describe QUÉ mostrar, no CÓMO renderizar |
| **Platform-agnostic** | Lógica universal, rendering específico por target |

### 1.3 Anti-patrones evitados

**❌ NO hacemos (lecciones de otros frameworks):**

```kyle
# Flutter: 200+ widgets, imposible de memorizar
Container(
  padding: EdgeInsets.all(16),
  child: Column(
    children: [
      Text("Hola"),
      ElevatedButton(onPressed: () {}, child: Text("Click"))
    ]
  )
)

# Kyle: Componentes nativos mínimos, composición libre
<vstack spacing=16>
    <text value="Hola" />
    <button text="Click" click=@handler />
</vstack>
```

**❌ NO hacemos:**
- Widgets pre-diseñados con estilos fijos (Material, Cupertino)
- Jerarquías profundas innecesarias
- Props mágicas que funcionan solo en ciertos contextos
- Mezcla de lógica de UI con lógica de negocio
- Tipado débil (any, dynamic)

**✅ SÍ hacemos:**
- 20-30 componentes nativos esenciales
- Estilos tipados y reutilizables
- Composición explícita con layouts
- Separación clara: `@(...)` para lógica, markup para estructura
- Todo fuertemente tipado, errores en compilación

---

## 2. Componentes Nativos

### 2.1 Categorías

| Categoría | Componentes | Propósito |
|-----------|-------------|-----------|
| **Layout** | `view`, `vstack`, `hstack`, `zstack`, `scroll`, `spacer`, `divider` | Estructura y organización |
| **Texto** | `text`, `link` | Contenido textual y navegación |
| **Input** | `button`, `text_field`, `password_field`, `checkbox`, `radio`, `switch`, `slider`, `select`, `file_picker` | Interacción del usuario |
| **Media** | `img`, `video`, `audio` | Contenido multimedia |
| **Feedback** | `progress`, `spinner`, `skeleton` | Indicadores de estado |
| **Overlay** | `modal`, `sheet`, `alert`, `tooltip`, `toast` | Capas superiores |
| **Navegación** | `app_bar`, `side_bar`, `tab_bar`, `bottom_nav` | Navegación entre vistas |
| **Datos** | `list`, `table`, `grid` | Mostrar colecciones |
| **Contenedor** | `app`, `router`, `route`, `layout`, `slot` | Estructura de la app |

### 2.2 Principios de diseño de componentes

**1. Cada componente es un tipo Kyle:**

```kyle
final class button:
    text: str
    style: style_class?
    disabled: bool = false
    loading: bool = false
    icon: str?
    
    # Eventos
    click: fn(event: click_event)?
    
    # Accesibilidad
    aria_label: str?
    aria_hidden: bool = false
```

**2. Props tipadas, no strings mágicos:**

```kyle
# ❌ Malo
<button type="primary" size="large" />

# ✅ Bueno
<button style=Primary size=size.large />
```

**3. snake_case consistente:**

```kyle
# ❌ Malo
<button onClick={...} borderRadius={8} />

# ✅ Bueno
<button click=@handler border_radius=8 />
```

---

## 3. Traducción Multiplataforma

### 3.1 Web (HTML/CSS/JS)

```kyx
<button style=Primary text="Click" click=@handler />
```

↓ Compila a:

```html
<button class="Primary" onclick="handler()">Click</button>
```

```css
.Primary {
    background: #0066FF;
    color: #FFFFFF;
    padding: 12px 24px;
    border-radius: 8px;
}
```

### 3.2 Desktop (SDL2/Skia)

```kyx
<button style=Primary text="Click" click=@handler />
```

↓ Compila a:

```kyle
fn render_button(x: f32, y: f32):
    skia_fill_rounded_rect(x, y, 100, 40, 8.0, color("#0066FF"))
    skia_draw_text(x + 50, y + 20, "Click", color("#FFFFFF"))
    
    if mouse_in_rect(x, y, 100, 40) and mouse_clicked():
        handler()
```

### 3.3 iOS (SwiftUI)

```kyx
<button style=Primary text="Click" click=@handler />
```

↓ Compila a:

```swift
Button(action: handler) {
    Text("Click")
}
.padding(.horizontal, 24)
.padding(.vertical, 12)
.background(Color(hex: "#0066FF"))
.foregroundColor(.white)
.cornerRadius(8)
```

### 3.4 Android (Jetpack Compose)

```kyx
<button style=Primary text="Click" click=@handler />
```

↓ Compila a:

```kotlin
Button(
    onClick = { handler() },
    modifier = Modifier.padding(horizontal = 24.dp, vertical = 12.dp),
    colors = ButtonDefaults.buttonColors(
        backgroundColor = Color.parseColor("#0066FF"),
        contentColor = Color.WHITE
    ),
    shape = RoundedCornerShape(8.dp)
) {
    Text("Click")
}
```

---

## 4. Navegación Universal

### 4.1 app_bar (reemplaza navbar)

`app_bar` funciona en todas las plataformas:

```kyx
<app_bar title="Mi App">
    <app_bar_actions>
        <button icon="search" click=@search />
        <button icon="menu" click=@toggle_menu />
    </app_bar_actions>
</app_bar>
```

**Web:** Barra superior fija
**Desktop:** Barra superior o lateral
**Mobile:** Barra superior con back button automático
**iOS:** NavigationBar nativo
**Android:** AppBar nativo

### 4.2 side_bar

```kyx
<side_bar width=250 collapsible=true>
    <side_bar_item icon="home" label="Inicio" href="/" />
    <side_bar_item icon="users" label="Usuarios" href="/users" />
</side_bar>
```

**Web/Desktop:** Sidebar visible
**Mobile:** Se convierte en drawer (slide desde izquierda)
**iOS:** Se convierte en NavigationView
**Android:** Se convierte en NavigationDrawer

### 4.3 bottom_nav (mobile-first)

```kyx
<bottom_nav>
    <bottom_nav_item icon="home" label="Inicio" href="/" />
    <bottom_nav_item icon="search" label="Buscar" href="/search" />
    <bottom_nav_item icon="profile" label="Perfil" href="/profile" />
</bottom_nav>
```

**Mobile:** Barra inferior nativa
**Web/Desktop:** Se ignora o se convierte en tabs superiores

---

## 5. Responsive Universal

### 5.1 Breakpoints tipados

```kyle
enum breakpoint:
    mobile      # < 640px
    tablet      # 640px - 1024px
    desktop     # 1024px - 1440px
    wide        # > 1440px
```

### 5.2 Condicionales por breakpoint

```kyx
<view>
    @if current_breakpoint() == breakpoint.mobile:
        <bottom_nav />
    @else:
        <side_bar />
</view>
```

### 5.3 Layouts adaptativos

```kyx
<adaptive_layout>
    <layout_for breakpoint=breakpoint.mobile>
        <vstack>...</vstack>
    </layout_for>
    <layout_for breakpoint=breakpoint.desktop>
        <hstack>...</hstack>
    </layout_for>
</adaptive_layout>
```

---

## 6. Rendimiento Multiplataforma

### 6.1 Virtualización

Listas grandes se virtualizan automáticamente:

```kyx
<list data=@big_list virtual=true item_height=48>
    <item>
        <text value=@item.name />
    </item>
</list>
```

**Web:** DOM virtual (solo renderiza items visibles)
**Desktop:** Renderizado directo (solo items en viewport)
**Mobile:** RecyclerView/LazyVStack nativo

### 6.2 Lazy loading

```kyx
<img src=@photo_url lazy=true placeholder=skeleton />
```

**Web:** IntersectionObserver
**Desktop:** Carga asíncrona con placeholder
**Mobile:** Glide/SDWebImage nativo

---

## 7. Accesibilidad Universal

### 7.1 ARIA automático

```kyx
<button text="Eliminar" click=@delete />
```

Genera automáticamente:
- **Web:** `role="button"`, `aria-label="Eliminar"`
- **Desktop:** Focusable, activable con Enter/Space
- **Mobile:** Accessibility element con label

### 7.2 Navegación por teclado

```kyx
<view focusable=true on_key=@handle_key>
    <button text="Aceptar" />
    <button text="Cancelar" />
</view>
```

**Web:** Tab order, Enter/Space activation
**Desktop:** Focus traversal nativo
**Mobile:** No aplica (touch-only)

---

## 8. Referencias

- [Componentes nativos](components/README.md)
- [Sistema de estilos](style-system.md)
- [Estado y eventos](state-events.md)
- [Routing](routing.md)
