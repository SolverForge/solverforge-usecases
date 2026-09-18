import { openAnalysis, openConstraintAnalysis, syncConstraintStatus } from './analysis.js';
import { buildApiGuideEndpoints } from './api.js';
import { cleanupTerminalJob, syncLifecycleMarkers } from './lifecycle.js';
import { renderAll } from './render.js';
import { destroyAllTimelines } from './schedule.js';
import { clonePlan } from './utils.js';

var SF = window.SF;

export async function startApp() {
  var responses = await Promise.all([
    fetch('/sf-config.json').then(function (response) { return response.json(); }),
    fetch('/generated/ui-model.json').then(function (response) { return response.json(); }),
  ]);
  var config = responses[0];
  var uiModel = responses[1];
  var app = document.getElementById('sf-app');
  var backend = SF.createBackend({ baseUrl: '' });
  var ctx = {
    app: app,
    backend: backend,
    config: config,
    uiModel: uiModel,
    currentPlan: null,
    lastAnalysis: null,
    activeTab: (uiModel.views && uiModel.views.length) ? uiModel.views[0].id : 'overview',
    viewPanels: {},
    viewTimelines: {},
    statusBar: null,
    overviewPanel: null,
    dataPanel: null,
    apiPanel: null,
    overviewContainer: null,
    tablesContainer: null,
    analysisModal: null,
    solver: null,
  };

  var statusBar = SF.createStatusBar({
    constraints: uiModel.constraints || [],
    onConstraintClick: function (index) {
      openConstraintAnalysis(ctx, (ctx.uiModel.constraints || [])[index]);
    },
  });
  ctx.statusBar = statusBar;

  var tabs = (uiModel.views || []).map(function (view, index) {
    return {
      id: view.id,
      label: view.label,
      icon: view.icon || 'fa-table-cells-large',
      active: index === 0,
    };
  });
  if (!tabs.length) {
    tabs.push({ id: 'overview', label: 'Overview', icon: 'fa-compass', active: true });
  }
  tabs.push({ id: 'data', label: 'Data', icon: 'fa-table' });
  tabs.push({ id: 'api', label: 'REST API', icon: 'fa-book' });

  var header = SF.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: config.title,
    subtitle: config.subtitle,
    tabs: tabs,
    actions: {
      onSolve: function () { loadAndSolve(ctx); },
      onPause: function () { pauseSolve(ctx); },
      onResume: function () { resumeSolve(ctx); },
      onCancel: function () { cancelSolve(ctx); },
      onAnalyze: function () { openAnalysis(ctx); },
    },
    onTabChange: function (tab) {
      ctx.activeTab = tab;
      Object.keys(ctx.viewPanels).forEach(function (key) {
        ctx.viewPanels[key].style.display = key === tab ? '' : 'none';
      });
      ctx.overviewPanel.style.display = tab === 'overview' ? '' : 'none';
      ctx.dataPanel.style.display = tab === 'data' ? '' : 'none';
      ctx.apiPanel.style.display = tab === 'api' ? '' : 'none';
    },
  });
  app.appendChild(header);
  statusBar.bindHeader(header);
  app.appendChild(statusBar.el);

  ctx.overviewPanel = SF.el('div', { className: 'sf-content', style: { display: ctx.activeTab === 'overview' ? '' : 'none' } });
  ctx.overviewContainer = SF.el('div', { id: 'sf-overview' });
  ctx.overviewPanel.appendChild(ctx.overviewContainer);
  app.appendChild(ctx.overviewPanel);

  (uiModel.views || []).forEach(function (view) {
    var panel = SF.el('div', { className: 'sf-content', style: { display: ctx.activeTab === view.id ? '' : 'none' } });
    panel.appendChild(SF.el('div', { id: 'view-' + view.id }));
    ctx.viewPanels[view.id] = panel;
    app.appendChild(panel);
  });

  ctx.dataPanel = SF.el('div', { className: 'sf-content', style: { display: 'none' } });
  ctx.tablesContainer = SF.el('div', { id: 'sf-tables' });
  ctx.dataPanel.appendChild(ctx.tablesContainer);
  app.appendChild(ctx.dataPanel);

  ctx.apiPanel = SF.el('div', { className: 'sf-content', style: { display: 'none' } });
  ctx.apiPanel.appendChild(SF.createApiGuide({
    endpoints: buildApiGuideEndpoints(),
  }));
  app.appendChild(ctx.apiPanel);

  app.appendChild(SF.createFooter({
    links: [
      { label: 'SolverForge', url: 'https://www.solverforge.org' },
      { label: 'Docs', url: 'https://www.solverforge.org/docs' },
    ],
  }));

  ctx.analysisModal = SF.createModal({ title: 'Score Analysis', width: '760px' });
  ctx.solver = SF.createSolver({
    backend: backend,
    statusBar: statusBar,
    onProgress: function (meta) {
      syncLifecycleMarkers(ctx, meta);
    },
    onPauseRequested: function (meta) {
      syncLifecycleMarkers(ctx, meta);
    },
    onSolution: function (snapshot, meta) {
      if (snapshot && snapshot.solution) {
        renderAll(ctx, snapshot.solution);
      }
      syncLifecycleMarkers(ctx, meta);
    },
    onPaused: function (snapshot, meta) {
      if (snapshot && snapshot.solution) {
        renderAll(ctx, snapshot.solution);
      }
      syncLifecycleMarkers(ctx, meta);
    },
    onResumed: function (meta) {
      syncLifecycleMarkers(ctx, meta);
    },
    onCancelled: function (snapshot, meta) {
      if (snapshot && snapshot.solution) {
        renderAll(ctx, snapshot.solution);
      }
      syncLifecycleMarkers(ctx, meta);
    },
    onComplete: function (snapshot, meta) {
      if (snapshot && snapshot.solution) {
        renderAll(ctx, snapshot.solution);
      }
      syncLifecycleMarkers(ctx, meta);
    },
    onFailure: function (message, meta, snapshot, analysis) {
      if (snapshot && snapshot.solution) {
        renderAll(ctx, snapshot.solution);
      }
      if (analysis) {
        ctx.lastAnalysis = analysis;
      }
      console.error('Solver job failed:', message);
      syncLifecycleMarkers(ctx, meta);
    },
    onAnalysis: function (analysis) {
      ctx.lastAnalysis = analysis;
      syncConstraintStatus(ctx);
    },
    onError: function (message) {
      console.error('Solver lifecycle failed:', message);
      syncLifecycleMarkers(ctx);
    },
  });

  fetch('/demo-data/STANDARD')
    .then(function (response) { return response.json(); })
    .then(function (data) { renderAll(ctx, data); })
    .catch(function (error) { console.error('Demo data load failed:', error); });

  window.addEventListener('beforeunload', function () {
    destroyAllTimelines(ctx.viewTimelines);
  });
}

function loadAndSolve(ctx) {
  if (ctx.solver.isRunning() || ctx.solver.getLifecycleState() === 'PAUSED') return;
  cleanupTerminalJob(ctx)
    .then(function () { return clonePlan(ctx.currentPlan); })
    .then(function (data) {
      return ctx.solver.start(data);
    })
    .then(function () {
      syncLifecycleMarkers(ctx);
    })
    .catch(function (err) { console.error('Solve start failed:', err); });
}

function pauseSolve(ctx) {
  ctx.solver.pause()
    .then(function () { syncLifecycleMarkers(ctx); })
    .catch(function (err) { console.error('Pause failed:', err); });
}

function resumeSolve(ctx) {
  ctx.solver.resume()
    .then(function () { syncLifecycleMarkers(ctx); })
    .catch(function (err) { console.error('Resume failed:', err); });
}

function cancelSolve(ctx) {
  ctx.solver.cancel()
    .then(function () { syncLifecycleMarkers(ctx); })
    .catch(function (err) { console.error('Cancel failed:', err); });
}
