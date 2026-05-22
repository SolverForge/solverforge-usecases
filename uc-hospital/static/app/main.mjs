import { buildAnalysisBody } from './models/analysis-modal.mjs';
import { createAppShell } from './ui/app-shell.mjs';
import { createAppState } from './ui/app-state.mjs';
import { loadAppConfig } from './ui/config-loader.mjs';
import { renderDataTables } from './ui/data-panel.mjs';
import { createSolverController } from './ui/solver-controller.mjs';
import { createViewRegistry } from './ui/registry.mjs';

// Browser entrypoint that wires together config loading, the shared UI shell,
// hospital-specific view renderers, and the retained-job controller.
export async function bootApp(root = globalThis) {
  const document = root.document;
  const sf = root.SF;
  const appElement = document && document.getElementById('sf-app');
  if (!document || !appElement) return null;
  if (!sf) {
    throw new Error('SolverForge UI must be loaded before bootApp()');
  }

  const { config, uiModel, backend, demoId } = await loadAppConfig(root);
  const state = createAppState(uiModel.views[0].id);
  const statusBar = sf.createStatusBar({ constraints: uiModel.constraints });
  const shell = createAppShell({
    root,
    sf,
    appElement,
    config,
    uiModel,
    demoId,
    statusBar,
    activeTab: state.activeTab,
    actions: {
      onSolve: () => startSolve(),
      onPause: () => controller.pause(),
      onResume: () => controller.resume(),
      onCancel: () => controller.cancel(),
      onAnalyze: () => openAnalysis(),
      onTabChange(tabId) {
        state.activeTab = tabId;
      },
    },
  });
  const views = createViewRegistry();
  const controller = createSolverController({
    sf,
    backend,
    statusBar,
    onPlan(plan) {
      renderAll(plan);
    },
    onAnalysis() {},
    onMeta() {},
    onLifecycle(markers) {
      shell.syncLifecycleMarkers(markers);
    },
    onError(error) {
      root.console.error('Solver lifecycle failed:', error);
    },
  });

  try {
    const demoData = await backend.getDemoData(demoId);
    renderAll(demoData);
  } catch (error) {
    root.console.error('Initial demo load failed:', error);
  }

  return {
    backend,
    controller,
    shell,
    state,
    uiModel,
  };

  // Re-renders both schedule tabs and the raw data tables from the latest plan.
  function renderAll(data) {
    state.currentPlan = clonePlan(data);
    renderViews(data);
    renderDataTables({ sf, container: shell.dataRoot, uiModel, data });
  }

  // Dispatches each configured view to the renderer registered for its `kind`.
  function renderViews(data) {
    uiModel.views.forEach((view) => {
      const container = shell.viewRoots[view.id];
      const renderView = views[view.kind];
      if (!container) return;
      container.innerHTML = '';
      if (!renderView) {
        container.appendChild(sf.el('p', null, `No renderer is registered for ${view.kind}.`));
        return;
      }
      renderView({ sf, container, data, view });
    });
  }

  // Starts solving from the last rendered plan snapshot.
  async function startSolve() {
    const plan = await resolvePlanForSolve();
    await controller.start(() => Promise.resolve(clonePlan(plan)));
  }

  // Lazily loads demo data if the user clicks Solve before the first fetch finishes.
  async function resolvePlanForSolve() {
    if (state.currentPlan) {
      return state.currentPlan;
    }

    const demoData = await backend.getDemoData(demoId);
    renderAll(demoData);
    return state.currentPlan;
  }

  // Fetches exact retained-snapshot analysis and opens it in the shared modal.
  async function openAnalysis() {
    if (!controller.getJobId()) return Promise.resolve();
    try {
      const currentAnalysis = await controller.analyzeSnapshot();
      shell.openAnalysis(buildAnalysisBody(document, currentAnalysis, uiModel.constraints));
    } catch (error) {
      root.console.error('Analysis failed:', error);
    }
  }
}

// Defensive deep clone so the UI never mutates the last backend payload in place.
function clonePlan(data) {
  return JSON.parse(JSON.stringify(data));
}

// MVC entry pattern compatibility
export async function boot() {
  return bootApp();
}
