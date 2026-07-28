# Server-Side Rendering (SSR)

**Status:** Draft v1.0
**Date:** 2026-07-10
**Documentación relacionada:**
- [routing.md](routing.md) — Routing
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target
- [i18n.md](i18n.md) — Internacionalización en SSR

---

## 1. Visión General

Kyle soporta SSR para el target web. El mismo componente `.kyx` se renderiza
en el servidor (HTML) y luego se hidrata en el cliente (JS interactivo).

```
Browser solicita /dashboard
    │
    ▼
Servidor Kyle
    │
    ├── Renderiza <dashboard_view /> a HTML string
    ├── Inyecta estado inicial como JSON (serializado)
    ├── Incluye <script> con ui-runtime.js
    └── Responde con HTML completo
            │
            ▼
Browser recibe HTML (visible inmediatamente)
    │
    ▼
Carga WASM + ui-runtime.js
    │
    ▼
Hidratación: eventos, binding, estado reactivo
```

---

## 2. SSR Automático

Por defecto, las vistas `.kyx` registradas en el `<router>` son SSR automático al
compilar para web con flag `--ssr`:

```bash
ky build app.kyx --target web --ssr
```

Esto genera:

```
dist/
├── index.html           # HTML renderizado (SSR)
├── ui-runtime.js        # Runtime JS interactivo
├── app.wasm             # Lógica Kyle compilada
└── pages/
    ├── home.wasm        # Lazy-loaded pages
    └── dashboard.wasm
```

---

## 3. SSR por Componente

### 3.1 SSR completo (default)

Todo el árbol se renderiza en servidor:

```kyx
<app>
    <error_boundary>      # SSR
        <home_view />     # SSR
    </error_boundary>
</app>
```

### 3.2 SSR deshabilitado (client-only)

```kyx
<dashboard_view ssr=false />
```

El servidor renderiza un placeholder. El contenido se genera en cliente:

```html
<div id="dashboard-placeholder">
    <!-- Cargando dashboard... -->
</div>
```

### 3.3 SSR parcial

```kyx
<view>
    <header>              # SSR
        <nav_items ssr=true />
    </header>
    <main>                # SSR
        <user_profile />  # SSR (contiene datos públicos)
        <user_posts ssr=false />  # Client-only (carga bajo demanda)
    </main>
    <footer ssr=true />   # SSR
</view>
```

---

## 4. Estado Inicial (Hydration)

El servidor serializa el estado inicial para que el cliente lo retome:

```javascript
// Generado automáticamente en el HTML
<script>
window.__KYLE_INITIAL_STATE__ = {
    user: { id: 42, name: "Juan" },
    theme: "dark",
    locale: "es",
    route: "/dashboard",
};
</script>
```

### 4.1 Serialización automática

El compilador detecta qué variables de estado se usan en SSR y las serializa:

```kyx
@(
    user: User            # → se serializa automáticamente
    theme: Theme          # → se serializa automáticamente
    local_count: ^i32 = 0 # → NO se serializa (solo cliente)
)
```

### 4.2 Exclusiones

```kyx
@(
    session: Session @no_ssr    # No serializar (datos sensibles)
    api_key: str @no_ssr        # No exponer en HTML
)
```

---

## 5. Ciclo de Vida SSR vs Cliente

| Hook | Servidor | Cliente |
|------|:--------:|:-------:|
| `on_created` | ✅ | ✅ |
| `on_mounted` | ❌ | ✅ |
| `on_updated` | ❌ | ✅ |
| `on_unmounted` | ❌ | ✅ |
| `on_error` | ✅ | ✅ |

### 5.1 Código condicional

```kyle
fn load_data():
    if is_server():
        # Cargar datos en SSR (fetch desde servidor)
        fetch_from_database()
    else:
        # Cargar datos en cliente (API call)
        fetch("/api/data")
```

### 5.2 Hooks SSR-only

```kyle
fn on_server_init():
    # Se llama solo en SSR, antes de renderizar
    # Ideal para fetch de datos iniciales
    data = fetch_from_database()
```

---

## 6. Data Fetching en SSR

### 6.1 Fetch durante SSR

```kyx
@(
    user: User? = None

    fn on_server_init():
        user = fetch_user_from_db(params.id)
)

<view>
    @if user != None:
        <text value=user.name />
    @else:
        <text value="Usuario no encontrado" />
</view>
```

### 6.2 Cache de datos

