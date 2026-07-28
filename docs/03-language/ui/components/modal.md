# modal — Ventana Modal

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class modal:
    # Estado
    open: ^bool                       # Controla si está abierto
    
    # Configuración
    title: str?                       # Título del modal
    size: modal_size = modal_size.md  # Tamaño
    closable: bool = true             # Permitir cerrar (X, ESC, click fuera)
    
    # Eventos
    on_close: fn()?                   # Callback al cerrar
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Modal simple

```kyx
<view>
    @(
        show_modal: ^bool = false
    )
    
    <button text="Abrir modal" click=@() => show_modal = true />
    
    <modal open=@show_modal on_close=@() => show_modal = false title="Confirmar">
        <vstack spacing=16>
            <text value="¿Estás seguro?" />
            <hstack spacing=8>
                <button text="Cancelar" style=Secondary click=@() => show_modal = false />
                <button text="Confirmar" style=Primary click=@confirm />
            </hstack>
        </vstack>
    </modal>
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `open` | `^bool` | — | **Requerido.** Controla visibilidad |
| `title` | `str?` | `none` | Título del modal |
| `size` | `modal_size` | `md` | Tamaño |
| `closable` | `bool` | `true` | Permitir cerrar |
| `on_close` | `fn()?` | `none` | Callback al cerrar |

---

## Tipos Relacionados

### modal_size

```kyle
enum modal_size:
    sm          # Small (400px width)
    md          # Medium (600px width)
    lg          # Large (800px width)
    xl          # Extra large (1000px width)
    full        # Pantalla completa
```

---

## Ejemplos Avanzados

### Modal de confirmación

```kyx
<view>
    @(
        show_confirm: ^bool = false
        item_to_delete: ^Item? = none
        
        fn request_delete(item: Item):
            item_to_delete = item
            show_confirm = true
        
        fn confirm_delete():
            if item_to_delete != none:
                api.delete(item_to_delete.id)
                show_confirm = false
                item_to_delete = none
    )
    
    <button text="Eliminar" style=Danger click=@() => request_delete(item) />
    
    <modal open=@show_confirm on_close=@() => show_confirm = false title="Confirmar eliminación">
        <vstack spacing=16>
            <text value=@"¿Eliminar " + item_to_delete.name + "?" />
            <text value="Esta acción no se puede deshacer." style=style(color: color("#666666")) />
            
            <hstack spacing=8 alignment=alignment.end>
                <button text="Cancelar" style=Secondary click=@() => show_confirm = false />
                <button text="Eliminar" style=Danger click=@confirm_delete />
            </hstack>
        </vstack>
    </modal>
</view>
```

### Modal de formulario

```kyx
<view>
    @(
        show_edit: ^bool = false
        form: ^UserForm = UserForm()
        
        fn open_edit(user: User):
            form = UserForm.from_user(user)
            show_edit = true
        
        fn handle_save():
            if form.validate().is_empty():
                api.update(form)
                show_edit = false
    )
    
    <button text="Editar" click=@() => open_edit(user) />
    
    <modal open=@show_edit on_close=@() => show_edit = false title="Editar usuario" size=modal_size.lg>
        <form model=@form submit=@handle_save>
            <vstack spacing=16>
                <text_field field="name" label="Nombre" />
                <text_field field="email" label="Email" type=input_type.email />
                <text_field field="phone" label="Teléfono" type=input_type.tel />
                
                <hstack spacing=8 alignment=alignment.end>
                    <button text="Cancelar" style=Secondary click=@() => show_edit = false />
                    <button text="Guardar" style=Primary type="submit" />
                </hstack>
            </vstack>
        </form>
    </modal>
</view>
```

### Modal con tabs

```kyx
<modal open=@show_settings on_close=@() => show_settings = false title="Configuración" size=modal_size.lg>
    <vstack spacing=16>
        <tab_bar>
            <tab name="general" label="General">
                <vstack spacing=12>
                    <text_field label="Nombre de la app" bind=@app_name />
                    <select label="Idioma" bind=@language options=@languages />
                </vstack>
            </tab>
            <tab name="security" label="Seguridad">
                <vstack spacing=12>
                    <switch label="Autenticación de dos factores" bind=@two_factor />
                    <text_field label="Timeout de sesión" bind=@session_timeout type=input_type.number />
                </vstack>
            </tab>
        </tab_bar>
        
        <hstack spacing=8 alignment=alignment.end>
            <button text="Cancelar" style=Secondary click=@() => show_settings = false />
            <button text="Guardar" style=Primary click=@save_settings />
        </hstack>
    </vstack>
