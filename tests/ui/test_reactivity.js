const { describe, it, expect } = require('./runner.js');

describe('ReactiveState', () => {
  it('should create state with initial values', () => {
    const state = new window.ReactiveState({ count: 0, name: 'test' });
    expect(state.get('count')).toEqual(0);
    expect(state.get('name')).toEqual('test');
  });

  it('should set and get values', () => {
    const state = new window.ReactiveState({});
    state.set('count', 42);
    expect(state.get('count')).toEqual(42);
  });

  it('should notify watchers on set', () => {
    const state = new window.ReactiveState({ x: 1 });
    let called = 0;
    let latest;
    state.watch('x', (v) => { called++; latest = v; });
    state.set('x', 2);
    expect(called).toEqual(1);
    expect(latest).toEqual(2);
  });

  it('should not notify when value unchanged', () => {
    const state = new window.ReactiveState({ x: 1 });
    let called = 0;
    state.watch('x', () => called++);
    state.set('x', 1);
    expect(called).toEqual(0);
  });

  it('should support multiple watchers', () => {
    const state = new window.ReactiveState({ x: 1 });
    let a = 0, b = 0;
    state.watch('x', () => a++);
    state.watch('x', () => b++);
    state.set('x', 2);
    expect(a).toEqual(1);
    expect(b).toEqual(1);
  });

  it('should batch updates', () => {
    const state = new window.ReactiveState({ a: 1, b: 2 });
    let calls = 0;
    state.onAnyChange(() => calls++);
    state.batch(() => {
      state.set('a', 10);
      state.set('b', 20);
    });
    expect(state.get('a')).toEqual(10);
    expect(state.get('b')).toEqual(20);
    expect(calls).toEqual(2);
  });

  it('should call onAnyChange on any value change', () => {
    const state = new window.ReactiveState({ x: 1 });
    let changed = [];
    state.onAnyChange((key) => changed.push(key));
    state.set('x', 2);
    state.set('y', 3);
    expect(changed).toContain('x');
    expect(changed).toContain('y');
  });

  it('should return unsubscribe from watch', () => {
    const state = new window.ReactiveState({ x: 1 });
    let called = 0;
    const unsub = state.watch('x', () => called++);
    state.set('x', 2);
    expect(called).toEqual(1);
    unsub();
    state.set('x', 3);
    expect(called).toEqual(1);
  });
});

describe('Binding — oneWay', () => {
  it('should set textContent from state', () => {
    const el = document.createElement('span');
    const state = new window.ReactiveState({ name: 'World' });
    window.Binding.oneWay(el, 'textContent', state, 'name');
    expect(el.textContent).toEqual('World');
  });

  it('should update textContent on state change', () => {
    const el = document.createElement('span');
    const state = new window.ReactiveState({ name: 'World' });
    window.Binding.oneWay(el, 'textContent', state, 'name');
    state.set('name', 'Kyle');
    expect(el.textContent).toEqual('Kyle');
  });

  it('should set input value from state', () => {
    const el = document.createElement('input');
    const state = new window.ReactiveState({ email: 'test@test.com' });
    window.Binding.oneWay(el, 'value', state, 'email');
    expect(el.value).toEqual('test@test.com');
  });

  it('should set checkbox checked from state', () => {
    const el = document.createElement('input');
    el.type = 'checkbox';
    const state = new window.ReactiveState({ ok: true });
    window.Binding.oneWay(el, 'checked', state, 'ok');
    expect(el.checked).toEqual(true);
    state.set('ok', false);
    expect(el.checked).toEqual(false);
  });
});

describe('Binding — twoWay', () => {
  it('should set initial value from state to input', () => {
    const el = document.createElement('input');
    const state = new window.ReactiveState({ name: 'Initial' });
    window.Binding.twoWay(el, state, 'name', 'input');
    expect(el.value).toEqual('Initial');
  });

  it('should update state when input changes', () => {
    const el = document.createElement('input');
    const state = new window.ReactiveState({ name: '' });
    window.Binding.twoWay(el, state, 'name', 'input');
    el.value = 'New Value';
    el.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('name')).toEqual('New Value');
  });

  it('should update input when state changes (backward)', () => {
    const el = document.createElement('input');
    const state = new window.ReactiveState({ name: 'A' });
    window.Binding.twoWay(el, state, 'name', 'input');
    state.set('name', 'B');
    expect(el.value).toEqual('B');
  });

  it('should work with checkbox type', () => {
    const el = document.createElement('input');
    el.type = 'checkbox';
    const state = new window.ReactiveState({ ok: false });
    window.Binding.twoWay(el, state, 'ok', 'input');
    expect(el.checked).toEqual(false);
    el.checked = true;
    el.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('ok')).toEqual(true);
  });
});

describe('Event Helpers', () => {
  it('should create Kyle event from DOM event', () => {
    const el = document.createElement('button');
    const domEvent = new window.MouseEvent('click', { clientX: 100, clientY: 200, button: 0 });
    const ke = window.createKyleEvent(domEvent);
    expect(ke.type).toEqual('click');
    expect(ke.x).toEqual(100);
    expect(ke.y).toEqual(200);
    expect(ke.button).toEqual(0);
  });

  it('should handle keyboard events', () => {
    const domEvent = new window.KeyboardEvent('keydown', { key: 'Enter', ctrlKey: true });
    const ke = window.createKyleEvent(domEvent);
    expect(ke.key).toEqual('Enter');
    expect(ke.ctrl_key).toEqual(true);
  });
});

describe('UITestRenderer', () => {
  it('should render a component and find elements', () => {
    const renderer = new window.UITestRenderer();
    renderer.setup();
    const root = renderer.render(() => {
      const el = document.createElement('div');
      el.innerHTML = '<span class="test">Hello</span>';
      return el;
    });
    expect(renderer.find('.test')).toBeDefined();
    expect(renderer.text()).toContain('Hello');
    renderer.teardown();
  });

  it('should support click simulation', () => {
    const renderer = new window.UITestRenderer();
    renderer.setup();
    let clicked = false;
    renderer.render(() => {
      const btn = document.createElement('button');
      btn.textContent = 'Click me';
      btn.addEventListener('click', () => { clicked = true; });
      return btn;
    });
    renderer.click('button');
    expect(clicked).toEqual(true);
    renderer.teardown();
  });

  it('should support type simulation', () => {
    const renderer = new window.UITestRenderer();
    renderer.setup();
    renderer.render(() => {
      const input = document.createElement('input');
      input.id = 'myinput';
      return input;
    });
    renderer.type('#myinput', 'Hello World');
    expect(document.querySelector('#myinput').value).toEqual('Hello World');
    renderer.teardown();
  });

  it('should take snapshots', () => {
    const renderer = new window.UITestRenderer();
    renderer.setup();
    renderer.render(() => {
      const el = document.createElement('div');
      el.innerHTML = '<p>Snapshot test</p>';
      return el;
    });
    const snap = renderer.snapshot();
    expect(snap).toContain('Snapshot test');
    renderer.teardown();
  });
});
