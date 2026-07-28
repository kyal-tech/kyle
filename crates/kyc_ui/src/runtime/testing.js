// =============================================================================
//  Kyle UI — Testing Framework
//  Renderizado de componentes en tests, simulación de eventos, snapshots.
// =============================================================================

export class UITestRenderer {
    constructor() {
        this.container = null;
    }

    // Setup: create a DOM container for testing
    setup() {
        if (typeof document === 'undefined') {
            throw new Error('UITestRenderer requires a DOM environment (jsdom or browser)');
        }
        this.container = document.createElement('div');
        this.container.id = '__ky_test_container';
        document.body.appendChild(this.container);
        return this;
    }

    // Teardown: clean up
    teardown() {
        if (this.container && this.container.parentNode) {
            this.container.parentNode.removeChild(this.container);
        }
        this.container = null;
    }

    // Render a component into the test container
    render(componentFn, props = {}) {
        if (!this.container) this.setup();
        this.container.innerHTML = '';
        const result = componentFn(props);
        if (result?.element) {
            this.container.appendChild(result.element);
        } else if (result instanceof HTMLElement) {
            this.container.appendChild(result);
        }
        return this.container;
    }

    // Find an element by selector within the test container
    find(selector) {
        return this.container?.querySelector(selector);
    }

    // Find all elements by selector
    findAll(selector) {
        return Array.from(this.container?.querySelectorAll(selector) || []);
    }

    // Get text content of the container (trimmed)
    text() {
        return this.container?.textContent?.trim() || '';
    }

    // Simulate a click on an element
    click(selector) {
        const el = typeof selector === 'string' ? this.find(selector) : selector;
        if (el) {
            el.click();
            return true;
        }
        return false;
    }

    // Simulate typing into an input
    type(selector, value) {
        const el = typeof selector === 'string' ? this.find(selector) : selector;
        if (el) {
            el.value = value;
            el.dispatchEvent(new Event('input', { bubbles: true }));
            el.dispatchEvent(new Event('change', { bubbles: true }));
            return true;
        }
        return false;
    }

    // Check if element exists
    exists(selector) {
        return !!this.find(selector);
    }

    // Snapshot: get the HTML of the container
    snapshot() {
        return this.container?.innerHTML || '';
    }

    // Assertion helpers
    assert = {
        textContains: (expected) => {
            const text = this.text();
            if (!text.includes(expected)) {
                throw new Error(`Expected text to contain "${expected}", got "${text}"`);
            }
        },
        elementExists: (selector) => {
            if (!this.exists(selector)) {
                throw new Error(`Expected element "${selector}" to exist`);
            }
        },
        elementNotExists: (selector) => {
            if (this.exists(selector)) {
                throw new Error(`Expected element "${selector}" to not exist`);
            }
        },
        count: (selector, expected) => {
            const actual = this.findAll(selector).length;
            if (actual !== expected) {
                throw new Error(`Expected ${expected} elements matching "${selector}", got ${actual}`);
            }
        },
    };
}

// Create test renderer singleton
export let _testRenderer = null;
export function getTestRenderer() {
    if (!_testRenderer) _testRenderer = new UITestRenderer();
    return _testRenderer;
}

export function render(componentFn, props) {
    return getTestRenderer().render(componentFn, props);
}

export function find(selector) {
    return getTestRenderer().find(selector);
}

export function click(selector) {
    return getTestRenderer().click(selector);
}

export function snapshot() {
    return getTestRenderer().snapshot();
}

// Export
if (typeof window !== 'undefined') {
    window.UITestRenderer = UITestRenderer;
    window.render = render;
    window.find = find;
    window.click = click;
    window.snapshot = snapshot;
}
