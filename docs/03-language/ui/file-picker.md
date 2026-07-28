# File Picker — Selector de Archivos Nativo

**Status:** Draft v1.0
**Date:** 2026-07-12

---

## 1. Filosofía

El `<file_picker>` es un componente nativo que abre el selector de archivos del sistema operativo. No es un `<input>` genérico — es un componente específico con semántica propia que se traduce a:

| Plataforma | Equivalente nativo |
|-----------|-------------------|
| **web** | `<input type="file">` + FileReader API |
| **macOS** | `NSOpenPanel` / `NSSavePanel` |
| **windows** | `IFileOpenDialog` (COM) |
| **linux** | `GtkFileChooserNative` |
| **ios** | `UIDocumentPickerViewController` |
| **android** | `ActivityResultContracts.GetContent()` |

---

## 2. Tipo `file_data`

`file_data` es un **tipo nativo** de Kyle. Se puede usar en clases, modelos de formulario, y como tipo de parámetro.

### 2.1 Definición del tipo

```kyle
final class file_data:
    name: str              # nombre del archivo (e.g. "foto.jpg")
    size: i64              # tamaño en bytes
    mime: str              # tipo MIME (e.g. "image/jpeg")
    content: bytes         # contenido binario del archivo (para enviar a backend)
    last_modified: i64     # timestamp UNIX de última modificación
```

### 2.2 Uso en clases

```kyx
final class UserForm:
    name: str = ""
    email: str = ""
    photo: file_data?       # ← file_data como tipo de campo
    age: i32 = 0
```

```kyx
final class UploadResult:
    file: file_data
    uploaded_at: i64

    fn summary() str:
        file.name + " (" + file.size.to_str() + " bytes)"
```

### 2.3 Acceso a campos

```kyx
<view>
    @(
        f: file_data       # ← variable de tipo file_data
    )
    <file_picker on_select=@fn (file):
        f = file           # ← file es de tipo file_data
        print(f.name)      # → "foto.jpg"
        print(f.size)      # → 102400
        print(f.mime)      # → "image/jpeg"
        print(f.content)   # → bytes (binario)
    />
</view>
```

---

## 3. Componente `<file_picker>`

### 3.1 Selector básico

```kyx
<file_picker on_select=@fn (file: file_data):
    print("Archivo seleccionado: " + file.name)
/>
```

### 3.2 Filtro por tipo

```kyx
# Solo imágenes
<file_picker accept="image/*" on_select=@handle_image />

# Extensiones específicas
<file_picker accept=".pdf,.doc,.docx" on_select=@handle_doc />

# Múltiples tipos
<file_picker accept="image/*,.pdf" on_select=@handle_file />
```

### 3.3 Múltiples archivos

```kyx
<file_picker multiple=true on_select=@fn (files: {file_data}):
    for file in files:
        print(" - " + file.name)
/>
```

### 3.4 Modo carpeta

```kyx
<file_picker mode=directory on_select=@fn (files: {file_data}):
    print("Archivos en carpeta: " + files.len().to_str())
/>
```

### 3.5 Con vista previa

```kyx
<view>
    @(
        preview: ^str?
        fn handle_file(file: file_data):
            if file.mime.starts_with("image/"):
                preview = URL.create_object_url(file.content)
    )

    <file_picker accept="image/*" on_select=@handle_file />
    @if preview != None:
        <image src=@preview width=300 />
</view>
```

---

## 4. Props del componente

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `accept` | `str` | `""` | Tipos MIME o extensiones aceptadas |
| `multiple` | `bool` | `false` | Permitir selección múltiple |
| `mode` | `str` | `"file"` | `"file"`, `"directory"`, `"save"` |
| `on_select` | `fn (file_data)` | – | Callback al seleccionar |
| `on_select_multi` | `fn ({file_data})` | – | Callback para múltiples archivos |
| `label` | `str` | `"Seleccionar archivo"` | Texto del botón |

---

## 5. Uso con HTTP upload

```kyx
<view>
    @(
        fn upload(file: file_data):
            loading = true
            http.post("/api/upload", form: {
                "file": file
            })
            loading = false
    )

    <file_picker accept="image/*,.pdf" label="Subir archivo"
        on_select=@upload />
</view>
```

---

## 6. Implementación por plataforma

### 6.1 Web

```javascript
// Generado automáticamente
function createFilePicker(accept, multiple, onSelect) {
    const input = document.createElement('input');
    input.type = 'file';
    if (accept) input.accept = accept;
    if (multiple) input.multiple = true;

    input.addEventListener('change', async () => {
        const files = [];
        for (const file of input.files) {
            const content = await file.arrayBuffer();
            files.push({
                name: file.name,
                size: file.size,
                mime: file.type,
                content: new Uint8Array(content),
                last_modified: file.lastModified,
            });
        }
        onSelect(multiple ? files : files[0]);
    });

    input.click();
}
```

### 6.2 Desktop (macOS)

```kyle
@link "-framework AppKit"
extern fn NSOpenPanel_runModal() i32
extern fn NSOpenPanel_URL() ptr

fn show_open_panel(accept: str) file_data:
    # Usa NSOpenPanel vía FFI
    panel = NSOpenPanel_alloc()
    NSOpenPanel_setAllowsMultipleSelection(panel, false)
    NSOpenPanel_setCanChooseFiles(panel, true)
    result = NSOpenPanel_runModal(panel)
    if result == 1:  # NSModalResponseOK
        url = NSOpenPanel_URL(panel)
        path = url_to_path(url)
        content = read_file(path)
        file_data(name: basename(path), content: content, ...)
```

### 6.3 iOS

```swift
// Generado automáticamente en ContentView.swift
struct FilePickerView: UIViewControllerRepresentable {
    var onSelect: (FileData) -> Void

    func makeUIViewController(context: Context) -> UIDocumentPickerViewController {
        let picker = UIDocumentPickerViewController(forOpeningContentTypes: [.image])
        picker.delegate = context.coordinator
        return picker
    }
}
```

---

## 7. Referencias

- [ui-syntax.md](../syntax/ui-syntax.md) — Sintaxis .kyx
- [state-events.md](state-events.md) — Estado y eventos
- [routing.md](routing.md) — Routing/Navegación
- [RFC-0005](../../10-design/rfc/0005-ui-rearchitecture-plan.md) — Arquitectura UI
