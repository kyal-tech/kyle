# img — Imagen

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class img:
    # Contenido
    src: str                          # URL o ruta local
    alt: str = ""                     # Texto alternativo (accesibilidad)
    
    # Dimensiones
    width: length?                    # Ancho
    height: length?                   # Alto
    object_fit: object_fit = object_fit.cover  # Cómo ajustar la imagen
    
    # Lazy loading
    lazy: bool = false                # Cargar solo cuando sea visible
    placeholder: placeholder_type?    # Qué mostrar mientras carga
    
    # Fallback
    fallback: str?                    # Imagen de respaldo si falla
    
    # Eventos
    on_load: fn()?                    # Cuando carga correctamente
    on_error: fn()?                   # Cuando falla la carga
    
    # Accesibilidad
    aria_label: str?                  # Label para screen readers
    aria_hidden: bool = false         # Ocultar de screen readers
```

---

## Uso Básico

### URL externa

```kyx
<img src="https://example.com/photo.jpg" alt="Foto de perfil" width=200 height=150 />
```

### Archivo local

```kyx
<img src="./assets/logo.png" alt="Logo" />
```

### Desde state (reactivo)

```kyx
<view>
    @(
        user: ^User = User()
    )
    
    <img src=@user.avatar alt=@"Avatar de " + user.name />
</view>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `src` | `str` | — | **Requerido.** URL o ruta local de la imagen |
| `alt` | `str` | `""` | Texto alternativo para accesibilidad |
| `width` | `length?` | `none` | Ancho de la imagen |
| `height` | `length?` | `none` | Alto de la imagen |
| `object_fit` | `object_fit` | `cover` | Cómo ajustar la imagen al contenedor |
| `lazy` | `bool` | `false` | Cargar solo cuando sea visible |
| `placeholder` | `placeholder_type?` | `none` | Qué mostrar mientras carga |
| `fallback` | `str?` | `none` | Imagen de respaldo si falla |
| `on_load` | `fn()?` | `none` | Callback cuando carga |
| `on_error` | `fn()?` | `none` | Callback cuando falla |

---

## Tipos Relacionados

### object_fit

```kyle
enum object_fit:
    cover       # Llena el contenedor, puede recortar
    contain     # Muestra completa, puede haber espacio
    fill        # Estira para llenar (distorsiona)
    none        # Tamaño original
    scale_down  # Como contain, pero nunca más grande que original
```

### placeholder_type

```kyle
enum placeholder_type:
    skeleton                    # Skeleton loader (animado)
    color(c: color)             # Color sólido
    blur_hash(hash: str)        # BlurHash (imagen borrosa)
    component(c: view)          # Componente personalizado
```

### length

```kyle
enum length:
    px(f32)                     # Píxeles
    percent(f32)                # Porcentaje del contenedor
    auto                        # Automático
    fill                        # Llena el espacio disponible
```

---

## Ejemplos Avanzados

### Lazy loading con skeleton

```kyx
<img 
    src=@user.avatar 
    lazy=true 
    placeholder=placeholder_type.skeleton 
    width=100 
    height=100 
/>
```

### Con fallback

```kyx
<img 
    src=@photo_url 
    fallback="./assets/default-avatar.png" 
    alt="Avatar" 
/>
```

### Galería de imágenes

```kyx
<view>
    @(
        images: ^{str} = {"./assets/1.jpg", "./assets/2.jpg", "./assets/3.jpg"}
    )
    
    <hstack spacing=8>
        @for(url in images):
            <img src=@url width=150 height=150 object_fit=object_fit.cover />
    </hstack>
</view>
```

### Imagen con overlay

```kyx
<zstack>
    <img src="./assets/hero.jpg" width=800 height=400 />
    <view style=style(background: color("#000000").with_alpha(0.5))>
        <text value="Título sobre imagen" style=style(color: color("#FFFFFF")) />
    </view>
</zstack>
```

### Responsive

```kyx
<img 
    src="./assets/banner.jpg" 
    width=length.percent(100) 
    height=length.auto 
    object_fit=object_fit.contain 
/>
```

---

## Traducción Multiplataforma

### Web

```html
<img src="https://example.com/photo.jpg" alt="Foto" width="200" height="150" loading="lazy" />
```

### Desktop (SDL2/Skia)

```kyle
fn render_img(x: f32, y: f32, w: f32, h: f32):
    texture = skia_load_texture("./assets/photo.jpg")
    skia_draw_texture(texture, x, y, w, h)
```

### iOS (SwiftUI)

```swift
AsyncImage(url: URL(string: "https://example.com/photo.jpg")) { image in
    image.resizable()
} placeholder: {
    ProgressView()
}
.frame(width: 200, height: 150)
```

### Android (Jetpack Compose)

```kotlin
AsyncImage(
    model = "https://example.com/photo.jpg",
    contentDescription = "Foto",
    modifier = Modifier.size(200.dp, 150.dp),
    contentScale = ContentScale.Crop
)
```

---

## Errores Comunes

### ❌ Malo: No especificar alt

```kyx
<img src="./assets/logo.png" />
```

**Problema:** No accesible para screen readers.

### ✅ Bueno: Siempre especificar alt

```kyx
<img src="./assets/logo.png" alt="Logo de la empresa" />
```

### ❌ Malo: Dimensiones fijas en responsive

```kyx
<img src="./assets/banner.jpg" width=1920 height=1080 />
```

**Problema:** No se adapta a pantallas pequeñas.

### ✅ Bueno: Usar percent o fill

```kyx
<img src="./assets/banner.jpg" width=length.percent(100) height=length.auto />
```

---

## Referencias

- [Arquitectura multiplataforma](../architecture.md)
- [Sistema de estilos](../style-system.md)
- [Componentes nativos](README.md)
