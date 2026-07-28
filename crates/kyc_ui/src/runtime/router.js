// =============================================================================
//  Kyle UI — Router
//  Client-side router with History API, route params, guards, lazy loading.
// =============================================================================

export class Route {
    constructor(pattern, view, options = {}) {
        this.pattern = pattern;
        this.view = view;           // component function or lazy loader
        this.options = options;
        this._paramNames = [];
        this._regex = this._compilePattern(pattern);
    }

    // Compile "/users/{id}" → /^\/users\/([^\/]+)$/
    // Special case: "*" → /^.*$/ (catch-all wildcard)
    _compilePattern(pattern) {
        this._paramNames = [];
        // Wildcard catch-all
        if (pattern === '*' || pattern === '/*') {
            return new RegExp('^.*$');
        }
        const parts = pattern.replace(/\*$/, '.*').split(/\{(\w+)\}/);
        let regexStr = '^';
        for (let i = 0; i < parts.length; i++) {
            if (i % 2 === 0) {
                regexStr += parts[i].replace(/[.+^${}()|[\]\\]/g, '\\$&');
            } else {
                this._paramNames.push(parts[i]);
                regexStr += '([^/]+)';
            }
        }
        // Optional trailing slash
        regexStr += '\\/?$';
        return new RegExp(regexStr);
    }

    match(path) {
        const m = path.match(this._regex);
        if (!m) return null;
        const params = {};
        for (let i = 0; i < this._paramNames.length; i++) {
            params[this._paramNames[i]] = decodeURIComponent(m[i + 1]);
        }
        return params;
    }

    async resolve() {
        if (typeof this.view === 'function') return this.view;
        if (typeof this.view === 'string') {
            // Lazy loading: dynamic import
            const mod = await import(this.view);
            return mod.render || mod.default;
        }
        return this.view;
    }
}

export class Router {
    constructor(options = {}) {
        this.routes = [];
        this.currentPath = '/';
        this.currentParams = {};
        this.currentView = null;
        this.container = options.container || document.getElementById('app');
        this.layout = options.layout || null;
        this._beforeEnterGuards = [];
        this._beforeLeaveGuards = [];
        this._listeners = [];

        // Handle popstate (back/forward)
        window.addEventListener('popstate', (e) => {
            const path = e.state?.path || this._getLocationPath();
            this._navigate(path, { replace: true, fromPopState: true });
        });
    }

    // Register a route
    register(pattern, view, options = {}) {
        this.routes.push(new Route(pattern, view, options));
        // Sort routes: literal paths first, then params, then wildcard
        this.routes.sort((a, b) => {
            const aScore = a.pattern.includes('{') ? 1 : a.pattern === '*' ? 2 : 0;
            const bScore = b.pattern.includes('{') ? 1 : b.pattern === '*' ? 2 : 0;
            return aScore - bScore;
        });
    }

    // Register multiple routes at once
    registerAll(routeMap) {
        for (const [pattern, view] of Object.entries(routeMap)) {
            this.register(pattern, view);
        }
    }

    // Add guard
    beforeEach(guard) { this._beforeEnterGuards.push(guard); }
    beforeLeave(guard) { this._beforeLeaveGuards.push(guard); }

    // Navigate to a path
    navigate(path, options = {}) {
        return this._navigate(path, options);
    }

    // Internal navigate
    async _navigate(path, options = {}) {
        // Run beforeLeave guards
        for (const guard of this._beforeLeaveGuards) {
            const result = await guard({ from: this.currentPath, to: path });
            if (result === false) return;
        }

        // Match route
        const { route, params } = this._matchRoute(path);
        if (!route) {
            console.warn(`Route not found: ${path}`);
            return this._handleNotFound(path);
        }

        // Run beforeEnter guards
        for (const guard of this._beforeEnterGuards) {
            const result = await guard({ from: this.currentPath, to: path, params });
            if (result === false) return;
            if (typeof result === 'string') {
                // Redirect
                return this.navigate(result);
            }
        }

        // Update history
        if (!options.replace && !options.fromPopState) {
            window.history.pushState({ path }, '', path);
        }

        // Resolve view component
        const viewFn = await route.resolve();
        this.currentPath = path;
        this.currentParams = params;

        // Render
        if (this.container) {
            this.container.innerHTML = '';
            if (typeof viewFn === 'function') {
                const result = viewFn(params);
                if (result instanceof HTMLElement) {
                    this.container.appendChild(result);
                } else if (result?.element) {
                    this.container.appendChild(result.element);
                }
            }
        }

        // Fire route change listeners
        for (const listener of this._listeners) {
            listener({ path, params, route: route.pattern });
        }
    }

    // Match a path against registered routes
    _matchRoute(path) {
        const cleanPath = this._cleanPath(path);
        for (const route of this.routes) {
            const params = route.match(cleanPath);
            if (params !== null) {
                return { route, params };
            }
        }
        // Try wildcard
        for (const route of this.routes) {
            if (route.pattern === '*' || route.pattern === '/*') {
                return { route, params: { wildcard: cleanPath } };
            }
        }
        return { route: null, params: {} };
    }

    // Clean path: remove query string, normalize
    _cleanPath(path) {
        return path.split('?')[0].split('#')[0] || '/';
    }

    // Get current path from browser location
    _getLocationPath() {
        return window.location.pathname || '/';
    }

    // Handle 404
    _handleNotFound(path) {
        console.warn(`No route matched: ${path}`);
    }

    // Listen for route changes
    onChange(listener) {
        this._listeners.push(listener);
        return () => {
            this._listeners = this._listeners.filter(l => l !== listener);
        };
    }

    // Start the router: navigate to current URL
    start() {
        const path = this._getLocationPath();
        return this.navigate(path, { replace: true });
    }

    // Destroy the router
    destroy() {
        this._listeners = [];
        this.routes = [];
    }
}

// Get route params (called from Kyle components)
export function routeParams() {
    return window.__router?.currentParams || {};
}

// Navigate programmatically (called from Kyle)
export function navigate(path) {
    return window.__router?.navigate(path);
}

export function navigateBack() {
    window.history.back();
}

export function navigateReplace(path) {
    return window.__router?.navigate(path, { replace: true });
}

// Set page title from Kyle
export function setTitle(title) {
    document.title = title;
}

// Set meta tags from Kyle
export function setMeta(name, content) {
    let el = document.querySelector(`meta[name="${name}"]`);
    if (!el) {
        el = document.createElement('meta');
        el.setAttribute('name', name);
        document.head.appendChild(el);
    }
    el.setAttribute('content', content);
}

// =============================================================================
// Auto-routing: scan .kyx view() declarations
// =============================================================================

export function createRouter(options = {}) {
    const router = new Router(options);
    window.__router = router;

    // Register routes from auto-routing declarations
    const routes = window.__KYLE_ROUTES || {};
    router.registerAll(routes);

    return router;
}

// Global fallback for direct script inclusion
// Snake_case aliases for Kyle-generated code
// navigate is already exported by name; alias the rest
export { setTitle as set_title, setMeta as set_meta, navigateBack as navigate_back, navigateReplace as navigate_replace };

if (typeof window !== 'undefined') {
    window.Router = Router;
    window.createRouter = createRouter;
    window.routeParams = routeParams;
    window.navigate = navigate;
    window.navigateBack = navigateBack;
    window.navigateReplace = navigateReplace;
    window.setTitle = setTitle;
    window.setMeta = setMeta;
    window.set_title = setTitle;
    window.set_meta = setMeta;
}
