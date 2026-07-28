# file_picker — Selector de Archivos

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class file_picker:
    # Valor
    bind: ^file_data?               # Binding al archivo seleccionado
    field: str?                     # Nombre del campo (con modelo)
    
    # Configuración
    accept: str?                    # Tipos aceptados (ej: "image/*", ".pdf,.doc")
    multiple: bool = false          # Permitir múltiples archivos
    label: str?                     # Etiqueta
    max_size: i64?                  # Tamaño máximo en bytes
    
    # Eventos
    change: fn(event: file_change_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Selector simple

```kyx
<view>
    @(
        selected_file: ^file_data? = none
    )
    
    <file_picker bind=@selected_file label="Seleccionar archivo" />
    
    @if selected_file != none:
        <text value=@"Archivo: " + selected_file.name />
</view>
```

### Solo imágenes

```kyx
<file_picker 
    bind=@avatar 
    accept="image/*" 
    label="Foto de perfil" 
/>
```

### Múltiples archivos

```kyx
<file_picker 
    bind=@documents 
    accept=".pdf,.doc,.docx" 
    multiple=true
    label="Documentos" 
/>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `bind` | `^file_data?` | `none` | Binding al archivo seleccionado |
| `field` | `str?` | `none` | Nombre del campo (con modelo) |
| `accept` | `str?` | `none` | Tipos aceptados |
| `multiple` | `bool` | `false` | Permitir múltiples archivos |
| `label` | `str?` | `none` | Etiqueta |
| `max_size` | `i64?` | `none` | Tamaño máximo en bytes |
| `change` | `fn(event: file_change_event)?` | `none` | Al cambiar selección |

---

## Tipos Relacionados

### file_data

```kyle
final class file_data:
    name: str                       # Nombre del archivo
    size: i64                       # Tamaño en bytes
    type: str                       # MIME type
    content: bytes                  # Contenido del archivo
    last_modified: i64              # Timestamp de modificación
    preview: str?                   # URL de preview (para imágenes)
```

### file_change_event

```kyle
final class file_change_event:
    files: {file_data}              # Archivos seleccionados
    target: str                     # ID del elemento
```

---

## Ejemplos Avanzados

### Upload de imagen con preview

```kyx
<view>
    @(
        avatar: ^file_data? = none
        
        fn handle_file_change(event: file_change_event):
            if event.files.len() > 0:
                avatar = event.files[0]
    )
    
    <vstack spacing=16>
        @if avatar != none:
            <img src=@avatar.preview width=150 height=150 />
            <text value=@avatar.name />
            <text value=@"Tamaño: " + (avatar.size / 1024).to_str() + " KB" />
        
        <file_picker 
            bind=@avatar 
            accept="image/*"
            change=@handle_file_change
            label="Seleccionar imagen"
        />
    </vstack>
</view>
```

### Upload con validación

```kyx
<view>
    @(
        document: ^file_data? = none
        error: ^str? = none
        max_size = 5 * 1024 * 1024  # 5 MB
        
        fn handle_file_change(event: file_change_event):
            error = none
            if event.files.len() > 0:
                file = event.files[0]
                if file.size > max_size:
                    error = "El archivo es demasiado grande (máx 5 MB)"
                else:
                    document = file
    )
    
    <vstack spacing=8>
        <file_picker 
            bind=@document
            accept=".pdf"
            change=@handle_file_change
            label="Subir PDF"
        />
        
        @if error != none:
            <text value=@error style=Error />
        
        @if document != none:
            <text value=@"Archivo listo: " + document.name />
            <button text="Subir" style=Primary click=@upload_document />
    </vstack>
</view>
```

### Con modelo

```kyx
<form model=@profile_form>
    <text_field field="name" label="Nombre" />
    <file_picker field="avatar" accept="image/*" label="Foto de perfil" />
    <file_picker field="resume" accept=".pdf" label="CV (PDF)" />
    
    <button text="Guardar" style=Primary type="submit" />
</form>
```

---

## Traducción Multiplataforma

### Web

```html
<div>
    <label>Seleccionar archivo</label>
    <input type="file" accept="image/*" />
</div>
```

### Desktop (SDL2)

```kyle
fn show_file_picker(accept: str?) -> file_data?:
    # Usar diálogo nativo del SO
    result = sdl_show_open_dialog(accept)
    if result != none:
        content = fs_read(result.path)
        return file_data(
            name: path_basename(result.path),
            size: content.len(),
            type: guess_mime_type(result.path),
            content: content
        )
    return none
```

### iOS (SwiftUI)

```swift
.fileImporter(
    isPresented: $show_file_picker,
    allowedContentTypes: [.image],
    allowsMultipleSelection: false
) { result in
    switch result {
    case .success(let urls):
        if let url = urls.first {
            // Cargar archivo
        }
    case .failure(let error):
        print(error)
    }
}
```

### Android (Jetpack Compose)

```kotlin
val launcher = rememberLauncherForActivityResult(
    ActivityResultContracts.GetContent()
) { uri: Uri? ->
    uri?.let {
        // Cargar archivo
    }
}

Button(onClick = { launcher.launch("image/*") }) {
    Text("Seleccionar imagen")
}
```

---

## Referencias

- [form](form.md)
- [img](img.md)
- [Componentes nativos](README.md)
