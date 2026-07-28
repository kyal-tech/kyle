# toast — Notificación Temporal

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
# Toast no es un componente visual, es una función del runtime

fn toast.success(message: str, duration: i32 = 3000)
fn toast.error(message: str, duration: i32 = 5000)
fn toast.info(message: str, duration: i32 = 3000)
fn toast.warning(message: str, duration: i32 = 4000)

fn toast.show(
    message: str,
    type: toast_type = toast_type.info,
    duration: i32 = 3000,
    action: str? = none,
    on_action: fn()? = none
)
```

---

## Uso Básico

### Notificaciones simples

```kyx
<view>
    @(
        fn handle_save():
            # Lógica de guardado
            toast.success("Guardado correctamente")
        
        fn handle_delete():
            # Lógica de eliminación
            toast.info("Item eliminado")
        
        fn handle_error():
            toast.error("Error al conectar con el servidor")
    )
    
    <button text="Guardar" click=@handle_save />
    <button text="Eliminar" click=@handle_delete />
</view>
```

---

## Tipos Relacionados

### toast_type

```kyle
enum toast_type:
    success         # Verde, ícono check
    error           # Rojo, ícono X
    info            # Azul, ícono info
    warning         # Amarillo, ícono warning
```

---

## Ejemplos Avanzados

### Toast con acción

```kyx
<view>
    @(
        fn handle_delete():
            item = get_selected_item()
            api.delete(item.id)
            
            toast.show(
                message: "Item eliminado",
                type: toast_type.info,
                duration: 5000,
                action: "Deshacer",
                on_action: @() => undo_delete(item)
            )
    )
    
    <button text="Eliminar" click=@handle_delete />
</view>
```

### Toast en formulario

```kyx
<view>
    @(
        form: ^UserForm = UserForm()
        
        fn handle_submit():
            errors = form.validate()
            if !errors.is_empty():
                toast.error("Por favor corrige los errores")
                return
            
            result = api.save(form)
            if result.is_ok:
                toast.success("Usuario creado correctamente")
                navigate("/users")
            else:
                toast.error(@"Error: " + result.error)
    )
    
    <form model=@form submit=@handle_submit>
        <text_field field="name" label="Nombre" />
        <text_field field="email" label="Email" />
        <button text="Guardar" style=Primary type="submit" />
    </form>
</view>
```

### Toast de progreso

```kyx
<view>
    @(
        fn handle_upload():
            toast_id = toast.show(
                message: "Subiendo archivo...",
                type: toast_type.info,
                duration: 0  # No auto-cerrar
            )
            
            result = api.upload(file)
            
            toast.dismiss(toast_id)
            
            if result.is_ok:
                toast.success("Archivo subido correctamente")
            else:
                toast.error("Error al subir archivo")
    )
    
    <button text="Subir" click=@handle_upload />
</view>
```

### Múltiples toasts

```kyx
<view>
    @(
        fn handle_batch_operation():
            # Mostrar progreso
            for i, item in items:
                api.process(item)
                toast.info(@"Procesando " + (i + 1).to_str() + " de " + items.len().to_str())
            
            toast.success("Operación completada")
    )
    
    <button text="Procesar todo" click=@handle_batch_operation />
</view>
```

---

## API Completa

### toast.show()

```kyle
toast.show(
    message: str,                      # Mensaje a mostrar
    type: toast_type = toast_type.info, # Tipo (success, error, info, warning)
    duration: i32 = 3000,              # Duración en ms (0 = no auto-cerrar)
    position: toast_position = toast_position.top_right,  # Posición
    action: str? = none,               # Texto del botón de acción
    on_action: fn()? = none            # Callback al hacer click en acción
) -> str  # Retorna toast_id
```

### toast.dismiss()

```kyle
toast.dismiss(toast_id: str)  # Cierra un toast específico
toast.dismiss_all()           # Cierra todos los toasts
```

### toast_position

```kyle
enum toast_position:
    top_left
    top_center
    top_right
    bottom_left
    bottom_center
    bottom_right
```

---

## Traducción Multiplataforma

### Web

```html
<div class="toast-container top-right">
    <div class="toast success">
        <span class="icon">✓</span>
        <span class="message">Guardado correctamente</span>
        <button class="close">×</button>
    </div>
</div>
```

```css
.toast-container {
    position: fixed;
    z-index: 9999;
}
.toast-container.top-right {
    top: 20px;
    right: 20px;
}
.toast {
    padding: 12px 16px;
    border-radius: 8px;
    margin-bottom: 8px;
    animation: slideIn 0.3s ease;
}
.toast.success {
    background: #10B981;
    color: white;
}
```

### Desktop (SDL2/Skia)

```kyle
fn render_toast(toast: Toast):
    x = screen_w - toast.width - 20
    y = 20
    
    # Background
    color = match toast.type:
        toast_type.success: color("#10B981")
        toast_type.error: color("#EF4444")
        toast_type.info: color("#3B82F6")
        toast_type.warning: color("#F59E0B")
    
    skia_fill_rounded_rect(x, y, toast.width, 60, 8.0, color)
    skia_draw_text(x + 16, y + 20, toast.message, color("#FFFFFF"))
    
    # Auto-dismiss
    if current_time() > toast.created_at + toast.duration:
        dismiss_toast(toast.id)
```

### iOS (SwiftUI)

```swift
// Usar biblioteca como Toast-Swift
self.view.makeToast("Guardado correctamente", duration: 3.0, position: .top)
```

### Android (Jetpack Compose)

```kotlin
val snackbarHostState = remember { SnackbarHostState() }

LaunchedEffect(Unit) {
    snackbarHostState.showSnackbar(
        message = "Guardado correctamente",
        duration = SnackbarDuration.Short
    )
}
```

---

## Errores Comunes

### ❌ Malo: Usar toast para errores críticos

```kyx
fn handle_critical_error():
    toast.error("Error crítico del sistema")
```

**Problema:** Toast desaparece, usuario puede no verlo.

### ✅ Bueno: Usar modal para errores críticos

```kyx
fn handle_critical_error():
    show_error_modal = true

<modal open=@show_error_modal title="Error crítico">
    <text value="Error crítico del sistema. Por favor contacta soporte." />
</modal>
```

### ❌ Malo: Demasiados toasts a la vez

```kyx
for item in items:
    toast.success(@"Item " + item.name + " procesado")
```

**Problema:** 10 toasts apilados, molesto.

### ✅ Bueno: Consolidar mensajes

```kyx
toast.success(@"10 items procesados correctamente")
```

### ❌ Malo: Toast sin duración para operaciones largas

```kyx
fn handle_long_operation():
    toast.info("Procesando...")
    # Operación de 10 segundos
    long_operation()
    # Toast sigue showing "Procesando..."
```

**Problema:** Toast muestra mensaje obsoleto.

### ✅ Bueno: Dismiss manual

```kyx
fn handle_long_operation():
    toast_id = toast.info("Procesando...", duration=0)
    long_operation()
    toast.dismiss(toast_id)
    toast.success("Operación completada")
```

---

## Referencias

- [modal](modal.md)
- [alert](alert.md)
- [Componentes nativos](README.md)
