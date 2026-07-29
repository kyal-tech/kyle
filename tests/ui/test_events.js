const { describe, it, expect } = require('./runner.js');

describe('DOM Events', () => {
  it('should handle click event', () => {
    const btn = document.createElement('button');
    let count = 0;
    btn.addEventListener('click', () => count++);
    btn.click();
    expect(count).toEqual(1);
  });

  it('should handle input event', () => {
    const input = document.createElement('input');
    let val = '';
    input.addEventListener('input', (e) => { val = e.target.value; });
    input.value = 'test';
    input.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(val).toEqual('test');
  });

  it('should handle change event', () => {
    const input = document.createElement('input');
    let changed = false;
    input.addEventListener('change', () => { changed = true; });
    input.dispatchEvent(new window.Event('change', { bubbles: true }));
    expect(changed).toEqual(true);
  });

  it('should handle focus and blur events', () => {
    const input = document.createElement('input');
    let focused = false;
    let blurred = false;
    input.addEventListener('focus', () => { focused = true; });
    input.addEventListener('blur', () => { blurred = true; });
    input.dispatchEvent(new window.Event('focus'));
    expect(focused).toEqual(true);
    input.dispatchEvent(new window.Event('blur'));
    expect(blurred).toEqual(true);
  });

  it('should handle keyboard events', () => {
    const input = document.createElement('input');
    let key = '';
    input.addEventListener('keydown', (e) => { key = e.key; });
    input.dispatchEvent(new window.KeyboardEvent('keydown', { key: 'Enter' }));
    expect(key).toEqual('Enter');
  });

  it('should handle mouse events', () => {
    const el = document.createElement('div');
    let entered = false;
    let left = false;
    el.addEventListener('mouseenter', () => { entered = true; });
    el.addEventListener('mouseleave', () => { left = true; });
    el.dispatchEvent(new window.MouseEvent('mouseenter'));
    expect(entered).toEqual(true);
    el.dispatchEvent(new window.MouseEvent('mouseleave'));
    expect(left).toEqual(true);
  });

  it('should support event preventDefault', () => {
    const form = document.createElement('form');
    let prevented = false;
    form.addEventListener('submit', (e) => { e.preventDefault(); prevented = true; });
    form.dispatchEvent(new window.Event('submit', { cancelable: true }));
    expect(prevented).toEqual(true);
  });

  it('should support event stopPropagation', () => {
    const outer = document.createElement('div');
    const inner = document.createElement('button');
    outer.appendChild(inner);
    let outerFired = false;
    outer.addEventListener('click', () => { outerFired = true; });
    inner.addEventListener('click', (e) => { e.stopPropagation(); });
    inner.dispatchEvent(new window.MouseEvent('click', { bubbles: true }));
    expect(outerFired).toEqual(false);
  });
});

describe('Form Events', () => {
  it('should submit form and collect data', () => {
    const form = document.createElement('form');
    const input = document.createElement('input');
    input.name = 'email';
    input.value = 'test@test.com';
    form.appendChild(input);

    let submitted = false;
    let formData = null;
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      submitted = true;
      const data = new window.FormData(form);
      formData = Object.fromEntries(data);
    });

    form.dispatchEvent(new window.Event('submit', { cancelable: true }));
    expect(submitted).toEqual(true);
    expect(formData.email).toEqual('test@test.com');
  });

  it('should validate required fields', () => {
    const form = document.createElement('form');
    const input = document.createElement('input');
    input.required = true;
    form.appendChild(input);
    expect(input.checkValidity()).toEqual(false);
    input.value = 'ok';
    expect(input.checkValidity()).toEqual(true);
  });
});

describe('Select Element', () => {
  it('should display selected option', () => {
    const select = document.createElement('select');
    const opt1 = document.createElement('option');
    opt1.value = 'us';
    opt1.textContent = 'United States';
    const opt2 = document.createElement('option');
    opt2.value = 'uk';
    opt2.textContent = 'United Kingdom';
    select.appendChild(opt1);
    select.appendChild(opt2);
    select.value = 'uk';
    expect(select.value).toEqual('uk');
    expect(select.options[select.selectedIndex].textContent).toEqual('United Kingdom');
  });

  it('should trigger change event on selection', () => {
    const select = document.createElement('select');
    const opt1 = document.createElement('option');
    opt1.value = 'a';
    const opt2 = document.createElement('option');
    opt2.value = 'b';
    select.appendChild(opt1);
    select.appendChild(opt2);
    let changed = false;
    select.addEventListener('change', () => { changed = true; });
    select.value = 'b';
    select.dispatchEvent(new window.Event('change', { bubbles: true }));
    expect(changed).toEqual(true);
  });
});
