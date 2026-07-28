# alert — Diálogo de Alerta

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class alert:
    # Estado
    open: ^bool                     # Controla si está abierto
    
    # Contenido
    title: str                      # Título
    message: str                    # Mensaje
    
    # Configuración
    type: alert_type = alert_type.info  # Tipo (info, success, warning, error)
    confirm_text: str = "OK"        # Texto del botón confirmar
    cancel_text: str?               # Texto del botón cancelar (none = solo OK)
    
    # Eventos
    on_confirm: fn()?               # Callback al confirmar
    on_cancel: fn()?                # Callback al cancelar
    on_close: fn()?                 # Callback al cerrar
```

---

## Uso Básico

### Alerta simple

```kyx
<view>
    @(
        show_alert: ^bool = false
    )
    
    <button text="Mostrar alerta" click=@() => show_alert = true />
    
    <alert 
        open=@show_alert 
        title="Información"
        message="Operación completada exitosamente"
        on_close=@() => show_alert = false
    />
</view>
```

### Alerta de confirmación

```kyx
<view>
    @(
        show_confirm: ^bool = false
        
        fn handle_delete():
            show_confirm = true
        
        fn confirm_delete():
            delete_item()
            show_confirm = false
    )
    
    <button text="Eliminar" click=@handle_delete />
    
    <alert 
        open=@show_confirm 
        title="Confirmar eliminación"
        message="¿Estás seguro de que deseas eliminar este item?"
        type=alert_type.warning
        confirm_text="Eliminar"
        cancel_text="Cancelar"
        on_confirm=@confirm_delete
        on_cancel=@() => show_confirm = false
    />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `open` | `^bool` | — | **Requerido.** Controla visibilidad |
| `title` | `str` | — | **Requerido.** Título |
| `message` | `str` | — | **Requerido.** Mensaje |
| `type` | `alert_type` | `info` | Tipo |
| `confirm_text` | `str` | `"OK"` | Texto del botón confirmar |
| `cancel_text` | `str?` | `none` | Texto del botón cancelar |
| `on_confirm` | `fn()?` | `none` | Callback al confirmar |
| `on_cancel` | `fn()?` | `none` | Callback al cancelar |
| `on_close` | `fn()?` | `none` | Callback al cerrar |

---

## Tipos Relacionados

### alert_type

```kyle
enum alert_type:
    info            # Azul, ícono info
    success         # Verde, ícono check
    warning         # Amarillo, ícono warning
    error           # Rojo, ícono error
```

---

## Ejemplos Avanzados

### Alerta de error

```kyx
<alert 
    open=@show_error 
    title="Error"
    message="No se pudo conectar al servidor. Por favor intenta nuevamente."
    type=alert_type.error
    on_close=@() => show_error = false
/>
```

### Alerta de éxito

```kyx
<alert 
    open=@show_success 
    title="¡Éxito!"
    message="Tu perfil ha sido actualizado correctamente."
    type=alert_type.success
    on_close=@() => show_success = false
/>
```

### Función helper para alertas

```kyx
<view>
    @(
        alert_state: ^alert_state = alert_state.hidden
        
        fn show_alert(title: str, message: str, type: alert_type):
            alert_state = alert_state.visible(title, message, type)
        
        fn close_alert():
            alert_state = alert_state.hidden
    )
    
    <button text="Info" click=@() => show_alert("Info", "Mensaje informativo", alert_type.info) />
    <button text="Error" click=@() => show_alert("Error", "Ocurrió un error", alert_type.error) />
    
    @match alert_state:
        alert_state.hidden:
            none
        alert_state.visible(title, message, type):
            <alert 
                open=true 
                title=@title
                message=@message
                type=@type
                on_close=@close_alert
            />
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div class="alert-overlay">
    <div class="alert">
        <h3>Confirmar eliminación</h3>
        <p>¿Estás seguro?</p>
        <div class="alert-actions">
            <button onclick="cancel()">Cancelar</button>
            <button onclick="confirm()">Eliminar</button>
        </div>
    </div>
</div>
```

### iOS (SwiftUI)

```swift
.alert("Confirmar eliminación", isPresented: $show_confirm) {
    Button("Cancelar", role: .cancel) { }
    Button("Eliminar", role: .destructive) { confirm_delete() }
} message: {
    Text("¿Estás seguro?")
}
```

### Android (Jetpack Compose)

```kotlin
AlertDialog(
    onDismissRequest = { show_confirm = false },
    title = { Text("Confirmar eliminación") },
    text = { Text("¿Estás seguro?") },
    confirmButton = {
        Button(onClick = { confirm_delete() }) { Text("Eliminar") }
    },
    dismissButton = {
        Button(onClick = { show_confirm = false }) { Text("Cancelar") }
    }
)
```

---

## Diferencia entre alert y modal

| Característica | alert | modal |
|----------------|-------|-------|
| Contenido | Título + mensaje + botones | Contenido personalizado |
| Complejidad | Simple | Compleja |
| Uso | Confirmaciones, notificaciones | Formularios, detalles |

**Usar alert cuando:**
- Confirmaciones simples
- Notificaciones de error/éxito
- Mensajes informativos

**Usar modal cuando:**
- Formularios
- Contenido complejo
- Múltiples acciones

---

## Referencias

- [modal](modal.md)
- [toast](toast.md)
- [Componentes nativos](README.md)
