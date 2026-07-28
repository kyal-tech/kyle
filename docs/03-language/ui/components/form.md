# form — Formulario

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class form:
    # Modelo
    model: ^any?                      # Modelo de datos (clase Kyle)
    
    # Validación
    validate: fn() -> {str: str}?     # Función de validación personalizada
    
    # Eventos
    submit: fn()?                     # Callback al enviar
    
    # Configuración
    auto_validate: bool = false       # Validar en cada cambio
    show_errors: bool = true          # Mostrar errores automáticamente
```

---

## Uso Básico

### Con modelo (recomendado)

```kyx
<view>
    @(
        form: ^UserForm = UserForm()
        
        fn handle_submit():
            if form.validate().is_empty():
                submit_to_api(form)
    )
    
    <form model=@form submit=@handle_submit>
        <text_field field="name" label="Nombre" />
        <text_field field="email" label="Email" type=input_type.email />
        <number_field field="age" label="Edad" />
        
        <button text="Guardar" style=Primary type="submit" />
    </form>
</view>
```

### Sin modelo (bind directo)

```kyx
<view>
    @(
        email: ^str = ""
        password: ^str = ""
        
        fn handle_submit():
            if email != "" and password.len() >= 8:
                login(email, password)
    )
    
    <form submit=@handle_submit>
        <text_field label="Email" bind=@email />
        <password_field label="Contraseña" bind=@password />
        
        <button text="Ingresar" style=Primary type="submit" />
    </form>
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `model` | `^any?` | `none` | Modelo de datos |
| `validate` | `fn() -> {str: str}?` | `none` | Validación personalizada |
| `submit` | `fn()?` | `none` | Callback al enviar |
| `auto_validate` | `bool` | `false` | Validar en cada cambio |
| `show_errors` | `bool` | `true` | Mostrar errores automáticamente |

---

## Modelo de Formulario

### Definición del modelo

```kyle
final class UserForm:
    name: str = ""
    email: str = ""
    age: i32 = 0
    avatar: file_data?
    
    fn validate(this) {str: str}:
        errors: {str: str} = {}
        
        if name == "":
            errors.set("name", "Nombre requerido")
        
        if email == "":
            errors.set("email", "Email requerido")
        elif !email.contains("@"):
            errors.set("email", "Email inválido")
        
        if age < 18:
            errors.set("age", "Debe ser mayor de edad")
        
        errors
    
    fn to_json(this) str:
        json.stringify({
            "name": name,
            "email": email,
            "age": age.to_str()
        })
```

### Uso del modelo

```kyx
<view>
    @(
        form: ^UserForm = UserForm()
        errors: ^{str: str} = {}
        
        fn handle_submit():
            errors = form.validate()
            if errors.is_empty():
                http.post("/api/users", body: form.to_json())
    )
    
    <form model=@form submit=@handle_submit>
        <text_field field="name" label="Nombre" error=@errors.get("name") />
        <text_field field="email" label="Email" error=@errors.get("email") />
        <number_field field="age" label="Edad" error=@errors.get("age") />
        
        <button text="Guardar" style=Primary type="submit" />
    </form>
</view>
```

---

## field vs bind

| Atributo | Uso | Descripción |
|----------|-----|-------------|
| `bind=@var` | Sin modelo | Binding directo a variable |
| `field="name"` | Con `model=@form` | Binding al campo del modelo |

### Ejemplo comparativo

```kyx
# Sin modelo (bind)
<text_field label="Email" bind=@email />

# Con modelo (field)
<form model=@form>
    <text_field field="email" label="Email" />
</form>
```

---

## Ejemplos Avanzados

### Formulario con validación en tiempo real

```kyx
<view>
    @(
        form: ^RegisterForm = RegisterForm()
        
        fn validate_field(field: str):
            match field:
                "email":
                    if !form.email.contains("@"):
                        return "Email inválido"
                "password":
                    if form.password.len() < 8:
                        return "Mínimo 8 caracteres"
            ""
    )
    
    <form model=@form auto_validate=true>
        <text_field field="email" label="Email" />
        <password_field field="password" label="Contraseña" />
        
        <button text="Registrarse" style=Primary type="submit" />
    </form>
</view>
```

### Formulario con file upload

```kyx
<view>
    @(
        form: ^ProfileForm = ProfileForm()
        
        fn handle_submit():
            http.upload("/api/profile", form)
    )
    
    <form model=@form submit=@handle_submit>
        <text_field field="name" label="Nombre" />
        <file_picker field="avatar" accept="image/*" label="Foto de perfil" />
        
        <button text="Guardar" style=Primary type="submit" />
    </form>
</view>
```

### Formulario con pasos (wizard)

```kyx
<view>
    @(
        step: ^i32 = 1
        form: ^MultiStepForm = MultiStepForm()
        
        fn next_step():
            if step == 1 and form.validate_step_1().is_empty():
                step += 1
            elif step == 2 and form.validate_step_2().is_empty():
                step += 1
        
        fn prev_step():
            if step > 1:
                step -= 1
    )
    
    <form model=@form>
        @if step == 1:
            <text_field field="name" label="Nombre" />
            <text_field field="email" label="Email" />
        @elif step == 2:
            <text_field field="address" label="Dirección" />
            <text_field field="phone" label="Teléfono" />
        @elif step == 3:
            <text value="Confirmar datos" />
        
        <hstack spacing=8>
            @if step > 1:
                <button text="Anterior" style=Secondary click=@prev_step />
            @if step < 3:
                <button text="Siguiente" style=Primary click=@next_step />
            @else:
                <button text="Enviar" style=Primary click=@submit />
        </hstack>
    </form>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<form onsubmit="handleSubmit(event)">
    <input type="text" name="name" />
    <input type="email" name="email" />
    <button type="submit">Guardar</button>
</form>
```

### Desktop (SDL2/Skia)

```kyle
# Form es un contenedor lógico, no visual
# Los inputs se renderizan individualmente
fn render_form():
    render_text_field(name_field)
    render_text_field(email_field)
    render_button(submit_button)
```

### iOS (SwiftUI)

```swift
Form {
    TextField("Nombre", text: $form.name)
    TextField("Email", text: $form.email)
    Button("Guardar") { handleSubmit() }
}
```

### Android (Jetpack Compose)

```kotlin
Column {
    OutlinedTextField(value = form.name, onValueChange = { form.name = it }, label = { Text("Nombre") })
    OutlinedTextField(value = form.email, onValueChange = { form.email = it }, label = { Text("Email") })
    Button(onClick = { handleSubmit() }) { Text("Guardar") }
}
```

---

## Errores Comunes

### ❌ Malo: Validar solo al enviar

```kyx
<form submit=@handle_submit>
    <text_field field="email" />
    <button text="Enviar" />
</form>
```

**Problema:** Usuario descubre errores solo al final.

### ✅ Bueno: Validación en tiempo real

```kyx
<form model=@form auto_validate=true>
    <text_field field="email" />
    <button text="Enviar" disabled=@!form.is_valid />
</form>
```

### ❌ Malo: Mezclar lógica de negocio en el form

```kyx
<form submit=@() => {
    # Lógica de API aquí
    http.post("/api/users", form.to_json())
}>
```

**Problema:** Dificulta testing y reutilización.

### ✅ Bueno: Separar lógica

```kyx
<view>
    @(
        fn handle_submit():
            if form.validate().is_empty():
                submit_to_api(form)
    )
    
    <form model=@form submit=@handle_submit>
        ...
    </form>
</view>
```

---

## Referencias

- [text_field](text_field.md)
- [Estado y eventos](../state-events.md)
- [Componentes nativos](README.md)
