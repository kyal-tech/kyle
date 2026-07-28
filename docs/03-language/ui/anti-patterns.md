# Anti-Patrones y Lecciones Aprendidas

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Filosofía de Kyle UI

Kyle UI aprende de los errores de otros frameworks para ofrecer una experiencia superior:

1. **Menos es más** — 30 componentes nativos, no 200
2. **Tipado fuerte** — Todo verificado en compilación
3. **Composición sobre configuración** — Componentes pequeños combinables
4. **Multiplataforma real** — Misma sintaxis, rendering nativo
5. **Sin magia** — Todo explícito y predecible

---

## Anti-Patrones de Otros Frameworks

### 1. Flutter: Widget Hell

**Problema:**
```dart
// Flutter: 200+ widgets, imposible de memorizar
Container(
  padding: EdgeInsets.all(16),
  child: Column(
    children: [
      Text("Hola"),
      ElevatedButton(
        onPressed: () {},
        child: Text("Click")
      )
    ]
  )
)
```

**Solución Kyle:**
```kyx
<vstack spacing=16>
    <text value="Hola" />
    <button text="Click" click=@handler />
</vstack>
```

**Lección:** No crear widgets para todo. Usar composición con layouts básicos.

---

### 2. React: Prop Drilling

**Problema:**
```jsx
// React: Pasar props por 10 niveles
<GrandParent user={user}>
    <Parent user={user}>
        <Child user={user}>
            <GrandChild user={user} />
        </Child>
    </Parent>
</GrandParent>
```

**Solución Kyle:**
```kyx
<view>
    @(
        context UserContext:
            user: User
    )
    
    <grand_child />  # Accede directamente al contexto
</view>
```

**Lección:** Usar context para estado compartido, no prop drilling.

---

### 3. Vue: Magic Strings

**Problema:**
```vue
<!-- Vue: Strings mágicos, errores en runtime -->
<input type="email" pattern="^[^@]+@[^@]+$" />
<script>
  // Error: typo en nombre de variable
  this.emal = "test@test.com"  // No error hasta runtime
</script>
```

**Solución Kyle:**
```kyx
<text_field type=input_type.email bind=@email />
```

**Lección:** Todo tipado, errores en compilación, no runtime.

---

### 4. Angular: Verbosidad Excesiva

**Problema:**
```typescript
// Angular: Decorators, modules, services, DI...
@Component({
  selector: 'app-user',
  template: `<div>{{user.name}}</div>`
})
export class UserComponent {
  @Input() user: User;
  constructor(private service: UserService) {}
}
```

**Solución Kyle:**
```kyx
<view>
    @(
        user: User  # Prop automática
    )
    <text value=@user.name />
</view>
```

**Lección:** Minimal boilerplate, máxima productividad.

---

### 5. SwiftUI: Implicit Magic

**Problema:**
```swift
// SwiftUI: Comportamiento implícito, difícil de debuggear
var body: some View {
    List(items) { item in
        Text(item.name)
    }
    // ¿Cuándo se actualiza? ¿Por qué no se actualiza?
}
```

**Solución Kyle:**
```kyx
<view>
    @(
        items: ^{Item} = {}  # ^ = mutable, reactivo
    )
    
    <list data=@items>
        <item>
            <text value=@item.name />
        </item>
    </list>
</view>
```

**Lección:** Reactividad explícita con `^` (mutable) y `@` (reactivo).

---

### 6. CSS: Cascade Nightmares

**Problema:**
```css
/* CSS: Especificidad impredecible */
.button { color: blue; }
.container .button { color: red; }
#main .button { color: green; }
/* ¿Qué color gana? Depende del orden y especificidad */
```

**Solución Kyle:**
```kyle
style<button> Primary:
    background = color("#0066FF")
    color = color("#FFFFFF")
```

```kyx
<button style=Primary />
```

**Lección:** Sin cascade, sin selectores. Estilos tipados y locales.

---

### 7. Tailwind: Class Explosion

**Problema:**
```html
<!-- Tailwind: 50 clases para un botón -->
<button class="px-4 py-2 bg-blue-500 text-white rounded-lg 
               hover:bg-blue-600 focus:outline-none focus:ring-2 
               focus:ring-blue-500 focus:ring-offset-2 
               transition duration-200 ease-in-out">
    Click
</button>
```

**Solución Kyle:**
```kyx
<button style=Primary text="Click" />
```

**Lección:** Estilos reutilizables, no clases inline.

---

### 8. Electron: Performance Issues

**Problema:**
```javascript
// Electron: App de 500MB para un "Hello World"
// Renderiza HTML en Chromium, pesado y lento
```