</modal>
```

---

## sheet — Hoja Deslizante

Variante de modal que se desliza desde abajo (mobile-first):

```kyx
<view>
    @(
        show_sheet: ^bool = false
    )
    
    <button text="Abrir sheet" click=@() => show_sheet = true />
    
    <sheet open=@show_sheet on_close=@() => show_sheet = false>
        <vstack spacing=16>
            <text value="Contenido del sheet" />
            <button text="Cerrar" click=@() => show_sheet = false />
        </vstack>
    </sheet>
</view>
```

### Traducción por plataforma

**Web:** Modal centrado con backdrop
**Mobile:** Se desliza desde abajo (bottom sheet)
**Desktop:** Modal centrado
**iOS:** `.sheet` nativo
**Android:** `BottomSheetDialog` nativo

---

## Traducción Multiplataforma

### Web

```html
<div class="modal-backdrop" onclick="close()">
    <div class="modal" onclick="event.stopPropagation()">
        <div class="modal-header">
            <h2>Confirmar</h2>
            <button class="close" onclick="close()">×</button>
        </div>
        <div class="modal-body">
            <p>¿Estás seguro?</p>
        </div>
        <div class="modal-footer">
            <button>Cancelar</button>
            <button>Confirmar</button>
        </div>
    </div>
</div>
```

### Desktop (SDL2/Skia)

```kyle
fn render_modal(open: bool):
    if open:
        # Backdrop
        skia_fill_rect(0, 0, screen_w, screen_h, color("#000000").with_alpha(0.5))
        
        # Modal
        modal_w = 600
        modal_h = 400
        x = (screen_w - modal_w) / 2
        y = (screen_h - modal_h) / 2
        
        skia_fill_rounded_rect(x, y, modal_w, modal_h, 8.0, color("#FFFFFF"))
        skia_draw_text(x + 20, y + 20, "Confirmar", title_color)
        
        # Handle close
        if key_pressed(ESCAPE) or mouse_click_outside_modal():
            on_close()
```

### iOS (SwiftUI)

```swift
.sheet(isPresented: $showModal) {
    VStack(spacing: 16) {
        Text("¿Estás seguro?")
        HStack {
            Button("Cancelar") { showModal = false }
            Button("Confirmar") { confirm() }
        }
    }
    .padding()
}
```

### Android (Jetpack Compose)

```kotlin
if (showModal) {
    AlertDialog(
        onDismissRequest = { showModal = false },
        title = { Text("Confirmar") },
        text = { Text("¿Estás seguro?") },
        confirmButton = {
            Button(onClick = { confirm() }) { Text("Confirmar") }
        },
        dismissButton = {
            Button(onClick = { showModal = false }) { Text("Cancelar") }
        }
    )
}
```

---

## Errores Comunes

### ❌ Malo: Modal sin on_close

```kyx
<modal open=@show_modal>
    <text value="Contenido" />
</modal>
```

**Problema:** Usuario no puede cerrar el modal.

### ✅ Bueno: Siempre proporcionar on_close

```kyx
<modal open=@show_modal on_close=@() => show_modal = false>
    <text value="Contenido" />
</modal>
```

### ❌ Malo: Modal muy grande en mobile

```kyx
<modal open=@show_modal size=modal_size.xl>
    <text value="Contenido" />
</modal>
```

**Problema:** No se ve bien en pantallas pequeñas.

### ✅ Bueno: Usar sheet en mobile

```kyx
<view>
    @if current_breakpoint() == breakpoint.mobile:
        <sheet open=@show_modal on_close=@() => show_modal = false>
            <text value="Contenido" />
        </sheet>
    @else:
        <modal open=@show_modal on_close=@() => show_modal = false>
            <text value="Contenido" />
        </modal>
</view>
```

---

## Referencias

- [alert](alert.md)
- [toast](toast.md)
- [Componentes nativos](README.md)
