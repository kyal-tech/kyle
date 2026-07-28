# Ciclo de Vida de Componentes

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Introducción

Cada componente en Kyle UI pasa por un ciclo de vida específico desde que se crea hasta que se destruye. Los hooks de ciclo de vida te permiten ejecutar código en momentos específicos de este ciclo.

---

## Diagrama del Ciclo de Vida

```
┌─────────────────────────────────────────────────────────────┐
│                    CREACIÓN                                  │
│  1. Constructor llamado                                      │
│  2. Props inicializadas                                      │
│  3. Estado inicial establecido                               │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    MONTAJE                                   │
│  4. on_created() - Antes de montar en DOM                    │
│  5. Render inicial                                           │
│  6. on_mounted() - Después de montar en DOM                  │
│  7. Componente visible e interactivo                         │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    ACTUALIZACIÓN                             │
│  8. Props o estado cambian                                   │
│  9. on_before_update() - Antes de re-renderizar              │
│ 10. Re-renderizado                                           │
│ 11. on_updated() - Después de re-renderizar                  │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                    DESMONTAJE                                │
│ 12. on_before_unmount() - Antes de remover                   │
│ 13. Removido del DOM                                         │
│ 14. on_unmounted() - Después de remover                      │
│ 15. Limpieza de recursos                                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Hooks Disponibles

### 1. `on_created()`

Se ejecuta **inmediatamente después** de que el componente es creado, pero **antes** de que se monte en el DOM.

**Casos de uso:**
- Inicializar variables de estado
- Configurar listeners
- Validar props

```kyx
<view>
    @(
        user: User? = none
        
        fn on_created():
            # Se ejecuta antes de montar
            user = load_user_from_cache()
            if user == none:
                user = User.default()
    )
    
    <text value=@user.name />
</view>
```

**Multiplataforma:**
- **Web:** Antes de `componentDidMount` / `onMounted`
- **iOS:** Antes de `onAppear`
- **Android:** Antes de `onAppear` en Composable

---

### 2. `on_mounted()`

Se ejecuta **después** de que el componente ha sido montado en el DOM y es visible.

**Casos de uso:**
- Cargar datos de API
- Iniciar animaciones
- Suscribirse a eventos globales
- Medir dimensiones del DOM

```kyx
<view>
    @(
        products: {Product} = {}
        loading: bool = true
        
        fn on_mounted():
            # Componente ya es visible
            products = await api.get_products()
            loading = false
    )
    
    @if loading:
        <spinner />
    @else:
        <list data=@products>
            <item>
                <text value=@item.name />
            </item>
        </list>
</view>
```

**Multiplataforma:**
- **Web:** Equivalente a `componentDidMount` / `onMounted`
- **iOS:** Equivalente a `onAppear`
- **Android:** Equivalente a `LaunchedEffect`

---

### 3. `on_before_update()`

Se ejecuta **antes** de que el componente se re-renderice debido a cambios en props o estado.

**Casos de uso:**
- Calcular valores derivados
- Validar cambios antes de aplicar
- Optimizar rendimiento

```kyx
<view>
    @(
        items: {Item} = {}
        total: f32 = 0.0
        
        fn on_before_update():
            # Se ejecuta antes de cada re-render
            total = items.fold(0.0, fn(acc, item): acc + item.price)
    )
    
    <text value=@"Total: $" + total.to_str() />
</view>
```

**Multiplataforma:**
- **Web:** Equivalente a `componentWillUpdate` / `onBeforeUpdate`
- **iOS:** No tiene equivalente directo (usar `onChange`)
- **Android:** No tiene equivalente directo (usar `derivedStateOf`)

---

### 4. `on_updated()`

Se ejecuta **después** de que el componente ha sido re-renderizado.

**Casos de uso:**
- Actualizar librerías de terceros
- Medir dimensiones después del render
- Sincronizar con sistemas externos

```kyx
<view>
    @(
        chart_data: {DataPoint} = {}
        
        fn on_updated():
            # Actualizar gráfico después de re-render
            chart.update(chart_data)
    )
    
    <chart data=@chart_data />
