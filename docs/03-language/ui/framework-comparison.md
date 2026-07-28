# Comparativa con Frameworks Nativos

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Introducción

Este documento compara Kyle UI con los frameworks nativos más populares de cada plataforma, extrayendo lo mejor de cada uno y mostrando cómo Kyle UI implementa o supera estas características.

---

## Resumen Ejecutivo

| Característica | Kyle UI | React | SwiftUI | Jetpack Compose | Flutter |
|----------------|---------|-------|---------|-----------------|---------|
| **Tipado** | ✅ Fuerte (Kyle) | ❌ Débil (JS) | ✅ Fuerte (Swift) | ✅ Fuerte (Kotlin) | ✅ Fuerte (Dart) |
| **Multiplataforma** | ✅ Nativo | ❌ Web only | ❌ Apple only | ❌ Android only | ✅ Propio |
| **Curva de aprendizaje** | 🟢 Baja | 🟡 Media | 🟡 Media | 🟡 Media | 🟡 Media |
| **Rendimiento** | ✅ Nativo | 🟡 Virtual DOM | ✅ Nativo | ✅ Nativo | 🟡 Bridge |
| **Tamaño bundle** | 🟢 <1MB | 🟡 ~100KB | ✅ Nativo | ✅ Nativo | 🔴 ~15MB |
| **Hot reload** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **DevTools** | 🟡 Básico | ✅ Excelentes | ✅ Xcode | ✅ Android Studio | ✅ Flutter DevTools |

---

## 1. Web: React / Vue / Svelte

### Lo Mejor de React

#### 1.1 Hooks

**React:**
```javascript
function Counter() {
  const [count, setCount] = useState(0);
  
  useEffect(() => {
    document.title = `Count: ${count}`;
  }, [count]);
  
  return <button onClick={() => setCount(count + 1)}>{count}</button>;
}
```

**Kyle UI:**
```kyx
<view>
    @(
        count: ^i32 = 0
        
        fn on_updated():
            # Equivalente a useEffect
            set_title(@"Count: " + count.to_str())
    )
    
    <button text=@count.to_str() click=@() => count += 1 />
</view>
```

**Ventaja Kyle UI:**
- ✅ Sin array de dependencias (automático)
- ✅ Tipado fuerte (no `any`)
- ✅ Sintaxis más limpia

#### 1.2 Component Composition

**React:**
```javascript
function Layout({ children }) {
  return (
    <div className="layout">
      <Header />
      <main>{children}</main>
      <Footer />
    </div>
  );
}

function App() {
  return (
    <Layout>
      <Content />
    </Layout>
  );
}
```

**Kyle UI:**
```kyx
# layout.kyx
<view>
    <app_bar title="Mi App" />
    <main>
        <slot />
    </main>
    <footer>
        <text value="© 2026" />
    </footer>
</view>

# app.kyx
<layout>
    <content />
</layout>
```

**Ventaja Kyle UI:**
- ✅ `<slot />` nativo (no props children)
- ✅ Separación clara de estructura

#### 1.3 State Management

**React (Redux):**
```javascript
// actions.js
export const increment = () => ({ type: 'INCREMENT' });

// reducer.js
export const counterReducer = (state = 0, action) => {
  switch (action.type) {
    case 'INCREMENT': return state + 1;
    default: return state;
  }
};

// component.js
const count = useSelector(state => state.count);
const dispatch = useDispatch();
dispatch(increment());
```

**Kyle UI (Context):**
```kyx
# counter_context.ky
context CounterContext:
    count: i32 = 0
    
    fn increment():
        count += 1

# component.kyx
<view>
    @(
        counter = use_context(CounterContext)
    )
    
    <text value=@counter.count.to_str() />
    <button text="+" click=@counter.increment />
</view>
```

**Ventaja Kyle UI:**
- ✅ Sin boilerplate (actions, reducers)
- ✅ Tipado fuerte
- ✅ Más intuitivo

---

### Lo Mejor de Vue

#### 2.1 Two-Way Binding

**Vue:**
```vue
<template>
  <input v-model="name" />
  <p>Hello, {{ name }}</p>
</template>

<script>
export default {
  data() {
    return { name: '' };
  }
};
</script>
```

**Kyle UI:**
```kyx
<view>
    @(
        name: ^str = ""
    )
    
    <text_field bind=@name />
    <text value=@"Hello, " + name />
</view>
```

**Ventaja Kyle UI:**
- ✅ Misma sintaxis `bind=`
- ✅ Tipado fuerte
- ✅ Sin directivas especiales

#### 2.2 Computed Properties

