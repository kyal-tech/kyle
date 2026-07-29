const { describe, it, expect } = require('./runner.js');

describe('Router', () => {
  it('should create router with options', () => {
    const router = new window.Router({ container: document.getElementById('app') });
    expect(router).toBeDefined();
    expect(router.routes).toBeDefined();
    expect(router.routes.length).toEqual(0);
  });

  it('should register routes', () => {
    const router = new window.Router({ container: document.getElementById('app') });
    const viewFn = () => document.createElement('div');
    router.register('/', viewFn);
    expect(router.routes.length).toEqual(1);
    expect(router.routes[0].pattern).toBeDefined();
  });

  it('should navigate between routes', async () => {
    const router = new window.Router({ container: document.getElementById('app') });
    router.register('/', () => {
      const el = document.createElement('div');
      el.textContent = 'Home';
      return el;
    });
    router.register('/about', () => {
      const el = document.createElement('div');
      el.textContent = 'About';
      return el;
    });

    await router.navigate('/');
    const app = document.getElementById('app');
    expect(app.textContent).toContain('Home');

    await router.navigate('/about');
    expect(app.textContent).toContain('About');
  });

  it('should support route params', async () => {
    const router = new window.Router({ container: document.getElementById('app') });
    let capturedParams = null;
    router.register('/user/:id', (params) => {
      capturedParams = params;
      const el = document.createElement('div');
      el.textContent = `User ${params.id}`;
      return el;
    });

    await router.navigate('/user/42');
    expect(capturedParams).toBeDefined();
    expect(capturedParams.id).toEqual('42');
    const app = document.getElementById('app');
    expect(app.textContent).toContain('User 42');
  });

  it('should handle 404 routes', async () => {
    const router = new window.Router({ container: document.getElementById('app') });
    router.register('/', () => {
      const el = document.createElement('div');
      el.textContent = 'Home';
      return el;
    });
    router.register('*', () => {
      const el = document.createElement('div');
      el.textContent = 'Not Found';
      return el;
    });

    await router.navigate('/nonexistent');
    const app = document.getElementById('app');
    expect(app.textContent).toContain('Not Found');
  });

  it('should support navigation guards', async () => {
    const router = new window.Router({ container: document.getElementById('app') });
    router.register('/', () => {
      const el = document.createElement('div');
      el.textContent = 'Home';
      return el;
    });
    router.register('/protected', () => {
      const el = document.createElement('div');
      el.textContent = 'Protected';
      return el;
    });

    // Add guard
    router.beforeEach = async (to, from) => {
      if (to === '/protected') {
        return '/'; // redirect to home
      }
      return to;
    };

    await router.navigate('/protected');
    const app = document.getElementById('app');
    expect(app.textContent).toContain('Home');
  });

  it('should support query parameters', () => {
    const url = '/search?q=test&page=1';
    const params = new window.URLSearchParams(url.split('?')[1]);
    expect(params.get('q')).toEqual('test');
    expect(params.get('page')).toEqual('1');
  });

  it('should support history pushState and popState', () => {
    window.history.pushState({}, '', '/test');
    expect(window.location.pathname).toEqual('/test');
    window.history.pushState({}, '', '/');
    expect(window.location.pathname).toEqual('/');
  });
});

describe('Router — Lifecycle', () => {
  it('should call on_mounted after route change', async () => {
    const router = new window.Router({ container: document.getElementById('app') });
    let mounted = false;
    router.register('/', () => {
      const el = document.createElement('div');
      el.textContent = 'Home';
      return { element: el, lifecycle: { mounted: () => { mounted = true; }, unmounted: null, updated: null } };
    });

    await router.navigate('/');
    expect(mounted).toEqual(true);
  });

  it('should call on_unmounted when leaving route', async () => {
    const router = new window.Router({ container: document.getElementById('app') });
    let unmounted = false;
    router.register('/', () => {
      const el = document.createElement('div');
      el.textContent = 'Home';
      return { element: el, lifecycle: { mounted: null, unmounted: () => { unmounted = true; }, updated: null } };
    });
    router.register('/other', () => {
      const el = document.createElement('div');
      el.textContent = 'Other';
      return el;
    });

    await router.navigate('/');
    await router.navigate('/other');
    expect(unmounted).toEqual(true);
  });
});