</view>
```

**Multiplataforma:**
- **Web:** Equivalente a `componentDidUpdate` / `onUpdated`
- **iOS:** Equivalente a `onChange`
- **Android:** Equivalente a `LaunchedEffect(key)`

---

### 5. `on_before_unmount()`

Se ejecuta **antes** de que el componente sea removido del DOM.

**Casos de uso:**
- Guardar estado antes de destruir
- Confirmar navegación
- Cancelar operaciones pendientes

```kyx
<view>
    @(
        form_data: FormData = FormData()
        
        fn on_before_unmount():
            # Guardar borrador antes de salir
            if form_data.has_changes():
                save_draft(form_data)
    )
    
    <form model=@form_data>
        <!-- campos del formulario -->
    </form>
</view>
```

**Multiplataforma:**
- **Web:** Equivalente a `componentWillUnmount` / `onBeforeUnmount`
- **iOS:** Equivalente a `onDisappear`
- **Android:** Equivalente a `DisposableEffect`

---

### 6. `on_unmounted()`

Se ejecuta **después** de que el componente ha sido removido del DOM.

**Casos de uso:**
- Limpiar listeners
- Cancelar timers
- Liberar recursos
- Cancelar suscripciones

```kyx
<view>
    @(
        timer_id: i32 = 0
        ws_connection: WebSocket? = none
        
        fn on_mounted():
            # Iniciar timer
            timer_id = set_interval(fn(): update_clock(), 1000)
            
            # Conectar WebSocket
            ws_connection = WebSocket.connect("wss://api.example.com")
        
        fn on_unmounted():
            # Limpiar timer
            clear_interval(timer_id)
            
            # Cerrar WebSocket
            if ws_connection != none:
                ws_connection.close()
    )
    
    <text value="Componente activo" />
</view>
```

**Multiplataforma:**
- **Web:** Equivalente a `componentWillUnmount` / `onUnmounted`
- **iOS:** Equivalente a `onDisappear` (cleanup)
- **Android:** Equivalente a `DisposableEffect` (cleanup)

---

## Orden de Ejecución

### Montaje Inicial

```
1. on_created()
2. Render inicial
3. on_mounted()
```

### Actualización

```
1. on_before_update()
2. Re-renderizado
3. on_updated()
```

### Desmontaje

```
1. on_before_unmount()
2. Remover del DOM
3. on_unmounted()
```

---

## Ejemplos Completos

### Carga de Datos con Loading State

```kyx
<view>
    @(
        users: {User} = {}
        loading: bool = true
        error: str? = none
        
        fn on_mounted():
            # Cargar datos cuando el componente se monta
            try:
                users = await api.get_users()
            catch e:
                error = e.message
            finally:
                loading = false
    )
    
    @if loading:
        <spinner />
    @elif error != none:
        <alert type=error message=@error />
    @else:
        <list data=@users>
            <item>
                <text value=@item.name />
            </item>
        </list>
</view>
```

### Suscripción a Eventos Globales

```kyx
<view>
    @(
        window_width: f32 = 0.0
        
        fn on_mounted():
            # Suscribirse a resize
            window.addEventListener("resize", handle_resize)
            window_width = window.innerWidth
        
        fn on_unmounted():
            # Limpiar listener
            window.removeEventListener("resize", handle_resize)
        
        fn handle_resize():
            window_width = window.innerWidth
    )
    
    <text value=@"Ancho: " + window_width.to_str() + "px" />
</view>
```

### Timer con Limpieza

```kyx
<view>
    @(
        seconds: i32 = 0
        timer_id: i32 = 0
        running: bool = false
        
        fn start():
            running = true
            timer_id = set_interval(fn():
                seconds += 1
            , 1000)
        
        fn stop():
            running = false
            clear_interval(timer_id)
        
        fn on_unmounted():
            # Asegurar limpieza
            if running:
                stop()
    )
    
    <vstack>
        <text value=@"Tiempo: " + seconds.to_str() + "s" />
        <hstack>
            <button text="Iniciar" click=@start disabled=@running />
            <button text="Detener" click=@stop disabled=@!running />
        </hstack>
    </vstack>