**Vue:**
```vue
<template>
  <p>{{ fullName }}</p>
</template>

<script>
export default {
  data() {
    return { firstName: 'John', lastName: 'Doe' };
  },
  computed: {
    fullName() {
      return `${this.firstName} ${this.lastName}`;
    }
  }
};
</script>
```

**Kyle UI:**
```kyx
<view>
    @(
        first_name: str = "John"
        last_name: str = "Doe"
        
        fn full_name() str:
            return first_name + " " + last_name
    )
    
    <text value=@full_name() />
</view>
```

**Ventaja Kyle UI:**
- ✅ Funciones normales (no magia)
- ✅ Tipado fuerte
- ✅ Más explícito

---

### Lo Mejor de Svelte

#### 3.1 Reactividad Automática

**Svelte:**
```svelte
<script>
  let count = 0;
  
  $: doubled = count * 2;
</script>

<button on:click={() => count++}>
  {count} (doubled: {doubled})
</button>
```

**Kyle UI:**
```kyx
<view>
    @(
        count: ^i32 = 0
        
        fn doubled() i32:
            return count * 2
    )
    
    <button text=@(count.to_str() + " (doubled: " + doubled().to_str() + ")") 
            click=@() => count += 1 />
</view>
```

**Ventaja Kyle UI:**
- ✅ Sin sintaxis especial (`$:`)
- ✅ Funciones normales
- ✅ Tipado fuerte

---

## 2. iOS: SwiftUI

### Lo Mejor de SwiftUI

#### 4.1 Declarative Syntax

**SwiftUI:**
```swift
struct ContentView: View {
    @State private var count = 0
    
    var body: some View {
        VStack {
            Text("Count: \(count)")
            Button("Increment") {
                count += 1
            }
        }
    }
}
```

**Kyle UI:**
```kyx
<view>
    @(
        count: ^i32 = 0
    )
    
    <vstack>
        <text value=@"Count: " + count.to_str() />
        <button text="Increment" click=@() => count += 1 />
    </vstack>
</view>
```

**Ventaja Kyle UI:**
- ✅ Sintaxis más limpia (no `@State`, `some View`)
- ✅ Multiplataforma (no solo iOS)
- ✅ Sin type erasure

#### 4.2 Property Wrappers

**SwiftUI:**
```swift
struct ProfileView: View {
    @Binding var userName: String
    @StateObject var viewModel = ProfileViewModel()
    @Environment(\.colorScheme) var colorScheme
    
    var body: some View {
        Text(userName)
            .foregroundColor(colorScheme == .dark ? .white : .black)
    }
}
```

**Kyle UI:**
```kyx
<view>
    @(
        # Props (equivalente a @Binding)
        user_name: str
        
        # Estado interno (equivalente a @StateObject)
        _view_model: ProfileViewModel = ProfileViewModel()
        
        # Context (equivalente a @Environment)
        color_scheme = use_context(ThemeContext).color_scheme
    )
    
    <text value=@user_name 
          style=@if color_scheme == color_scheme.dark:
              style(color: color("#FFFFFF"))
          @else:
              style(color: color("#000000"))
    />
</view>
```

**Ventaja Kyle UI:**
- ✅ Sin property wrappers (más simple)
- ✅ Convención sobre configuración (`_` prefix)
- ✅ Context explícito

#### 4.3 Navigation

**SwiftUI:**
```swift
NavigationView {
    List(items) { item in
        NavigationLink(destination: DetailView(item: item)) {
            Text(item.name)
        }
    }
    .navigationTitle("Items")
}
```

**Kyle UI:**
```kyx
<app>
    <router>
        <route path="/" component=ListView />
        <route path="/item/{id}" component=DetailView />
    </router>
</app>

# ListView.kyx
<view>
    <app_bar title="Items" />
    <list data=@items>
        <item>
            <link href=@"/item/" + item.id.to_str()>
                <text value=@item.name />
            </link>
        </item>
    </list>
</view>
```

**Ventaja Kyle UI:**
- ✅ Routing centralizado (no anidado)
- ✅ URL-based (deep linking nativo)
- ✅ Multiplataforma

---

## 3. Android: Jetpack Compose

### Lo Mejor de Jetpack Compose

#### 5.1 State Hoisting

**Jetpack Compose:**
```kotlin
@Composable
fun MyScreen() {
    var name by remember { mutableStateOf("") }
    
    NameInput(
        name = name,
        onNameChange = { name = it }
    )
}

@Composable
fun NameInput(name: String, onNameChange: (String) -> Unit) {
    TextField(
        value = name,
        onValueChange = onNameChange
    )
}
```

**Kyle UI:**
```kyx
# MyScreen.kyx
<view>
    @(
        name: ^str = ""
    )
    
    <name_input bind=@name />
</view>

# NameInput.kyx
<view>
    @(
        # Props con bind
        bind: ^str
    )
    
    <text_field bind=@bind />
</view>
```

