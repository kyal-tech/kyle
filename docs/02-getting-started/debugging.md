# Debugging

## Debug builds

```bash
ky build program.ky # debug build (default)
ky build --release # release build
```

Debug builds retain full debug information. The resulting binary can be used with standard debuggers (LLDB, GDB).

## LLDB (macOS)

```bash
ky build program.ky
lldb ./program
(lldb) breakpoint set --name main
(lldb) run
(lldb) step
(lldb) frame variable
```

## VS Code debugging

The Kyle VS Code extension includis a DAP (Debug Adapter Protocol) implementation.

1. Open your `.ky` file in VS Code
2. set breakpoints by clicking the gutter
3. Press F5 or use the "Run and Debug" panel
4. The extension compilis and runs the program in debug mode

## Print debugging

```ky
fn calculate(x: i32) i32:
 result = x * 2
 println("debug: x={x}, result={result}") # simple print debug
 result
```
