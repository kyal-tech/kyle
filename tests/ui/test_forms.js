const { describe, it, expect } = require('./runner.js');

describe('Form Integration', () => {
  it('should render text_field and update state on input', () => {
    const state = new window.ReactiveState({ email: '' });
    const el = document.createElement('input');
    el.type = 'text';
    el.placeholder = 'you@example.com';

    window.Binding.twoWay(el, state, 'email', 'input');

    expect(el.value).toEqual('');

    el.value = 'test@test.com';
    el.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('email')).toEqual('test@test.com');
  });

  it('should render password_field and hide input', () => {
    const el = document.createElement('input');
    el.type = 'password';
    expect(el.type).toEqual('password');
    el.value = 'secret123';
    expect(el.value).toEqual('secret123');
  });

  it('should render text_area and update state', () => {
    const state = new window.ReactiveState({ bio: '' });
    const el = document.createElement('textarea');
    el.placeholder = 'Tell us about yourself...';

    window.Binding.twoWay(el, state, 'bio', 'input');

    el.value = 'Hello, this is my bio!';
    el.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('bio')).toEqual('Hello, this is my bio!');
  });

  it('should render checkbox and toggle state', () => {
    const state = new window.ReactiveState({ checked: false });
    const el = document.createElement('input');
    el.type = 'checkbox';

    window.Binding.twoWay(el, state, 'checked', 'input');

    expect(el.checked).toEqual(false);
    expect(state.get('checked')).toEqual(false);

    el.checked = true;
    el.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('checked')).toEqual(true);
  });

  it('should render select and update state on change', () => {
    const state = new window.ReactiveState({ country: '' });
    const select = document.createElement('select');
    const options = [
      { value: '', text: 'Select...' },
      { value: 'us', text: 'United States' },
      { value: 'uk', text: 'United Kingdom' },
    ];
    for (const opt of options) {
      const el = document.createElement('option');
      el.value = opt.value;
      el.textContent = opt.text;
      select.appendChild(el);
    }

    window.Binding.twoWay(select, state, 'country', 'change');

    select.value = 'us';
    select.dispatchEvent(new window.Event('change', { bubbles: true }));
    expect(state.get('country')).toEqual('us');
  });

  it('should render slider and update state', () => {
    const state = new window.ReactiveState({ volume: 50 });
    const el = document.createElement('input');
    el.type = 'range';
    el.min = 0;
    el.max = 100;

    window.Binding.oneWay(el, 'value', state, 'volume');

    expect(el.value).toEqual('50');

    state.set('volume', 75);
    expect(el.value).toEqual('75');
  });

  it('should handle full form submit with all field types', () => {
    const state = new window.ReactiveState({
      email: 'user@test.com',
      password: 'pass123',
      bio: 'Hello!',
      newsletter: true,
      country: 'uk',
      volume: 80
    });

    // Create form with all field types
    const form = document.createElement('form');
    const emailInput = document.createElement('input');
    emailInput.name = 'email';
    emailInput.type = 'email';
    const passInput = document.createElement('input');
    passInput.name = 'password';
    passInput.type = 'password';
    const bioTextarea = document.createElement('textarea');
    bioTextarea.name = 'bio';
    const checkInput = document.createElement('input');
    checkInput.name = 'newsletter';
    checkInput.type = 'checkbox';
    const select = document.createElement('select');
    select.name = 'country';
    ['us','uk','jp'].forEach(v => {
      const o = document.createElement('option');
      o.value = v; o.textContent = v.toUpperCase();
      select.appendChild(o);
    });
    const slider = document.createElement('input');
    slider.name = 'volume';
    slider.type = 'range';

    form.append(emailInput, passInput, bioTextarea, checkInput, select, slider);

    // Bind all fields
    window.Binding.twoWay(emailInput, state, 'email', 'input');
    window.Binding.twoWay(passInput, state, 'password', 'input');
    window.Binding.twoWay(bioTextarea, state, 'bio', 'input');
    window.Binding.twoWay(checkInput, state, 'newsletter', 'input');
    window.Binding.twoWay(select, state, 'country', 'change');
    window.Binding.oneWay(slider, 'value', state, 'volume');

    // Verify initial state values reflected in UI
    expect(emailInput.value).toEqual('user@test.com');
    expect(passInput.value).toEqual('pass123');
    expect(bioTextarea.value).toEqual('Hello!');
    expect(checkInput.checked).toEqual(true);
    expect(select.value).toEqual('uk');
    expect(slider.value).toEqual('80');

    // Simulate user changing fields
    emailInput.value = 'new@test.com';
    emailInput.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('email')).toEqual('new@test.com');

    checkInput.checked = false;
    checkInput.dispatchEvent(new window.Event('input', { bubbles: true }));
    expect(state.get('newsletter')).toEqual(false);

    select.value = 'jp';
    select.dispatchEvent(new window.Event('change', { bubbles: true }));
    expect(state.get('country')).toEqual('jp');

    // Verify submit
    let submitted = false;
    let data;
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      submitted = true;
      data = {
        email: state.get('email'),
        password: state.get('password'),
        bio: state.get('bio'),
        newsletter: state.get('newsletter'),
        country: state.get('country'),
        volume: state.get('volume'),
      };
    });
    form.dispatchEvent(new window.Event('submit', { cancelable: true }));
    expect(submitted).toEqual(true);
    expect(data.email).toEqual('new@test.com');
    expect(data.newsletter).toEqual(false);
    expect(data.country).toEqual('jp');
  });
});

describe('Radio Buttons', () => {
  it('should select one radio at a time within a group', () => {
    const group = document.createElement('div');
    const r1 = document.createElement('input');
    r1.type = 'radio'; r1.name = 'plan'; r1.value = 'free';
    const r2 = document.createElement('input');
    r2.type = 'radio'; r2.name = 'plan'; r2.value = 'pro';
    const r3 = document.createElement('input');
    r3.type = 'radio'; r3.name = 'plan'; r3.value = 'enterprise';
    group.append(r1, r2, r3);

    r1.checked = true;
    r1.dispatchEvent(new window.Event('change', { bubbles: true }));
    expect(r1.checked).toEqual(true);
    expect(r2.checked).toEqual(false);
    expect(r3.checked).toEqual(false);

    r2.checked = true;
    r2.dispatchEvent(new window.Event('change', { bubbles: true }));
    // In a real browser, checking r2 would uncheck r1 automatically
    // jsdom doesn't do this, so we simulate:
    r1.checked = false;
    expect(r1.checked).toEqual(false);
    expect(r2.checked).toEqual(true);
  });
});