**Solución Kyle:**
- Web: HTML/CSS/JS nativo (sin overhead)
- Desktop: SDL2/Skia (ligero, rápido)
- Mobile: SwiftUI/Jetpack Compose (nativo)

**Lección:** Rendering nativo por plataforma, no web embebido.

---

## Anti-Patrones a Evitar en Kyle UI

### ❌ 1. Lógica de negocio en el markup

```kyx
# Malo
<button click=@() => {
    if validate_form():
        if check_permissions():
            api.save(data)
            toast.success("Guardado")
} />
```

### ✅ Solución: Separar lógica

```kyx
<view>
    @(
        fn handle_save():
            if !validate_form(): return
            if !check_permissions(): return
            api.save(data)
            toast.success("Guardado")
    )
    
    <button text="Guardar" click=@handle_save />
</view>
```

---

### ❌ 2. Estados inconsistentes

```kyx
# Malo: loading y success pueden ser true al mismo tiempo
@(
    loading: ^bool = false
    success: ^bool = false
    error: ^str? = none
)
```

### ✅ Solución: Enum de estados

```kyx
@(
    state: ^state = state.idle
)

@match state:
    state.idle:
        <form />
    state.loading:
        <spinner />
    state.success:
        <text value="Guardado" />
    state.error(msg):
        <text value=@msg style=style(color: color("#FF0000")) />
```

---

### ❌ 3. Componentes god (hacen todo)

```kyx
# Malo: Componente de 500 líneas que hace todo
<user_dashboard 
    show_stats=true 
    show_chart=true 
    show_table=true 
    show_form=true 
/>
```

### ✅ Solución: Composición

```kyx
<view>
    <user_stats />
    <user_chart />
    <user_table />
    <user_form />
</view>
```

---

### ❌ 4. Tipado débil

```kyx
# Malo: any, dynamic
@(
    data: any  # ¿Qué es? ¿Qué campos tiene?
)
```

### ✅ Solución: Tipos explícitos

```kyx
@(
    user: User  # Tipo específico, autocompletado
)
```

---

### ❌ 5. Efectos secundarios en render

```kyx
# Malo: Modificar estado durante render
<view>
    @(
        count: ^i32 = 0
        count += 1  # ¡Se ejecuta en cada render!
    )
</view>
```

### ✅ Solución: Efectos explícitos

```kyx
<view>
    @(
        count: ^i32 = 0
        
        fn on_mounted():
            count += 1  # Solo se ejecuta una vez
    )
</view>
```

---

### ❌ 6. No manejar estados de error

```kyx
# Malo: Asumir que todo siempre funciona
<view>
    @(
        user = api.get_user()  # ¿Y si falla?
    )
    <text value=@user.name />
</view>
```

### ✅ Solución: Manejar errores

```kyx
<view>
    @(
        user: User? = none
        error: str? = none
        
        fn load_user():
            result = api.get_user()
            match result:
                ok(u): user = u
                error(e): error = e
    )
    
    @if error != none:
        <text value=@error style=style(color: color("#FF0000")) />
    @elif user != none:
        <text value=@user.name />
    @else:
        <spinner />
</view>
```

---

### ❌ 7. Accesibilidad como afterthought

```kyx
# Malo: No considerar accesibilidad
<button icon="x" click=@close />
```

### ✅ Solución: Accesibilidad desde el inicio

```kyx
<button icon="x" click=@close aria_label="Cerrar" />
```

---

### ❌ 8. No considerar multiplataforma

```kyx
# Malo: Asumir solo web
<button style=style(cursor: cursor.pointer) />
```

### ✅ Solución: Diseño universal

```kyx
<button style=Primary />  # Funciona en todas las plataformas
```

---

## Principios de Diseño

### 1. Explícito sobre implícito

```kyx
# ✅ Claro lo que hace
<modal open=@show_modal on_close=@close title="Confirmar">
```

### 2. Composición sobre herencia

```kyx
# ✅ Componentes pequeños combinables
<vstack spacing=16>
    <header />
    <content />
    <footer />
</vstack>
```

### 3. Tipado sobre strings

```kyx
# ✅ Enum tipado
<input type=input_type.email />

# ❌ String mágico
<input type="email" />
```

### 4. snake_case consistente

```kyx
# ✅ Consistente
<text_field 
    border_radius=8 
    on_focus=@handle_focus 
    aria_label="Email" 
/>

# ❌ Mezcla de convenciones
<textField 
    borderRadius={8} 
    onFocus={handleFocus} 
    ariaLabel="Email" 
/>
```

---

## Referencias

- [Arquitectura multiplataforma](architecture.md)
- [Componentes nativos](components/README.md)
- [Sistema de estilos](style-system.md)
