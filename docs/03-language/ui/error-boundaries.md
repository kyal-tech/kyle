# Error Boundaries

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Routing (guardias on_error)

---

## 1. Qué es un Error Boundary

Un error boundary es un componente que **captura errores** en su subárbol de
componentes hijos y muestra una **UI de fallback** en lugar de dejar que toda
la app se rompa.

```
┌──────────────────────────────┐
│   ErrorBoundary              │
│   ┌────────────────────────┐ │
│   │   Componente hijo      │─┼── ❌ Error!
│   │   (se rompe)           │ │
│   └────────────────────────┘ │
│                              │
│   ┌────────────────────────┐ │
│   │   UI de fallback       │ │
│   │   "Algo salió mal"     │ │
│   └────────────────────────┘ │
└──────────────────────────────┘
```

### ¿Qué errores captura?

| Tipo | Captura |
|------|:-------:|
| Render error | ✅ |
| Event handler error | ✅ |
| Lifecycle hook error | ✅ |
| Async error (Promise) | ✅ (con `try`/`catch` en el handler) |
| Error en contexto | ✅ |
| Errores en componentes hijos | ✅ (recursivo) |

### ¿Qué NO captura?

- Errores en event listeners del target nativo (ej: `addEventListener` directo)
- Errores en timers (`setTimeout`/`setInterval`)
- Errores en streams asíncronos no manejados

---

## 2. Sintaxis

### 2.1 ErrorBoundary component

```kyx
<error_boundary>
    <child_component />
</error_boundary>
```

### 2.2 Con UI de fallback

```kyx
<error_boundary>
    @(    fallback: ^view = error_view())
    <profile_card user_id=@user_id />
</error_boundary>
```

```kyx
# error_view.kyx
@(
    on_retry: ^&(fn ())
)
<view>
    <vstack alignment=alignment.center padding=32>
        <icon name="alert" size=48 color=@theme.error />
        <text value="Algo salió mal" />
        <text value="Intenta de nuevo más tarde" />
        <button text="Reintentar" click=@on_retry />
    </vstack>
</view>
```

### 2.3 Inline fallback

```kyx
<error_boundary>
    <slot name="fallback">
        <view padding=16>
            <text value="Error al cargar el componente" />
        </view>
    </slot>
    <profile_card user_id=@user_id />
</error_boundary>
```

---

## 3. Ciclo de Vida del Error

```
Error en componente hijo
    │
    ▼
on_error(caught_error) se dispara en el boundary
    │
    ├── Si hay on_error handler → ejecuta lógica personalizada
    │       │
    │       ├── log_error(error) → envía a servidor
    │       ├── set_error_state(error) → muestra fallback
    │       └── retry() → reintenta renderizar
    │
    └── Si NO hay on_error handler → muestra fallback por defecto
            │
            └── "Algo salió mal" + botón Reintentar
```

---

## 4. API Completa

### 4.1 Atributos

| Atributo | Tipo | Descripción |
|----------|------|-------------|
| `on_error` | `fn(Error) → RouteResult?` | Handler personalizado |
| `fallback` | `View` | UI de reemplazo |
| `max_retries` | `i32 = 3` | Reintentos automáticos |
| `reset_on_change` | `{str}` | Resetear error si cambian ciertos props |
| `log_to` | `str?` | URL de logging remoto |

### 4.2 Eventos

```kyx
@(
    fn handle_error(error: Error):
        log_error_to_server(error)
        show_snackbar("Error reportado")
)

<error_boundary on_error=@handle_error>
    <child_component />
</error_boundary>
```

### 4.3 Reintento automático

```kyx
<error_boundary max_retries=3>
    <weather_widget city=@city />
</error_boundary>
```

Si falla, reintenta hasta 3 veces con backoff exponencial (1s, 2s, 4s).

### 4.4 Reset en cambio de props

```kyx
<error_boundary reset_on_change={"user_id", "section"}>
    <profile_section user_id=@user_id section=@section />
</error_boundary>
```

Si `user_id` o `section` cambian, el boundary se resetea automáticamente
(intenta renderizar de nuevo aunque esté en estado de error).

---

## 5. Error Boundaries Anidados

Cada boundary captura errores de su propio subárbol. Si un boundary falla,
el error lo captura el boundary padre:

```kyx
<error_boundary>                    # ← captura errores de sidebar + content
    <sidebar>
        <error_boundary>            # ← captura errores de nav_items
            <nav_items />
        </error_boundary>
    </sidebar>

    <content>
        <error_boundary>            # ← captura errores de profile
            <profile_section />
        </error_boundary>
    </content>
</error_boundary>
```