**Ventaja Kyle UI:**
- ✅ `bind=` nativo (no callbacks manuales)
- ✅ Menos boilerplate
- ✅ Tipado fuerte

#### 5.2 Side Effects

**Jetpack Compose:**
```kotlin
@Composable
fun MyScreen() {
    var data by remember { mutableStateOf(emptyList<Item>()) }
    
    LaunchedEffect(Unit) {
        data = api.fetchItems()
    }
    
    DisposableEffect(Unit) {
        val listener = registerListener()
        onDispose {
            listener.unregister()
        }
    }
}
```

**Kyle UI:**
```kyx
<view>
    @(
        data: ^{Item} = {}
        
        fn on_mounted():
            data = await api.fetch_items()
        
        fn on_unmounted():
            unregister_listener()
    )
    
    <list data=@data>
        <item>
            <text value=@item.name />
        </item>
    </list>
</view>
```

**Ventaja Kyle UI:**
- ✅ Hooks con nombres claros (no `LaunchedEffect`)
- ✅ Más intuitivo
- ✅ Menos verboso

#### 5.3 Modifiers

**Jetpack Compose:**
```kotlin
Text(
    text = "Hello",
    modifier = Modifier
        .padding(16.dp)
        .background(Color.Blue)
        .border(2.dp, Color.Black)
        .clickable { onClick() }
)
```

**Kyle UI:**
```kyx
<text 
    value="Hello"
    style=style(
        padding: spacing.all(16),
        background: color("#0000FF"),
        border: border(2, color("#000000"), border_style.solid)
    )
    click=@on_click
/>
```

**Ventaja Kyle UI:**
- ✅ `style=` tipado (no cadena de modifiers)
- ✅ Reutilizable (style classes)
- ✅ Más legible

---

## 4. Cross-Platform: Flutter

### Lo Mejor de Flutter

#### 6.1 Widget Tree

**Flutter:**
```dart
Widget build(BuildContext context) {
  return Scaffold(
    appBar: AppBar(title: Text('My App')),
    body: Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text('Count: $count'),
          ElevatedButton(
            onPressed: () => setState(() => count++),
            child: Text('Increment'),
          ),
        ],
      ),
    ),
  );
}
```

**Kyle UI:**
```kyx
<view>
    @(
        count: ^i32 = 0
    )
    
    <app_bar title="My App" />
    
    <vstack alignment=alignment.center spacing=16>
        <text value=@"Count: " + count.to_str() />
        <button text="Increment" style=Primary click=@() => count += 1 />
    </vstack>
</view>
```

**Ventaja Kyle UI:**
- ✅ Sintaxis declarativa pura (no `build()`, `setState()`)
- ✅ Componentes nativos (no `Scaffold`, `AppBar`)
- ✅ Más legible

#### 6.2 State Management

**Flutter (Provider):**
```dart
// provider.dart
class CounterProvider extends ChangeNotifier {
  int _count = 0;
  int get count => _count;
  
  void increment() {
    _count++;
    notifyListeners();
  }
}

// widget.dart
class MyWidget extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    final counter = Provider.of<CounterProvider>(context);
    return Text('${counter.count}');
  }
}
```

**Kyle UI:**
```kyx
# counter_context.ky
context CounterContext:
    count: i32 = 0
    
    fn increment():
        count += 1

# widget.kyx
<view>
    @(
        counter = use_context(CounterContext)
    )
    
    <text value=@counter.count.to_str() />
</view>
```

**Ventaja Kyle UI:**
- ✅ Sin boilerplate (ChangeNotifier, notifyListeners)
- ✅ Tipado fuerte
- ✅ Más simple

#### 6.3 Hot Reload

**Flutter:**
```bash
flutter run
# Edit code
# Press 'r' to hot reload
```

**Kyle UI:**
```bash
ky run web
# Edit code
# Auto-reload en navegador
```

**Ventaja Kyle UI:**
- ✅ Auto-reload (no manual)
- ✅ Más rápido
- ✅ Preserva estado

---

## 5. Características Únicas de Kyle UI

### 7.1 Multi-Backend Nativo

**Kyle UI:**
```kyx
# Mismo código para todas las plataformas
<view>
    <button text="Click" click=@handler />
</view>
```

**Compila a:**
- **Web:** HTML/CSS/JS
- **iOS:** SwiftUI
- **Android:** Jetpack Compose
- **Desktop:** SDL2/Skia
- **KYOS OS:** Nativo

**Otros frameworks:**
- React: Solo web
- SwiftUI: Solo Apple
- Jetpack Compose: Solo Android
- Flutter: Propio (no nativo)

### 7.2 Tipado Fuerte Multiplataforma

