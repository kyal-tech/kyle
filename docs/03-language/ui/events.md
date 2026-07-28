# Eventos en Kyle UI

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Filosofía

Todos los eventos en Kyle UI son:
- **Tipados** — Cada evento tiene un tipo específico
- **Multiplataforma** — Misma sintaxis en web, desktop, iOS, Android
- **snake_case** — Consistente en todo el framework
- **Declarativos** — Se definen como props de componentes

---

## Lista Completa de Eventos

### Eventos de Mouse

| Evento | Tipo | Descripción | Plataformas |
|--------|------|-------------|-------------|
| `click` | `click_event` | Click simple | Web, Desktop, Mobile (tap) |
| `dblclick` | `click_event` | Doble click | Web, Desktop |
| `mouse_enter` | `mouse_event` | Cursor entra al elemento | Web, Desktop |
| `mouse_leave` | `mouse_event` | Cursor sale del elemento | Web, Desktop |
| `mouse_down` | `mouse_event` | Botón presionado | Web, Desktop |
| `mouse_up` | `mouse_event` | Botón liberado | Web, Desktop |
| `mouse_move` | `mouse_event` | Cursor se mueve | Web, Desktop |
| `context_menu` | `mouse_event` | Click derecho | Web, Desktop |

### Eventos de Touch (Mobile)

| Evento | Tipo | Descripción | Plataformas |
|--------|------|-------------|-------------|
| `touch_start` | `touch_event` | Dedo toca la pantalla | Mobile |
| `touch_end` | `touch_event` | Dedo se levanta | Mobile |
| `touch_move` | `touch_event` | Dedo se mueve | Mobile |
| `long_press` | `touch_event` | Presión prolongada (>500ms) | Mobile |

### Eventos de Teclado

| Evento | Tipo | Descripción | Plataformas |
|--------|------|-------------|-------------|
| `keydown` | `key_event` | Tecla presionada | Web, Desktop |
| `keyup` | `key_event` | Tecla liberada | Web, Desktop |
| `keypress` | `key_event` | Tecla presionada (deprecated, usar keydown) | Web, Desktop |

### Eventos de Formulario

| Evento | Tipo | Descripción | Plataformas |
|--------|------|-------------|-------------|
| `submit` | `submit_event` | Formulario enviado | Web, Desktop, Mobile |
| `change` | `change_event` | Valor cambió (al perder foco) | Web, Desktop, Mobile |
| `input` | `input_event` | Valor cambió (en tiempo real) | Web, Desktop, Mobile |
| `focus` | `focus_event` | Elemento enfocado | Web, Desktop, Mobile |
| `blur` | `focus_event` | Elemento perdió foco | Web, Desktop, Mobile |

### Eventos de Scroll

| Evento | Tipo | Descripción | Plataformas |
|--------|------|-------------|-------------|
| `scroll` | `scroll_event` | Elemento scrolleado | Web, Desktop, Mobile |
| `scroll_end` | `scroll_event` | Scroll terminó | Mobile |

### Eventos de Ciclo de Vida

| Evento | Tipo | Descripción | Plataformas |
|--------|------|-------------|-------------|
| `on_mounted` | `fn()` | Componente montado | Todas |
| `on_unmounted` | `fn()` | Componente desmontado | Todas |
| `on_updated` | `fn(changed: {str})` | Componente actualizado | Todas |

---

## Tipos de Eventos

### click_event

```kyle
final class click_event:
    x: f32                      # Coordenada X
    y: f32                      # Coordenada Y
    target: str                 # ID del elemento
    button: i32                 # 0=izq, 1=medio, 2=der
    alt_key: bool               # Alt presionado
    ctrl_key: bool              # Ctrl presionado
    shift_key: bool             # Shift presionado
    meta_key: bool              # Meta (Cmd/Win) presionado
```

### mouse_event

```kyle
final class mouse_event:
    x: f32                      # Coordenada X
    y: f32                      # Coordenada Y
    client_x: f32               # X relativo al viewport
    client_y: f32               # Y relativo al viewport
    screen_x: f32               # X relativo a la pantalla
    screen_y: f32               # Y relativo a la pantalla
    button: i32                 # Botón del mouse
    buttons: i32                # Botones presionados (bitmask)
    alt_key: bool
    ctrl_key: bool
    shift_key: bool
    meta_key: bool
```

### touch_event

```kyle
final class touch_event:
    touches: {touch_point}      # Puntos de contacto actuales
    changed_touches: {touch_point}  # Puntos que cambiaron
    target: str
    
final class touch_point:
    identifier: i32             # ID único del dedo
    x: f32                      # Coordenada X
    y: f32                      # Coordenada Y
    force: f32                  # Presión (0.0-1.0)
    radius_x: f32               # Radio del contacto X
    radius_y: f32               # Radio del contacto Y
```

