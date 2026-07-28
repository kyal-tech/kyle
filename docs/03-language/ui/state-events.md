# Estado y Eventos

**Status:** Draft v2.0
**Date:** 2026-07-12
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [style-system.md](style-system.md) — Sistema de estilos
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target

---

## 1. Estado de Componente

### 1.1 Estado local

Todo el código Kyle va dentro del bloque `@(...)`:

```kyx
<view>
    @(
        count: ^i32 = 0

        fn increment():
            count += 1
    )

    <text value=@"Contador: " + count.to_str() />
    <button text="+" click=@increment />
</view>
```

### 1.2 Estado derivado

```kyx
<view>
    @(
        items: ^{str} = {}
        filter_text: ^str = ""
        filtered: ^{str} = {}

        fn update_filter():
            filtered = items.filter(fn(x): x.contains(filter_text))
    )
</view>
```

### 1.3 Props vía visibilidad

Los props son variables **públicas** dentro de `@(...)`. Lo interno se marca con `_`:

| Declaración dentro de `@(...)` | ¿Es prop? | Visible desde fuera |
|-------------------------------|-----------|-------------------|
| `name: str` | ✅ Sí | Se pasa como atributo |
| `count: ^i32` | ✅ Sí | Binding reactivo |
| `_internal: str` | ❌ No | Uso interno |
| `__cache: [i32]` | ❌ No | Privado del módulo |
| `fn on_click(this)` | ✅ Sí | Callback |

```kyx
<!-- Definición en UserCard.kyx -->
<view>
    @(
        name: str                      # prop requerido
        age: i32 = 0                   # prop opcional con default
        on_click: ^&(fn ())            # callback como prop
    )

    <text value=@"Nombre: " + name />
    <text value=@"Edad: " + age.to_str() />
</view>
```

```kyx
<!-- Uso desde padre -->
<user_card name="Juan" age=30 on_click=@handle_click />
```

### 1.4 Estado global (Context)

Para estado compartido entre componentes:

```kyx
<view>
    @(
        context AuthContext:
            user: User?
            token: str?
            fn login(email: str, password: str):
                # ...
    )
</view>
```

Consumir contexto:

```kyx
<view>
    @(
        auth = use_context(AuthContext)
    )
    @if auth.user != None:
        <text value=@"Bienvenido, " + auth.user.name />
    @else:
        <button text="Login" click=@show_login />
</view>
```

---

## 2. Binding Bidireccional

### 2.1 One-way binding (estado → UI)

```kyx
<text value=@name />               # se actualiza cuando name cambia
```

### 2.2 Two-way binding (estado ↔ UI)

```kyx
<text_field bind=@email />          # email se actualiza al escribir
<checkbox bind=@accept_terms />
```

### 2.3 Binding a props de componentes

```kyx
<child_component prop=@parent_var />
```

Cuando `parent_var` cambia, `prop` se actualiza automáticamente.

### 2.4 Cómo funciona el binding

Web target:
```javascript
const state = { email: '' };
const input = document.createElement('input');
input.addEventListener('input', () => {
    state.email = input.value;
    wasm.exports.onStateChange('email', state.email);
});
```

---

## 3. Eventos

### 3.1 Eventos de UI

```kyx
<button click=@handle_click />
<text_field change=@handle_change />
<text_field input=@on_input />
<checkbox change=@on_toggle />
<select change=@on_select />
<form submit=@handle_submit />
<view mouse_enter=@on_hover />
<view mouse_leave=@on_leave />
<view focus=@on_focus />
<view blur=@on_blur />
<view keydown=@on_keypress />
<view scroll=@on_scroll />
```

### 3.2 Parámetros de evento

```kyx
<view>
    @(
        fn handle_click(event: MouseEvent):
            print("click en: " + event.x.to_str() + ", " + event.y.to_str())

        fn handle_keypress(event: KeyboardEvent):
            if event.key == "Enter":
                submit()
    )
</view>
```

### 3.3 Eventos personalizados

Los componentes pueden emitir eventos personalizados via callbacks:

```kyx
<!-- Definición -->
<view>
    on_save: ^&(fn (data: str))
    data: ^str = ""

    @(
        fn save():
            on_save(data)
    )

    <text_field bind=@data />
    <button text="Guardar" click=@save />
</view>
```

```kyx
<!-- Uso -->
<my_form on_save=@handle_save />
```

### 3.4 Event bubbling

Por defecto, los eventos burbujean hacia el padre:

```kyx
<view>
    @(
        fn handle_click(event: MouseEvent):
            event.stop_propagation()
    )
</view>
```

---

## 4. Reactividad

### 4.1 Cómo se actualiza la UI

```
count = count + 1
     │
     ▼
Runtime detecta el cambio
     │
     ▼
Busca qué componentes dependen de count
     │
     ▼
Re-renderiza solo esos componentes
```

### 4.2 Implementación por target

**Web:** Proxy/setters en JS:
```javascript
function createState(initial) {
    const state = reactive({ ...initial });
    return new Proxy(state, {
        set(target, key, value) {
            target[key] = value;
            notifyWatchers(key, value);
            return true;
        }
    });
}
```

