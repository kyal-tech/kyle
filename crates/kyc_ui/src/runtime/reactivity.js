// =============================================================================
//  Kyle UI — Reactivity System
//  Proxy-based reactive state with one-way and two-way binding.
// =============================================================================

export class ReactiveState {
    constructor(initial = {}) {
        this._watchers = new Map();   // key → Set<callback>
        this._onAnyChange = null;     // called on ANY state change (for on_updated)
        this._batchDepth = 0;
        this._batchedUpdates = new Set();

        this._state = new Proxy(initial, {
            set: (target, key, value) => {
                const old = target[key];
                target[key] = value;
                if (old !== value) {
                    this._notify(key, value);
                }
                return true;
            },
            get: (target, key) => {
                return target[key];
            }
        });
    }

    onAnyChange(callback) {
        this._onAnyChange = callback;
    }

    // Get current state
    get state() { return this._state; }

    // Watch a key for changes
    watch(key, callback) {
        if (!this._watchers.has(key)) {
            this._watchers.set(key, new Set());
        }
        this._watchers.get(key).add(callback);
        return () => this._watchers.get(key)?.delete(callback); // unsubscribe
    }

    // Batch multiple updates together
    batch(fn) {
        this._batchDepth++;
        try { fn(); } finally {
            this._batchDepth--;
            if (this._batchDepth === 0) {
                for (const key of this._batchedUpdates) {
                    this._notifyImmediate(key, this._state[key]);
                }
                this._batchedUpdates.clear();
            }
        }
    }

    // Set a value and trigger watchers
    set(key, value) {
        this._state[key] = value;
    }

    // Get a value
    get(key) {
        return this._state[key];
    }

    _notify(key, value) {
        if (this._batchDepth > 0) {
            this._batchedUpdates.add(key);
            return;
        }
        this._notifyImmediate(key, value);
    }

    _notifyImmediate(key, value) {
        const watchers = this._watchers.get(key);
        if (watchers) {
            for (const cb of watchers) {
                try { cb(value); } catch (e) { console.warn('Watcher error:', e); }
            }
        }
        // Trigger on_updated with changed key
        if (this._onAnyChange) {
            try { this._onAnyChange(key); } catch (e) { console.warn('Lifecycle error:', e); }
        }
    }
}

// =============================================================================
//  Binding Helpers
// =============================================================================

export class Binding {
    // One-way binding: state → UI
    static oneWay(el, prop, state, key) {
        const update = (value) => {
            if (prop === 'textContent' || prop === 'innerText') {
                el.textContent = value ?? '';
            } else if (prop === 'value') {
                if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT') {
                    if (el.value !== value) el.value = value ?? '';
                }
            } else if (prop === 'checked') {
                el.checked = !!value;
            } else if (prop === 'disabled') {
                el.disabled = !!value;
            } else if (prop === 'class' || prop === 'className') {
                el.className = value ?? '';
            } else if (prop === 'style') {
                if (typeof value === 'string') el.style.cssText = value;
                else Object.assign(el.style, value ?? {});
            } else if (prop.startsWith('data-')) {
                el.setAttribute(prop, value ?? '');
            } else {
                el[prop] = value;
            }
        };

        // Initial value
        update(state.get(key));
        // Watch for changes
        return state.watch(key, update);
    }

    // Two-way binding: state ↔ UI
    static twoWay(el, state, key, eventType = 'input') {
        // One-way: state → UI
        const unsub1 = Binding.oneWay(el, 'value', state, key);

        // UI → state
        const handler = () => {
            const val = el.type === 'checkbox' ? el.checked : el.value;
            state.set(key, val);
        };
        el.addEventListener(eventType, handler);

        // Return unsubscribe function
        return () => {
            unsub1();
            el.removeEventListener(eventType, handler);
        };
    }

    // Class binding: toggle className based on expression
    static classBinding(el, state, classMap) {
        const update = () => {
            for (const [className, expr] of Object.entries(classMap)) {
                const val = typeof expr === 'function' ? expr(state) : expr;
                if (val) {
                    el.classList.add(className);
                } else {
                    el.classList.remove(className);
                }
            }
        };
        update();
        // Watch all referenced keys
        const unsubs = [];
        for (const key of Object.keys(classMap)) {
            unsubs.push(state.watch(key, update));
        }
        return () => unsubs.forEach(fn => fn());
    }
}

// =============================================================================
//  Event Helpers
// =============================================================================

// Create a Kyle-compatible event object from a DOM event
export function createKyleEvent(domEvent) {
    const base = {
        type: domEvent.type,
        target: domEvent.target,
        key: domEvent.key ?? '',
        ctrl_key: domEvent.ctrlKey ?? false,
        shift_key: domEvent.shiftKey ?? false,
        alt_key: domEvent.altKey ?? false,
        meta_key: domEvent.metaKey ?? false,
        button: domEvent.button ?? 0,
        prevent_default: () => domEvent.preventDefault(),
        stop_propagation: () => domEvent.stopPropagation(),
    };
    // Mouse/click events
    if (domEvent.clientX !== undefined) {
        base.x = domEvent.clientX;
        base.y = domEvent.clientY;
        base.client_x = domEvent.clientX;
        base.client_y = domEvent.clientY;
        base.screen_x = domEvent.screenX ?? 0;
        base.screen_y = domEvent.screenY ?? 0;
        base.buttons = domEvent.buttons ?? 0;
    }
    // Touch events — convert TouchList to array
    if (domEvent.touches !== undefined) {
        base.touches = Array.from(domEvent.touches).map(t => ({
            identifier: t.identifier,
            x: t.clientX,
            y: t.clientY,
            force: t.force ?? 0,
            radius_x: t.radiusX ?? 0,
            radius_y: t.radiusY ?? 0,
        }));
        base.changed_touches = Array.from(domEvent.changedTouches).map(t => ({
            identifier: t.identifier,
            x: t.clientX,
            y: t.clientY,
            force: t.force ?? 0,
            radius_x: t.radiusX ?? 0,
            radius_y: t.radiusY ?? 0,
        }));
    }
    // Scroll events
    if (domEvent.scrollX !== undefined) {
        base.scroll_x = domEvent.scrollX;
        base.scroll_y = domEvent.scrollY;
    }
    return base;
}

// Track long press timers per element
const _longPressTimers = new WeakMap();

// Set up long press detection on an element
export function enableLongPress(element, callback) {
    let timer = null;
    const start = (e) => {
        timer = setTimeout(() => {
            callback(createKyleEvent(e));
            timer = null;
        }, 500);
    };
    const end = () => {
        if (timer) {
            clearTimeout(timer);
            timer = null;
        }
    };
    element.addEventListener('touchstart', start, { passive: true });
    element.addEventListener('touchend', end);
    element.addEventListener('touchmove', end);
    element.addEventListener('mousedown', start);
    element.addEventListener('mouseup', end);
    element.addEventListener('mouseleave', end);
    _longPressTimers.set(element, { start, end });
}

// Global fallback for direct script inclusion
if (typeof window !== 'undefined') {
    window.ReactiveState = ReactiveState;
    window.Binding = Binding;
    window.createKyleEvent = createKyleEvent;
}
//# sourceURL=reactivity.js
