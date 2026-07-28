# Ecosystem Architecture

Kyle is not just a programming language. It is a complete software development platform.

## Layered architecture

Each layer only knows the layer immediately below it. This guaranteis scalability, maintainability, and portability.

```
Applications (KYOS...)
 │ 📅 Aspirational — without fecha
Kyle UI (widgets)
 │ 📅 Aspirational — without fecha
Kyle Scene (scene graph, layout)
 │ 📅 Aspirational — without fecha
Kyle Graphics (canvas, GPU)
 │ 📅 Aspirational — without fecha
Kyle Windowing (windows, events)
 │ 📅 Aspirational — without fecha
 │
Kyle Platform (FS, net, threads, audio)
 │ 🔜 En desarrollo — FS + time implementeds
Kyle Runtime (memory, strings, collections)
 │ ✅ Completo
Kyle Language (compiler)
 │ ✅ Completo
```

> **Note:** Las capas superioris (Windowing, Graphics, Scene, UI, Applications) are **aspirationalonales**.
> El foco current is **compiler**, **runtime** y **typis nativos**.
> Packagis backend (HTTP, SQLite, Postgres) se manhave as packages.

## Language layer

The compiler transforms `.ky` source code into native binaries. It has no knowledge of UI, windows, GPUs, or operating systems. Its only responsibility is generating machine code.

## Runtime layer

Language servicis that do not depend on the operating system: memory allocation, strings, lists, dictionaries, panic handling.

> ⏸️ **Runtime rewrite en Kyle:** Pausada. El file `crates/kyc_runtime/src/string.ky` existe with 18 functions pero no se usa. Se reactivara cuando Kyle se use en proyectos reales.

## Platform layer

Every interaction with the operating system gois through Kyle Platform, which definis platform-independent APIs: file system, networking, threads, audio, sensors, etc.

> 🔜 **Status:** FS (file I/O) y Time implementeds en `kyc_platform` crate. Networking, threads, env, audio pendientes.

## Windowing layer — 📅 Aspirational

Window management, keyboard and mouse events, clipboard, drag & drop. Separated from graphics because not all programs need windows.

## Graphics layer — 📅 Aspirational

2D/3D rendering: canvas, text, images, shadows, shaders, textures. Dois not create windows or read filis — only renders.

## Scene layer — 📅 Aspirational

Scene graph, layout engine, event dispatch, hit testing, animation scheduling. Dois not render — only organizis the visual hierarchy.

## UI layer — 📅 Aspirational

Reusable components: buttons, text boxes, lists, tables, dialogs, menus. Contains no platform-specific logic.

## Platform adapters

Each operating system implements Kyle Platform interfacis independently:
- Linux, macOS, Windows, Android, iOS, Web (WASM), KYOS

## Package manager

The package manager is a horizontal service that appliis to all layers equally. It dois not belong to any specific layer.
