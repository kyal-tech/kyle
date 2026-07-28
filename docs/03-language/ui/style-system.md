# Sistema de Estilos Tipado

**Status:** Draft v2.0
**Date:** 2026-07-12
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target

---

## 1. Filosofía

Kyle NO usa CSS. Los estilos son **tipos Kyle** con propiedades fuertemente tipadas,
verificadas en tiempo de compilación. No hay strings mágicos, no hay selectores CSS,
no hay hojas de estilo externas.

Todo es **snake_case**: funciones, tipos, constructores, propiedades.

| Aspecto | CSS | Kyle Style |
|---------|:---:|:-----------:|
| Sintaxis | `.clase { prop: val }` | `style<comp> Name: prop = val` |
| Tipado | ❌ Strings | ✅ Tipos Kyle verificados |
| Scope | Global (cascade) | Local al componente |
| Compilación | Texto | AST → target nativo |
| Variables | `--var` (custom props) | `theme.Name` (tipado) |

---

## 2. Tipos de Estilo

### 2.1 color

```kyle
final class color:
    r: u8
    g: u8
    b: u8
    a: f32 = 1.0

    # Constructores
    static fn from_hex(hex: str) color
    static fn from_rgba(r: u8, g: u8, b: u8, a: f32) color
    static fn from_hsl(h: f32, s: f32, l: f32) color

    # Colores predefinidos
    static fn transparent() color
    static fn black() color
    static fn white() color
    static fn primary() color       # del theme activo
    static fn accent() color

    # Modificación
    fn darken(this, amount: f32) color     # 0.0 = no change, 1.0 = black
    fn lighten(this, amount: f32) color    # 0.0 = no change, 1.0 = white
    fn with_alpha(this, a: f32) color      # nueva alpha
```

Uso:

```kyx
color("#0066FF")
color.from_rgba(0, 102, 255, 1.0)
color.black()
color.transparent()
```

### 2.2 spacing

```kyle
final class spacing:
    top: f32
    right: f32
    bottom: f32
    left: f32

    static fn all(value: f32) spacing
    static fn symmetric(horizontal: f32, vertical: f32) spacing
    static fn only(top: f32, right: f32, bottom: f32, left: f32) spacing
```

### 2.3 length

```kyle
enum length:
    px(f32)          # píxeles
    percent(f32)     # porcentaje del contenedor (0.0-100.0)
    auto             # automático
    fill             # llena el espacio disponible
    vw(f32)          # viewport width %
    vh(f32)          # viewport height %
```

### 2.4 border

```kyle
final class border:
    width: f32
    color: color
    style: border_style  # solid, dashed, dotted, none

final class shadow:
    x: f32
    y: f32
    blur: f32
    spread: f32
    color: color
```

### 2.5 Tipografía

```kyle
enum font_weight:
    thin       # 100
    light      # 300
    normal     # 400
    medium     # 500
    semi_bold  # 600
    bold       # 700
    black      # 900

enum text_align:
    left
    center
    right
    justify

final class text_style:
    font_family: str
    font_size: f32
    font_weight: font_weight
    line_height: f32
    letter_spacing: f32
    color: color
    align: text_align
    decoration: text_decoration  # none, underline, line_through
```

### 2.6 Layout

```kyle
enum display:
    flex
    grid
    none

enum flex_direction:
    row
    column
    row_reverse
    column_reverse

enum alignment:
    start
    center
    end
    stretch
    baseline

enum overflow:
    visible
    hidden
    scroll
    auto

enum position:
    static
    relative
    absolute(pos: absolute_position)
    fixed(pos: absolute_position)
    sticky

final class absolute_position:
    top: length
    left: length
    right: length
    bottom: length
```

### 2.7 Transform

```kyle
final class transform:
    translate_x: f32
    translate_y: f32
    scale_x: f32 = 1.0
    scale_y: f32 = 1.0
    rotate: f32          # grados
    skew_x: f32
    skew_y: f32

final class transition:
    property: str        # "opacity", "transform", "background", "all"
    duration: i32        # milisegundos
    easing: easing       # ease, linear, ease_in, ease_out, ease_in_out, cubic_bezier(f1,f2,f3,f4)
    delay: i32           # milisegundos
```

### 2.8 Style class completo

```kyle
final class style:
    background: color?
    background_image: str?
    color: color?
    font: text_style?
    padding: spacing?
    margin: spacing?
    border: border?
    border_top: border?
    border_right: border?
    border_bottom: border?
    border_left: border?
    border_radius: f32?
    shadow: shadow?
    shadow_inner: shadow?
    width: length?
    height: length?
    min_width: length?
    max_width: length?
    min_height: length?
    max_height: length?
    display: display?
    flex_direction: flex_direction?
    flex_wrap: bool?
    flex_grow: f32?
    flex_shrink: f32?
    align_items: alignment?
    align_self: alignment?
    justify_content: alignment?
    gap: f32?
    grid_columns: i32?
    grid_rows: i32?
    grid_column_span: i32?
    grid_row_span: i32?
    position: position?
    opacity: f32?
    cursor: cursor?        # pointer, default, text, wait, crosshair, not_allowed
    pointer_events: bool?
    overflow: overflow?
    transform: transform?
    transform_origin_x: f32?
    transform_origin_y: f32?
    transition: transition?
    animation: animation?
    clip: bool?
    clip_path: str?
```

---

## 3. Declaración de Estilos

### 3.1 Styles

