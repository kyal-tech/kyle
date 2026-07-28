// =============================================================================
//  Kyle WASM Runtime — JS Glue
//  Bridges the Kyle WASM module with the browser/Node.js environment.
// =============================================================================

export class KyleRuntime {
    constructor() {
        this.wasm = null;
        this.memory = null;
        this.exports = null;
        this.stringCache = new Map();  // ptr → string
    }

    // -----------------------------------------------------------------------
    //  Initialize: instantiate WASM module with import object
    // -----------------------------------------------------------------------
    async init(wasmModule) {
        const importObj = this._buildImports();
        const result = await WebAssembly.instantiate(wasmModule, importObj);
        this.wasm = result.instance;
        this.exports = result.instance.exports;
        this.memory = this.exports.memory;
        return this;
    }

    // -----------------------------------------------------------------------
    //  Build the import object that WASM expects
    // -----------------------------------------------------------------------
    _buildImports() {
        const env = {
            // Memory
            memory: new WebAssembly.Memory({ initial: 256, maximum: 65536 }),

            // Print / console
            ky_print: (ptr) => {
                const s = this._readStr(ptr);
                console.log(s);
            },
            ky_println: (ptr) => {
                const s = this._readStr(ptr);
                console.log(s);
            },

            // I/O stubs (can be overridden by host)
            ky_open: () => -1,
            ky_read_str: () => 0,
            ky_write_str: () => 0,
            ky_close: () => 0,
            ky_sleep: () => 0,
            ky_now: () => Date.now(),

            // crypto stubs
            ky_sha256: () => 0,
            ky_random_bytes: () => 0,

            // Thread stubs (single-threaded in WASM)
            ky_spawn_task: () => 0,
            ky_await_task: () => 0,
            ky_yield: () => {},
            ky_spawn_thread: () => 0,
            ky_join_thread: () => 0,

            // Channel stubs
            ky_channel_new: () => 0,
            ky_channel_send: () => 0,
            ky_channel_recv: () => 0,
            ky_channel_close: () => 0,
            ky_channel_len: () => 0,
            ky_channel_free: () => 0,

            // Network stubs
            ky_tcp_listen: () => -1,
            ky_tcp_accept: () => -1,
            ky_tcp_read: () => 0,
            ky_tcp_write: () => 0,
            ky_tcp_close: () => 0,

            // Mutex/Atomic stubs (single-threaded, no-ops)
            ky_mutex_new: () => 0,
            ky_mutex_lock: () => {},
            ky_mutex_store: () => {},
            ky_mutex_free: () => {},
            ky_atomic_i64_new: (v) => v,
            ky_atomic_i64_load: (p) => p,
            ky_atomic_i64_store: () => {},
            ky_atomic_i64_add: (p, v) => v,
            ky_atomic_i64_free: () => {},
            ky_atomic_bool_new: (v) => v ? 1 : 0,
            ky_atomic_bool_load: (v) => v,
            ky_atomic_bool_store: () => {},
            ky_atomic_bool_free: () => {},
        };
        return { env };
    }

    // -----------------------------------------------------------------------
    //  Read a null-terminated string from WASM memory
    // -----------------------------------------------------------------------
    _readStr(ptr) {
        if (!ptr || !this.memory) return '';
        if (this.stringCache.has(ptr)) return this.stringCache.get(ptr);

        const buf = new Uint8Array(this.memory.buffer);
        let end = ptr;
        while (buf[end]) end++;
        const str = new TextDecoder('utf-8').decode(buf.slice(ptr, end));
        this.stringCache.set(ptr, str);
        return str;
    }

    // -----------------------------------------------------------------------
    //  Write a string to WASM memory (allocates via ky_alloc)
    // -----------------------------------------------------------------------
    _writeStr(str) {
        const encoded = new TextEncoder().encode(str + '\0');
        const ptr = this.exports.ky_alloc(encoded.length);
        if (!ptr) return 0;
        const buf = new Uint8Array(this.memory.buffer);
        buf.set(encoded, Number(ptr));
        this.stringCache.set(Number(ptr), str);
        return Number(ptr);
    }

    // -----------------------------------------------------------------------
    //  Call a Kyle function by name with JS values
    // -----------------------------------------------------------------------
    call(fnName, ...args) {
        const fn = this.exports[fnName];
        if (!fn) throw new Error(`Function '${fnName}' not found in WASM exports`);
        // Convert JS strings to WASM pointers
        const wasmArgs = args.map(a => {
            if (typeof a === 'string') return this._writeStr(a);
            if (typeof a === 'boolean') return a ? 1 : 0;
            return a;
        });
        const result = fn(...wasmArgs);
        // Convert WASM string pointers back to JS strings
        if (typeof result === 'bigint') return Number(result);
        return result;
    }

    // -----------------------------------------------------------------------
    //  Run main() function — the Kyle entry point
    // -----------------------------------------------------------------------
    runMain() {
        if (this.exports.main) {
            return this.exports.main();
        }
        if (this.exports.kyle_main) {
            return this.exports.kyle_main(0);
        }
        console.warn('No main() or kyle_main() found in WASM module');
        return 0;
    }
}

// Global fallback
if (typeof window !== 'undefined') {
    window.KyleRuntime = KyleRuntime;
}
