// =============================================================================
//  Kyle UI — Accessibility (a11y) Runtime
//  ARIA, keyboard navigation, focus management, reduced motion.
// =============================================================================

export class A11yManager {
    constructor() {
        this.focusStack = [];
        this.keyboardHandlers = new Map();
        this._reducedMotion = null;
        this._initReducedMotion();
    }

    // -----------------------------------------------------------------------
    //  ARIA Auto-generation
    // -----------------------------------------------------------------------

    // Set ARIA attributes on an element based on its Kyle component type
    static applyAria(el, tag, attrs = {}) {
        const aria = A11yManager._ariaForTag(tag, attrs);
        for (const [key, val] of Object.entries(aria)) {
            if (val !== undefined && val !== null && val !== false) {
                el.setAttribute(key, val === true ? 'true' : String(val));
            }
        }
    }

    // Map Kyle components to ARIA roles
    static _ariaForTag(tag, attrs) {
        const roleMap = {
            'button': { role: 'button' },
            'link': { role: 'link' },
            'text_field': { role: 'textbox' },
            'password_field': { role: 'textbox' },
            'textarea': { role: 'textbox' },
            'checkbox': { role: 'checkbox', 'aria-checked': attrs.checked },
            'radio': { role: 'radio', 'aria-checked': attrs.checked },
            'switch': { role: 'switch', 'aria-checked': attrs.checked },
            'slider': { role: 'slider', 'aria-valuenow': attrs.value },
            'progress': { role: 'progressbar', 'aria-valuenow': attrs.value },
            'dialog': { role: 'dialog', 'aria-modal': attrs.modal ?? true },
            'tooltip': { role: 'tooltip' },
            'tab': { role: 'tab' },
            'tab_list': { role: 'tablist' },
            'tab_panel': { role: 'tabpanel' },
            'navigation': { role: 'navigation' },
            'main': { role: 'main' },
            'banner': { role: 'banner' },
            'complementary': { role: 'complementary' },
            'form': { role: 'form' },
            'search': { role: 'search' },
            'img': { role: 'img' },
            'list': { role: 'list' },
            'list_item': { role: 'listitem' },
        };

        const base = roleMap[tag] || {};
        // Add aria-label from label attribute
        if (attrs.label) base['aria-label'] = attrs.label;
        if (attrs.aria_label) base['aria-label'] = attrs.aria_label;
        if (attrs.aria_describedby) base['aria-describedby'] = attrs.aria_describedby;
        if (attrs.disabled) base['aria-disabled'] = 'true';
        if (attrs.required) base['aria-required'] = 'true';
        if (attrs.invalid) base['aria-invalid'] = 'true';
        if (attrs.expanded !== undefined) base['aria-expanded'] = String(attrs.expanded);
        if (attrs.haspopup) base['aria-haspopup'] = String(attrs.haspopup);

        return base;
    }

    // -----------------------------------------------------------------------
    //  Focus Management
    // -----------------------------------------------------------------------

    // Focus an element by ID or element reference
    focus(el) {
        if (typeof el === 'string') {
            el = document.getElementById(el);
        }
        if (el && typeof el.focus === 'function') {
            el.focus();
            this.focusStack.push(el);
        }
    }

    // Trap focus within a container (for modals, dialogs)
    trapFocus(container) {
        const focusable = container.querySelectorAll(
            'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
        );
        if (focusable.length === 0) return;

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        const handler = (e) => {
            if (e.key !== 'Tab') return;
            if (e.shiftKey) {
                if (document.activeElement === first) {
                    e.preventDefault();
                    last.focus();
                }
            } else {
                if (document.activeElement === last) {
                    e.preventDefault();
                    first.focus();
                }
            }
        };

        container.addEventListener('keydown', handler);
        // Focus first element
        first.focus();
        return () => container.removeEventListener('keydown', handler);
    }

    // Restore focus to previous element
    restoreFocus() {
        const el = this.focusStack.pop();
        if (el && typeof el.focus === 'function') {
            el.focus();
        }
    }