### key_event

```kyle
final class key_event:
    key: str                    # Tecla (ej: "a", "Enter", "ArrowUp")
    code: str                   # Código físico (ej: "KeyA", "Enter")
    alt_key: bool
    ctrl_key: bool
    shift_key: bool
    meta_key: bool
    repeat: bool                # True si es auto-repeat
```

### input_event

```kyle
final class input_event:
    value: str                  # Valor actual del input
    target: str                 # ID del elemento
    input_type: str             # "insert", "delete", "none"
    data: str?                  # Datos insertados
```

### change_event

```kyle
final class change_event:
    value: str                  # Nuevo valor
    target: str                 # ID del elemento
    previous_value: str         # Valor anterior
```

### focus_event

```kyle
final class focus_event:
    target: str                 # ID del elemento
    related_target: str?        # Elemento relacionado (blur/focus)
```

### submit_event

```kyle
final class submit_event:
    target: str                 # ID del form
    prevent_default: fn()       # Prevenir envío default
```

### scroll_event

```kyle
final class scroll_event:
    scroll_x: f32               # Scroll horizontal
    scroll_y: f32               # Scroll vertical
    target: str                 # ID del elemento
```

---

## Uso Básico

### Click simple

```kyx
<view>
    @(
        fn handle_click(event: click_event):
            print(@"Click en: " + event.x.to_str() + ", " + event.y.to_str())
    )
    
    <button text="Click me" click=@handle_click />
</view>
```

### Hover (mouse enter/leave)

```kyx
<view>
    @(
        is_hovered: ^bool = false
        
        fn handle_mouse_enter(event: mouse_event):
            is_hovered = true
        
        fn handle_mouse_leave(event: mouse_event):
            is_hovered = false
    )
    
    <button 
        text="Hover me" 
        mouse_enter=@handle_mouse_enter
        mouse_leave=@handle_mouse_leave
        style=@if is_hovered: PrimaryHover else: Primary
    />
</view>
```

### Double click

```kyx
<view>
    @(
        fn handle_dblclick(event: click_event):
            print("Doble click detectado")
    )
    
    <view dblclick=@handle_dblclick>
        <text value="Haz doble click aquí" />
    </view>
</view>
```

### Long press (mobile)

```kyx
<view>
    @(
        fn handle_long_press(event: touch_event):
            print("Presión larga detectada")
            show_context_menu()
    )
    
    <view long_press=@handle_long_press>
        <text value="Mantén presionado" />
    </view>
</view>
```

### Keyboard events

```kyx
<view>
    @(
        fn handle_keydown(event: key_event):
            if event.key == "Enter":
                submit_form()
            elif event.key == "Escape":
                close_modal()
            
            if event.ctrl_key and event.key == "s":
                event.prevent_default()
                save()
    )
    
    <text_field 
        bind=@search_query 
        keydown=@handle_keydown
        placeholder="Buscar... (Ctrl+S para guardar)"
    />
</view>
```

### Input en tiempo real

```kyx
<view>
    @(
        search: ^str = ""
        
        fn handle_input(event: input_event):
            search = event.value
            filter_results(search)
    )
    
    <text_field 
        bind=@search 
        input=@handle_input
        placeholder="Buscar..."
    />
</view>
```

### Focus/Blur

```kyx
<view>
    @(
        email_focused: ^bool = false
        
        fn handle_focus(event: focus_event):
            email_focused = true
        
        fn handle_blur(event: focus_event):
            email_focused = false
            validate_email()
    )
    
    <text_field 
        bind=@email 
        focus=@handle_focus
        blur=@handle_blur
        placeholder="Email"
        style=@if email_focused: FocusedStyle else: NormalStyle
    />
</view>
```

### Scroll

```kyx
<view>
    @(
        scroll_y: ^f32 = 0.0
        
        fn handle_scroll(event: scroll_event):
            scroll_y = event.scroll_y
            
            # Load more when reaching bottom
            if scroll_y > content_height - viewport_height - 100:
                load_more_items()
    )
    
    <scroll scroll=@handle_scroll height=length.px(400)>
        @for(item in items):
            <item_view item=@item />
    </scroll>
</view>
```

---

## Eventos por Componente

### button

```kyx
<button 
    text="Click"
    click=@handle_click
    mouse_enter=@handle_hover
    mouse_leave=@handle_hover_end
    focus=@handle_focus
    blur=@handle_blur
    keydown=@handle_key
/>
```

### text_field

```kyx
<text_field 
    bind=@value
    input=@handle_input
    change=@handle_change
    focus=@handle_focus
    blur=@handle_blur
    keydown=@handle_key
    keyup=@handle_key_up
/>
```

### checkbox

```kyx
<checkbox 
    bind=@checked
    change=@handle_change
    click=@handle_click
/>
```