**Desktop:** Runtime Kyle maneja reactive updates vía diff del árbol de componentes.

### 4.3 Optimización

Solo los componentes afectados se re-renderizan:

```
count ───→ CounterView (depende de count)
           └──→ Button (no depende → skip)
```

---

## 5. Comunicación entre Componentes

### 5.1 Padre → Hijo (props)

```kyx
<child_component title="Mi Título" items=@data_list />
```

### 5.2 Hijo → Padre (callbacks)

```kyx
<child_component on_delete=@handle_delete on_select=@handle_select />
```

### 5.3 Entre hermanos (vía padre)

```kyx
<view>
    @(
        selected_id: ^i32 = -1

        fn on_select(id: i32):
            selected_id = id
    )

    <item_list items=@items on_select=@on_select />
    <item_detail item_id=@selected_id />
</view>
```

### 5.4 Entre componentes no relacionados (vía context)

```kyx
<view>
    @(
        context AppState:
            theme: Theme = LightTheme
            user: User?
            notifications: {Notification}

        state = use_context(AppState)
    )
</view>
```

---

## 6. Ciclo de Vida de Eventos

```
Usuario hace click
     │
     ▼
Evento nativo (click del browser / touch / mouse)
     │
     ▼
[Web] addEventListener captura
[Desktop] Event loop captura
[iOS] UIControl action
     │
     ▼
Traduce a evento Kyle (MouseEvent, KeyboardEvent)
     │
     ▼
Ejecuta el callback Kyle
     │
     ▼
Callback modifica estado
     │
     ▼
Runtime detecta cambios → re-renderiza
     │
     ▼
[Web] Actualiza DOM
[Desktop] Redibuja con Skia
[iOS] Actualiza SwiftUI View
```

---

## 7. Formularios

### 7.1 Form básico

```kyx
<view>
    @(
        email: ^str = ""
        password: ^str = ""
        errors: ^{str} = {}
        loading: ^bool = false

        fn validate() bool:
            errors = {}
            if email == "":
                errors.push("Email requerido")
            if password == "":
                errors.push("Contraseña requerida")
            if password.len() < 6:
                errors.push("Mínimo 6 caracteres")
            errors.len() == 0

        fn handle_submit():
            if !validate():
                return
            loading = true
    )

    <form submit=@handle_submit>
        <text_field label="Email" bind=@email
            error=if errors.contains("Email requerido"): "Requerido" else: "" />
        <password_field label="Contraseña" bind=@password
            error=if errors.contains("Contraseña requerida"): "Requerido" else: "" />
        <button style=Primary text=@"Ingresar" disabled=@loading />
    </form>
</view>
```

---

## 8. Form Models (Class Binding)

### 8.1 Problema

Sin form models, cada campo requiere una variable separada:

```kyx
<view>
    @(
        name: ^str = ""
        email: ^str = ""
        age: ^i32 = 0
        errors: ^{str} = {}
    )
</view>
```

20 campos = 20 variables. Frustrante, repetitivo, propenso a errores.

### 8.2 Solución: `model=` en `<form>`

Se declara una clase como **modelo** y el `<form>` la usa automáticamente:

```kyx
# models/user.ky
final class UserForm:
    name: str = ""
    email: str = ""
    age: i32 = 0
    avatar: file_data?       # ← tipo file_data para la foto

    fn validate() {str}:
        e = {}
        if name == "": e.set("name", "Requerido")
        if !email.contains("@"): e.set("email", "Email inválido")
        if age < 18: e.set("age", "Debe ser mayor de edad")
        e

    fn to_api() {str}:
        {
            "name": name,
            "email": email,
            "age": age.to_str(),
            "avatar": avatar.content  # ← bytes para enviar
        }
```

```kyx
# views/register.kyx
<view>
    @(
        form: ^UserForm = UserForm()

        fn handle_submit():
            errs = form.validate()
            if errs.is_empty():
                http.post("/api/users", json: form.to_api())
    )

    <form model=@form submit=@handle_submit>
        <text_field field="name" label="Nombre" />
        <text_field field="email" label="Email" />
        <number_field field="age" label="Edad" />
        <file_picker field="avatar" accept="image/*" label="Foto de perfil" />
        <button style=Primary text="Guardar" />
    </form>
</view>
```

### 8.3 `field=` vs `bind=`

| Atributo | Uso | Descripción |
|----------|-----|-------------|
| `bind=@var` | Sin modelo | Binding a variable suelta |
| `field="name"` | Con `model=@form` | Binding automático al campo del modelo |

### 8.4 Validación automática

```kyx
# Los errores se muestran automáticamente
<text_field field="email" label="Email" />
# Si form.errors tiene "email", se muestra el error debajo del campo
```

### 8.5 Validaciones declarativas (post-MVP)

```kyx
final class UserForm:
    @required name: str
    @email email: str
    @min(18) age: str
    @pattern("^[0-9]+$") phone: str
```

Estas anotaciones generarían automáticamente las reglas de validación sin escribir `fn validate()`.

---

## 9. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target
- [style-system.md](style-system.md) — Sistema de estilos
- [routing.md](routing.md) — Routing/Navegación
- [file-picker.md](file-picker.md) — Selector de archivos nativo