```kyle
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    font_size = 14
    font_weight = font_weight.bold
    padding = spacing.all(12)
    border_radius = 8
    cursor = cursor.pointer

style<button> PrimaryHover: Primary:
    background = color("#0052CC")

style<button> Secondary:
    background = color("transparent")
    color = color("#0066FF")
    border = border(2, color("#0066FF"), border_style.solid)
    padding = spacing.all(12)
    border_radius = 8
```

### 3.2 Layouts

Los layouts se definen inline con props de flex en cada componente:

```kyx
<vstack alignment=alignment.center spacing=16>
    <text value="Centrado" />
</vstack>

<hstack alignment=alignment.center spacing=8>
    <button text="Aceptar" />
    <button text="Cancelar" />
</hstack>
```

No hay `layout<>` nombrados — se usa directamente `alignment=`, `spacing=`, `gap=`.

### 3.4 Themes

```kyle
theme LightTheme:
    primary = color("#0066FF")
    on_primary = color("#FFFFFF")
    background = color("#FFFFFF")
    on_background = color("#1A1A1A")
    surface = color("#F5F5F5")
    on_surface = color("#1A1A1A")
    error = color("#DC3545")
    outline = color("#CCCCCC")

    font_display = text_style(
        font_family: "Inter",
        font_size: 32,
        font_weight: font_weight.bold,
    )
    font_headline = text_style(
        font_family: "Inter",
        font_size: 24,
        font_weight: font_weight.semi_bold,
    )
    font_body = text_style(
        font_family: "Inter",
        font_size: 14,
        font_weight: font_weight.normal,
    )

    spacing_xs = 4
    spacing_sm = 8
    spacing_md = 16
    spacing_lg = 24
    spacing_xl = 32

    radius_sm = 4
    radius_md = 8
    radius_lg = 16
    radius_full = 9999

theme DarkTheme: LightTheme:
    primary = color("#4D8EFF")
    background = color("#121212")
    on_background = color("#E0E0E0")
    surface = color("#1E1E1E")
```

---

## 4. Uso de Estilos en Componentes

### 4.1 Aplicar estilo

```kyx
<button style=Primary text="Ingresar" />
<button style=Secondary text="Cancelar" />
<card style=Elevated>
    <text value="Contenido" />
</card>
```

### 4.2 Estilos inline (casos específicos)

```kyx
<text
    style=style(color: color("#FF0000"), font_size: 16)
    value="Error"
/>
```

### 4.3 Combinar estilos

```kyx
<button
    style=style.combine(Primary, style(margin: spacing.all(8)))
    text="Combinado"
/>
```

---

## 5. Compilación por Target

### 5.0 CSS Reset

Por defecto, el HTML generado incluye un CSS reset que elimina estilos default del navegador:

```css
* { margin: 0; padding: 0; box-sizing: border-box; }
button, input, select, textarea { font: inherit; border: none; background: none; outline: none; }
button { cursor: pointer; }
a { color: inherit; text-decoration: none; }
```

Esto asegura que los componentes Kyle se vean igual en todos los navegadores, sin margenes, paddings, ni bordes predeterminados.

### 5.1 Web target

```kyle
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    border_radius = 8
```

→ JavaScript generado:
```javascript
const styles = {
  Primary: {
    background: '#0066FF',
    color: '#FFFFFF',
    borderRadius: '8px',
    padding: '12px 24px',
    cursor: 'pointer',
  }
};
```

### 5.2 Desktop target (Skia)

```kyle
style<button> Primary:
    background = color("#0066FF")
    border_radius = 8
    padding = spacing.all(12)
```

→ Código Kyle generado:
```kyle
fn render_primary_button(x: f32, y: f32, w: f32, h: f32):
    skia_draw_rounded_rect(x, y, w, h, 8.0, color("#0066FF"))
```

### 5.3 Mobile target

**Android:**
```java
Button btn = new Button(context);
btn.setBackgroundColor(Color.parse("#0066FF"));
btn.setTextColor(Color.WHITE);
btn.setPadding(12, 24, 12, 24);
```

**iOS:**
```swift
let btn = UIButton()
btn.backgroundColor = UIColor(hex: "#0066FF")
btn.setTitleColor(.white, for: .normal)
btn.contentEdgeInsets = UIEdgeInsets(12, 24, 12, 24)
```

---

## 6. Responsive

### 6.1 Media queries en Kyle

```kyle
style<view> Card:
    width = length.percent(100)
    padding = spacing.all(8)

    @media(min_width: 640):
        width = length.px(400)
        padding = spacing.all(16)

    @media(min_width: 1024):
        max_width = length.px(800)
        margin = spacing(left: length.auto, right: length.auto)
```

### 6.2 Compilación responsive

Web target → `@media` queries:
```javascript
const styles = {
  Card: {
    base: { width: '100%', padding: '8px' },
    '@media (min-width: 640px)': { width: '400px', padding: '16px' },
    '@media (min-width: 1024px)': { maxWidth: '800px', margin: '0 auto' },
  }
};
```

Desktop target:
```kyle
fn render_card(viewport_w: f32):
    style = if viewport_w >= 1024:
        card_desktop_style
    elif viewport_w >= 640:
        card_tablet_style
    else:
        card_mobile_style
```

---

## 7. Modo oscuro

```kyx
<view>
    @(theme = current_theme())
    <text value=if theme == DarkTheme: "Modo oscuro" else: "Modo claro" />
    <button text="Toggle theme" click=@toggle_theme />
</view>
```

---

## 8. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target
- [state-events.md](state-events.md) — Estado y eventos
- [animation.md](animation.md) — Animaciones
