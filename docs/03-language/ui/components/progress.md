# progress — Barra de Progreso

**Status:** Specification v2.0
**Date:** 2026-07-28

---

## Tipo Kyle

```kyle
final class progress:
    # Valor
    value: f32?                     # Valor actual (0.0 - max)
    max: f32 = 100.0                # Valor máximo
    
    # Configuración
    label: str?                     # Etiqueta
    show_value: bool = false        # Mostrar valor numérico
    indeterminate: bool = false     # Progreso indeterminado
    size: size = size.md            # Tamaño
    
    # Accesibilidad
    aria_label: str?
    aria_value_now: f32?
    aria_value_min: f32 = 0.0
    aria_value_max: f32 = 100.0
```

---

## Uso Básico

### Progreso determinado

```kyx
<view>
    @(
        upload_progress: ^f32 = 45.0
    )
    
    <progress value=@upload_progress max=100.0 show_value=true />
</view>
```

### Progreso indeterminado

```kyx
<progress indeterminate=true label="Cargando..." />
```

---

## Props

| Prop | Tipo | Default | Descripción |
|------|------|---------|-------------|
| `value` | `f32?` | `none` | Valor actual |
| `max` | `f32` | `100.0` | Valor máximo |
| `label` | `str?` | `none` | Etiqueta |
| `show_value` | `bool` | `false` | Mostrar valor numérico |
| `indeterminate` | `bool` | `false` | Progreso indeterminado |
| `size` | `size` | `md` | Tamaño |

---

## Ejemplos Avanzados

### Upload con progreso

```kyx
<view>
    @(
        upload_progress: ^f32 = 0.0
        uploading: ^bool = false
        
        fn start_upload():
            uploading = true
            for i in 0..100:
                upload_chunk(i)
                upload_progress = i as f32
            uploading = false
    )
    
    <vstack spacing=8>
        @if uploading:
            <progress value=@upload_progress max=100.0 show_value=true />
            <text value=@"Subiendo... " + upload_progress.to_str() + "%" />
        @else:
            <button text="Subir archivo" click=@start_upload />
    </vstack>
</view>
```

### Múltiples tareas

```kyx
<view>
    @(
        tasks: ^{Task} = {}
        
        fn total_progress() f32:
            if tasks.len() == 0: return 0.0
            completed = tasks.filter(fn(t): t.completed).len()
            (completed as f32 / tasks.len() as f32) * 100.0
    )
    
    <vstack spacing=8>
        <text value="Progreso general" />
        <progress value=@total_progress() max=100.0 show_value=true />
        
        @for(task in tasks):
            <hstack spacing=8>
                <text value=@task.name />
                <progress value=@task.progress max=100.0 style=style(width: length.percent(50)) />
            </hstack>
    </vstack>
</view>
```

---

## Traducción Multiplataforma

### Web

```html
<div>
    <label>Progreso: 45%</label>
    <progress value="45" max="100"></progress>
</div>
```

### iOS (SwiftUI)

```swift
ProgressView(value: upload_progress, total: 100) {
    Text("Subiendo...")
}
```

### Android (Jetpack Compose)

```kotlin
Column {
    Text("Progreso: ${upload_progress.toInt()}%")
    LinearProgressIndicator(
        progress = upload_progress / 100f
    )
}
```

---

## Referencias

- [spinner](spinner.md)
- [skeleton](skeleton.md)
- [Componentes nativos](README.md)
