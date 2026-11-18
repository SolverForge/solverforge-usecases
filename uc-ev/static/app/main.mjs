// Boot and lifecycle wiring for SolverForge EV.
//
// The solver lifecycle is the stock `SF.createSolver` state machine, wired the
// same way as the CLI scaffold. Road geometry is fetched per snapshot and
// only drawn when it belongs to the snapshot currently on screen.

import { createLayout } from './layout.mjs';
import { clonePlan, constraintLabel, statusBarConstraints } from './model.mjs';
import {
  buildAnalysisBody,
  renderData,
  renderMap,
  renderRouteList,
  renderSchedule,
  renderSummary,
} from './render.mjs';

export async function boot() {
  const app = document.getElementById('sf-app');
  const config = await requestJson('/sf-config.json', 'app config');
  const statusBar = labelledStatusBar(SF.createStatusBar({ constraints: statusBarConstraints() }));
  const catalog = await requestJson('/demo-data', 'demo data catalog').catch((error) => ({ error }));

  let currentPlan = null;
  let currentRoutes = null;
  let routeToken = 0;
  // Geometry takes the server a moment per snapshot, and early in a solve
  // snapshots arrive faster than that. One request is in flight at a time and
  // only the newest snapshot waits behind it: a request per snapshot queued
  // dozens of reads ahead of Pause and Cancel, which browsers send over the
  // same six connections per host.
  let routesInFlight = false;
  let routesWanted = null;
  let focusedIndex = null;
  let mapCtrl = null;
  let locationKey = null;

  const layout = createLayout({
    app,
    config,
    statusBar,
    actions: {
      onSolve: () => loadAndSolve(),
      onPause: () => solver.pause().then(() => syncMarkers()).catch(reportError),
      onResume: () => solver.resume().then(() => syncMarkers()).catch(reportError),
      onCancel: () => solver.cancel().then(() => syncMarkers()).catch(reportError),
      onAnalyze: () => openAnalysis(),
    },
    onTabChange: (tab) => {
      Object.entries(layout.panels).forEach(([id, panel]) => panel.classList.toggle('is-active', id === tab));
      if (tab === 'overview' && mapCtrl) setTimeout(() => window.dispatchEvent(new Event('resize')), 0);
    },
  });

  const solver = SF.createSolver({
    backend: SF.createBackend({ baseUrl: '' }),
    statusBar,
    onProgress: (meta) => syncMarkers(meta),
    onPauseRequested: (meta) => syncMarkers(meta),
    onSolution: (snapshot, meta) => handleSnapshot(snapshot, meta),
    onPaused: (snapshot, meta) => handleSnapshot(snapshot, meta),
    onResumed: (meta) => syncMarkers(meta),
    onCancelled: (snapshot, meta) => handleSnapshot(snapshot, meta),
    onComplete: (snapshot, meta) => handleSnapshot(snapshot, meta),
    onFailure: (message, meta, snapshot, analysis) => {
      handleSnapshot(snapshot, meta);
      if (analysis) layout.analysisModal.setBody(buildAnalysisBody(analysis));
      reportError(new Error(message));
    },
    onAnalysis: (analysis) => layout.analysisModal.setBody(buildAnalysisBody(analysis)),
    onError: (message) => reportError(new Error(message)),
  });

  if (catalog.error) {
    reportBootstrapError(catalog.error);
  } else {
    await loadDemo(catalog.defaultId);
  }

  async function loadDemo(demoId) {
    if (solver.isRunning()) return;
    try {
      const plan = await requestJson(`/demo-data/${encodeURIComponent(demoId)}`, `demo data "${demoId}"`);
      clearBootstrapError();
      showPlan(plan, null);
    } catch (error) {
      reportBootstrapError(error);
    }
  }

  async function loadAndSolve() {
    if (!currentPlan || solver.isRunning() || solver.getLifecycleState() === 'PAUSED') return;
    await cleanupTerminalJob();
    // The server downloads (first run) and caches road data before solving.
    SF.showToast({ title: 'Preparing road network', message: 'Routing every pharmacy and charger…', variant: 'success', delay: 4000 });
    try {
      await solver.start(clonePlan(currentPlan));
    } catch (error) {
      reportError(error);
    }
    syncMarkers();
  }

  async function cleanupTerminalJob() {
    const state = solver.getLifecycleState();
    if (!solver.getJobId() || state === 'IDLE' || state === 'PAUSED' || solver.isRunning()) return;
    try {
      await solver.delete();
    } catch (_error) {
      // Deleting the previous terminal job is best effort before a new start.
    }
  }

  async function openAnalysis() {
    if (!solver.getJobId()) return;
    try {
      layout.analysisModal.setBody(buildAnalysisBody(await solver.analyzeSnapshot()));
      layout.analysisModal.open();
    } catch (error) {
      reportError(error);
    }
  }

  function handleSnapshot(snapshot, meta) {
    if (snapshot && snapshot.solution) showPlan(snapshot.solution, meta);
    syncMarkers(meta);
  }

  function showPlan(plan, meta) {
    currentPlan = clonePlan(plan);
    currentRoutes = null;
    routeToken += 1;
    renderAll();
    const jobId = meta?.jobId != null ? String(meta.jobId) : solver.getJobId();
    const revision = meta?.snapshotRevision ?? solver.getSnapshotRevision();
    if (meta && jobId && revision != null) requestRoutes(jobId, revision, routeToken);
  }

  function requestRoutes(jobId, revision, token) {
    routesWanted = { jobId, revision, token };
    if (!routesInFlight) loadLatestRoutes();
  }

  async function loadLatestRoutes() {
    routesInFlight = true;
    while (routesWanted) {
      const { jobId, revision, token } = routesWanted;
      routesWanted = null;
      if (token !== routeToken) continue;
      try {
        const response = await requestJson(`/jobs/${jobId}/routes?snapshot_revision=${revision}`, 'route geometry');
        if (token !== routeToken || String(response.snapshotRevision) !== String(revision)) continue;
        currentRoutes = response.routes;
        drawMap(false);
      } catch (_error) {
        // Geometry is decoration over the scored plan; keep markers without lines.
      }
    }
    routesInFlight = false;
  }

  function renderAll() {
    renderSummary(layout.summaryCard, currentPlan);
    renderRouteList(layout.routeList, currentPlan, focusedIndex, focusVehicle);
    renderSchedule(layout.timeline, currentPlan);
    renderData(layout.dataBody, currentPlan);
    const nextKey = (currentPlan.customers || []).map((c) => `${c.lat},${c.lng}`).join('|');
    drawMap(nextKey !== locationKey);
    locationKey = nextKey;
  }

  function drawMap(fitBounds) {
    if (!mapCtrl) {
      const hub = currentPlan?.vehicles?.[0];
      mapCtrl = SF.map.create({ container: 'ev-map', center: [hub?.homeLat ?? 45.67, hub?.homeLng ?? 9.66], zoom: 10 });
    }
    renderMap(mapCtrl, currentPlan, currentRoutes, focusedIndex, fitBounds);
  }

  function focusVehicle(index) {
    focusedIndex = focusedIndex === index ? null : index;
    renderRouteList(layout.routeList, currentPlan, focusedIndex, focusVehicle);
    drawMap(false);
  }

  function syncMarkers(meta) {
    const jobId = solver.getJobId();
    const revision = meta?.snapshotRevision ?? solver.getSnapshotRevision();
    const state = meta?.lifecycleState || solver.getLifecycleState();
    if (jobId) app.dataset.jobId = String(jobId);
    else delete app.dataset.jobId;
    if (revision != null) app.dataset.snapshotRevision = String(revision);
    else delete app.dataset.snapshotRevision;
    if (state && state !== 'IDLE') app.dataset.lifecycleState = state;
    else delete app.dataset.lifecycleState;
  }

  function reportBootstrapError(error) {
    layout.bootstrapNotice.textContent = `Demo data bootstrap failed: ${error?.message || error}`;
    layout.bootstrapNotice.hidden = false;
    app.dataset.bootstrapError = 'true';
    setSolveEnabled(false);
  }

  function clearBootstrapError() {
    layout.bootstrapNotice.hidden = true;
    delete app.dataset.bootstrapError;
    setSolveEnabled(true);
  }

  function setSolveEnabled(enabled) {
    const button = [...layout.header.querySelectorAll('button')].find((b) => b.textContent.trim() === 'Solve');
    if (!button) return;
    button.disabled = !enabled;
    button.setAttribute('aria-disabled', enabled ? 'false' : 'true');
  }
}

// Shows readable constraint labels on the status-bar dots that each analysis
// payload repaints (the stock bar uses the raw `name`).
function labelledStatusBar(statusBar) {
  const paint = statusBar.colorDotsFromAnalysis;
  if (typeof paint === 'function') {
    statusBar.colorDotsFromAnalysis = (constraints) =>
      paint.call(statusBar, (constraints || []).map((c) => ({ ...c, name: constraintLabel(c.name) })));
  }
  return statusBar;
}

function reportError(error) {
  SF.showError('SolverForge EV', error?.message || String(error));
}

async function requestJson(path, label) {
  const response = await fetch(path);
  if (!response.ok) throw new Error(`${label} returned HTTP ${response.status}`);
  return response.json();
}
