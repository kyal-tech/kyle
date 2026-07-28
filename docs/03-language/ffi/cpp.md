# C++ Interop

> Interoperabilidad with C++.
> **Pending de implementation.** Actualmente Kyle solo supports C FFI directo.

## Status current

Kyle can llamar functions C (via `extern "C"`) pero NO can llamar functions
C++ directamente. Para usar libraries C++, necesitas un wrapper C.

## Workaround: wrapper C

```cpp
// cpp_lib_wrapper.cpp
extern "C" {
 #include "cpp_lib.h"
 // Wrapper functions that call C++ internally
 void* create_object() { return new CppClass(); }
 void destroy_object(void* obj) { delete static_cast<CppClass*>(obj); }
 int call_method(void* obj, int arg) {
 return static_cast<CppClass*>(obj)->method(arg);
 }
}
```

```ky
@link "cpp_lib_wrapper"
extern fn create_object() ptr
extern fn destroy_object(obj: ptr)
extern fn call_method(obj: ptr, arg: i32) i32
```

## Limitations

| Aspecto | Support |
|---------|---------|
| C `extern "C"` | ✅ Completo |
| C++ name mangling | ❌ No supportsdo |
| C++ classis directo | ❌ No supportsdo |
| C++ exceptions | ❌ No supportsdo |
| C++ templatis | ❌ No supportsdo |
| Wrapper C manual | ✅ Funciona |

## See also

- `abi.md` — ABI y calling convention
- `native-libraries.md` — Linkear libraries nativas
