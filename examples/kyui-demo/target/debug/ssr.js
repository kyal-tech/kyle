// =============================================================================
//  Kyle UI — Server-Side Rendering (SSR)
//  Renderiza componentes Kyle a HTML strings para el servidor.
// =============================================================================

export class SSRRenderer {
    // Render a component to HTML string
    static render(componentFn, props = {}) {
        const result = componentFn(props);
        if (typeof result === 'string') return result;
        if (result instanceof HTMLElement) return SSRRenderer._elementToHTML(result);
        if (result?.element) return SSRRenderer._elementToHTML(result.element);
        return '';
    }

    // Convert DOM element tree to HTML string
    static _elementToHTML(el) {
        if (el.nodeType === Node.TEXT_NODE) {
            return SSRRenderer._escape(el.textContent || '');
        }
        if (el.nodeType === Node.DOCUMENT_FRAGMENT_NODE) {
            let html = '';
            for (const child of el.childNodes) {
                html += SSRRenderer._elementToHTML(child);
            }
            return html;
        }

        const tag = el.tagName?.toLowerCase() || 'div';
        let html = `<${tag}`;

        // Attributes
        for (const attr of el.attributes || []) {
            html += ` ${attr.name}="${SSRRenderer._escape(attr.value)}"`;
        }

        // Style
        const style = el.getAttribute?.('style');
        if (style) {
            html += ` style="${SSRRenderer._escape(style)}"`;
        }

        html += '>';

        // Children
        for (const child of el.childNodes || []) {
            html += SSRRenderer._elementToHTML(child);
        }

        // Self-closing tags
        const voidElements = new Set(['br', 'hr', 'img', 'input', 'meta', 'link', 'area', 'base', 'col', 'embed', 'source', 'track', 'wbr']);
        if (voidElements.has(tag)) {
            html = `<${tag}`;
            for (const attr of el.attributes || []) {
                html += ` ${attr.name}="${SSRRenderer._escape(attr.value)}"`;
            }
            html += ' />';
        } else {
            html += `</${tag}>`;
        }

        return html;
    }

    // Escape HTML special characters
    static _escape(str) {
        return String(str)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#39;');
    }

    // Render full HTML page with state serialization
    static renderPage(componentFn, props = {}, options = {}) {
        const content = SSRRenderer.render(componentFn, props);
        const title = options.title || 'Kyle App';
        const lang = options.lang || 'en';
        const dir = options.dir || 'ltr';
        const scripts = options.scripts || [];
        const styles = options.styles || [];

        return `<!DOCTYPE html>
<html lang="${lang}" dir="${dir}">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>${SSRRenderer._escape(title)}</title>
    ${styles.map(s => `<link rel="stylesheet" href="${s}" />`).join('\n    ')}
</head>
<body>
    <div id="app">${content}</div>
    <script>window.__KYLE_INITIAL_STATE__ = ${JSON.stringify(props)};</script>
    ${scripts.map(s => `<script src="${s}"></script>`).join('\n    ')}
</body>
</html>`;
    }
}

// Export
if (typeof window !== 'undefined') {
    window.SSRRenderer = SSRRenderer;
}
