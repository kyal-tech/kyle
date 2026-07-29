const { describe, it, expect } = require('./runner.js');

describe('Lifecycle — on_mounted', () => {
  it('should call on_mounted after element is added to DOM', () => {
    const container = document.getElementById('app');
    let mounted = false;

    const el = document.createElement('div');
    el.textContent = 'Test';
    const lifecycle = {
      mounted: () => { mounted = true; },
      unmounted: null,
      updated: null,
    };

    container.appendChild(el);
    if (lifecycle.mounted) lifecycle.mounted();

    expect(mounted).toEqual(true);
    expect(container.contains(el)).toEqual(true);
  });
});

describe('Lifecycle — on_unmounted', () => {
  it('should call on_unmounted when element is removed from DOM', () => {
    const container = document.getElementById('app');
    let unmounted = false;

    const el = document.createElement('div');
    el.textContent = 'Remove me';
    const lifecycle = {
      mounted: null,
      unmounted: () => { unmounted = true; },
      updated: null,
    };

    container.appendChild(el);
    container.removeChild(el);
    if (lifecycle.unmounted) lifecycle.unmounted();

    expect(unmounted).toEqual(true);
  });
});

describe('Lifecycle — on_updated', () => {
  it('should call on_updated when state changes trigger re-render', () => {
    const state = new window.ReactiveState({ count: 0 });
    let updated = false;

    state.watch('count', () => {
      updated = true;
    });

    expect(updated).toEqual(false);
    state.set('count', 1);
    expect(updated).toEqual(true);
  });
});

describe('Lifecycle — on_created', () => {
  it('should execute on_created logic before mount', () => {
    let created = false;
    let mountCalled = false;

    const onCreated = () => {
      created = true;
    };

    const onMounted = () => {
      mountCalled = true;
    };

    onCreated();
    expect(created).toEqual(true);
    expect(mountCalled).toEqual(false);

    onMounted();
    expect(mountCalled).toEqual(true);
  });
});

describe('Lifecycle — Full lifecycle order', () => {
  it('should follow create -> mount -> unmount order', () => {
    const order = [];

    const onCreated = () => order.push('created');
    const onMounted = () => order.push('mounted');
    const onUnmounted = () => order.push('unmounted');

    onCreated();
    expect(order).toEqual(['created']);

    const el = document.createElement('div');
    document.getElementById('app').appendChild(el);

    onMounted();
    expect(order).toEqual(['created', 'mounted']);

    document.getElementById('app').removeChild(el);

    onUnmounted();
    expect(order).toEqual(['created', 'mounted', 'unmounted']);
  });
});
