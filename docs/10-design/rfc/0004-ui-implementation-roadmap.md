# RFC-0004: UI Framework — Roadmap de Implementación

**Status:** Reemplazado por [RFC-0005](0005-ui-rearchitecture-plan.md)
**Date:** 2026-07-10 (original)

> ⚠️ Este roadmap ha sido reemplazado por [RFC-0005](0005-ui-rearchitecture-plan.md)
> que define la nueva arquitectura UI-IR + Backend system + CLI unificado.
> Las FASIS 0-12 originales han sido refactorizadas en FASIS A-F.
**Documentación relacionada:**
- [0002-ui-architecture.md](0002-ui-architecture.md) — Arquitectura
- [0003-ui-translation.md](0003-ui-translation.md) — Traducción multi-target
- [ui-syntax.md](../../03-language/syntax/ui-syntax.md) — Sintaxis .kyx
- [style-system.md](../../03-language/ui/style-system.md) — Estilos
- [state-events.md](../../03-language/ui/state-events.md) — Estado/eventos
- [animation.md](../../03-language/ui/animation.md) — Animaciones
- [routing.md](../../03-language/ui/routing.md) — Routing
- [accessibility.md](../../03-language/ui/accessibility.md) — a11y
- [i18n.md](../../03-language/ui/i18n.md) — Internacionalización
- [portals.md](../../03-language/ui/portals.md) — Portales/teleport
- [error-boundaries.md](../../03-language/ui/error-boundaries.md) — Error boundaries
- [composition.md](../../03-language/ui/composition.md) — Composición
- [context-patterns.md](../../03-language/ui/context-patterns.md) — Context avanzado
- [ssr.md](../../03-language/ui/ssr.md) — SSR
- [testing.md](../../03-language/ui/testing.md) — Testing

---

## 1. Filosofía de Implementación

| Principio | Significado |
|-----------|-------------|
| **Funcional primero** | Cada fase produce algo que FUNCIONA, no solo código muerto |
| **Sin dependencias externas** | No npm, no Node.js, no CDN. Todo es Kyle + LLVM |
| **Web primero, desktop después** | Web (WASM + JS) es el target inicial por ser el más accesible |
| **Runtime en Kyle** | Los componentes UI se escriben en Kyle (como http, json, sqlite) |
| **Un parser, múltiples backends** | .kyx → AST Kyle → LLVM/WASM + JS generator |

---

## 2. FASE 0: CLI y Target Triples

**Objetivo:** `ky build --target wasm32` funciona.

### Tareas

| # | Tarea | Archivos | Esfuerzo |
|---|-------|----------|:--------:|
| 0.1 | Flag `--target` en CLI | `crates/kyc_cli/src/main.rs` | Pequeño |
| 0.2 | Target triple en Codegen | `crates/kyc_backend/src/codegen.rs` | Pequeño |
| 0.3 | Target triple en Pipeline | `crates/kyc_driver/src/pipeline.rs` | Pequeño |
| 0.4 | Linker por target triple | `crates/kyc_backend/src/linker.rs` | Medio |
| 0.5 | Compilar kyc_runtime a WASM | `crates/kyc_runtime/` | Medio |
| 0.6 | JS glue runtime básico | `runtimes/js/glue.js` | Medio |
| 0.7 | `ky build login.kyx` produce .wasm + .js | Pipeline completo | Medio |

### Entregable

```bash
ky build --target wasm32 app.kyx
# → app.wasm + ui-runtime.js + index.html
```

### Validación

```bash
cd /tmp/test-ui
ky new webapp my-app
cd my-app
ky build
# Abrir index.html en browser → ver "Hello from Kyle"
```

---

## 3. FASE 1: Parser .kyx + Traductor JS

**Objetivo:** Parsear .kyx y generar JS que manipula el DOM.

### Tareas

| # | Tarea | Archivos | Esfuerzo |
|---|-------|----------|:--------:|
| 1.1 | Nueva crate `kyc_ui` | `crates/kyc_ui/src/lib.rs` | Medio |
| 1.2 | Parser XML .kyx → AstType | `crates/kyc_ui/src/parser.rs` | Grande |
| 1.3 | Registrar `.kyx` en module resolver | `crates/kyc_frontend/src/resolver.rs` | Pequeño |
| 1.4 | JS generator: componentes básicos | `crates/kyc_ui/src/js_gen.rs` | Grande |
| 1.5 | Soporte `@expr` y `@(bloque)` | `crates/kyc_ui/src/parser.rs` | Medio |
| 1.6 | Soporte `@if`, `@for`, `@match` | `crates/kyc_ui/src/js_gen.rs` | Medio |
| 1.7 | Soporte slots y children | `crates/kyc_ui/src/js_gen.rs` | Medio |

### Entregable

```kyx
# hello.kyx
<view>
    <text value="Hola Mundo" />
</view>
```

