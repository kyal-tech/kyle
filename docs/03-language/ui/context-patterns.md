# Patrones de Contexto

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [state-events.md](state-events.md) — Estado y eventos (context básico)
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [routing.md](routing.md) — Context en rutas

---

## 1. Qué es Context

Context permite compartir estado entre componentes **sin pasar props manualmente**
por cada nivel del árbol. Es la solución al "prop drilling":

```
Sin Context:         Con Context:
A ──prop──→ B        A ──context──→ B
             ↘                        ↘
               C ──prop──→ D            C ──context──→ D
```

---

## 2. Context vs Props

| Aspecto | Props | Context |
|---------|:-----:|:-------:|
| Ámbito | Padre → hijo directo | Todo el subárbol |
| Tipado | ✅ Fuerte | ✅ Fuerte |
| Reactividad | ✅ Automática | ✅ Automática |
| Multi-nivel | Prop drilling | Acceso directo |
| Overhead | Mínimo | Mínimo (solo referencias) |

**Regla:** Props para datos específicos del componente. Context para datos
globales o compartidos.

---

## 3. Declaración de Context

### 3.1 Context básico

```kyx
@(
    context ThemeContext:
        theme: Theme = DarkTheme
        primary_color: color = theme.primary

    # Componentes hijos acceden con use_context()
    ctx = use_context(ThemeContext)
)
```

### 3.2 Context con métodos

```kyx
@(
    context AuthContext:
        user: User?
        token: str?
        is_authenticated: bool = false

        fn login(email: str, password: str):
            # lógica de login
            user = result.user
            token = result.token
            is_authenticated = true

        fn logout():
            user = None
            token = None
            is_authenticated = false
)
```

### 3.3 Context con signals

```kyx
@(
    context NotificationContext:
        notifications: ^Queue{Notification} = Queue{Notification}()

        fn notify(msg: str, type: NotificationType):
            notifications.push(Notification(msg, type))
            # Los componentes que usen este context se actualizan

            # Auto-dismiss después de 3s
            schedule(fn():
                notifications.pop()
            , 3000)
)
```

---

## 4. Consumo de Context

### 4.1 use_context

```kyx
@(
    auth = use_context(AuthContext)
)
<view>
    @if auth.is_authenticated:
        <text value="Bienvenido, " + auth.user.name />
        <button text="Cerrar sesión" click=@auth.logout />
    @else:
        <button text="Iniciar sesión" click=@show_login />
</view>
```

### 4.2 Context como prop implícito

Cualquier componente dentro del árbol del provider puede accederlo sin
props intermedios:

```kyx
# app.kyx — provee ThemeContext
@(
    context ThemeContext: ...
)
<app>
    <sidebar>
        <nav_item>      # ← usa use_context(ThemeContext) directamente
            <icon />
            <text />
        </nav_item>
    </sidebar>
</app>
```

### 4.3 use_context con default

```kyx
@(
    # Si no hay provider, usa el default
    theme = use_context(ThemeContext, default: LightTheme)
)
```

---

## 5. Context Provider Anidados

### 5.1 Override de context

```kyx
@(
    # Provee ThemeContext global
    context ThemeContext:
        theme = LightTheme
)
<app>
    <home_page />

    <section style=@dark_theme_style>
        @(
            # Override del context para esta sección
            context ThemeContext:
                theme = DarkTheme
        )
        <dark_card />   # ← usa DarkTheme
    </section>
</app>
```

### 5.2 Multi-context

```kyx
@(
    context AuthContext: ...
    context ThemeContext: ...
    context LocaleContext: ...
)
<app>
    <home_page />
</app>

# home_page usa los 3 context
@(
    auth = use_context(AuthContext)
    theme = use_context(ThemeContext)
    locale = use_context(LocaleContext)
)
```

---

## 6. Patrones Avanzados

### 6.1 Context Factory

```kyle
fn create_api_context(base_url: str) -> type:
    context ApiContext:
        url: str = base_url
        headers: {str: str} = {}

        fn get<T>(path: str) T:
            fetch(url + path, headers: headers)
```

