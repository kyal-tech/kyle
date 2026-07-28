# Accesibilidad (a11y)

**Status:** Draft v1.0
**Date:** 2026-07-10

---

## 1. Principios

Kyle UI debe ser accesible por defecto. Todo componente built-in debe cumplir
con WCAG 2.1 AA como mínimo.

| Principio | Significado |
|-----------|-------------|
| Perceptible | La información debe presentarse de forma que todos puedan percibirla |
| Operable | Los componentes deben ser utilizables con teclado, voz, etc. |
| Comprensible | La interfaz debe ser predecible y clara |
| Robusto | Debe funcionar con tecnologías de asistencia |

---

## 2. ARIA Attributes

### 2.1 Atributos built-in

Cada componente genera ARIA automáticamente:

```kyx
<button text="Cerrar" click=@close />
```

Web target → HTML generado automáticamente:
```html
<button role="button" aria-label="Cerrar">Cerrar</button>
```

### 2.2 Atributos explícitos

```kyx
<text_field
    label="Email"
    aria_label="Correo electrónico"
    aria_describedby="email-hint"
    aria_invalid=if has_error: "true" else: "false"
    aria_required="true"
/>
<text id="email-hint" value="Ingrese su correo electrónico" />
```

### 2.3 Roles ARIA

```kyx
<view role="navigation" aria_label="Menú principal">
    <link to="/" text="Inicio" />
    <link to="/about" text="Acerca de" />
</view>
```

---

## 3. Navegación por Teclado

### 3.1 Orden de tabulación

```kyx
<text_field tab_index=1 />
<button tab_index=2 text="Enviar" />
```

### 3.2 Focus management

```kyx
@(
    fn on_mounted():
        focus("email-input")  # auto-focus al montar
)

<text_field id="email-input" bind=@email />
```

### 3.3 Atajos de teclado

```kyx
<view keydown=@handle_keyboard>
    <!-- Ctrl+S para guardar -->
</view>

@(
    fn handle_keyboard(event: KeyboardEvent):
        if event.ctrl_key && event.key == "s":
            event.prevent_default()
            save()
)
```

---

## 4. Screen Readers

### 4.1 Texto alternativo

```kyx
<image src="logo.png" alt="Logo de la empresa" />
<icon name="search" aria_label="Buscar" />
```

### 4.2 Live regions

```kyx
<view aria_live="polite" aria_atomic="true">
    <text value="Se encontraron " + results.len().to_str() + " resultados" />
</view>
```

### 4.3 Mensajes de estado

```kyx
<view role="status" aria_live="polite">
    @if loading:
        <text value="Cargando..." />
    @else:
        <text value="Listo" />
</view>
```

---

## 5. Contraste y Tamaño

### 5.1 Contraste mínimo

El theme system debe garantizar contraste suficiente:

```kyle
# Validación en tiempo de compilación
theme AccessibleTheme: LightTheme:
    # Error de compilación si el contraste es < 4.5:1
    on_surface = color("#666666")  # ❌ Muy claro sobre #FFFFFF
```

### 5.2 Texto escalable

Todos los tamaños de fuente deben ser relativos al tamaño base del sistema:

```kyle
style<text> Body:
    font_size = 14       # relativo al theme.font_size_base
    line_height = 1.5    # relativo al font_size
```

### 5.3 Reduced motion

```kyle
style<button> MotionAware:
    transition = transition("background", 200, easing.ease_in_out, 0)
    @media(reduced_motion: true):
        transition = transition.none()  # sin animación si el usuario lo prefiere
```

---

## 6. Compilación por Target

### 6.1 Web

Los atributos ARIA se convierten directamente en atributos HTML:

```javascript
// Generado automáticamente
const btn = document.createElement('button');
btn.setAttribute('role', 'button');
btn.setAttribute('aria-label', 'Cerrar');
btn.setAttribute('tabindex', '0');
```

### 6.2 Desktop (Skia)

En desktop nativo, la accesibilidad se delega a la plataforma:

- macOS: `NSAccessibility` protocol
- Windows: `IAccessible` (UI Automation)
- Linux: `AT-SPI`

---

## 7. Referencias

- [WCAG 2.1](https://www.w3.org/TR/WCAG21/)
- [ARIA Authoring Practices](https://www.w3.org/TR/wai-aria-practices/)
- [ui-syntax.md](../syntax/ui-syntax.md)
