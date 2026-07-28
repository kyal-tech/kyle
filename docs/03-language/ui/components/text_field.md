# text_field — Campo de Texto

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class text_field:
    # Valor
    bind: ^str?                     # Binding bidireccional (sin modelo)
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    label: str?                     # Etiqueta del campo
    placeholder: str = ""           # Texto placeholder
    type: input_type = input_type.text  # Tipo de input
    
    # Validación
    required: bool = false          # Campo requerido
    min_length: i32?                # Longitud mínima
    max_length: i32?                # Longitud máxima
    pattern: str?                   # Regex pattern
    error: str?                     # Mensaje de error
    
    # Estado
    disabled: bool = false          # Deshabilitado
    readonly: bool = false          # Solo lectura
    
    # Eventos
    on_input: fn(event: input_event)?    # Al escribir
    on_change: fn(event: change_event)?  # Al perder foco
    on_focus: fn()?                      # Al enfocar
    on_blur: fn()?                       # Al perder foco
    
    # Accesibilidad
    aria_label: str?
    aria_describedby: str?
```

---

## Uso Básico

### Con bind (sin modelo)

```kyx
<view>
    @(
        email: ^str = ""
    )
    
    <text_field label="Email" placeholder="tu@email.com" bind=@email />
</view>
```

### Con modelo (form)

```kyx
<view>
    @(
        form: ^UserForm = UserForm()
    )
    
    <form model=@form>
        <text_field field="email" label="Email" placeholder="tu@email.com" />
        <text_field field="name" label="Nombre" />
    </form>
</view>
```

### Con validación

```kyx
<text_field 
    label="Email" 
    bind=@email 
    required=true 
    pattern="^[^@]+@[^@]+\.[^@]+$"
    error=@email_error 
/>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^str?` | `none` | Binding bidireccional |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `label` | `str?` | `none` | Etiqueta del campo |
| `placeholder` | `str` | `""` | Texto placeholder |
| `type` | `input_type` | `text` | Tipo de input |
| `required` | `bool` | `false` | Campo requerido |
| `min_length` | `i32?` | `none` | Longitud mínima |
| `max_length` | `i32?` | `none` | Longitud máxima |
| `pattern` | `str?` | `none` | Regex pattern |
| `error` | `str?` | `none` | Mensaje de error |
| `disabled` | `bool` | `false` | Deshabilitado |
| `readonly` | `bool` | `false` | Solo lectura |

---

## Tipos Relacionados

### input_type

```kyle
enum input_type:
    text            # Texto normal
    email           # Email (validación automática)
    password        # Contraseña (oculto)
    number          # Número
    tel             # Teléfono
    url             # URL
    search          # Búsqueda
    date            # Fecha
    time            # Hora
    datetime        # Fecha y hora
```

---

## Ejemplos Avanzados

### Password field

```kyx
<password_field label="Contraseña" bind=@password required=true min_length=8 />
```

### Con helper text

```kyx
<text_field 
    label="Usuario" 
    bind=@username 
    placeholder="ej: juan123"
    helper_text="Solo letras y números, 4-20 caracteres"
    min_length=4
    max_length=20
/>
```

### Con ícono

```kyx
<text_field 
    label="Buscar" 
    bind=@search_query 
    icon="search"
    placeholder="Buscar..."
/>
```

### Formulario completo

```kyx
<view>
    @(
        form: ^RegisterForm = RegisterForm()
        errors: ^{str: str} = {}
        
        fn validate():
            errors = {}
            if form.email == "":
                errors.set("email", "Email requerido")
            elif !form.email.contains("@"):
                errors.set("email", "Email inválido")
            
            if form.password.len() < 8:
                errors.set("password", "Mínimo 8 caracteres")
            
            if form.password != form.password_confirm:
                errors.set("password_confirm", "Las contraseñas no coinciden")
        
        fn handle_submit():
            validate()
            if errors.is_empty():
                submit_form(form)
    )
    
    <form model=@form submit=@handle_submit>
        <text_field field="email" label="Email" type=input_type.email error=@errors.get("email") />
        <password_field field="password" label="Contraseña" error=@errors.get("password") />
        <password_field field="password_confirm" label="Confirmar" error=@errors.get("password_confirm") />
        
        <button text="Registrarse" style=Primary type="submit" />
    </form>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div class="text_field">
    <label>Email</label>
    <input type="email" placeholder="tu@email.com" value="" />
    <span class="error">Email inválido</span>
</div>
```

### Desktop (SDL2/Skia)

```kyle
fn render_text_field(x: f32, y: f32, w: f32, h: f32):
    # Border
    skia_draw_rect(x, y, w, h, border_color)
    
    # Label
    skia_draw_text(x, y - 20, "Email", label_color)
    
    # Input text
    skia_draw_text(x + 8, y + 8, value, text_color)
    
    # Error
    if error != "":
        skia_draw_text(x, y + h + 4, error, error_color)
    
    # Handle keyboard input
    if focused and keyboard_input():
        value += keyboard_text()
```

### iOS (SwiftUI)

```swift
VStack(alignment: .leading) {
    Text("Email")
    TextField("tu@email.com", text: $email)
        .textFieldStyle(RoundedBorderTextFieldStyle())
        .keyboardType(.emailAddress)
    if error != nil {
        Text(error!).foregroundColor(.red).font(.caption)
    }
}
```

### Android (Jetpack Compose)

```kotlin
Column {
    OutlinedTextField(
        value = email,
        onValueChange = { email = it },
        label = { Text("Email") },
        placeholder = { Text("tu@email.com") },
        isError = error != null,
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email)
    )
    if (error != null) {
        Text(error!!, color = Color.Red, fontSize = 12.sp)
    }
}
```

---

## Errores Comunes

### ❌ Malo: No manejar errores

```kyx
<text_field label="Email" bind=@email />
<button text="Enviar" click=@submit />
```

**Problema:** No hay validación, usuario puede enviar datos inválidos.

### ✅ Bueno: Validar y mostrar errores

```kyx
<text_field label="Email" bind=@email error=@email_error />
<button text="Enviar" disabled=@has_errors click=@submit />
```

### ❌ Malo: Usar strings para tipos

```kyx
<text_field type="email" />
```

**Problema:** No tipado, errores en runtime.

### ✅ Bueno: Usar enum tipado

```kyx
<text_field type=input_type.email />
```

---

## Referencias

- [Formulario](form.md)
- [Estado y eventos](../state-events.md)
- [Componentes nativos](README.md)
