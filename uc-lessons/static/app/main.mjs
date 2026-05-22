/* main.mjs — Controller for solverforge-lessons */

import {
  clonePlan,
  title,
  entityLabel,
  toneForKey,
  buildScalarViewPayload,
  buildListViewPayload,
  buildStatusBarConstraints,
  buildAnalysisHtml,
  describeError,
  requestJson,
  fetchDemoCatalog,
  fetchDemoPlan,
} from './models/index.mjs';
import {
  createLayout,
  setActiveTab,
  renderOverview,
  renderViews,
  renderTimelinePanel,
  destroyAllTimelines,
  renderByGroup,
  renderByRoom,
  renderByTeacher,
  renderTables,
  renderApiGuide,
} from './ui/index.mjs';

const SLOT_MINUTES = 60;
const DEFAULT_VIEWPORT_SLOTS = 12;
const TIMELINE_TONES = ['emerald', 'blue', 'amber', 'rose', 'violet', 'slate'];

const DEFAULT_DEMO = 'LARGE';

function findHeaderButton(header, label) {
  var buttons = header.querySelectorAll('button');
  for (var i = 0; i < buttons.length; i += 1) {
    var text = (buttons[i].textContent || '').trim();
    if (text === label) {
      return buttons[i];
    }
  }
  return null;
}

function canSolve(bootstrapError, demoCatalog) {
  return !bootstrapError && !!demoCatalog.defaultId;
}

function updateSolveActionAvailability(layout, solver, bootstrapError, demoCatalog) {
  var solveButton = findHeaderButton(layout.header, 'Solve');
  var disabled = !canSolve(bootstrapError, demoCatalog);
  if (!solveButton) return;
  solveButton.disabled = disabled;
  solveButton.setAttribute('aria-disabled', disabled ? 'true' : 'false');
  solveButton.title = disabled
    ? (bootstrapError ? 'Demo data bootstrap failed.' : 'Loading demo data catalog...')
    : '';
}

function reportBootstrapError(layout, err, demoCatalog, SF) {
  var bootstrapError = describeError(err);
  layout.bootstrapNotice.textContent = 'Demo data bootstrap failed: ' + bootstrapError;
  layout.bootstrapNotice.style.display = '';
  layout.app.dataset.bootstrapError = 'true';
  renderApiGuide(layout.apiGuideContainer, demoCatalog, SF);
  updateSolveActionAvailability(layout, solver, bootstrapError, demoCatalog);
  console.error('Demo data bootstrap failed:', err);
}

function clearBootstrapError(layout) {
  layout.bootstrapNotice.textContent = '';
  layout.bootstrapNotice.style.display = 'none';
  delete layout.app.dataset.bootstrapError;
}

