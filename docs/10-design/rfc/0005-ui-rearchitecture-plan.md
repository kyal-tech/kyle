# RFC-0005: Re-arquitectura UI Framework — Multiplataforma desde el núcleo

**Status:** Implementado (FASES A-C) — Diseño v2 aprobado
**Date:** 2026-07-12
**Documentación relacionada:**
- [0002-ui-architecture.md](0002-ui-architecture.md) — Arquitectura original
- [0003-ui-translation.md](0003-ui-translation.md) — Traducción multi-target
- [0004-ui-implementation-roadmap.md](0004-ui-implementation-roadmap.md) — Roadmap anterior

---

## 0. Resumen Ejecutivo

El framework UI tiene una **capa intermedia agnóstica (UI-IR)** que permite
a `.kyx` compilar a cualquier plataforma. El diseño v2 introduce:

- **Targets desglosados por SO**: `web`, `macos`, `windows`, `linux`, `ios`, `android`
- **Rutas centralizadas** en el `<router>` con `<route>`, NO en las vistas
- **Props vía visibilidad**: público = prop, `_` = interno, `__` = privado
- **Layouts persistentes** con `<layout>` + `<slot />` (navbar/sidebar no se refrescan)
- **Sin `view("/path")`**: los `.kyx` son solo componentes, las rutas están en `app.kyx`
- **Todo snake_case**: `color()`, `spacing.all()`, `font_weight.bold`, etc.

---

## 1. Arquitectura Aprobada

```
.ky / .kyx (código fuente)
     │
     ▼
┌──────────────────────────────┐
│      .kyx Parser             │  ← XML → AST Kyle → UI-IR
│   (kyc_ui/parser.rs)         │
└──────────┬───────────────────┘
           ▼
┌──────────────────────────────┐
│   UI-IR (Intermediate Repr)  │  ← Agnóstico, tipado (ir.rs)
│   UiNode, ComponentTag       │
└──────────┬───────────────────┘
           │
     ┌─────┴───────────────────────────┐
     │          │          │           │
     ▼          ▼          ▼           ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌──────────┐
│  Web   │ │  macOS  │ │Windows │ │  Linux   │
│ Backend│ │ Backend │ │Backend │ │ Backend  │
└────────┘ └────────┘ └────────┘ └──────────┘
     │          │          │           │
     ▼          ▼          ▼           ▼
┌────────┐ ┌────────┐
│  iOS   │ │ Android│
│ Backend│ │ Backend│
└────────┘ └────────┘
```

### 1.1 Principios de diseño

1. **Rutas centralizadas en `<router>`** — Sin `view("/path")` en componentes
2. **Props via visibilidad** — `name` = prop, `_name` = interno, `__name` = privado
3. **Layouts persistentes** — `<layout>` + `<slot />` para wrappers que no se refrescan
4. **Todo snake_case** — `color()`, `spacing.all()`, `font_weight.bold`
5. **Targets por SO** — `web`, `macos`, `windows`, `linux`, `ios`, `android`

### 1.2 Jerarquía de componentes nativos

```
<app>                   ← Raíz (1 por app)
<router>                ← Navegador
<route>                 ← Definición de ruta (path, component, layout, title, guard)
<layout>                ← Layout persistente
<slot />                ← Punto de inserción
<view>                  ← Contenedor genérico (equivalente a div)
<vstack>/<hstack>/<zstack>  ← Layout flex
<text>/<button>/<image>/<link>  ← Elementos
```

### 1.3 Tipos nativos

Cada componente es un `final class`. No hay mapeo explícito a plataformas — eso es interno del backend.

```kyle
final class app, router, route, layout, view
final class vstack, hstack, zstack, spacer, divider
final class text, button, image, link, input, text_field
final class group, scroll, modal, sheet, alert, navbar, sidebar
```

---

## 2. Cambios del Diseño v1 → v2

