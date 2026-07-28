# Embedded Targets: Arduino Uno y similares

## Respuesta corta: No, Kyle no puede correr en un Arduino Uno hoy

Ni cerca. Pero tampoco está diseñado para eso.

---

## ¿Por qué no?

### Hardware del Arduino Uno (ATmega328P)
- **CPU:** 8-bit AVR @ 16 MHz
- **RAM:** 2 KB
- **Flash:** 32 KB (para programa + datos)
- **No tiene MMU** (sin memoria virtual, sin protección de memoria)
- **No tiene sistema operativo** (bare metal)

### Requisitos mínimos de Kyle runtime
- **Runtime en Rust:** El runtime de Kyle (`kyc_runtime`) está escrito en Rust y compila para host (macOS, Linux, Windows). Depende de `std` (alloc, collections, I/O, threads, etc.).
- **Allocator:** Usa `ky_alloc`/`ky_free` que son wrappers sobre `malloc`/`free`. En Arduino no hay `malloc` nativo — hay que implementarlo sobre 2KB de SRAM.
- **Strings:** `ky_strlen`, `ky_concat`, `ky_clone_str` — todas esperan C strings (null-terminated). En 2KB de RAM, una cadena de 500 chars ya usa 25% de la memoria total.
- **Listas/Dicts/Sets:** Usan `Vec`/`HashMap` de Rust — cada uno requiere heap allocation. Con 2KB de RAM, una lista de 10 `i64` (80 bytes + overhead del Vec) ya consume ~200 bytes.
- **LLVM codegen:** El compilador de Kyle genera código vía LLVM. LLVM no tiene backend para AVR de 8 bits (el target AVR de LLVM es para ATmega modernos con >32KB flash, y aun así es experimental).

### ¿Serviría Kyle para el Uno aunque el runtime fuera mínimo?
No. El lenguaje Kyle asume:
- Enteros de 32/64 bits (`i32`, `i64`)
- Strings heap-allocated
- Recolector basado en ownership (no GC, pero requiere free)
- Funciones con múltiples returns (tuplas)
- Indirección de llamadas (closures, function pointers)
- LLVM como backend

Todo esto es incompatible con un MCU de 8 bits con 2KB de RAM.

---

## ¿Qué se necesitaría para que Kyle funcione en embebidos?

### Opción A: Targets grandes (ARM Cortex-M, RISC-V, ESP32)

Estos tienen:
- 32-bit CPU @ 80-480 MHz
- 256KB - 16MB Flash
- 32KB - 8MB RAM
- MMU opcional (depende del chip)

**Lo que haría falta:**
1. **Runtime sin `std`:** Reimplementar `kyc_runtime` en `#![no_std]` + alloc (usando `alloc::vec::Vec` con un allocador global). Esto es posible hoy.
2. **Backend LLVM:** Kyle ya usa LLVM. Los targets ARM, RISC-V, Xtensa (ESP32) tienen soporte en LLVM. Solo habría que pasar el triple correcto.
3. **Linker script:** Para generar binarios desnudos (`.elf` → `.bin`).
4. **Startup code:** Vector table, init de .bss/.data, handler de interrupts.

**Tiempo estimado:** ~1-2 semanas para un port básico a ESP32.

### Opción B: Targets chicos (AVR de 8 bits, Arduino Uno, ATtiny)

Aquí LLVM no llega o es experimental. Habría que:
1. Usar un backend alternativo (ej: compilar Kyle a C con ky2c.ky, luego compilar con `avr-gcc`).
2. Reimplementar TODO el runtime en C bare-metal sin alloc.
3. Eliminar features del lenguaje que requieran heap (listas, dicts, sets, strings dinámicos).
4. Restringir enteros a 8/16 bits.

**Tiempo estimado:** Meses, y el resultado sería un Kyle irreconocible.

---

## ¿Hay planes para esto?

No. El roadmap de Kyle prioriza:
1. Completar el lenguaje para systems programming en desktop/server
2. Self-hosting (Kyle compilando Kyle)
3. UI framework

El soporte embebido sería **FASE 6+** si hay interés. Pero el runtime sin `std` para ARM Cortex-M es un paso factible a corto plazo si alguien lo impulsa.

---

## Conclusión

| Dispositivo | ¿Kyle hoy? | ¿Posible? | Esfuerzo |
|-------------|-----------|-----------|----------|
| Arduino Uno (AVR 8-bit) | ❌ | Técnicamente posible via ky2c → C → avr-gcc, pero impráctico | Muy alto |
| ESP32 (Xtensa 32-bit) | ❌ | ✅ Con runtime no-std + backend LLVM | 1-2 semanas |
| Raspberry Pi Pico (ARM Cortex-M0) | ❌ | ✅ Similar al ESP32 | 1-2 semanas |
| STM32 (ARM Cortex-M4) | ❌ | ✅ Similar al ESP32 | 1-2 semanas |
| Linux embebido (ARM Cortex-A) | ✅ | Ya funciona (es solo Linux) | 0 |
