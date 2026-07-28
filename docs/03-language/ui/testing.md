# Testing de UI

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Routing

---

## 1. Filosofía

Los tests de UI en Kyle se escriben en **Kyle puro**, no en JavaScript ni en un
DSL separado. El framework de testing está integrado en el compilador:

```bash
ky test              # todos los tests
ky test ui/          # tests de UI
ky test --watch      # watch mode
ky test --coverage   # cobertura
```

---

## 2. Tipos de Tests UI

| Tipo | Qué prueba | Velocidad |
|------|-----------|:---------:|
| **Unit test** | Componente aislado (props → output) | ⚡ ms |
| **Interaction test** | Eventos, estado, binding | ⚡ ms |
| **Integration test** | Múltiples componentes juntos | ⏱️ s |
| **Snapshot test** | Output visual vs referencia | ⚡ ms |
| **E2E test** | App completa en browser/dispositivo | 🐢 s |

---

## 3. Unit Tests

### 3.1 Renderizar un componente

```kyle
# tests/button_test.ky
use ~lib.{test, expect, render}

test("Button renders with text"):
    # Renderiza el componente en aislamiento
    btn = render("""
        <button text="Click me" style=Primary />
    """)

    expect(btn.text).to_equal("Click me")
    expect(btn.tag).to_equal("button")
    expect(btn.props.style).to_contain("Primary")
```

### 3.2 Test con props

```kyle
test("Button with disabled state"):
    btn = render("<button disabled=true text="OK" />")

    expect(btn.props.disabled).to_be_true()
    expect(btn.props.aria_disabled).to_equal("true")
```

### 3.3 Test de slots

```kyle
test("Card renders slot content"):
    card = render("""
        <card style=Elevated>
            <text value="Contenido" />
        </card>
    """)

    expect(card.slot("default").length).to_equal(1)
    expect(card.slot("default")[0].text).to_equal("Contenido")
```

---

## 4. Interaction Tests

### 4.1 Eventos

```kyle
test("Button fires click event"):
    clicked = false

    btn = render("""
        <button text="OK" click=@handle_click />
    """, context: {
        "handle_click": fn():
            clicked = true
    })

    btn.click()
    expect(clicked).to_be_true()
```

### 4.2 Binding bidireccional

```kyle
test("TextField updates state on input"):
    state = { "email": "" }

    field = render("""
        <text_field bind=@email />
    """, state: state)

    field.simulate_input("test@example.com")
    expect(state.email).to_equal("test@example.com")
```

### 4.3 Estado del componente

```kyle
test("Counter increments on click"):
    counter = render("""
        @(count: ^i32 = 0)
        <view>
            <text value=count.to_str() />
            <button text="+" click=@count = count + 1 />
        </view>
    """)

    expect(counter.text()).to_equal("0")
    counter.find("button").click()
    expect(counter.text()).to_equal("1")
```

---

## 5. Testing con Context

```kyle
test("Component uses context"):
    mock_auth = {
        "user": User(name: "Test"),
        "token": "mock-token",
    }

    profile = render("""
        @(auth = use_context(AuthContext))
        <text value=auth.user.name />
    """, context_providers: {
        AuthContext: mock_auth,
    })

    expect(profile.text()).to_equal("Test")
```

---

## 6. Async Tests

```kyle
test("Async data loading"):
    loader = render("""
        @(data: ^str? = None)
        @(
            fn on_mounted():
                fetch_data().then(fn(res):
                    data = res
                )
        )
        <view>
            @if data == None:
                <spinner />
            @else:
                <text value=data />
        </view>
    """)

    expect(loader.find("spinner")).to_exist()
    
    await loader.wait_for_state_change()
    expect(loader.text()).to_equal("loaded data")
```

---

## 7. Snapshot Tests

### 7.1 Snapshot básico

```kyle
test("Button snapshot"):
    btn = render("""
        <button text="OK" style=Primary />
    """)

    expect(btn).to_match_snapshot("button-primary")
```

### 7.2 Actualizar snapshots

```bash
ky test --update-snapshots
```

### 7.3 Snapshots por target

```kyle
test("Button renders correctly on all targets"):
    btn = render("<button text="OK" />")

    # Los snapshots se generan por target
    expect(btn).to_match_snapshot("button-web", target: "web")
    expect(btn).to_match_snapshot("button-macos", target: Target.macos)
```

---

## 8. Integration Tests

### 8.1 Múltiples componentes

```kyle
test("Form with validation"):
    form = render("""
        @(
            email: ^str = ""
            errors: ^{str} = {}

            fn validate() bool:
                errors = {}
                if email == "":
                    errors.push("Email requerido")
                errors.len() == 0

            fn submit():
                if validate():
                    # submit...
        )

        <form submit=@submit>
            <text_field bind=@email />
            <button text="Enviar" />
        </form>
    """)

    form.find("button").click()
    expect(form.text()).to_contain("Email requerido")

    form.find("text_field").simulate_input("test@test.com")
    form.find("button").click()
    expect(form.text()).not_to_contain("Email requerido")
```

### 8.2 Con routing

```kyle
test("Navigation between views"):
    app = render_with_router("""
        <router>
            <home_view />
            <login_view />
        </router>
    """)

    expect(app.current_route()).to_equal("/")
    
    app.navigate("/login")
    expect(app.current_route()).to_equal("/login")
    expect(app.find("password_field")).to_exist()
```

---

## 9. E2E Tests

Para tests end-to-end en browser/dispositivo:

```kyle
# tests/e2e/login_e2e.ky
use ~lib.{e2e, browser}

e2e("Login flow"):
    page = browser.new_page()

    page.navigate("http://localhost:3000/login")
    page.fill("text_field[bind=email]", "user@test.com")
    page.fill("password_field[bind=password]", "123456")
    page.click("button")

    await page.wait_for_navigation()
    expect(page.url()).to_contain("/dashboard")
    expect(page.text("text")).to_contain("Bienvenido")
```

---

## 10. Mocks y Stubs

### 10.1 Mock de componente

```kyle
test("Parent with mocked child"):
    # Reemplaza un componente hijo por un mock
    profile = render("""
        <user_profile user_id=@uid />
    """, mocks: {
        "user_profile": TestComponent("mocked-profile"),
    })

    expect(profile.find("user_profile")).to_exist()
```

### 10.2 Mock de contexto

```kyle
test("Component with mocked API"):
    mock_api = MockAPI()
    mock_api.when("fetch_user", fn(id):
        User(id: id, name: "Mock")
    )

    render("""
        @(api = use_context(ApiContext))
        <!-- ... -->
    """, context_providers: {
        ApiContext: mock_api,
    })
```

---

## 11. Coverage

```bash
$ ky test --coverage

File                          %    Lines  Missed  Coverage
─────────────────────────────────────────────────────────
components/button.kyx       100       42       0   ✅
components/login.kyx         85      120      18   ⚠️
components/user_card.kyx     92       65       5   ✅
─────────────────────────────────────────────────────────
Total                        89      227      23   ✅
```

---

## 12. Buenas Prácticas

| Práctica | Descripción |
|----------|-------------|
| **Test por comportamiento** | No por implementación — testea qué hace, no cómo |
| **Un test por caso** | Un `test()` por escenario |
| **Mockear APIs** | No hagas requests reales en unit tests |
| **Snapshot mínimo** | Solo componentes estables, no WIP |
| **Test de accesibilidad** | Verifica `aria-*`, roles, tab order |
| **Coherencia targets** | Tests cross-platform donde sea posible |

---

## 13. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Routing en tests de integración