| Aspecto | v1 (anterior) | v2 (aprobado) |
|---------|---------------|---------------|
| Rutas | `view("/path")` en cada `.kyx` | Centralizadas en `<router>` con `<route>` |
| Props | Bloque `@(...)` en .kyx | Visibilidad: `name` = prop, `_` = interno |
| Layout | No existía | `<layout>` + `<slot />` persistente |
| Target | `@target("web")` string | `target(Target.web)` enum tipado |
| `Color` | `Color("#0066FF")` | `color("#0066ff")` |
| `class` en link | `class="nav-link"` | Eliminado (usar `style=Style.nav_link`) |
| Desktop target | `desktop` (genérico) | `macos`, `windows`, `linux` |

---

## 3. UI-IR (UI Intermediate Representation)

Definido en `crates/kyc_ui/src/ir.rs`:

```rust
pub struct UiProgram {
    pub view_paths: Vec<String>,
    pub code_blocks: Vec<String>,
    pub styles: Vec<StyleDecl>,
    pub animations: Vec<AnimDecl>,
    pub body: Vec<UiNode>,
}

pub enum UiNode {
    Element { tag: ComponentTag, attrs: Vec<UiAttr>, children: Vec<UiNode> },
    SelfClosing { tag: ComponentTag, attrs: Vec<UiAttr> },
    Text(String), Slot { name, fallback },
    If { condition, then_branch, else_branch },
    For { item, list, body },
    Match { expr, cases },
    Expr(String), CodeBlock(String),
}
```

---

## 4. Sistema de Backends

Trait `UiBackend`:

```rust
pub trait UiBackend {
    fn name(&self) -> &str;
    fn target_triple(&self) -> &str;
    fn generate(&self, program: &UiProgram) -> BackendOutput;
}
```

Backends registrados:
- `web` (wasm32) — ✅ Implementado
- `macos` (arm64) — 📅 FASE D
- `windows` (x64) — 📅 FASE D
- `linux` (x64/arm64) — 📅 FASE D
- `ios` (arm64) — 📅 FASE E
- `android` (arm64) — 📅 FASE E

---

## 5. Plan de Implementación

### FASE R: Routing + Module Resolver (v0.8.0)

| Tarea | Esfuerzo | Archivos |
|-------|:--------:|----------|
| Parser: `<route path="..." component=@comp layout=@layout>` | Medio | `parser.rs`, `ir.rs` |
| Parser: props via visibility (`_name`, `__name`) | Medio | `parser.rs` |
| Module resolver: `<home_view />` busca `views/home.kyx` | Grande | `resolver.rs` |
| Multi-file: compilar N .kyx → 1 IR con N rutas | Grande | `pipeline.rs` |
| `<layout>` + `<slot />` en UI-IR | Medio | `ir.rs`, `web_backend.rs` |
| `target(Target.web)` enum en parser | Pequeño | `parser.rs` |

### FASE C: Config + CLI (v0.8.0)

| Tarea | Esfuerzo |
|-------|:--------:|
| `app.kyx` como entry point único | Medio |
| `ky new kyui` genera `app.kyx + views/ + layouts/ + components/` | Medio |
| Bloque `config:` con `target(Target.web): port = 8080` | Medio |

### FASE D: Desktop Nativos (Skia/SDL2/GLFW)

| Tarea | Esfuerzo |
|-------|:--------:|
| Backend macOS: UI-IR → AppKit/SwiftUI | Grande |
| Backend Windows: UI-IR → Win32 | Grande |
| Backend Linux: UI-IR → GTK | Grande |
| Layout engine (flexbox) | Grande |

### FASE E: Mobile (iOS + Android)

| Tarea | Esfuerzo |
|-------|:--------:|
| Backend iOS: UI-IR → SwiftUI | Grande |
| Backend Android: UI-IR → Compose | Grande |

---

## 6. Glosario

| Término | Significado |
|---------|-------------|
| UI-IR | UI Intermediate Representation — AST agnóstico de plataforma |
| Backend | Traductor de UI-IR a código de plataforma específica |
| ComponentTag | Enum de tags conocidos (View, Text, Button, etc.) |
| Target | Enum: `web`, `macos`, `windows`, `linux`, `ios`, `android` |
| `<route>` | Elemento que define una ruta (path + component + layout) |
| `<layout>` | Wrapper persistente con `<slot />` para contenido dinámico |
| Props via visibility | `name` = prop público, `_name` = interno, `__name` = privado |
