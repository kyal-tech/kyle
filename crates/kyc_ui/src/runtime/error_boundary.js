// =============================================================================
//  Kyle UI — Error Boundaries
//  Captura errores en componentes hijos con UI de fallback y recovery.
// =============================================================================

export class ErrorBoundary {
    constructor(options = {}) {
        this.state = 'success'; // 'success' | 'error' | 'recovering'
        this.error = null;
        this.fallback = options.fallback || this._defaultFallback;
        this.onError = options.onError || null;
        this.maxRetries = options.maxRetries || 3;
        this.retryCount = 0;
        this.resetOnChange = options.resetOnChange || null;
        this._lastSuccessContent = null;
        this._errorListeners = [];
    }

    // Wrap a render function with error catching
    wrap(renderFn) {
        return (...args) => {
            if (this.state === 'error') {
                return this._renderFallback();
            }
            try {
                const result = renderFn(...args);
                // If render returns a promise, catch async errors
                if (result && typeof result.then === 'function') {
                    return result.catch((err) => {
                        return this._handleError(err);
                    });
                }
                this._lastSuccessContent = result;
                return result;
            } catch (err) {
                return this._handleError(err);
            }
        };
    }

    // Handle an error
    _handleError(error) {
        this.state = 'error';
        this.error = error;

        // Log error
        console.error('[ErrorBoundary]', error);

        // Fire error listeners
        for (const listener of this._errorListeners) {
            try { listener(error); } catch (e) { /* ignore */ }
        }

        // Call onError callback
        if (this.onError) {
            try { this.onError(error); } catch (e) { /* ignore */ }
        }

        // Auto-retry with exponential backoff
        if (this.retryCount < this.maxRetries) {
            const delay = Math.pow(2, this.retryCount) * 1000; // 1s, 2s, 4s
            this.retryCount++;
            console.log(`[ErrorBoundary] Retry ${this.retryCount}/${this.maxRetries} in ${delay}ms`);
            setTimeout(() => this.reset(), delay);
        }

        return this._renderFallback();
    }

    // Reset the boundary (retry rendering children)
    reset() {
        this.state = 'success';
        this.error = null;
        // Trigger re-render
        if (this._onChange) this._onChange();
    }

    // Listen for errors
    onError(callback) {
        this._errorListeners.push(callback);
        return () => {
            this._errorListeners = this._errorListeners.filter(l => l !== callback);
        };
    }

    // Register a change callback for re-rendering
    onChange(callback) {
        this._onChange = callback;
    }

    // Check if props changed and reset if needed
    checkReset(props) {
        if (!this.resetOnChange || this.state !== 'error') return;
        for (const key of this.resetOnChange) {
            if (props?.[key] !== undefined && props[key] !== this._lastProps?.[key]) {
                this.reset();
                break;
            }
        }
        this._lastProps = { ...props };
    }

    // Default fallback UI
    _defaultFallback(error) {
        const container = document.createElement('div');
        container.style.cssText = `
            padding: 16px; border: 1px solid #DC3545;
            border-radius: 8px; background: #FFF5F5;
            color: #DC3545; font-family: sans-serif;
        `;

        const title = document.createElement('div');
        title.textContent = 'Something went wrong';
        title.style.cssText = 'font-weight: bold; margin-bottom: 8px;';
        container.appendChild(title);

        const msg = document.createElement('div');
        msg.textContent = error?.message || String(error);
        msg.style.cssText = 'font-size: 12px; margin-bottom: 12px; opacity: 0.8;';
        container.appendChild(msg);

        const retryBtn = document.createElement('button');
        retryBtn.textContent = 'Retry';
        retryBtn.style.cssText = `
            padding: 6px 16px; border: none; border-radius: 4px;
            background: #DC3545; color: white; cursor: pointer;
        `;
        retryBtn.onclick = () => this.reset();
        container.appendChild(retryBtn);

        return container;
    }

    // Render the fallback UI
    _renderFallback() {
        if (typeof this.fallback === 'function') {
            return this.fallback(this.error);
        }
        return this.fallback || this._defaultFallback(this.error);
    }
}

// Create an error boundary wrapper for components
export function withErrorBoundary(componentFn, options = {}) {
    const boundary = new ErrorBoundary(options);
    const wrapped = boundary.wrap(componentFn);
    wrapped.boundary = boundary;
    wrapped.reset = () => boundary.reset();
    return wrapped;
}

// Export
if (typeof window !== 'undefined') {
    window.ErrorBoundary = ErrorBoundary;
    window.withErrorBoundary = withErrorBoundary;
}
