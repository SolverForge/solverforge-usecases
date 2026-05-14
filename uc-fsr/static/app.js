/* app.js - lifecycle bootstrap for the Bergamo field service routing demo */

(async function () {
  'use strict';

  var DAY_START = 8 * 60;
  var DAY_END = 18 * 60;
  var DEFAULT_CENTER = [45.698, 9.677];
  var utils = window.FSR.utils;

  var config = await fetch('/sf-config.json').then(function (response) { return response.json(); });
  var uiModel = await fetch('/generated/ui-model.json').then(function (response) { return response.json(); });
  var app = document.getElementById('sf-app');
  app.className = 'sf-app fsr-app';

  var backend = SF.createBackend({ baseUrl: '' });
  var statusBar = SF.createStatusBar({ constraints: uiModel.constraints || [] });
  var currentPlan = null;
  var currentRoutes = null;
  var demoData = null;
  var bootstrapError = null;
  var lastAnalysis = null;
  var routeMap = null;
  var renderer = null;
  var routeGeometry = null;
  var focusedRouteId = null;

  var layout = window.FSR.createAppLayout({
    SF: SF,
    app: app,
    config: config,
    statusBar: statusBar,
    onSolve: function () { loadAndSolve(); },
    onPause: function () { pauseSolve(); },
    onResume: function () { resumeSolve(); },
    onCancel: function () { cancelSolve(); },
    onAnalyze: function () { openAnalysis(); },
    onMapTabShown: function () {
      window.setTimeout(function () {
        if (routeMap && routeMap.map && routeMap.map.invalidateSize) routeMap.map.invalidateSize();
        renderCurrentPlan();
      }, 80);
    },
  });

  demoData = window.FSR.createDemoDataController({
    SF: SF,
    getCurrentPlan: function () { return currentPlan; },
    onCatalog: function () { if (renderer) renderer.renderApiGuide(); },
    onError: reportBootstrapError,
    onLoadingChange: updateSolveActionAvailability,
    onPlan: handleDemoPlanLoaded,
    utils: utils,
  });
  app.insertBefore(demoData.el, layout.bootstrapNotice.nextSibling);

  routeMap = SF.map.create({ container: 'fsr-map', center: DEFAULT_CENTER, zoom: 13 });
  renderer = window.FSR.createRenderer({
    SF: SF,
    apiGuideContainer: layout.apiGuideContainer,
    dayEnd: DAY_END,
    dayStart: DAY_START,
    getFocusedRouteId: function () { return focusedRouteId; },
    getDemoCatalog: function () { return demoData.getCatalog(); },
    getSelectedDemoId: function () { return demoData.getSelectedId(); },
    onFocusRoute: focusRoute,
    routeCards: layout.routeCards,
    routeMap: routeMap,
    summaryContainer: layout.summaryContainer,
    tablesContainer: layout.tablesContainer,
    timelineContainer: layout.timelineContainer,
  });

  var analysisModal = SF.createModal({ title: 'Score Analysis', width: '760px' });
  var solver = SF.createSolver({
    backend: backend,
    statusBar: statusBar,
    onProgress: function (meta) { syncLifecycleMarkers(meta); },
    onPauseRequested: function (meta) { syncLifecycleMarkers(meta); },
    onSolution: function (snapshot, meta) {
      renderSnapshot(snapshot, meta);
      syncLifecycleMarkers(meta);
    },
    onPaused: function (snapshot, meta) {
      renderSnapshot(snapshot, meta);
      syncLifecycleMarkers(meta);
    },
    onResumed: function (meta) { syncLifecycleMarkers(meta); },
    onCancelled: function (snapshot, meta) {
      renderSnapshot(snapshot, meta);
      syncLifecycleMarkers(meta);
    },
    onComplete: function (snapshot, meta) {
      renderSnapshot(snapshot, meta);
      syncLifecycleMarkers(meta);
    },
    onFailure: function (message, meta, snapshot, analysis) {
      renderSnapshot(snapshot, meta);
      if (analysis) lastAnalysis = analysis;
      console.error('Solver job failed:', message);
      syncLifecycleMarkers(meta);
    },
    onAnalysis: function (analysis) {
      lastAnalysis = analysis;
      syncLifecycleMarkers();
    },
    onError: function (message) {
      console.error('Solver lifecycle failed:', message);
      syncLifecycleMarkers();
    },
  });

  routeGeometry = window.FSR.createRouteGeometryController({
    solver: solver,
    utils: utils,
    onClearFocus: function () { focusedRouteId = null; },
    onRoutesChange: function (routes) {
      currentRoutes = routes;
      renderCurrentPlan();
    },
  });

  renderer.renderApiGuide();
  updateSolveActionAvailability();
  demoData.bootstrap();
  window.addEventListener('beforeunload', function () { renderer.destroy(); });

  function loadAndSolve() {
    if (solver.isRunning() || solver.getLifecycleState() === 'PAUSED' || !canSolve()) return;
    routeGeometry.invalidate();
    cleanupTerminalJob()
      .then(function () { return demoData.resolvePlan(); })
      .then(function (data) { return solver.start(data); })
      .then(function () { syncLifecycleMarkers(); })
      .catch(function (err) { console.error('Solve start failed:', err); });
  }

  function pauseSolve() {
    solver.pause().then(function () { syncLifecycleMarkers(); }).catch(function (err) { console.error('Pause failed:', err); });
  }

  function resumeSolve() {
    solver.resume().then(function () { syncLifecycleMarkers(); }).catch(function (err) { console.error('Resume failed:', err); });
  }

  function cancelSolve() {
    solver.cancel().then(function () { syncLifecycleMarkers(); }).catch(function (err) { console.error('Cancel failed:', err); });
  }

  function openAnalysis() {
    var jobId = solver.getJobId();
    if (jobId == null || jobId === '') return;
    solver.analyzeSnapshot()
      .then(function (analysis) {
        lastAnalysis = analysis;
        analysisModal.setBody(renderer.buildAnalysisBody(analysis));
        analysisModal.open();
      })
      .catch(function (err) { console.error('Analysis failed:', err); });
  }

  function cleanupTerminalJob() {
    var state = solver.getLifecycleState();
    var jobId = solver.getJobId();
    if (jobId == null || jobId === '' || state === 'IDLE' || state === 'PAUSED' || solver.isRunning()) {
      return Promise.resolve(null);
    }
    return solver.delete().then(function () {
      lastAnalysis = null;
      syncLifecycleMarkers();
    });
  }

  function renderSnapshot(snapshot, meta) {
    var identity = routeGeometry.identityFrom(meta);
    if (snapshot && snapshot.solution) {
      routeGeometry.setPlanIdentity(identity);
      renderPlan(snapshot.solution);
    }
    routeGeometry.load(identity);
  }

  function handleDemoPlanLoaded(plan) {
    lastAnalysis = null;
    routeGeometry.invalidate();
    clearBootstrapError();
    renderPlan(plan);
    renderer.renderApiGuide();
    syncLifecycleMarkers();
  }

  function renderPlan(plan) {
    currentPlan = utils.clonePlan(plan);
    if (focusedRouteId && !hasRoute(currentPlan, focusedRouteId)) focusedRouteId = null;
    renderCurrentPlan();
  }

  function renderCurrentPlan() {
    if (renderer && currentPlan) renderer.renderAll(currentPlan, currentRoutes);
  }

  function focusRoute(routeId) {
    focusedRouteId = focusedRouteId === routeId ? null : routeId;
    renderCurrentPlan();
  }

  function hasRoute(plan, routeId) {
    return (plan.technician_routes || []).some(function (route, idx) {
      return routeKey(route, idx) === routeId;
    });
  }

  function routeKey(route, idx) {
    return String(route.id || route.technician_name || ('route-' + idx));
  }

  function canSolve() {
    return !bootstrapError && !!currentPlan && !demoData.isLoading();
  }

  function reportBootstrapError(err) {
    bootstrapError = err && err.message ? err.message : String(err || 'unknown error');
    layout.bootstrapNotice.textContent = 'Demo data bootstrap failed: ' + bootstrapError;
    layout.bootstrapNotice.style.display = '';
    app.dataset.bootstrapError = 'true';
    updateSolveActionAvailability();
    console.error('Demo data bootstrap failed:', err);
  }

  function clearBootstrapError() {
    bootstrapError = null;
    layout.bootstrapNotice.textContent = '';
    layout.bootstrapNotice.style.display = 'none';
    delete app.dataset.bootstrapError;
  }

  function updateSolveActionAvailability() {
    var solveButton = utils.findHeaderButton(layout.header, 'Solve');
    if (!solveButton) return;
    var disabled = !canSolve();
    solveButton.disabled = disabled;
    solveButton.setAttribute('aria-disabled', disabled ? 'true' : 'false');
    solveButton.title = disabled
      ? (bootstrapError ? 'Bergamo road data could not be loaded.' : 'Loading Bergamo demo data...')
      : '';
  }

  function syncLifecycleMarkers(meta) {
    var jobId = solver.getJobId();
    var snapshotRevision = solver.getSnapshotRevision();
    var lifecycleState = meta && meta.lifecycleState ? meta.lifecycleState : solver.getLifecycleState();

    if (jobId) app.dataset.jobId = String(jobId);
    else delete app.dataset.jobId;
    if (snapshotRevision != null) app.dataset.snapshotRevision = String(snapshotRevision);
    else delete app.dataset.snapshotRevision;
    if (lifecycleState && lifecycleState !== 'IDLE') app.dataset.lifecycleState = lifecycleState;
    else delete app.dataset.lifecycleState;
    updateSolveActionAvailability();
  }
})();
