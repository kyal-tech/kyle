// =============================================================================
//  Kyle UI — Reactivity System
//  Proxy-based reactive state with one-way and two-way binding.
// =============================================================================

export class ReactiveState {
    constructor(initial = {}) {
        this._watchers = new Map();   // key → Set<callback>
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
    return {
        type: domEvent.type,
        target: domEvent.target,
        x: domEvent.clientX ?? 0,
        y: domEvent.clientY ?? 0,
        key: domEvent.key ?? '',
        ctrl_key: domEvent.ctrlKey ?? false,
        shift_key: domEvent.shiftKey ?? false,
        alt_key: domEvent.altKey ?? false,
        meta_key: domEvent.metaKey ?? false,
        button: domEvent.button ?? 0,
        prevent_default: () => domEvent.preventDefault(),
        stop_propagation: () => domEvent.stopPropagation(),
    };
}

// Global fallback for direct script inclusion
if (typeof window !== 'undefined') {
    window.ReactiveState = ReactiveState;
    window.Binding = Binding;
    window.createKyleEvent = createKyleEvent;
}
//# sourceURL=reactivity.js