Si `nav_items` falla → el boundary interno muestra fallback del nav.
El sidebar y el content siguen funcionando.
Si el boundary interno falla → lo captura el boundary externo.

---

## 6. Estrategias de Fallback

### 6.1 Fallback minimalista

```kyx
<error_boundary>
    <slot name="fallback">
        <text value="❌" />
    </slot>
    <heavy_component />
</error_boundary>
```

### 6.2 Fallback con acción

```kyx
<error_boundary>
    @(
        fallback: ^View = error_retry_view(fn():
            # el boundary se resetea
        )
    )
    <data_table />
</error_boundary>
```

### 6.3 Fallback específico por error

```kyx
@(
    fn handle_error(error: Error):
        if error.type == ErrorType.Network:
            fallback = network_error_view()
        elif error.type == ErrorType.Permission:
            fallback = permission_error_view()
        else:
            fallback = generic_error_view()
)

<error_boundary on_error=@handle_error>
    <dashboard />
</error_boundary>
```

### 6.4 Fallback con reporte

```kyx
<error_boundary log_to="/api/log-error">
    <profile_section />
</error_boundary>
```

El boundary captura el error, lo envía al servidor, y muestra el fallback.

---

## 7. Error Recovery

### 7.1 Reintento manual

```kyx
@(
    boundary_ref: ^ErrorBoundaryRef

    fn retry():
        boundary_ref.reset()
)

<error_boundary ref=@boundary_ref>
    <profile_section />
</error_boundary>

<button text="Reintentar" click=@retry />
```

### 7.2 Reintento con estado degradado

```kyx
@(
    fn handle_error(error: Error):
        # Mostrar versión simplificada en lugar de fallback
        show_simplified = true
)

<error_boundary on_error=@handle_error>
    @if show_simplified:
        <profile_summary user_id=@user_id />
    @else:
        <profile_full user_id=@user_id />
</error_boundary>
```

### 7.3 Cache de último estado exitoso

El boundary puede mantener el último estado exitoso y mostrarlo hasta que
se recupere:

```kyx
<error_boundary cache_last_success=true>
    <weather_widget />
</error_boundary>
```

Si el widget falla al actualizar, muestra los últimos datos válidos en vez
de un mensaje de error.

---

## 8. Integración con Routing

Error boundaries en rutas:

```kyx
# app.kyx
<router>
    <error_boundary>
        <home_view />
    </error_boundary>
    <error_boundary>
        <route path="/dashboard" component=dashboard />
    </error_boundary>
</router>
```

Cada ruta tiene su propio boundary. Si el dashboard falla, el home sigue
funcionando.

Ver [routing.md](routing.md) sección de guardias `on_error`.

---

## 9. Compilación por Target

### 9.1 Web

```javascript
// Generado automáticamente
class ErrorBoundary {
    constructor(fallback, onError, maxRetries) {
        this.state = 'error';  // o 'success'
        this.fallback = fallback;
        this.onError = onError;
        this.maxRetries = maxRetries;
    }

    tryRender(children) {
        try {
            return this.renderChildren(children);
        } catch (error) {
            this.state = 'error';
            if (this.onError) this.onError(error);
            return this.renderFallback();
        }
    }

    reset() {
        this.state = 'success';
        this.retryCount = 0;
        this.forceUpdate();
    }
}
```

### 9.2 Desktop

```kyle
# Generado automáticamente
final class error_boundary:
    state: ^boundary_state = boundary_state.success
    children: ^View
    fallback: View
    max_retries: i32 = 3
    retry_count: ^i32 = 0

    fn render(this) View:
        if this.state == BoundaryState.Error:
            this.render_fallback()
        else:
            try:
                this.render_children()
            catch error:
                this.state = BoundaryState.Error
                this.on_error(error)
                this.render_fallback()
```

---

## 10. Buenas Prácticas

| Práctica | Descripción |
|----------|-------------|
| **Boundaries por sección** | No uno solo para toda la app — cada sección independiente |
| **Fallback significativo** | No solo "error", da contexto y acción |
| **Logging** | Siempre loguear errores, no solo atraparlos |
| **Reset on change** | Si el error es por datos inválidos, resetear al cambiar props |
| **Límite de reintentos** | No reintentar infinitamente — usar `max_retries` |
| **Boundary en rutas** | Cada ruta debe tener su propio boundary |

---

## 11. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Guardias on_error en rutas
