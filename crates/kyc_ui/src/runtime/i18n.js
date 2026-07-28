// =============================================================================
//  Kyle UI — Internacionalización (i18n)
//  Traducciones, plurales, formato fechas/números, RTL.
// =============================================================================

export class I18nManager {
    constructor(options = {}) {
        this.locale = options.locale || navigator?.language?.split('-')[0] || 'en';
        this.fallbackLocale = options.fallbackLocale || 'en';
        this.translations = options.translations || {};
        this._localeListeners = [];
        this._rtlLocales = new Set(['ar', 'he', 'fa', 'ur', 'yi', 'dv', 'ps', 'sd']);
    }

    // Current locale dir
    get dir() {
        return this._rtlLocales.has(this.locale) ? 'rtl' : 'ltr';
    }

    get isRTL() {
        return this.dir === 'rtl';
    }

    // Register translations for a locale
    addTranslations(locale, data) {
        if (!this.translations[locale]) {
            this.translations[locale] = {};
        }
        Object.assign(this.translations[locale], data);
    }

    // Switch locale
    setLocale(locale) {
        if (locale === this.locale) return;
        this.locale = locale;
        for (const listener of this._localeListeners) {
            try { listener(locale); } catch (e) { /* ignore */ }
        }
    }

    // Listen for locale changes
    onLocaleChange(listener) {
        this._localeListeners.push(listener);
        return () => {
            this._localeListeners = this._localeListeners.filter(l => l !== listener);
        };
    }

    // Translate a key
    t(key, params = {}) {
        const dict = this.translations[this.locale] || this.translations[this.fallbackLocale] || {};
        let msg = dict[key];
        if (msg === undefined) {
            console.warn(`[i18n] Missing translation: "${key}" for locale "${this.locale}"`);
            return key;
        }

        // Handle plural forms
        if (typeof msg === 'object' && msg !== null) {
            const count = params.count ?? params.n ?? 0;
            const pluralForm = this._pluralForm(count);
            msg = msg[pluralForm] || msg['other'] || key;
        }

        // Replace {params}
        if (typeof msg === 'string') {
            msg = msg.replace(/\{(\w+)\}/g, (_, name) => {
                return params[name] !== undefined ? String(params[name]) : `{${name}}`;
            });
        }

        return String(msg);
    }

    // ICU-like plural form selection
    _pluralForm(n) {
        const num = Number(n);
        // CLDR plural rules for common locales
        const rules = PLURAL_RULES[this.locale] || PLURAL_RULES['en'];

        if (rules.cardinal) {
            for (const [form, test] of Object.entries(rules.cardinal)) {
                if (test(num)) return form;
            }
        }
        return 'other';
    }

    // Format number according to locale
    formatNumber(n, style = 'decimal', options = {}) {
        try {
            return new Intl.NumberFormat(this.locale, { style, ...options }).format(n);
        } catch {
            return String(n);
        }
    }

    // Format currency
    formatCurrency(n, currency = 'USD') {
        return this.formatNumber(n, 'currency', { currency });
    }

    // Format percent
    formatPercent(n) {
        return this.formatNumber(n, 'percent');
    }

    // Format date
    formatDate(date, style = 'medium') {
        const styles = {
            short: { dateStyle: 'short' },
            medium: { dateStyle: 'medium' },
            long: { dateStyle: 'long' },
            full: { dateStyle: 'full' },
        };
        try {
            return new Intl.DateTimeFormat(this.locale, styles[style] || styles.medium).format(new Date(date));
        } catch {
            return String(date);
        }
    }

    // Format time
    formatTime(date, style = 'short') {
        const styles = {
            short: { timeStyle: 'short' },
            medium: { timeStyle: 'medium' },
            long: { timeStyle: 'long' },
        };
        try {
            return new Intl.DateTimeFormat(this.locale, styles[style] || styles.short).format(new Date(date));
        } catch {
            return String(date);
        }
    }
}

// CLDR plural rules (simplified)
const PLURAL_RULES = {
    'en': {
        cardinal: {
            one: (n) => n === 1,
        }
    },
    'es': {
        cardinal: {
            one: (n) => n === 1,
        }
    },
    'ru': {
        cardinal: {
            one: (n) => n % 10 === 1 && n % 100 !== 11,
            few: (n) => n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 10 || n % 100 >= 20),
            many: (n) => n % 10 === 0 || (n % 10 >= 5 && n % 10 <= 9) || (n % 100 >= 11 && n % 100 <= 14),
        }
    },
    'ar': {
        cardinal: {
            one: (n) => n === 1,
            two: (n) => n === 2,
            few: (n) => n % 100 >= 3 && n % 100 <= 10,
            many: (n) => n % 100 >= 11,
        }
    },
    'pl': {
        cardinal: {
            one: (n) => n === 1,
            few: (n) => n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 10 || n % 100 >= 20),
            many: (n) => n !== 1 && (n % 10 === 0 || n % 10 >= 5 && n % 10 <= 9 || n % 100 >= 12 && n % 100 <= 14),
        }
    },
    'zh': { cardinal: {} },
    'ja': { cardinal: {} },
    'ko': { cardinal: {} },
};

// Create singleton
let _instance = null;
export function getI18n(options) {
    if (!_instance) _instance = new I18nManager(options || {});
    return _instance;
}

// Global t() function
export function t(key, params) {
    return getI18n().t(key, params);
}

// Global locale switch
export function setLocale(locale) {
    getI18n().setLocale(locale);
}

// Export
if (typeof window !== 'undefined') {
    window.I18nManager = I18nManager;
    window.t = t;
    window.setLocale = setLocale;
}