```kyx
@(
    context ApiContext = create_api_context("https://api.example.com")
)
```

### 6.2 Context con Reducer (como useReducer)

```kyx
@(
    context CartContext:
        state: CartState = CartState(items: {})
        dispatch: fn(action: CartAction)

    fn cart_reducer(state: ^CartState, action: CartAction):
        match action.type:
            "ADD_ITEM":
                state.items.push(action.payload)
            "REMOVE_ITEM":
                state.items = state.items.filter(fn(i): i.id != action.payload.id)
            "CLEAR":
                state.items = {}

    dispatch = fn(action):
        cart_reducer(state, action)
)
```

### 6.3 Context Selector

Para evitar re-renders innecesarios, los componentes pueden suscribirse
a solo una parte del context:

```kyx
@(
    # Solo se re-renderiza si cambia auth.user
    user = use_context(AuthContext, select: fn(ctx): ctx.user)
)
```

### 6.4 Context Temporal (Scoped)

Context que solo existe dentro de un bloque:

```kyx
@(
    # FormContext existe solo dentro de este form
    context FormContext:
        values: {str: str} = {}
        errors: {str: str} = {}
        is_submitting: bool = false

    fn handle_submit():
        # validar y enviar
)
<form submit=@handle_submit>
    <text_field name="email" />     # usa FormContext
    <text_field name="password" />
    <button text="Enviar" />
</form>
```

---

## 7. Reactividad del Context

### 7.1 Cambios en cascada

```
ctx.user = new_user
    │
    ▼
Runtime detecta cambio en AuthContext.user
    │
    ▼
Busca todos los use_context(AuthContext) en el árbol
    │
    ▼
Re-renderiza solo los componentes que usan AuthContext
    │
    ▼
Si esos componentes tienen sub-árbol, se renderizan normalmente
```

### 7.2 Optimización con selectores

Sin selector → el componente se re-renderiza si **cualquier** campo del context
cambia. Con selector → solo si cambia el campo seleccionado.

```kyx
# Sin selector: se re-renderiza aunque cambie theme (no usa theme)
user = use_context(AppContext)

# Con selector: solo se re-renderiza si cambia user
user = use_context(AppContext, select: fn(ctx): ctx.user)
```

---

## 8. Testing con Context

```kyle
test("Component with mock context"):
    render("""
        @(auth = use_context(AuthContext))
        <text value=auth.user.name />
    """, context_providers: {
        AuthContext: {
            user: User(name: "Test"),
            token: "mock",
        },
    })
```

Ver [testing.md](testing.md) para más detalles.

---

## 9. Compilación por Target

### 9.1 Web

```javascript
// Generado automáticamente
const ContextSystem = {
    providers: new Map(),
    consumers: new Map(),

    provide(key, value) {
        this.providers.set(key, value);
        this.notify(key);
    },

    use(key, selector) {
        const value = this.providers.get(key);
        if (selector) return selector(value);
        return value;
    },

    notify(key) {
        const consumers = this.consumers.get(key) || [];
        for (const fn of consumers) fn();
    }
};
```

### 9.2 Desktop

```kyle
# Generado automáticamente
final class context_manager:
    providers: ^{context_key: any} = {}

    fn provide<T>(this, key: ContextKey, value: T):
        this.providers.set(key, value)
        this.notify(key)

    fn use<T>(this, key: ContextKey, select: fn(any) T? = None) T:
        value = this.providers.get(key)
        if select != None:
            select(value)
        else:
            value
```

---

## 10. Buenas Prácticas

| Práctica | Descripción |
|----------|-------------|
| **Context específico** | No un solo `GlobalContext` — separar por dominio |
| **Selectores** | Usar `select` para evitar re-renders innecesarios |
| **Context anidado** | Override para secciones específicas (temas, locales) |
| **No abusar** | Props siguen siendo mejores para datos locales |
| **Context inmutable** | No mutar el context directamente — reemplazar |
| **Documentar** | Cada contexto debe tener un propósito claro |

---

## 11. Referencias

- [state-events.md](state-events.md) — Estado y eventos (context básico)
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [testing.md](testing.md) — Testing con contexto mockeado
