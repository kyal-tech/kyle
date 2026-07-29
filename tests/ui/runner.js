const fs = require('fs');
const path = require('path');
const { JSDOM } = require('jsdom');

// Configuration
const RUNTIME_DIR = path.resolve(__dirname, '../../examples/kyui-demo/target/debug');
const RUNTIME_FILES = [
  'reactivity.js',
  'testing.js',
  'a11y.js',
  'router.js',
  'portal.js',
  'error_boundary.js',
  'i18n.js',
  'ssr.js',
];

// ── DOM Setup ──────────────────────────────────────────────────────────
let dom, window, document;

function setupDOM() {
  dom = new JSDOM('<!DOCTYPE html><html><body><div id="app"></div></body></html>', {
    url: 'http://localhost:3000/',
    runScripts: 'dangerously',
    pretendToBeVisual: true,
  });
  window = dom.window;
  document = window.document;
  global.window = window;
  global.document = document;
  global.navigator = window.navigator;
}

// ── Runtime Loader ─────────────────────────────────────────────────────
function loadRuntime() {
  for (const file of RUNTIME_FILES) {
    const filePath = path.join(RUNTIME_DIR, file);
    if (!fs.existsSync(filePath)) {
      console.warn(`  ⚠ Runtime file not found: ${file}`);
      continue;
    }
    const code = fs.readFileSync(filePath, 'utf8');
    // Strip ESM exports/imports so they work in CommonJS context
    const esmStripped = code
      .replace(/^export (default )?(class|function|let|const|var) /gm, '$2 ')
      .replace(/^import .+ from .+;?$/gm, '')
      .replace(/^import .+;?$/gm, '')
      .replace(/^export \{ .* \};?$/gm, '')
      .replace(/^export \* from .+;?$/gm, '');
    try {
      window.eval(esmStripped);
    } catch (e) {
      console.error(`  ✗ Error loading ${file}: ${e.message}`);
    }
  }
}

// ── Test Framework ─────────────────────────────────────────────────────
const results = { pass: 0, fail: 0, suites: [] };
let currentSuite = null;
let currentTests = [];

function describe(name, fn) {
  currentSuite = name;
  currentTests = [];
  fn();
  const suiteResults = { name, tests: [] };
  for (const test of currentTests) {
    try {
      test.fn();
      suiteResults.tests.push({ name: test.name, status: 'pass' });
      results.pass++;
    } catch (e) {
      suiteResults.tests.push({ name: test.name, status: 'fail', error: e.message });
      results.fail++;
    }
  }
  results.suites.push(suiteResults);
  currentSuite = null;
  currentTests = [];
}

function it(name, fn) {
  currentTests.push({ name, fn });
}

function expect(actual) {
  return {
    toEqual(expected) {
      if (Array.isArray(actual) && Array.isArray(expected)) {
        if (actual.length !== expected.length) {
          throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
        }
        for (let i = 0; i < actual.length; i++) {
          if (actual[i] !== expected[i]) {
            throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
          }
        }
        return;
      }
      if (actual !== expected) {
        throw new Error(`Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
      }
    },
    toBe(expected) {
      if (actual !== expected) {
        throw new Error(`Expected ${expected}, got ${actual}`);
      }
    },
    toBeTrue() {
      if (actual !== true) {
        throw new Error(`Expected true, got ${actual}`);
      }
    },
    toBeFalse() {
      if (actual !== false) {
        throw new Error(`Expected false, got ${actual}`);
      }
    },
    toContain(expected) {
      if (typeof actual === 'string' && !actual.includes(expected)) {
        throw new Error(`Expected "${actual}" to contain "${expected}"`);
      }
      if (Array.isArray(actual) && !actual.includes(expected)) {
        throw new Error(`Expected array to contain ${JSON.stringify(expected)}`);
      }
    },
    toBeGreaterThan(expected) {
      if (!(actual > expected)) {
        throw new Error(`Expected ${actual} > ${expected}`);
      }
    },
    toBeLessThan(expected) {
      if (!(actual < expected)) {
        throw new Error(`Expected ${actual} < ${expected}`);
      }
    },
    toBeNull() {
      if (actual !== null) {
        throw new Error(`Expected null, got ${actual}`);
      }
    },
    toBeDefined() {
      if (actual === undefined) {
        throw new Error(`Expected defined, got undefined`);
      }
    },
    toMatch(pattern) {
      if (typeof actual === 'string' && !pattern.test(actual)) {
        throw new Error(`Expected "${actual}" to match ${pattern}`);
      }
    },
    not: {
      toEqual(expected) {
        if (actual === expected) {
          throw new Error(`Expected not ${JSON.stringify(expected)}, got ${JSON.stringify(expected)}`);
        }
      },
      toContain(expected) {
        if (typeof actual === 'string' && actual.includes(expected)) {
          throw new Error(`Expected "${actual}" not to contain "${expected}"`);
        }
      },
      toBeNull() {
        if (actual === null) {
          throw new Error(`Expected not null, got null`);
        }
      },
    }
  };
}

// ── Test Helpers ───────────────────────────────────────────────────────

function createApp(initialState = {}) {
  const state = new window.ReactiveState(initialState);
  const el = document.createDocumentFragment();
  return { state, el, container: document.getElementById('app') };
}

function renderKyx(componentFn, props = {}) {
  return window.render(componentFn, props);
}

function find(selector) {
  return window.find(selector);
}

function click(selector) {
  return window.click(selector);
}

function type(selector, value) {
  const el = typeof selector === 'string' ? find(selector) : selector;
  if (!el) throw new Error(`Element not found: ${selector}`);
  el.value = value;
  el.dispatchEvent(new window.Event('input', { bubbles: true }));
  el.dispatchEvent(new window.Event('change', { bubbles: true }));
}

function cleanup() {
  document.getElementById('app').innerHTML = '';
  if (window._testRenderer) {
    window._testRenderer.teardown();
    window._testRenderer = null;
  }
}

// ── Report ─────────────────────────────────────────────────────────────

function report() {
  let output = '\n';
  for (const suite of results.suites) {
    output += `\n  ${suite.name}\n`;
    for (const test of suite.tests) {
      const icon = test.status === 'pass' ? '✓' : '✗';
      output += `    ${icon} ${test.name}\n`;
      if (test.error) {
        output += `      └ ${test.error}\n`;
      }
    }
  }
  const total = results.pass + results.fail;
  output += `\n  ─────────────────────────────────\n`;
  output += `  Total: ${total}  |  Pass: ${results.pass}  |  Fail: ${results.fail}\n`;
  console.log(output);
  return results.fail === 0 ? 0 : 1;
}

// ── Main ───────────────────────────────────────────────────────────────

function run(testsDir) {
  setupDOM();
  loadRuntime();

  const testFiles = fs.readdirSync(testsDir)
    .filter(f => f.startsWith('test_') && f.endsWith('.js'))
    .sort();

  if (testFiles.length === 0) {
    console.log('No test files found');
    return 0;
  }

  console.log(`\n  Kyle UI — Integration Tests`);
  console.log(`  ${testFiles.length} test file(s) loaded`);

  for (const file of testFiles) {
    const filePath = path.join(testsDir, file);
    delete require.cache[filePath]; // fresh for each run
    require(filePath);
  }

  return report();
}

module.exports = { describe, it, expect, setupDOM, loadRuntime, createApp, renderKyx, find, click, type, cleanup, run };

// ── CLI ────────────────────────────────────────────────────────────────

if (require.main === module) {
  const testsDir = path.resolve(__dirname);
  const exitCode = run(testsDir);
  process.exit(exitCode);
}
