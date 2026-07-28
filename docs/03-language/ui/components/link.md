# link — Enlace de Navegación

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class link:
    # Navegación
    href: str                       # URL o ruta
    target: link_target = link_target.self  # Cómo abrir
    
    # Contenido
    text: str?                      # Texto del enlace
    
    # Configuración
    external: bool = false          # Enlace externo (abre en nueva pestaña)
    
    # Eventos
    click: fn(event: click_event)?
    
    # Accesibilidad
    aria_label: str?
```

---

## Uso Básico

### Enlace interno (navegación de la app)

```kyx
<link href="/about" text="Acerca de" />
```

### Enlace externo

```kyx
<link href="https://kyle.ai" text="Sitio web" external=true />
```

### Con contenido personalizado

```kyx
<link href="/profile">
    <hstack spacing=8>
        <img src=@user.avatar width=32 height=32 />
        <text value=@user.name />
    </hstack>
</link>
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `href` | `str` | — | **Requerido.** URL o ruta |
| `target` | `link_target` | `self` | Cómo abrir |
| `text` | `str?` | `none` | Texto del enlace |
| `external` | `bool` | `false` | Enlace externo |
| `click` | `fn(event: click_event)?` | `none` | Al hacer click |

---

## Tipos Relacionados

### link_target

```kyle
enum link_target:
    self            # Misma ventana/pestaña
    blank           # Nueva ventana/pestaña
    parent          # Ventana padre
    top             # Ventana superior
```

---

## Ejemplos Avanzados

### Enlace con parámetros

```kyx
<link href=@"/users/" + user.id.to_str() text="Ver perfil" />
```

### Navegación programática

```kyx
<view>
    @(
        fn handle_click(event: click_event):
            if user.is_admin:
                navigate("/admin")
            else:
                navigate("/dashboard")
    )
    
    <link href="/dashboard" click=@handle_click text="Ir al dashboard" />
</view>
```

### Enlace con estilo

```kyx
<link href="/docs" style=style(
    color: color("#0066FF"),
    text_decoration: text_decoration.underline
) text="Documentación" />
```

---

## Traducción Multiplataforma

### Web

```html
<a href="/about">Acerca de</a>
<a href="https://kyle.ai" target="_blank" rel="noopener">Sitio web</a>
```

### Desktop (SDL2)

```kyle
fn render_link(x: f32, y: f32):
    skia_draw_text(x, y, "Acerca de", color("#0066FF"))
    
    if mouse_in_rect(x, y, text_width, text_height) and mouse_clicked():
        navigate("/about")
```

### iOS (SwiftUI)

```swift
NavigationLink(destination: AboutView()) {
    Text("Acerca de")
}

Link("Sitio web", destination: URL(string: "https://kyle.ai")!)
```

### Android (Jetpack Compose)

```kotlin
// Interno
TextButton(onClick = { navController.navigate("about") }) {
    Text("Acerca de")
}

// Externo
ClickableText(
    text = AnnotatedString("Sitio web"),
    onClick = {
        val intent = Intent(Intent.ACTION_VIEW, Uri.parse("https://kyle.ai"))
        context.startActivity(intent)
    }
)
```

---

## Referencias

- [button](button.md)
- [routing](../routing.md)
- [Componentes nativos](README.md)
