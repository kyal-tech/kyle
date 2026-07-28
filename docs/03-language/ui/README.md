# Kyle UI Framework — Documentación

**Status:** Specification v2.0
**Date:** 2026-07-28

Kyle UI es un framework UI declarativo multiplataforma que compila `.kyx` a web (HTML/CSS/JS), desktop (SDL2/Skia), iOS (SwiftUI), y Android (Jetpack Compose).

---

## Filosofia

- **30 componentes nativos**, no 200 como Flutter
- **Tipado fuerte** — Todo verificado en compilación
- **snake_case** — Consistente en todo
- **Composición** — Componentes pequeños combinables
- **Multiplataforma real** — Misma sintaxis, rendering nativo

---

## Core

| Documento | Descripción |
|-----------|-------------|
| [architecture.md](architecture.md) | Arquitectura multiplataforma, anti-patrones, traducción por target |
| [style-system.md](style-system.md) | Sistema de estilos tipado (color, spacing, layout, theme, responsive) |
| [state-events.md](state-events.md) | Estado, eventos, binding, formularios, validación, modelos |
| [animation.md](animation.md) | Animaciones y transiciones tipadas |
| [routing.md](routing.md) | Routing, `<route>`, `<layout>`, `<slot>`, guards, navegación |
| [accessibility.md](accessibility.md) | Accesibilidad WCAG 2.1 AA, ARIA, teclado, screen readers |
| [anti-patterns.md](anti-patterns.md) | Anti-patrones de otros frameworks y cómo Kyle los evita |

## Componentes Nativos

| Documento | Descripción |
|-----------|-------------|
| [components/README.md](components/README.md) | Índice de todos los componentes nativos |
| [components/img.md](components/img.md) | `<img>` — Imágenes (URL, local, lazy loading) |
| [components/button.md](components/button.md) | `<button>` — Botones (filled, outlined, text, icon) |
| [components/text_field.md](components/text_field.md) | `<text_field>` — Campos de texto y contraseña |
| [components/form.md](components/form.md) | `<form>` — Formularios con modelos y validación |
| [components/list.md](components/list.md) | `<list>` — Listas con virtualización y paginación |
| [components/table.md](components/table.md) | `<table>` — Tablas con sort, paginación, selección |
| [components/modal.md](components/modal.md) | `<modal>` y `<sheet>` — Ventanas modales y hojas |
| [components/toast.md](components/toast.md) | `toast.*` — Notificaciones temporales |

## Patrones Avanzados

| Documento | Descripción |
|-----------|-------------|
| [composition.md](composition.md) | Slots, render props, compound components |
| [context-patterns.md](context-patterns.md) | Context avanzado: selectores, reducers, multi-context |
| [portals.md](portals.md) | Portales: modals, tooltips fuera del árbol |
| [error-boundaries.md](error-boundaries.md) | Captura de errores, fallback UI, recovery |

## Infraestructura

| Documento | Descripción |
|-----------|-------------|
| [ssr.md](ssr.md) | Server-Side Rendering, streaming, hidratación |
| [i18n.md](i18n.md) | Internacionalización, plurales, fechas, RTL |
| [testing.md](testing.md) | Testing: unit, interacción, snapshots, E2E |
| [file-picker.md](file-picker.md) | File picker nativo multiplataforma |

## Documentos Relacionados

- [Sintaxis .kyx](../syntax/ui-syntax.md) — Sintaxis completa de archivos .kyx
- [RFC-0002: Arquitectura UI](../../10-design/rfc/0002-ui-architecture.md)
- [RFC-0003: Traducción multi-target](../../10-design/rfc/0003-ui-translation.md)
- [RFC-0005: UI v2](../../10-design/rfc/0005-ui-rearchitecture-plan.md)

---

## Quick Start

```bash
ky new kyui mi-app
cd mi-app
ky run web
```

```kyx
# app.kyx
from views.home import home

<app title="Mi App">
    <router>
        <route path="/" component=home />
    </router>
</app>
```

```kyx
# src/views/home.kyx
<view>
    @(
        count: ^i32 = 0
        fn increment():
            count += 1
    )
    
    <vstack spacing=16 alignment=alignment.center>
        <text value=@"Contador: " + count.to_str() />
        <button text="+" style=Primary click=@increment />
    </vstack>
</view>
```