### select

```kyx
<select 
    bind=@selected
    change=@handle_change
    focus=@handle_focus
    blur=@handle_blur
/>
```

### view (contenedor)

```kyx
<view 
    click=@handle_click
    dblclick=@handle_dblclick
    mouse_enter=@handle_hover
    mouse_leave=@handle_hover_end
    mouse_down=@handle_mouse_down
    mouse_up=@handle_mouse_up
    mouse_move=@handle_mouse_move
    touch_start=@handle_touch_start
    touch_end=@handle_touch_end
    touch_move=@handle_touch_move
    long_press=@handle_long_press
    keydown=@handle_key
    scroll=@handle_scroll
    focus=@handle_focus
    blur=@handle_blur
/>
```

---

## Traducción Multiplataforma

### Web

```javascript
// click
element.addEventListener('click', (e) => {
    const event = {
        x: e.clientX,
        y: e.clientY,
        target: e.target.id,
        button: e.button,
        alt_key: e.altKey,
        ctrl_key: e.ctrlKey,
        shift_key: e.shiftKey,
        meta_key: e.metaKey
    };
    handle_click(event);
});

// mouse_enter
element.addEventListener('mouseenter', (e) => {
    handle_mouse_enter(convert_mouse_event(e));
});

// touch_start
element.addEventListener('touchstart', (e) => {
    handle_touch_start(convert_touch_event(e));
});

// long_press (custom)
let press_timer;
element.addEventListener('touchstart', (e) => {
    press_timer = setTimeout(() => {
        handle_long_press(convert_touch_event(e));
    }, 500);
});
element.addEventListener('touchend', () => {
    clearTimeout(press_timer);
});
```

### Desktop (SDL2)

```kyle
fn handle_sdl_event(event: SDL_Event):
    match event.type:
        SDL_MOUSEBUTTONDOWN:
            handle_click(click_event(
                x: event.button.x,
                y: event.button.y,
                button: event.button.button
            ))
        
        SDL_MOUSEMOTION:
            handle_mouse_move(mouse_event(
                x: event.motion.x,
                y: event.motion.y
            ))
        
        SDL_KEYDOWN:
            handle_keydown(key_event(
                key: sdl_key_to_string(event.key.keysym.sym),
                alt_key: event.key.keysym.mod & KMOD_ALT != 0,
                ctrl_key: event.key.keysym.mod & KMOD_CTRL != 0
            ))
```

### iOS (SwiftUI)

```swift
// Tap (click)
Button(action: { handle_click(...) }) {
    Text("Click")
}

// Long press
Text("Mantén presionado")
    .onLongPressGesture(minimumDuration: 0.5) {
        handle_long_press(...)
    }

// Drag (touch_move)
Text("Arrastra")
    .gesture(
        DragGesture()
            .onChanged { value in
                handle_touch_move(...)
            }
    )
```

### Android (Jetpack Compose)

```kotlin
// Click
Button(onClick = { handle_click(...) }) {
    Text("Click")
}

// Long click
Text(
    "Mantén presionado",
    modifier = Modifier.combinedClickable(
        onClick = { },
        onLongClick = { handle_long_press(...) }
    )
)

// Touch events
Box(
    modifier = Modifier.pointerInput(Unit) {
        detectTapGestures(
            onTap = { handle_click(...) },
            onLongPress = { handle_long_press(...) }
        )
    }
)
```

---

## Errores Comunes

### ❌ Malo: No prevenir default en submit

```kyx
<form submit=@() => {
    # Form se recarga automáticamente
    save_data()
}>
```

**Problema:** Página se recarga antes de ejecutar lógica.

### ✅ Bueno: Prevenir default

```kyx
<form submit=@(event: submit_event) => {
    event.prevent_default()
    save_data()
}>
```

### ❌ Malo: Usar click en mobile para gestos complejos

```kyx
<view click=@handle_interaction>
```

**Problema:** No distingue entre tap, swipe, long press.

### ✅ Bueno: Usar eventos touch específicos

```kyx
<view 
    touch_start=@handle_touch_start
    touch_end=@handle_touch_end
    touch_move=@handle_touch_move
    long_press=@handle_long_press
>
```

### ❌ Malo: No manejar eventos de teclado para accesibilidad

```kyx
<button text="Submit" click=@submit />
```

**Problema:** No se puede activar con Enter/Space desde teclado.

### ✅ Bueno: Soportar teclado

```kyx
<button 
    text="Submit" 
    click=@submit
    keydown=@(event: key_event) => {
        if event.key == "Enter" or event.key == " ":
            submit()
    }
/>
```

---

## Referencias

- [Componentes nativos](components/README.md)
- [Arquitectura multiplataforma](architecture.md)
- [Accesibilidad](accessibility.md)
