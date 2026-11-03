import { buildBrowserModel } from './app-model.mjs';

// Lightweight JSON fetch helper used during browser boot.
async function requestJson(root, path, options = undefined) {
  const fetchFn = root.fetch || globalThis.fetch;
  if (typeof fetchFn !== 'function') {
    throw new Error(`No fetch implementation is available for ${path}`);
  }
  const response = await fetchFn(path, options);
  if (!response || typeof response.json !== 'function') {
    throw new Error(`Expected JSON response for ${path}`);
  }
  return response.json();
}

// Guards the human-authored config file before the app uses it.
function validateConfig(config) {
  if (!config || typeof config.title !== 'string' || typeof config.subtitle !== 'string') {
    throw new Error('sf-config.json must define title and subtitle');
  }
  if (!String(config.defaultDemoId || '').trim()) {
    throw new Error('sf-config.json must define defaultDemoId');
  }
}

// Guards the CLI-generated UI model before the app reads collections and constraint ids from it.
function validateUiModel(uiModel) {
  const valid = uiModel
    && Array.isArray(uiModel.constraints)
    && Array.isArray(uiModel.entities)
    && Array.isArray(uiModel.facts);
  if (!valid) {
    throw new Error('generated/ui-model.json must define constraints, entities, and facts arrays');
  }
}

// Guards the app-owned labels, constraint catalog, and schedule views.
function validateAppModel(appModel) {
  const valid = appModel
    && appModel.collections
    && typeof appModel.collections === 'object'
    && Array.isArray(appModel.constraints)
    && Array.isArray(appModel.views);
  if (!valid) {
    throw new Error('app/app-model.json must define collections, constraints, and views');
  }
  if (!appModel.views.length) {
    throw new Error('app/app-model.json must define at least one view');
  }
}

// Loads every browser-boot dependency and returns the assembled boot context.
export async function loadAppConfig(root = globalThis) {
  const [config, generatedUiModel, appModel] = await Promise.all([
    requestJson(root, '/sf-config.json'),
    requestJson(root, '/generated/ui-model.json'),
    requestJson(root, '/app/app-model.json'),
  ]);

  validateConfig(config);
  validateUiModel(generatedUiModel);
  validateAppModel(appModel);
  const uiModel = buildBrowserModel(generatedUiModel, appModel);

  const backend = root.SF.createBackend({ type: 'axum', baseUrl: '' });
  const availableDemoIds = await backend.listDemoData();
  const demoId = String(config.defaultDemoId).trim();
  if (!Array.isArray(availableDemoIds) || !availableDemoIds.includes(demoId)) {
    throw new Error(`Configured demo "${demoId}" is not exposed by /demo-data`);
  }

  return {
    config,
    uiModel,
    backend,
    demoId,
    availableDemoIds,
  };
}
