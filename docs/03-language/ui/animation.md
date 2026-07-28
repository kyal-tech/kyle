# Animaciones y Transiciones

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [style-system.md](style-system.md) — Sistema de estilos
- [state-events.md](state-events.md) — Estado y eventos

---

## 1. Tipos de Animación

### 1.1 animation_state

Estado animable en un fotograma. Es un tipo Kyle con todas las propiedades
que pueden animarse:

```kyle
final class animation_state:
    opacity: f32?          # 0.0 a 1.0
    translate_x: f32?      # desplazamiento en píxeles
    translate_y: f32?
    scale_x: f32?          # 1.0 = tamaño original
    scale_y: f32?
    rotate: f32?           # grados
    background: color?
    color: color?
    border_radius: f32?
    shadow: shadow?
    width: length?
    height: length?
```

### 1.2 animation_frame

Un fotograma en la línea de tiempo:

```kyle
final class animation_frame:
    progress: f32           # 0.0 a 1.0 (porcentaje de la animación)
    state: animation_state   # estado en ese punto

    static fn at(progress: f32, state: animation_state) animation_frame
```

### 1.3 Declaración de animaciones (tipada)

Las animaciones se declaran con sintaxis `animation<componente> Nombre:`,
igual que los estilos con `style<componente>`, asegurando type-safety:

```kyle
# Animación completa con fotogramas
animation<view> FadeIn:
    duration = 300              # milisegundos
    easing = easing.ease_out
    fill_mode = fill_mode.forwards  # mantener estado final
    frames = {
        animation_frame.at(0.0, animation_state(opacity: 0.0)),
        animation_frame.at(1.0, animation_state(opacity: 1.0)),
    }

# Animación con from/to (azúcar sintáctico para 2 fotogramas)
animation<view> SlideIn:
    from = animation_state(
        translate_x: -100,
        opacity: 0.0,
    )
    to = animation_state(
        translate_x: 0,
        opacity: 1.0,
    )
    duration = 400
    easing = easing.ease_out_cubic

# Keyframes múltiples
animation<view> Pulse:
    frames = {
        animation_frame.at(0.0,  animation_state(scale_x: 1.0, scale_y: 1.0)),
        animation_frame.at(0.5,  animation_state(scale_x: 1.05, scale_y: 1.05)),
        animation_frame.at(1.0,  animation_state(scale_x: 1.0, scale_y: 1.0)),
    }
    duration = 1000
    easing = easing.ease_in_out
    iterations = animation_iteration.infinite
```

### 1.4 Transiciones (en estilos)

Las transiciones se declaran DENTRO de los estilos, no como bloque separado.
Son parte del `Transition` type:

```kyle
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
    border_radius = 8
    transition = transition(
        property: "background",
        duration: 200,
        easing: easing.ease_in_out,
        delay: 0,
    )

# El hover cambia solo el background, la transición se aplica automáticamente
style<button> PrimaryHover: Primary:
    background = color("#0052CC")
```

### 1.5 Micro-interacciones

```kyle
animation<button> Ripple:
    type = animation_type.ripple
    duration = 600
    color = color("#FFFFFF")
    max_opacity = 0.3

animation<button> ScalePress:
    type = animation_type.scale
    frames = {
        animation_frame.at(0.0, animation_state(scale_x: 1.0, scale_y: 1.0)),
        animation_frame.at(1.0, animation_state(scale_x: 0.95, scale_y: 0.95)),
    }
    duration = 100
    easing = easing.ease_out
```

---

## 2. Tipos de animación

### 2.1 Easing

```kyle
enum Easing:
    Linear
    Ease
    EaseIn
    EaseOut
    EaseInOut
    EaseInSine
    EaseOutSine
    EaseInOutSine
    EaseInCubic
    EaseOutCubic
    EaseInOutCubic
    EaseInBack
    EaseOutBack
    EaseInOutBack
    Elastic(amplitude: f32, period: f32)
    Bounce
    CubicBezier(x1: f32, y1: f32, x2: f32, y2: f32)
```