    // -----------------------------------------------------------------------
    //  Keyboard Navigation
    // -----------------------------------------------------------------------

    // Add keyboard navigation to a list of items (arrow keys, home/end)
    keyboardNav(container, itemSelector, options = {}) {
        const items = () => container.querySelectorAll(itemSelector);
        const { loop = true, horizontal = false, onSelect } = options;
        const key = horizontal ? 'Horizontal' : 'Vertical';

        if (this.keyboardHandlers.has(container)) {
            container.removeEventListener('keydown', this.keyboardHandlers.get(container));
        }

        const handler = (e) => {
            const current = document.activeElement;
            const allItems = items();
            const currentIdx = Array.from(allItems).indexOf(current);

            let nextIdx = -1;
            if (e.key === 'ArrowDown' || (horizontal && e.key === 'ArrowRight')) {
                e.preventDefault();
                nextIdx = currentIdx + 1;
                if (nextIdx >= allItems.length) nextIdx = loop ? 0 : currentIdx;
            } else if (e.key === 'ArrowUp' || (horizontal && e.key === 'ArrowLeft')) {
                e.preventDefault();
                nextIdx = currentIdx - 1;
                if (nextIdx < 0) nextIdx = loop ? allItems.length - 1 : 0;
            } else if (e.key === 'Home') {
                e.preventDefault();
                nextIdx = 0;
            } else if (e.key === 'End') {
                e.preventDefault();
                nextIdx = allItems.length - 1;
            } else if (e.key === 'Enter' || e.key === ' ') {
                if (current && onSelect) {
                    e.preventDefault();
                    onSelect(current, currentIdx);
                }
            }

            if (nextIdx >= 0 && nextIdx < allItems.length && allItems[nextIdx]) {
                allItems[nextIdx].focus();
            }
        };

        container.addEventListener('keydown', handler);
        this.keyboardHandlers.set(container, handler);
        return () => container.removeEventListener('keydown', handler);
    }

    // -----------------------------------------------------------------------
    //  Reduced Motion
    // -----------------------------------------------------------------------

    _initReducedMotion() {
        if (typeof window === 'undefined') { this._reducedMotion = false; return; }
        const mql = window.matchMedia('(prefers-reduced-motion: reduce)');
        this._reducedMotion = mql.matches;
        mql.addEventListener('change', (e) => {
            this._reducedMotion = e.matches;
        });
    }

    get prefersReducedMotion() {
        return this._reducedMotion;
    }

    // Wrap an animation call — skip if reduced motion
    animate(el, keyframes, options) {
        if (this._reducedMotion) {
            // Jump to final state instead of animating
            if (keyframes.length > 0) {
                const final = keyframes[keyframes.length - 1];
                Object.assign(el.style, final);
            }
            return null;
        }
        return el.animate(keyframes, options);
    }

    // -----------------------------------------------------------------------
    //  Live Regions (for dynamic content updates)
    // -----------------------------------------------------------------------

    static makeLiveRegion(el, politeness = 'polite') {
        el.setAttribute('aria-live', politeness);
        el.setAttribute('aria-atomic', 'true');
    }

    // Announce a message to screen readers
    static announce(message, politeness = 'polite') {
        let el = document.getElementById('__ky_a11y_announcer');
        if (!el) {
            el = document.createElement('div');
            el.id = '__ky_a11y_announcer';
            el.setAttribute('aria-live', politeness);
            el.setAttribute('aria-atomic', 'true');
            el.style.position = 'absolute';
            el.style.width = '1px';
            el.style.height = '1px';
            el.style.overflow = 'hidden';
            el.style.clip = 'rect(0,0,0,0)';
            document.body.appendChild(el);
        }
        el.textContent = '';
        // Use setTimeout to ensure the clear triggers a new announcement
        setTimeout(() => { el.textContent = message; }, 50);
    }
}

// Global fallback
if (typeof window !== 'undefined') {
    window.A11yManager = A11yManager;
}
