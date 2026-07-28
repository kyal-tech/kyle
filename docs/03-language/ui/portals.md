# Portales (Teleport)

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos

---

## 1. Qué son los Portales

Un portal permite renderizar un componente **fuera del árbol jerárquico** del
componente padre. El contenido se renderiza en otro lugar del DOM (web) o del
árbol de vistas (desktop), pero el contexto lógico (eventos, estado, props)
sigue perteneciendo al componente que lo declara.

### ¿Por qué existen?

| Problema | Sin portal | Con portal |
|----------|:----------:|:----------:|
| Modal dentro de un `overflow: hidden` | Se corta visualmente | Se renderiza al final del body |
| Tooltip en contenedor con `z-index` bajo | Aparece debajo de otros elementos | Aparece siempre al frente |
| Dropdown en scroll container | Se mueve con el scroll | Se posiciona relativo al viewport |

---

## 2. Sintaxis

### 2.1 Portal básico

```kyx
<portal target="body">
    <modal>
        <text value="Contenido del modal" />
    </modal>
</portal>
```

### 2.2 Portal con target CSS selector

```kyx
<portal target="#app-root">
    <tooltip text="Ayuda" />
</portal>

<portal target=".overlay-container">
    <dropdown items=@menu_items />
</portal>
```

### 2.3 Portal a slot del componente

```kyx
# header.kyx
<portal target="slot:header-actions">
    <button text="Cerrar Sesión" click=@logout />
</portal>
```

### 2.4 Portal con named outlet

```kyx
# app.kyx — define outlets
<outlet name="modals">
    # los portales con target="outlet:modals" se renderizan aquí
</outlet>
<outlet name="tooltips" />

# componente.kyx — usa el outlet
<portal target="outlet:modals">
    <confirm_dialog message="¿Estás seguro?" />
</portal>
```

---

## 3. Casos de Uso

### 3.1 Modal

```kyx
@(
    show_modal: ^bool = false
    fn open_modal():
        show_modal = true
    fn close_modal():
        show_modal = false
)

<button text="Abrir" click=@open_modal />

@if show_modal:
    <portal target="body">
        <overlay>
            <dialog style=Elevated>
                <text value="¿Confirmar acción?" />
                <hstack gap=8>
                    <button text="Aceptar" click=@confirm />
                    <button text="Cancelar" click=@close_modal />
                </hstack>
            </dialog>
        </overlay>
    </portal>
```

### 3.2 Tooltip

```kyx
@(
    show_tooltip: ^bool = false
)

<view
    mouse_enter=@show_tooltip = true
    mouse_leave=@show_tooltip = false
>
    <text value="Pasa el mouse" />
    @if show_tooltip:
        <portal target="body" position=@mouse_pos>
            <tooltip text="Información adicional" />
        </portal>
</view>
```

### 3.3 Dropdown con portal

```kyx
<portal target="body">
    <dropdown
        items=@options
        position=@dropdown_pos
        visible=@show_dropdown
    />
</portal>
```

Sin portal, el dropdown se cortaría si el padre tiene `overflow: hidden`.

### 3.4 Toast / Notificaciones

```kyx
<portal target="outlet:toasts">
    <toast_container position="top-right">
        @for(msg in @messages):
            <toast
                type=@msg.type
                text=@msg.text
                duration=3000
                on_close=@dismiss(msg.id)
            />
    </toast_container>
</portal>
```

---

## 4. Posicionamiento

### 4.1 Posición relativa al origen

```kyx
<portal target="body" position=@element_pos>
    <tooltip text="Tooltip contextual" />
</portal>
```

### 4.2 Estrategias de posición

```kyle
enum PortalPosition:
    Absolute(x: f32, y: f32)
    RelativeTo(element_id: str, offset_x: f32, offset_y: f32)
    Center
    Mouse
    Auto  # calcula la mejor posición para evitar overflow
```

### 4.3 Auto-posicionamiento

```kyx
<portal target="body" position=PortalPosition.Auto(
    anchor: "button-ref",
    preferred: Direction.Bottom,
    fallback: Direction.Top,
    offset: 8,
)>
    <dropdown items=@options />
</portal>
```

---

## 5. Portal con Slot Props

El portal puede exponer datos al outlet:

```kyx
# provider.kyx
<portal target="outlet:sidebar" data=@sidebar_data>
    <slot />
</portal>
```

```kyx
# app.kyx — outlet recibe data del portal
<outlet name="sidebar">
    @(sidebar_data = outlet_data())
    <sidebar items=@sidebar_data.menu_items />
</outlet>
```

---

## 6. Ciclo de Vida del Portal

```
Componente se monta
    │
    ▼
Portal detecta target ("body", "outlet:modals"…)
    │
    ▼
Crea el contenido en el target (fuera del árbol actual)
    │
    ▼
Eventos del portal siguen propagándose al componente padre
    │
    ▼
Componente se desmonta
    │
    ▼
Portal limpia el contenido del target
```

### 6.1 Desmontaje diferido

Para animaciones de salida, el portal puede retrasar el desmontaje:

```kyx
<portal target="body" unmount_delay=300>
    @if show_modal:
        <dialog animation=ScaleOut>
            <text value="Adiós" />
        </dialog>
</portal>
```

---

## 7. Compilación por Target

### 7.1 Web

```javascript
// Generado automáticamente
function createPortal(content, targetSelector) {
    const target = document.querySelector(targetSelector);
    const el = render(content);

    if (!target) {
        console.warn(`Portal target "${targetSelector}" not found`);
        return;
    }

    target.appendChild(el);

    return {
        update: (newContent) => { /* re-render en target */ },
        destroy: () => {
            if (el.parentNode) el.parentNode.removeChild(el);
        }
    };
}
```

### 7.2 Desktop (Skia)

```kyle
# Generado automáticamente
fn create_portal(content, target_id: str):
    match target_id:
        "body" =>
            # Renderiza en la capa raíz del window
            render_on_overlay_layer(content)
        "outlet:modals" =>
            # Busca el outlet por id
            outlet = find_outlet("modals")
            outlet.set_content(content)
```

---

## 8. Buenas Prácticas

| Práctica | Descripción |
|----------|-------------|
| **Usar outlets** | Preferir `target="outlet:nombre"` a selectores frágiles como `".clase"` |
| **Unmount delay** | Para animaciones de salida usa `unmount_delay` |
| **Accesibilidad** | Los portales deben mantener `aria-*` attributes y `tabIndex` |
| **Event bubbling** | Los eventos del portal burbujean al contexto lógico, no al DOM |
| **No abusar** | Portal es para escapes del layout, no para organización |

---

## 9. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [animation.md](animation.md) — Animaciones de entrada/salida