export async function boot() {
  var config = await fetch('/sf-config.json').then(function (response) { return response.json(); });
  var uiModel = await fetch('/generated/ui-model.json').then(function (response) { return response.json(); });
  var appElement = document.getElementById('sf-app');

  var backend = SF.createBackend({ baseUrl: '' });
  var demoCatalog = { defaultId: null, availableIds: [] };
  var bootstrapError = null;
  var currentPlan = null;
  var lastAnalysis = null;
  var customTimelines = {};
  var viewTimelines = {};

  var statusBar = SF.createStatusBar({
    constraints: buildStatusBarConstraints(uiModel.constraints || config.constraints || []),
    onConstraintClick: function () { openAnalysis(); },
  });

  var layout = createLayout({
    app: appElement,
    config: config,
    statusBar: statusBar,
    actions: {
      onSolve: function () { loadAndSolve(); },
      onPause: function () { pauseSolve(); },
      onResume: function () { resumeSolve(); },
      onCancel: function () { cancelSolve(); },
      onAnalyze: function () { openAnalysis(); },
    },
    onTabChange: function (tab) { setActiveTab(layout, tab); },
    SF: SF,
  });

  var analysisModal = layout.analysisModal;

  var solver = SF.createSolver({
    backend: backend,
    statusBar: statusBar,
    onProgress: syncLifecycleMarkers,
    onPauseRequested: syncLifecycleMarkers,
    onSolution: handleSnapshotEvent,
    onPaused: handleSnapshotEvent,
    onResumed: syncLifecycleMarkers,
    onCancelled: handleSnapshotEvent,
    onComplete: handleSnapshotEvent,
    onFailure: function (message, meta, snapshot, analysis) {
      handleSnapshotEvent(snapshot, meta);
      if (analysis) { lastAnalysis = analysis; }
      console.error('Solver job failed:', message);
      syncLifecycleMarkers(meta);
    },
    onAnalysis: function (analysis) {
      lastAnalysis = analysis;
      analysisModal.setBody(buildAnalysisHtml(analysis, SF));
      syncLifecycleMarkers();
    },
    onError: function (message) {
      console.error('Solver lifecycle failed:', message);
      syncLifecycleMarkers();
    },
  });

  window.addEventListener('beforeunload', function () { destroyAllTimelines(viewTimelines, customTimelines); });

  renderApiGuide(layout.apiGuideContainer, demoCatalog, SF);
  updateSolveActionAvailability(layout, solver, bootstrapError, demoCatalog);
  bootstrapDemoData();

  function handleSnapshotEvent(snapshot, meta) {
    if (snapshot && snapshot.solution) {
      renderAll(snapshot.solution);
    }
    syncLifecycleMarkers(meta);
  }

  function renderAll(data) {
    currentPlan = clonePlan(data);
    renderOverview(layout.overviewContainer, uiModel, data);
    renderViews(data, uiModel, layout.viewPanels, function (container, viewId, payload, emptyMessage) {
      renderTimelinePanel(container, viewId, payload, emptyMessage, viewTimelines, SF);
    }, buildListViewPayload, buildScalarViewPayload);
    renderTables(layout.tablesContainer, uiModel, data);
    renderByGroup(data, layout.byGroupContainer, SF, toneForKey, entityLabel, customTimelines);
    renderByRoom(data, layout.byRoomContainer, SF, toneForKey, entityLabel, customTimelines);
    renderByTeacher(data, layout.byTeacherContainer, SF, toneForKey, entityLabel, customTimelines);
  }

  function loadAndSolve() {
    if (solver.isRunning() || solver.getLifecycleState() === 'PAUSED' || !canSolve(bootstrapError, demoCatalog)) return;
    cleanupTerminalJob()
      .then(function (data) {
        return data || resolvePlanForSolve();
      })
      .then(function (data) {
        return solver.start(data);
      })
      .then(function () {
        syncLifecycleMarkers();
      })
      .catch(function (err) { console.error('Solve start failed:', err); });
  }

  function pauseSolve() {
    solver.pause()
      .then(syncLifecycleMarkers)
      .catch(function (err) { console.error('Pause failed:', err); });
  }

  function resumeSolve() {
    solver.resume()
      .then(syncLifecycleMarkers)
      .catch(function (err) { console.error('Resume failed:', err); });
  }

  function cancelSolve() {
    solver.cancel()
      .then(syncLifecycleMarkers)
      .catch(function (err) { console.error('Cancel failed:', err); });
  }

  function openAnalysis() {
    if (!solver.getJobId()) return;
    solver.analyzeSnapshot()
      .then(function (analysis) {
        lastAnalysis = analysis;
        analysisModal.setBody(buildAnalysisHtml(analysis, SF));
        analysisModal.open();
      })
      .catch(function () { });
  }

  function resolvePlanForSolve() {
    if (currentPlan) {
      return Promise.resolve(clonePlan(currentPlan));
    }
    if (!demoCatalog.defaultId) {
      return Promise.reject(new Error('demo data catalog is unavailable'));
    }
    return fetchDemoPlan(demoCatalog.defaultId);
  }

  function cleanupTerminalJob() {
    var state = solver.getLifecycleState();
    if (!solver.getJobId() || state === 'IDLE' || state === 'PAUSED' || solver.isRunning()) {
      return Promise.resolve(null);
    }
    return solver.delete()
      .then(function () {
        lastAnalysis = null;
        syncLifecycleMarkers();
        return null;
      })
      .catch(function (err) {
        console.error('Delete failed:', err);
        throw err;
      });
  }

  function syncLifecycleMarkers(meta) {
    var jobId = solver.getJobId();
    var snapshotRevision = solver.getSnapshotRevision();
    var lifecycleState = meta && meta.lifecycleState ? meta.lifecycleState : solver.getLifecycleState();

    if (jobId) {
      layout.app.dataset.jobId = String(jobId);
    } else {
      delete layout.app.dataset.jobId;
    }
    if (snapshotRevision != null) {
      layout.app.dataset.snapshotRevision = String(snapshotRevision);
    } else {
      delete layout.app.dataset.snapshotRevision;
    }
    if (lifecycleState && lifecycleState !== 'IDLE') {
      layout.app.dataset.lifecycleState = lifecycleState;
    } else {
      delete layout.app.dataset.lifecycleState;
    }
    updateSolveActionAvailability(layout, solver, bootstrapError, demoCatalog);
  }

  function bootstrapDemoData() {
    fetchDemoCatalog()
      .then(function (catalog) {
        demoCatalog = catalog;
        clearBootstrapError(layout);
        renderApiGuide(layout.apiGuideContainer, demoCatalog, SF);
        return fetchDemoPlan(catalog.defaultId);
      })
      .then(function (data) {
        renderAll(data);
        updateSolveActionAvailability(layout, solver, bootstrapError, demoCatalog);
      })
      .catch(function (err) {
        reportBootstrapError(layout, err, demoCatalog, SF);
      });
  }
}