```bash
ky build --target wasm32 hello.kyx
# → hello.wasm + ui-runtime.js
# → Browser muestra "Hola Mundo"
```

---

## 4. FASE 2: Estilos Tipados

**Objetivo:** `style<button> Primary:` compila a estilos inline en JS.

### Tareas

| # | Tarea | Archivos | Esfuerzo |
|---|-------|----------|:--------:|
| 2.1 | Types Kyle: Color, Spacing, etc. | `packages/ui/src/styles/types.ky` | Pequeño |
| 2.2 | Parser `style<>`, `layout<>`, `tpl<>`, `theme` | `crates/kyc_ui/src/parser.rs` | Medio |
| 2.3 | JS generator: style → inline styles | `crates/kyc_ui/src/style_gen.rs` | Medio |
| 2.4 | Theme system (light/dark) | `packages/ui/src/styles/theme.ky` | Medio |
| 2.5 | Responsive `@media()` | `crates/kyc_ui/src/style_gen.rs` | Medio |

### Entregable

```kyx
style<button> Primary:
    background = Color("#0066FF")
    color = Color("#FFFFFF")
    border_radius = 8

<button tpl=Primary text="Estilizado" />
```

---

## 5. FASE 3: Eventos y Binding

**Objetivo:** `click=@fn`, `bind=@var`, eventos funcionan.

### Tareas

| # | Tarea | Archivos | Esfuerzo |
|---|-------|----------|:--------:|
| 3.1 | DOM events: click, input, change | `crates/kyc_ui/src/events_gen.rs` | Medio |
| 3.2 | Event → callback WASM bridge | `runtimes/js/glue.js` | Medio |
| 3.3 | One-way binding (estado → UI) | `runtimes/js/reactivity.js` | Medio |
| 3.4 | Two-way binding (UI ↔ estado) | `runtimes/js/reactivity.js` | Medio |
| 3.5 | Event object (MouseEvent, KeyboardEvent) | `packages/ui/src/events.ky` | Pequeño |

### Entregable

```kyx
@(
    count: ^i32 = 0
    fn increment():
        count = count + 1
)
<view>
    <text value="Contador: " + count.to_str() />
    <button text="+" click=@increment />
</view>
```

---

## 6. FASE 4: Componentes UI

**Objetivo:** Componentes built-in funcionando (view, text, button, text_field, etc.).

### Tareas

| # | Tarea | Prioridad |
|---|-------|:---------:|
| 4.1 | View, Text, Button | Alta |
| 4.2 | Column, Row, Spacer | Alta |
| 4.3 | TextField, PasswordField, Checkbox | Alta |
| 4.4 | Image, Icon, Divider | Media |
| 4.5 | Card, Surface, Scroll | Media |
| 4.6 | Select, Slider, Switch, Radio | Media |
| 4.7 | Dialog, Tooltip, Snackbar | Baja |
| 4.8 | Navigation: Tabs, AppBar, Drawer | Baja |
| 4.9 | Progress, Spinner, Skeleton | Baja |

Cada componente es su propio archivo en `packages/ui/src/components/`.

### Entregable

```kyx
<view>
    <column layout=Center spacing=16>
        <text typography=Title value="Login" />
        <text_field bind=@email label="Email" />
        <password_field bind=@password label="Password" />
        <button tpl=Primary text="Ingresar" click=@login />
    </column>
</view>
```

---

## 7. FASE 5: Routing

**Objetivo:** Navegación entre vistas con `view("/path")`.

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 5.1 | Router component | Medio |
| 5.2 | History API bridge (web) | Pequeño |
| 5.3 | Route params parsing | Medio |
| 5.4 | Lazy loading de vistas | Medio |
| 5.5 | Guardias (before_enter, before_leave) | Medio |

### Entregable

```kyx
# app.kyx — auto-routing con <router>
<app>
    <router>
        <home_view />
        <login_view />
        <user_detail />
    </router>
</app>
```

```kyx
# home.kyx
view("/")
<view>
    <text value="Inicio" />
</view>
```

```kyx
# user_detail.kyx
view("/users/{id}")
@(
    params: {str: str} = route_params()
    id: str = params.get("id") ?? ""
)
<view>
    <text value="Usuario: " + id />
</view>
```

---

## 8. FASE 6: Animaciones

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 6.1 | Animation type system | Pequeño |
| 6.2 | CSS animations generator | Medio |
| 6.3 | Web Animations API bridge | Medio |
| 6.4 | Transition in styles | Medio |

---

## 9. FASE 7: a11y

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 7.1 | ARIA auto-generation en JS gen | Medio |
| 7.2 | Keyboard navigation | Medio |
| 7.3 | Focus management | Pequeño |
| 7.4 | Reduced motion support | Pequeño |

---
# FASE 8: Portales

