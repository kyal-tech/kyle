// =============================================================================
//  Kyle UI — Portal (Teleport) Runtime
//  Renderiza contenido fuera del árbol padre del componente.
// =============================================================================

export class PortalManager {
    constructor() {
        this.portals = new Map();
        this.outlets = new Map();
    }

    // Register a named outlet
    registerOutlet(name, el) {
        this.outlets.set(name, el);
        return () => this.outlets.delete(name);
    }

    // Get outlet element
    getOutlet(name) {
        if (this.outlets.has(name)) return this.outlets.get(name);
        // Try DOM query
        const el = document.getElementById(name) || document.querySelector(name);
        return el || document.body;
    }

    // Create a portal
    create(content, target, options = {}) {
        const targetEl = this.getOutlet(target);
        if (!targetEl) {
            console.warn(`Portal target "${target}" not found`);
            return null;
        }

        const wrapper = document.createElement('div');
        wrapper.style.display = 'contents';

        if (typeof content === 'string') {
            wrapper.innerHTML = content;
        } else if (content instanceof Node) {
            wrapper.appendChild(content);
        } else if (content?.element) {
            wrapper.appendChild(content.element);
        }

        // Apply position if specified
        if (options.position) {
            this._applyPosition(wrapper, options.position, targetEl);
        }

        targetEl.appendChild(wrapper);

        const portalId = Symbol('portal');
        this.portals.set(portalId, { wrapper, targetEl });

        return {
            id: portalId,
            update: (newContent) => {
                wrapper.innerHTML = '';
                if (newContent instanceof Node) wrapper.appendChild(newContent);
            },
            destroy: () => {
                if (wrapper.parentNode) wrapper.parentNode.removeChild(wrapper);
                this.portals.delete(portalId);
            }
        };
    }

    // Create a modal portal (centered overlay)
    createModal(content, options = {}) {
        const overlay = document.createElement('div');
        overlay.style.cssText = `
            position: fixed; top: 0; left: 0; right: 0; bottom: 0;
            background: rgba(0,0,0,0.5); display: flex;
            align-items: center; justify-content: center;
            z-index: ${options.zIndex || 1000};
        `;

        const modalContainer = document.createElement('div');
        modalContainer.style.cssText = `
            background: white; border-radius: 8px;
            padding: 24px; min-width: 300px;
            box-shadow: 0 8px 32px rgba(0,0,0,0.2);
        `;

        if (typeof content === 'string') {
            modalContainer.innerHTML = content;
        } else if (content instanceof Node) {
            modalContainer.appendChild(content);
        } else if (content?.element) {
            modalContainer.appendChild(content.element);
        }

        overlay.appendChild(modalContainer);

        // Close on overlay click (optional)
        if (options.closeOnOverlay !== false) {
            overlay.addEventListener('click', (e) => {
                if (e.target === overlay) {
                    if (options.onClose) options.onClose();
                }
            });
        }

        // Close on Escape
        if (options.closeOnEscape !== false) {
            const keyHandler = (e) => {
                if (e.key === 'Escape' && options.onClose) options.onClose();
            };
            document.addEventListener('keydown', keyHandler);
        }

        document.body.appendChild(overlay);

        return {
            element: overlay,
            close: () => {
                if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
            }
        };
    }

    // Position strategies
    _applyPosition(wrapper, position, targetEl) {
        const rect = targetEl.getBoundingClientRect();

        if (position === 'center' || position?.type === 'center') {
            wrapper.style.cssText = `
                position: fixed; top: 50%; left: 50%;
                transform: translate(-50%, -50%);
                z-index: ${position?.zIndex || 1000};
            `;
            return;
        }

        if (position?.type === 'absolute') {
            wrapper.style.position = 'absolute';
            if (position.x !== undefined) wrapper.style.left = `${position.x}px`;
            if (position.y !== undefined) wrapper.style.top = `${position.y}px`;
            wrapper.style.zIndex = String(position.zIndex || 1000);
            return;
        }

        if (position?.type === 'relative') {
            wrapper.style.position = 'fixed';
            const offsetX = position.offsetX || 0;
            const offsetY = position.offsetY || 0;

            switch (position.placement || 'bottom') {
                case 'bottom':
                    wrapper.style.top = `${rect.bottom + offsetY}px`;
                    wrapper.style.left = `${rect.left + offsetX}px`;
                    break;
                case 'top':
                    wrapper.style.top = `${rect.top - wrapper.offsetHeight - offsetY}px`;
                    wrapper.style.left = `${rect.left + offsetX}px`;
                    break;
                case 'left':
                    wrapper.style.top = `${rect.top + offsetY}px`;
                    wrapper.style.left = `${rect.left - wrapper.offsetWidth - offsetX}px`;
                    break;
                case 'right':
                    wrapper.style.top = `${rect.top + offsetY}px`;
                    wrapper.style.left = `${rect.right + offsetX}px`;
                    break;
            }
            wrapper.style.zIndex = String(position.zIndex || 1000);
        }
    }

    // Cleanup all portals
    destroyAll() {
        for (const [, portal] of this.portals) {
            if (portal.wrapper.parentNode) {
                portal.wrapper.parentNode.removeChild(portal.wrapper);
            }
        }
        this.portals.clear();
    }
}

// Singleton
export const portalManager = new PortalManager();

// Export
if (typeof window !== 'undefined') {
    window.PortalManager = PortalManager;
    window.portalManager = portalManager;
}
