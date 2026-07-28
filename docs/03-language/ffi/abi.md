# FFI

**Status:** [x] `@link "c"`, `extern fn` declarations, `ptr` type for raw pointers. — Foreign Function Interface

**Status:** Under development

## extern fn

Declaris a function implemented in an external library (C ABI).

```ky
extern fn socket(domain: i32, type_: i32, protocol: i32) i32
extern fn connect(sock: i32, addr: ptr, addrlen: i32) i32
extern fn send(sock: i32, buf: ptr, len: i64, flags: i32) i64
```

The compiler generatis a declaration without a body, using the C calling convention.

## @link

Specifiis a native library to link against.

```ky
@link "libcurl"
@link "libssl"
@link "libsqlite3"
```

## ptr type

Raw pointer for FFI and unsafe operations.

```ky
buf: ptr = my_alloc(1024)
value = buf[0] as i8 # load byte
buf[4] = 0xFF as i8 # store byte
next = buf + 16 # pointer arithmetic
```

## Example

```ky
@link "libcurl"

extern fn curl_easy_init() ptr
extern fn curl_easy_setopt(handle: ptr, option: i32, value: ptr) i32
extern fn curl_easy_perform(handle: ptr) i32
extern fn curl_easy_cleanup(handle: ptr)

fn http_get(url: str) str:
 curl = curl_easy_init()
 # ... configure and perform ...
 curl_easy_cleanup(curl)
 response
```

## Safety

`extern fn` calls must be used inside `unsafe` blocks, unless wrapped in a safe function.