### 2.2 FillMode

```kyle
enum FillMode:
    None       # no aplicar estilo después de la animación
    Forwards   # mantener el estado final
    Backwards  # aplicar estado inicial antes de empezar
    Both       # forwards + backwards
```

### 2.3 AnimationIteration

```kyle
enum AnimationIteration:
    Count(i32)     # número específico de repeticiones
    Infinite       # loop infinito
```

### 2.4 AnimationType

```kyle
enum AnimationType:
    Standard       # keyframes normales
    Ripple         # efecto de onda (click)
    Scale          # escalado (press)
    Slide          # deslizamiento
    Fade           # opacidad
    Spring         # efecto muelle (físico)
```

---

## 3. Uso en Componentes

```kyx
# app.kyx — ejemplo completo
@(
    visible: ^bool = false
    fn toggle():
        visible = !visible
)
<view>
    <button
        style=Primary
        text="Mostrar / Ocultar"
        click=@toggle
    />

    <modal
        visible=@visible
        animation=FadeIn
    >
        <vstack alignment=alignment.center padding=16>
            <text value="Contenido del diálogo" />
            <button
                text="Cerrar"
                click=@toggle
            />
        </vstack>
    </modal>
</view>
```

---

## 4. Compilación por Target

### 4.1 Web

Las animaciones Kyle se compilan a CSS animations/transitions + Web Animations API:

```javascript
// ui-runtime.js — generado automáticamente
const animations = {
    FadeIn: {
        keyframes: [
            { opacity: '0' },
            { opacity: '1' },
        ],
        options: {
            duration: 300,
            easing: 'ease-out',
            fill: 'forwards',
        }
    },
    SlideIn: {
        keyframes: [
            { transform: 'translateX(-100px)', opacity: '0' },
            { transform: 'translateX(0)', opacity: '1' },
        ],
        options: {
            duration: 400,
            easing: 'cubic-bezier(0.22, 1, 0.36, 1)',
        }
    },
};

function applyAnimation(element, name) {
    const anim = animations[name];
    if (!anim) return;
    element.animate(anim.keyframes, anim.options);
}
```

### 4.2 Desktop (Skia)

```kyle
# Generado automáticamente por el compilador de animaciones
fn animate_fade_in(el, start_time: i64):
    elapsed = now() - start_time
    progress = min(elapsed / 300.0, 1.0)
    eased = ease_out(progress)
    el.set_opacity(eased)
    if progress < 1.0:
        request_animation_frame(fn():
            animate_fade_in(el, start_time)
        )
```

---

## 5. Rendimiento

| Técnica | Descripción |
|---------|-------------|
| **GPU acelerada** | transform y opacity usan GPU (composite layers en web) |
| **Off-screen** | Animaciones fuera de pantalla no gastan recursos |
| **Throttle** | Máximo 60fps, menor si la app está en background |
| **Cancelación** | Al desmontar el componente, se cancelan sus animaciones |
| **Reduced motion** | Si el usuario prefiere `reduced-motion`, se saltan las animaciones |

---

## 6. Ejemplo completo

```kyx
# card.kyx — componente con animaciones
@(
    hover: ^bool = false
)

<style<card>> Elevated:
    background = color("#FFFFFF")
    border_radius = 8
    shadow = shadow(0, 2, 4, 0, Color.black().with_alpha(0.1))
    transition = transition("all", 200, easing.ease_out, 0)

<style<card>> ElevatedHover: Elevated:
    shadow = shadow(0, 8, 16, 0, Color.black().with_alpha(0.2))

<card
    style=if hover: ElevatedHover else: Elevated
    mouse_enter=@hover = true
    mouse_leave=@hover = false
>
    <slot />
</card>
```

---

## 7. Referencias

- [style-system.md](style-system.md) — Sistema de estilos
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Page transitions
