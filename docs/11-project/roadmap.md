# Kyle — Roadmap de Desarrollo

> UI Framework: [`docs/10-design/rfc/0005-ui-rearchitecture-plan.md`](docs/10-design/rfc/0005-ui-rearchitecture-plan.md)

---

## Estado Actual

| Área | Estado |
|------|--------|
| **Compilador** (Fases 1-17) | ✅ Completo |
| **Runtime** (memoria, strings, colecciones) | ✅ Completo |
| **Borrow checker** | ✅ Completo |
| **Cross-platform** (macOS, Linux, Windows) | ✅ Completo |
| **Tooling** (LSP, formatter, VS Code, package manager) | ✅ Completo |
| **Paquetes** (http, json, sqlite) | ✅ Completo |
| **FFI** (extern fn, @link, ptr) | ✅ Completo |
| **Tipos especializados** (46 tipos) | ✅ Todos implementados |

### UI Framework (RFC-0005 v2)

| Fase | Estado |
|:----:|:------:|
| **UI-IR + Backend system** | ✅ Completo |
| **JS Runtimes → ESM** | ✅ Completo |
| **`ky run` + `ky new kyui`** | ✅ Completo |
| **Routing: `<router>` + `<route>` + `<layout>` + `<slot>`** | ✅ **v0.8.0 — COMPLETADO** |
| **Module resolver multi-archivo** | ✅ **COMPLETADO** |
| **`app.kyx` entry point + template kyui** | ✅ **COMPLETADO** |
| **`navigate()` / `set_title()` / `<link>` built-in** | ✅ **COMPLETADO** |
| **File picker** (`<file_picker>`, `file_data` type) | ✅ **COMPLETADO** |
| **Form models** (`model=`, `field=`, class binding) | ✅ **COMPLETADO** |
| **VSCode extension** (grammar, snippets, closing tags) | ✅ **COMPLETADO** |
| **CSS reset** (`* { margin:0; padding:0 }` en HTML) | ✅ **COMPLETADO** |
| **Web backend funcional** | ✅ **Funciona** |
| **File picker nativo** (`<file_picker>`) | ✅ **Funciona en web** |
| **Form models + class binding** (`model=`, `field=`) | ✅ **Funciona en web** |
| **VSCode extension** (grammar, snippets, theme) | ✅ **Completo** |
| **Desktop backend (SDL2)** | 🟡 **Roto** — Sin `SDL_PollEvent`, eventos, ni ventana responsive |
| **iOS backend (SwiftUI)** | 🟡 **Roto** — Swift inválido (`.fontWeight(.bold())`, `Color(hex:)`), routes ignorados |
| **WASM target** | ❌ **No probado** |
| **Android backend** | ❌ **No existe** |
| **Config block global** | 📅 **Pendiente** |
| **Terminal / TUI backend** | 📅 **Pendiente** |

---

## Plan de Implementación Priorizado

### 🔴 FASE 1: Arreglar Desktop Backend (3-5 días)

**Objetivo:** Que `ky run desktop` funcione — ventana responsive, con eventos SDL2.

| Tarea | Esfuerzo |
|-------|:--------:|
| 1.1 `SDL_PollEvent` + manejar `SDL_QUIT` | Medio |
| 1.2 Arreglar `SDL_WINDOWPOS_UNDEFINED` | Pequeño |
| 1.3 Usar `SDL_RenderFillRect` (no drawLine loop) | Pequeño |
| 1.4 Soportar `@if`, `@for`, `@match`, `@expr` | Medio |
| 1.5 Soportar 30+ ComponentTags (hoy solo 11/46) | Grande |
| 1.6 `@link` cross-platform (macOS/Linux/Windows) | Medio |

### 🔴 FASE 2: Arreglar iOS Backend (3-5 días)

**Objetivo:** `ky build ios` genere SwiftUI compilable.

| Tarea | Esfuerzo |
|-------|:--------:|
| 2.1 `.fontWeight(.bold())` → `.fontWeight(.bold)` | Pequeño |
| 2.2 Extensión `Color(hex:)` en código generado | Pequeño |
| 2.3 `Package.swift` con `.library` (no `.executable`) | Medio |
| 2.4 No ignorar `Slot`, `Match`, `Expr`, `CodeBlock` | Medio |
| 2.5 Soportar 30+ ComponentTags (hoy solo 12/46) | Grande |
| 2.6 Routing: `NavigationStack` + `NavigationLink` | Grande |

### 🟡 FASE 3: WASM + Performance (2-4 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 3.1 Probar y arreglar build WASM | Grande |
| 3.2 Code splitting por ruta (lazy loading) | Medio |

### 🟡 FASE 4: Config Block Global (2-3 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 4.1 Parsear bloque `config:` con propiedades tipadas | Medio |
| 4.2 `target(Target.web): port = 8080` | Medio |
| 4.3 Tipografía global (fuente base, tamaños) | Medio |

### 🟢 FASE 5: Android Backend (5-7 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 5.1 UI-IR → Jetpack Compose | Grande |
| 5.2 `ky run android` | Medio |

### 🟢 FASE 6: Terminal / TUI (3-5 días)

| Tarea | Esfuerzo |
|-------|:--------:|
| 6.1 UI-IR → ratatui/NCurses | Grande |
| 6.2 `ky run terminal` | Medio |

---

## Estado de Backends UI

| Backend | Estado | Próximo paso |
|---------|:------:|-------------|
| **Web** | ✅ **Funcional** | Mantener + WASM |
| **Desktop (SDL2)** | 🟡 **Roto** — Sin eventos, 11/46 tags | 🔴 **FASE 1** |
| **iOS (SwiftUI)** | 🟡 **Roto** — Swift inválido, 12/46 tags | 🔴 **FASE 2** |
| **WASM** | ❌ **No probado** | 🟡 **FASE 3** |
| **Android** | ❌ **No existe** | 🟢 **FASE 5** |
| **Terminal / TUI** | ❌ **No existe** | 🟢 **FASE 6** |

---

## Testing

```bash
# Rust tests
cargo test --workspace --exclude kyc_runtime_wasm

# Build release
cargo build --release --bin ky

# Create test project
ky new kyui /tmp/test
cd /tmp/test

# Build web
ky build web
# Serve
ky run web        # Dev server on :8080

# Desktop (WIP)
ky build desktop  # Genera main.ky + compila

# iOS (WIP)
ky build ios      # Genera ios-app/ (necesita arreglos)
```

---

*Última actualización: 2026-07-12 · Plan priorizado por fases*