**Objetivo:** Renderizar contenido fuera del árbol padre.

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 8.1 | Portal component en .kyx parser | Pequeño |
| 8.2 | JS generator: render en target selector | Medio |
| 8.3 | Desktop: overlay layer | Medio |
| 8.4 | Position strategies (Auto, RelativeTo, Center) | Medio |

---

## 9. FASE 9: Error Boundaries

**Objetivo:** Captura y recuperación de errores en UI.

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 9.1 | ErrorBoundary component en parser | Pequeño |
| 9.2 | Error capture en runtime JS | Medio |
| 9.3 | Fallback UI system | Medio |
| 9.4 | Retry + recovery logic | Pequeño |

---

## 10. FASE 10: i18n

**Objetivo:** Sistema completo de internacionalización.

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 10.1 | `locale ""` parser | Medio |
| 10.2 | ICU plural rules compiler | Grande |
| 10.3 | Date/number format compiler | Medio |
| 10.4 | RTL layout support | Medio |
| 10.5 | Lazy locale loading | Medio |

---

## 11. FASE 11: SSR

**Objetivo:** Server-Side Rendering de vistas.

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 11.1 | SSR renderer (Kyle → HTML) | Grande |
| 11.2 | State serialization para hydration | Medio |
| 11.3 | Streaming SSR | Grande |
| 11.4 | `ky serve` dev server | Medio |
| 11.5 | SEO meta tags | Pequeño |

---

## 12. FASE 12: Testing

**Objetivo:** Framework de testing UI integrado.

### Tareas

| # | Tarea | Esfuerzo |
|---|-------|:--------:|
| 12.1 | Test renderer (componente aislado) | Medio |
| 12.2 | Event simulation | Medio |
| 12.3 | Snapshot system | Medio |
| 12.4 | Context mocking | Pequeño |
| 12.5 | `ky test --coverage` | Medio |

---

## 13. Diagrama de dependencias

```
FASE 0: CLI + Targets
  │
  ▼
FASE 1: Parser .kyx + JS gen  ◄── EMPEZAMOS AQUÍ
  │
  ├──► FASE 2: Estilos
  │       │
  │       ▼
  ├──► FASE 3: Eventos + Binding
  │       │
  │       ▼
  ├──► FASE 4: Componentes UI
  │       │
  │       ▼
  ├──► FASE 5: Routing
  │
└──► FASE 6: Animaciones
        │
        ▼
     FASE 7: a11y
        │
        ▼
     FASE 8: Portales
        │
        ▼
     FASE 9: Error Boundaries
        │
        ▼
     FASE 10: i18n
        │
        ▼
     FASE 11: SSR
        │
        ▼
     FASE 12: Testing
```

---

## 14. Orden de implementación recomendado

```
Semana 1-2:   FASE 0 (CLI + Targets)
              FASE 1.1-1.3 (Parser .kyx básico)

Semana 3-4:   FASE 1.4-1.7 (JS generator)
              FASE 2.1-2.3 (Estilos básicos)

Semana 5-6:   FASE 3 (Eventos + Binding)
              FASE 4.1-4.3 (View, Text, Button, Column, Row)

Semana 7-8:   FASE 4.4-4.6 (TextField, Image, Card, etc.)
              FASE 2.4-2.5 (Theme, Responsive)

Semana 9-10:  FASE 5 (Routing)
              FASE 6 (Animaciones)

Semana 11-12: FASE 7 (a11y)
              FASE 8 (Portales)

Semana 13-14: FASE 9 (Error Boundaries)
              FASE 10.1-10.2 (i18n básico)

Semana 15-16: FASE 10.3-10.5 (i18n avanzado)
              FASE 11.1-11.3 (SSR básico)

Semana 17-18: FASE 11.4-11.5 (SSR + SEO)
              FASE 12 (Testing)
```

---

## 15. Glosario de implementación

| Término | Significado |
|---------|-------------|
| `kyc_ui` | Nueva crate Rust que parsea .kyx y genera JS |
| `js_gen.rs` | Generador de código JS a partir de AST Kyle |
| `style_gen.rs` | Traduce `style<>` a inline styles JS |
| `events_gen.rs` | Traduce eventos a addEventListener |
| `ui-runtime.js` | Runtime JS generado que contiene componentes |
| `glue.js` | Código JS que conecta WASM con el DOM |
| `reactivity.js` | Sistema reactivo (Proxy + watchers) |

---

## 16. Referencias

- [RFC-0002](0002-ui-architecture.md) — Arquitectura
- [RFC-0003](0003-ui-translation.md) — Traducción
- [i18n.md](../../03-language/ui/i18n.md) — Internacionalización
- [portals.md](../../03-language/ui/portals.md) — Portales
- [error-boundaries.md](../../03-language/ui/error-boundaries.md) — Error boundaries
- [ssr.md](../../03-language/ui/ssr.md) — SSR
- [testing.md](../../03-language/ui/testing.md) — Testing UI
