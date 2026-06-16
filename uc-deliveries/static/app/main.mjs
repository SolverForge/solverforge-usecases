import {
  clonePlan,
  refreshPlan,
  refreshServerPlan,
} from './models.mjs';
import { fetchJson, showError } from './ui/components.mjs';
import { renderData } from './ui/data-tables.mjs';
import { createLayout } from './ui/layout.mjs';
import { sameRouteIdentity, syncLifecycleDataset } from './ui/lifecycle.mjs';
import { buildAnalysisBody, buildRecommendationBody } from './ui/modals.mjs';
import { renderMap, renderRouteList, renderSummary, renderTimelines } from './ui/overview.mjs';

const DEFAULT_DEMO = 'PHILADELPHIA';
const ROUTING_MODE = 'road_network';

export async function boot() {
  const config = await fetchJson('/sf-config.json');
  const backend = SF.createBackend({ baseUrl: '' });
  const app = document.getElementById('sf-app');

  let currentPlan = null;
  let currentRoutes = null;
  let currentDemo = DEFAULT_DEMO;
  let mapCtrl = null;
  let focusedVehicleId = null;
  let routeRequestToken = 0;
  // Route geometry is only trustworthy for one retained job and snapshot
  // revision. This identity prevents stale map lines from surviving dataset
  // switches or newer solver snapshots.
  let activeRouteIdentity = null;
  let activeTab = 'overview';
  let mapLocationSignature = null;

  const statusBar = SF.createStatusBar({
    constraints: [
      'all_deliveries_assigned',
      'vehicle_capacity',
      'delivery_time_windows',
      'total_travel_time',
    ],
  });

  const layout = createLayout({
    app,
    config,
    statusBar,
    actions: {
      onSolve: () => loadAndSolve(),
      onPause: () => solver.pause().catch(showError),
      onResume: () => solver.resume().catch(showError),
      onCancel: () => solver.cancel().catch(showError),
      onAnalyze: () => openAnalysis(),
    },
    onTabChange: (tabId) => setActiveTab(tabId),
  });

  const solver = SF.createSolver({
    backend,
    statusBar,
    onProgress: syncLifecycle,
    onPauseRequested: syncLifecycle,
    onSolution: async (snapshot, meta) => handleSnapshotEvent(snapshot, meta),
    onPaused: async (snapshot, meta) => handleSnapshotEvent(snapshot, meta),
    onResumed: syncLifecycle,
    onCancelled: async (snapshot, meta) => handleSnapshotEvent(snapshot, meta),
    onComplete: async (snapshot, meta) => handleSnapshotEvent(snapshot, meta),
    onFailure: async (_message, meta, snapshot, analysis) => {
      await handleSnapshotEvent(snapshot, meta);
      if (analysis) layout.analysisModal.setBody(buildAnalysisBody(analysis));
    },
    onAnalysis: (analysis) => layout.analysisModal.setBody(buildAnalysisBody(analysis)),
    onError: showError,
  });

  layout.reloadButton.addEventListener('click', async () => loadDemoData(layout.demoField.select.value));
  layout.demoField.select.addEventListener('change', async () => loadDemoData(layout.demoField.select.value));

  setActiveTab(activeTab);
  await loadDemoData(DEFAULT_DEMO);

  function setActiveTab(tabId) {
    activeTab = tabId;
    Object.entries(layout.panels).forEach(([id, panel]) => {
      panel.classList.toggle('is-active', id === tabId);
    });
  }

  async function loadDemoData(demoId) {
    currentDemo = demoId;
    invalidateRoutes();
    if (currentPlan) {
      renderMapView();
      renderRouteListView();
    }
    const plan = await fetchJson(`/demo-data/${demoId}`);
    plan.routingMode = plan.routingMode || ROUTING_MODE;
    renderServerPlan(plan);
  }

  async function loadAndSolve() {
    if (!currentPlan) return;
    invalidateRoutes();
    renderDraftPlan({ ...currentPlan, routingMode: ROUTING_MODE });
    await cleanupTerminalJob();
    await solver.start(clonePlan(currentPlan));
    syncLifecycle();
  }

  async function cleanupTerminalJob() {
    const state = solver.getLifecycleState();
    if (!solver.getJobId() || state === 'IDLE' || state === 'PAUSED' || solver.isRunning()) return;
    try {
      await solver.delete();
    } catch (_error) {
      // Retained terminal cleanup is opportunistic before starting a new job.
    }
  }

  async function handleSnapshotEvent(snapshot, meta) {
    if (snapshot?.solution) {
      renderServerPlan(snapshot.solution, meta);
    }
    await loadRoutes(meta);
    syncLifecycle(meta);
  }

  async function loadRoutes(meta) {
    const requested = routeIdentityFrom(meta);
    if (!requested || !activeRouteIdentity || !sameRouteIdentity(requested, activeRouteIdentity)) return;
    const token = ++routeRequestToken;
    try {
      const routes = await fetchJson(`/jobs/${requested.jobId}/routes?snapshot_revision=${requested.snapshotRevision}`);
      const routingMode = routes.routingMode || requested.routingMode;
      if (!routeResponseStillCurrent(token, requested, routingMode)) return;
      currentRoutes = { ...routes, jobId: requested.jobId, snapshotRevision: requested.snapshotRevision, routingMode };
      renderMapView();
      renderRouteListView();
    } catch (_error) {
      if (token === routeRequestToken && activeRouteIdentity && sameRouteIdentity(requested, activeRouteIdentity)) {
        currentRoutes = null;
        renderMapView();
        renderRouteListView();
      }
    }
  }

  function routeResponseStillCurrent(token, requested, routingMode) {
    return (
      token === routeRequestToken &&
      activeRouteIdentity &&
      sameRouteIdentity(requested, activeRouteIdentity) &&
      currentPlan?.routingMode === requested.routingMode &&
      routingMode === requested.routingMode
    );
  }

  async function openAnalysis() {
    if (!solver.getJobId()) return;
    const analysis = await solver.analyzeSnapshot();
    layout.analysisModal.setBody(buildAnalysisBody(analysis));
    layout.analysisModal.open();
  }

  async function openRecommendations(deliveryId) {
    if (!currentPlan) return;
    const response = await fetch('/recommendations/delivery-insertions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ plan: currentPlan, deliveryId, limit: 8 }),
    });
    if (!response.ok) throw new Error('Failed to load recommendations');
    const payload = await response.json();
    layout.recommendationModal.setBody(buildRecommendationBody(payload, (previewPlan) => {
      renderServerPlan(previewPlan);
      layout.recommendationModal.close();
    }));
    layout.recommendationModal.open();
  }

  function renderDraftPlan(plan) {
    renderPlan(plan, refreshPlan);
  }

  function renderServerPlan(plan, meta = null) {
    invalidateRoutes();
    renderPlan(plan, refreshServerPlan);
    // A solver snapshot is the only source that can authorize `/routes` reads.
    activeRouteIdentity = meta ? routeIdentityFrom(meta) : null;
  }

  function renderPlan(plan, refresh) {
    const nextMapLocationSignature = locationSignature(plan);
    const shouldFitMap = nextMapLocationSignature !== mapLocationSignature;
    currentPlan = refresh({ ...plan, routingMode: plan.routingMode || ROUTING_MODE });
    mapLocationSignature = nextMapLocationSignature;
    renderSummaryView();
    renderRouteListView();
    renderMapView({ fitBounds: shouldFitMap });
    renderTimelinesView();
    renderDataView();
  }

  function invalidateRoutes() {
    routeRequestToken += 1;
    currentRoutes = null;
    focusedVehicleId = null;
    // Route tables and map geometry must be reloaded for the next snapshot.
    activeRouteIdentity = null;
  }

  function routeIdentityFrom(meta) {
    const jobId = meta?.jobId != null ? String(meta.jobId) : solver.getJobId();
    const snapshotRevision =
      meta?.snapshotRevision != null ? meta.snapshotRevision : solver.getSnapshotRevision();
    const routingMode = currentPlan?.routingMode || null;
    if (!jobId || snapshotRevision == null || !routingMode) return null;
    return { jobId: String(jobId), snapshotRevision: String(snapshotRevision), routingMode };
  }

  function renderSummaryView() {
    renderSummary({ currentDemo, currentPlan, summaryCard: layout.summaryCard, summaryMetrics: layout.summaryMetrics });
  }

  function renderRouteListView() {
    renderRouteList({
      currentPlan,
      focusedVehicleId,
      routeList: layout.routeList,
      onFocusVehicle: (vehicleId) => {
        focusedVehicleId = focusedVehicleId === vehicleId ? null : vehicleId;
        renderMapView();
        renderRouteListView();
      },
    });
  }

  function renderMapView({ fitBounds = false } = {}) {
    renderMap({
      currentPlan,
      currentRoutes,
      focusedVehicleId,
      fitBounds,
      mapCtrl,
      setMapCtrl: (next) => {
        mapCtrl = next;
      },
    });
  }

  function renderTimelinesView() {
    renderTimelines({
      currentPlan,
      vehicleTimeline: layout.vehicleTimeline,
      deliveryTimeline: layout.deliveryTimeline,
    });
  }

  function renderDataView() {
    renderData({
      currentPlan,
      dataBody: layout.dataBody,
      onRecommend: async (deliveryId) => {
        try {
          await openRecommendations(deliveryId);
        } catch (error) {
          showError(error);
        }
      },
    });
  }

  function syncLifecycle(meta) {
    syncLifecycleDataset(app, solver, meta);
  }

  function locationSignature(plan) {
    const deliveries = (plan.deliveries || []).map((delivery) => `${delivery.id}:${delivery.lat}:${delivery.lng}`);
    const vehicles = (plan.vehicles || []).map((vehicle) => `${vehicle.id}:${vehicle.homeLat}:${vehicle.homeLng}`);
    return `${deliveries.join('|')}::${vehicles.join('|')}`;
  }
}