**Kyle UI:**
```kyx
@(
    user: User  # Tipado fuerte en todas las plataformas
)
```

**Otros frameworks:**
- React: `any` (JavaScript)
- Flutter: `dynamic` (Dart)
- SwiftUI: Fuerte pero solo Apple
- Jetpack Compose: Fuerte pero solo Android

### 7.3 Tamaño de Bundle

| Plataforma | Kyle UI | Flutter | React Native |
|------------|---------|---------|--------------|
| **Web** | <1MB | ~2MB (WASM) | ~500KB |
| **iOS** | Nativo | ~15MB | ~10MB |
| **Android** | Nativo | ~15MB | ~10MB |
| **Desktop** | ~5MB | ~50MB | N/A |

### 7.4 Rendimiento

**Kyle UI:**
- **Web:** DOM directo (no virtual DOM)
- **iOS:** SwiftUI nativo
- **Android:** Jetpack Compose nativo
- **Desktop:** SDL2/Skia nativo

**Otros frameworks:**
- **React:** Virtual DOM (overhead)
- **Flutter:** Bridge JS-Nativo (overhead)
- **React Native:** Bridge JS-Nativo (overhead)

---

## 6. Mejores Prácticas de Todos los Frameworks

### De React

✅ **Hooks para lógica reutilizable**
```kyx
# use_counter.ky
fn use_counter(initial: i32) CounterState:
    count: ^i32 = initial
    
    fn increment():
        count += 1
    
    fn decrement():
        count -= 1
    
    return CounterState(count: count, increment: increment, decrement: decrement)
```

### De Vue

✅ **Two-way binding nativo**
```kyx
<text_field bind=@name />
```

### De SwiftUI

✅ **Declarative syntax**
```kyx
<vstack spacing=16>
    <text value="Hello" />
    <button text="Click" />
</vstack>
```

### De Jetpack Compose

✅ **State hoisting**
```kyx
# Parent
<child bind=@value />

# Child
@(
    bind: ^str
)
<text_field bind=@bind />
```

### De Flutter

✅ **Widget composition**
```kyx
<card>
    <vstack>
        <text value="Title" />
        <text value="Content" />
    </vstack>
</card>
```

### De Svelte

✅ **Reactividad automática**
```kyx
@(
    count: ^i32 = 0
    doubled = count * 2  # Automático
)
```

---

## 7. Ventajas Competitivas de Kyle UI

### 8.1 Un Solo Lenguaje

**Kyle UI:**
- Kyle para lógica
- Kyle para UI (.kyx)
- Kyle para estilos
- Kyle para tests

**Otros frameworks:**
- React: JavaScript + CSS + HTML
- Flutter: Dart + YAML (pubspec)
- SwiftUI: Swift + Storyboards (opcional)
- Jetpack Compose: Kotlin + XML (layouts legacy)

### 8.2 Tipado Fuerte en Todo

**Kyle UI:**
```kyx
@(
    user: User  # Error en compilación si es incorrecto
)
```

**Otros frameworks:**
- React: `any` en runtime
- Flutter: `dynamic` en runtime
- Vue: `any` en templates

### 8.3 Multiplataforma Real

**Kyle UI:**
- Web: HTML/CSS/JS nativo
- iOS: SwiftUI nativo
- Android: Jetpack Compose nativo
- Desktop: SDL2/Skia nativo
- KYOS OS: Nativo

**Otros frameworks:**
- React: Solo web
- SwiftUI: Solo Apple
- Jetpack Compose: Solo Android
- Flutter: Propio (no nativo)

### 8.4 Sin Bridge

**Kyle UI:**
- Compila a código nativo de cada plataforma
- No hay bridge JS-Nativo
- Rendimiento nativo

**Otros frameworks:**
- React Native: Bridge JS-Nativo
- Flutter: Bridge Dart-Nativo
- Xamarin: Bridge C#-Nativo

---

## 8. Conclusión

Kyle UI toma lo mejor de cada framework:

| De | Característica |
|----|----------------|
| **React** | Hooks, composition |
| **Vue** | Two-way binding, computed |
| **Svelte** | Reactividad automática |
| **SwiftUI** | Declarative syntax, navigation |
| **Jetpack Compose** | State hoisting, side effects |
| **Flutter** | Widget tree, hot reload |

Y agrega características únicas:

✅ **Multiplataforma real** (no bridge)
✅ **Tipado fuerte** en todo
✅ **Un solo lenguaje** (Kyle)
✅ **Rendimiento nativo**
✅ **Tamaño mínimo**
✅ **KYOS OS** (futuro)

---

## Referencias

- [Architecture](architecture.md)
- [Lifecycle](lifecycle.md)
- [Events](events.md)
- [Components](components/README.md)
