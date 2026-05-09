const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { execFileSync } = require('node:child_process');
const { pathToFileURL } = require('node:url');

const { createDom } = require('../../support/fake-dom');

const ROOT = path.resolve(__dirname, '../../..');

// Resolve the stock UI bundle through Cargo so tests exercise the crates.io
// dependency selected by Cargo.toml instead of an in-tree source copy.
function resolveSolverForgeUiRuntime() {
  const metadata = JSON.parse(execFileSync('cargo', ['metadata', '--format-version', '1'], {
    cwd: ROOT,
    encoding: 'utf8',
  }));
  const uiPackage = metadata.packages.find((pkg) => pkg.name === 'solverforge-ui');
  if (!uiPackage) {
    throw new Error('cargo metadata did not include solverforge-ui');
  }
  return path.resolve(path.dirname(uiPackage.manifest_path), 'static/sf/sf.js');
}

const SF_RUNTIME_PATH = resolveSolverForgeUiRuntime();
const SF_SOURCE = fs.readFileSync(SF_RUNTIME_PATH, 'utf8');

// Tiny JSON response stub that behaves just enough like `fetch()` for these tests.
function createJsonResponse(value) {
  return {
    ok: true,
    status: 200,
    statusText: 'OK',
    headers: {
      get(name) {
        return String(name).toLowerCase() === 'content-type' ? 'application/json' : null;
      },
    },
    json: async () => value,
    text: async () => JSON.stringify(value),
  };
}

// Loads the shipped SolverForge UI runtime into the fake browser environment.
function loadSfRuntime(window, document, Node) {
  const wrapped = `(function(window, document, Node) { ${SF_SOURCE}\n return window.SF; })`;
  const factory = vm.runInThisContext(wrapped, {
    filename: SF_RUNTIME_PATH,
  });
  return factory(window, document, Node);
}

// Saves one global so the test harness can restore the real environment later.
function saveGlobal(name) {
  return {
    name,
    existed: Object.prototype.hasOwnProperty.call(globalThis, name),
    value: globalThis[name],
  };
}

// Restores every saved global after a test finishes.
function restoreGlobals(saved) {
  saved.forEach((entry) => {
    if (entry.existed) globalThis[entry.name] = entry.value;
    else delete globalThis[entry.name];
  });
}

// Adds a cache-busting query string so repeated dynamic imports reload the module.
function uniqueModuleUrl(relativePath) {
  const url = new URL(pathToFileURL(path.resolve(ROOT, relativePath)).href);
  url.searchParams.set('t', `${Date.now()}-${Math.random()}`);
  return url.href;
}

// Convenience wrapper used by the test files.
async function importModule(relativePath) {
  return import(uniqueModuleUrl(relativePath));
}

// Creates a fake browser environment, runs the callback, then restores globals.
async function withBrowserEnv(options, run) {
  const settings = options || {};
  const { document, window, Node } = createDom();
  const errors = [];
  const appRoot = settings.appRoot === false ? null : document.createElement('div');
  if (appRoot) {
    appRoot.id = 'sf-app';
    document.body.appendChild(appRoot);
  }

  const fauxConsole = {
    log: console.log,
    warn: console.warn,
    info: console.info,
    error(...args) {
      errors.push(args.map((arg) => String(arg)).join(' '));
    },
  };
  const fetchStub = settings.fetch || (async function unexpectedFetch(url) {
    throw new Error(`unexpected fetch ${url}`);
  });
  const eventSourceStub = settings.EventSource || function EventSource() {
    throw new Error('EventSource should not be used in this test');
  };
  const location = settings.location || { origin: 'http://localhost:7861' };

  window.window = window;
  window.document = document;
  window.Node = Node;
  window.console = fauxConsole;
  window.fetch = fetchStub;
  window.EventSource = eventSourceStub;
  window.location = location;

  const saved = ['window', 'document', 'Node', 'fetch', 'EventSource', 'location', 'SF', 'console']
    .map(saveGlobal);
  globalThis.window = window;
  globalThis.document = document;
  globalThis.Node = Node;
  globalThis.fetch = fetchStub;
  globalThis.EventSource = eventSourceStub;
  globalThis.location = location;
  globalThis.console = fauxConsole;
  globalThis.SF = loadSfRuntime(window, document, Node);
  window.SF = globalThis.SF;

  try {
    return await run({
      ROOT,
      document,
      window,
      Node,
      appRoot,
      errors,
      importModule,
      createJsonResponse,
    });
  } finally {
    restoreGlobals(saved);
  }
}

// High-level helper that boots the full app with stubbed config and demo data.
async function loadFullApp(options = {}) {
  const sfConfig = options.sfConfig || JSON.parse(
    fs.readFileSync(path.resolve(ROOT, 'static/sf-config.json'), 'utf8'),
  );
  const uiModel = options.uiModel || JSON.parse(
    fs.readFileSync(path.resolve(ROOT, 'static/generated/ui-model.json'), 'utf8'),
  );
  const demoData = options.demoData || { employees: [], shifts: [] };

  return withBrowserEnv({
    fetch: async (url) => {
      if (url === '/sf-config.json') return createJsonResponse(sfConfig);
      if (url === '/generated/ui-model.json') return createJsonResponse(uiModel);
      if (url === '/demo-data') return createJsonResponse([String(sfConfig.defaultDemoId || 'LARGE')]);
      if (url === `/demo-data/${String(sfConfig.defaultDemoId || 'LARGE')}`) return createJsonResponse(demoData);
      throw new Error(`unexpected fetch ${url}`);
    },
  }, async ({ document, errors, importModule, window }) => {
    const { bootApp } = await importModule('static/app/main.mjs');
    await bootApp(window);
    await new Promise((resolve) => setTimeout(resolve, 0));
    return { document, errors };
  });
}

module.exports = {
  ROOT,
  createJsonResponse,
  importModule,
  loadFullApp,
  withBrowserEnv,
};