El servidor mantiene un cache de fetchs durante SSR para evitar duplicados:

```kyle
fn fetch_user(id: i32) User:
    # El runtime SSR cachea automáticamente
    # Si dos componentes piden el mismo user, solo un fetch
    database.query("SELECT * FROM users WHERE id = ?", id)
```

### 6.3 Timeout SSR

```kyle
fn on_server_init():
    # Timeout de 5s para SSR
    data = fetch_with_timeout("/api/data", 5000)
```

Si el timeout expira, el servidor renderiza con los datos parciales.

---

## 7. Streaming SSR

Para páginas grandes, el servidor puede hacer streaming del HTML:

```bash
ky build app.kyx --target web --ssr=stream
```

```
Browser:
    │
    ▼
<head>...</head>                              ← inmediato
<div id="app">                                ← inmediato
  <header>...</header>                        ← 100ms
  <main>                                      ← 100ms
    <sidebar>...</sidebar>                    ← 200ms (carga datos)
    <content>...</content>                    ← 500ms (carga datos)
  </main>
  <footer>...</footer>                        ← 600ms
</div>
<script>...</script>                          ← 600ms
```

### 7.1 Suspense boundaries

```kyx
<view>
    <header />                                # flush inmediato
    <suspense fallback=<spinner />>
        <slow_component />                     # flush cuando esté listo
    </suspense>
    <footer />                                # flush inmediato
</view>
```

---

## 8. SEO

### 8.1 Meta tags dinámicos

```kyx
@(
    title: str = "Dashboard"
    description: str = "Panel de control"
)

<head>
    <title value=@title />
    <meta name="description" content=@description />
    <meta property="og:title" content=@title />
    <meta property="og:description" content=@description />
</head>
```

### 8.2 Sitemap

Generación automática de `sitemap.xml` basado en las rutas declaradas:

```xml
<!-- Generado automáticamente -->
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url><loc>https://example.com/</loc><priority>1.0</priority></url>
    <url><loc>https://example.com/login</loc><priority>0.5</priority></url>
    <url><loc>https://example.com/dashboard</loc><priority>0.8</priority></url>
</urlset>
```

---

## 9. Compilación por Target

### 9.1 Kyle SSR Server

```bash
ky run web app.kyx --port 3000
```

Inicia un servidor HTTP que:
1. Escucha requests
2. Resuelve la ruta
3. Renderiza el componente a HTML
4. Responde con HTML completo + estado inicial + scripts

### 9.2 Integración con Node.js

```javascript
// server.js — generado automáticamente por ky build --ssr
import { render } from './ssr-runtime.js';

app.get('/*', (req, res) => {
    const html = render(req.path, {
        url: req.url,
        headers: req.headers,
        cookies: req.cookies,
    });

    res.send(html);
});
```

### 9.3 Integración con cualquier backend

El SSR de Kyle produce HTML puro. Puede integrarse con:

- **Node.js** (Express, Fastify, Next.js)
- **Python** (FastAPI, Django)
- **Go** (net/http)
- **Rust** (Axum, Actix)
- **Cualquier HTTP server** via subprocess

```bash
# CLI mode: genera HTML para una ruta
ky render app.kyx /dashboard > dashboard.html
```

---

## 10. Rendimiento SSR

| Técnica | Mejora |
|---------|--------|
| **Streaming** | TTFB reducido (primeros bytes inmediatos) |
| **Cache de componentes** | Componentes sin props dinámicos se cachean |
| **Cache de fetchs** | Datos duplicados = un solo fetch |
| **Lazy SSR** | Componentes pesados se difieren |
| **Compresión** | HTML comprimido con gzip/brotli |
| **Inline crítico** | CSS crítico inlined, el resto lazy |

---

## 11. Buenas Prácticas

| Práctica | Descripción |
|----------|-------------|
| **SSR para rutas públicas** | Home, landing, blog, docs |
| **Client-only para dashboards** | Apps internas no necesitan SSR |
| **No datos sensibles en SSR** | `@no_ssr` en tokens, passwords |
| **Streaming para páginas grandes** | Mejora perceived performance |
| **Suspense para datos lentos** | No bloquees el SSR completo |
| **Cachea lo que puedas** | Componentes puramente presentacionales |

---

## 12. Referencias

- [routing.md](routing.md) — Routing y lazy loading
- [RFC-0003](../../10-design/rfc/0003-ui-translation.md) — Traducción multi-target
- [i18n.md](i18n.md) — i18n en SSR