</view>
```

### Guardado Automático de Borrador

```kyx
<view>
    @(
        content: str = ""
        last_saved: datetime? = none
        
        fn on_created():
            # Cargar borrador guardado
            draft = local_storage.get("draft")
            if draft != none:
                content = draft
        
        fn on_before_unmount():
            # Guardar borrador antes de salir
            if content != "":
                local_storage.set("draft", content)
        
        fn on_content_change():
            # Auto-guardar cada 30 segundos
            last_saved = datetime.now()
            local_storage.set("draft", content)
    )
    
    <text_area bind=@content />
    @if last_saved != none:
        <text value=@"Último guardado: " + last_saved.to_str() />
</view>
```

---

## Comparación con Otros Frameworks

| Kyle UI | React | Vue | SwiftUI | Jetpack Compose |
|---------|-------|-----|---------|-----------------|
| `on_created()` | Constructor | `created()` | `init()` | Constructor |
| `on_mounted()` | `useEffect(() => {}, [])` | `mounted()` | `onAppear` | `LaunchedEffect` |
| `on_before_update()` | `useEffect` (sin deps) | `beforeUpdate()` | - | - |
| `on_updated()` | `useEffect(() => {}, [deps])` | `updated()` | `onChange` | `LaunchedEffect(key)` |
| `on_before_unmount()` | - | `beforeUnmount()` | `onDisappear` | `DisposableEffect` |
| `on_unmounted()` | `useEffect` cleanup | `unmounted()` | `onDisappear` cleanup | `DisposableEffect` cleanup |

---

## Mejores Prácticas

### ✅ Hacer

1. **Limpiar recursos en `on_unmounted()`**
```kyx
fn on_unmounted():
    clear_interval(timer_id)
    remove_event_listener()
```

2. **Cargar datos en `on_mounted()`**
```kyx
fn on_mounted():
    data = await api.fetch_data()
```

3. **Validar en `on_created()`**
```kyx
fn on_created():
    if props.user_id == none:
        raise Error("user_id requerido")
```

### ❌ No Hacer

1. **No manipular DOM en `on_created()`**
```kyx
# ❌ Mal
fn on_created():
    element.focus()  # Elemento aún no existe

# ✅ Bien
fn on_mounted():
    element.focus()  # Elemento ya existe
```

2. **No hacer operaciones pesadas en `on_before_update()`**
```kyx
# ❌ Mal
fn on_before_update():
    heavy_computation()  # Se ejecuta en cada update

# ✅ Bien
fn on_before_update():
    cached_value = compute_if_needed()
```

3. **No olvidar limpiar listeners**
```kyx
# ❌ Mal
fn on_mounted():
    window.addEventListener("resize", handler)
# Falta cleanup

# ✅ Bien
fn on_mounted():
    window.addEventListener("resize", handler)

fn on_unmounted():
    window.removeEventListener("resize", handler)
```

---

## SSR y Ciclo de Vida

En Server-Side Rendering, algunos hooks no se ejecutan:

| Hook | SSR | CSR |
|------|-----|-----|
| `on_created()` | ✅ | ✅ |
| `on_mounted()` | ❌ | ✅ |
| `on_before_update()` | ❌ | ✅ |
| `on_updated()` | ❌ | ✅ |
| `on_before_unmount()` | ❌ | ✅ |
| `on_unmounted()` | ❌ | ✅ |

**Implicación:** Solo usa `on_created()` para lógica que debe ejecutarse en ambos lados.

---

## Referencias

- [State & Events](state-events.md)
- [Composition](composition.md)
- [SSR](ssr.md)
- [Testing](testing.md)
