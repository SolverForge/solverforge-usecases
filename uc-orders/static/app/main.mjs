import { buildPlanModel, buildRouteTimeline, clonePlan, formatLocation } from './orders-model.mjs';

const app = document.getElementById('sf-app');
const [config, uiModel] = await Promise.all([
  fetch('/sf-config.json').then(readJson),
  fetch('/generated/ui-model.json').then(readJson),
]);
const backend = SF.createBackend({ baseUrl: '' });
const statusBar = SF.createStatusBar({
  constraints: uiModel.constraints.map((name) => ({ name: name.replaceAll('_', ' '), type: name === 'required_buckets' ? 'hard' : 'soft' })),
});
let currentPlan;
let timeline;

const header = SF.createHeader({
  logo: '/sf/img/ouroboros.svg',
  title: config.title,
  subtitle: config.subtitle,
  tabs: [
    { id: 'overview', label: 'Overview', icon: 'fa-gauge-high', active: true },
    { id: 'routes', label: 'Routes / Picking Plan', icon: 'fa-route' },
    { id: 'data', label: 'Warehouse / Data', icon: 'fa-warehouse' },
    { id: 'api', label: 'REST API', icon: 'fa-code' },
  ],
  onTabChange: showPanel,
  actions: {
    onSolve: startSolve,
    onPause: () => solver.pause(),
    onResume: () => solver.resume(),
    onCancel: () => solver.cancel(),
    onAnalyze: openAnalysis,
  },
});
app.appendChild(header);
statusBar.bindHeader(header);
app.appendChild(statusBar.el);

const panels = {};
['overview', 'routes', 'data', 'api'].forEach((id) => {
  panels[id] = SF.el('main', { className: `sf-content orders-panel${id === 'overview' ? ' is-active' : ''}`, id: `panel-${id}` });
  app.appendChild(panels[id]);
});
panels.api.appendChild(SF.createApiGuide({ endpoints: [
  { method: 'GET', path: '/demo-data', description: 'Discover deterministic datasets' },
  { method: 'GET', path: '/demo-data/STANDARD', description: 'Load 145 order-item pick steps' },
  { method: 'POST', path: '/jobs', description: 'Start a retained solve' },
  { method: 'GET', path: '/jobs/{id}/status', description: 'Read lifecycle and telemetry' },
  { method: 'GET', path: '/jobs/{id}/snapshot', description: 'Read the latest picking plan' },
  { method: 'GET', path: '/jobs/{id}/analysis?snapshot_revision={n}', description: 'Analyze an exact scored snapshot' },
  { method: 'POST', path: '/jobs/{id}/pause', description: 'Pause search' },
  { method: 'POST', path: '/jobs/{id}/resume', description: 'Resume search' },
  { method: 'POST', path: '/jobs/{id}/cancel', description: 'Stop and retain the best plan' },
  { method: 'GET', path: '/jobs/{id}/events', description: 'Observe lifecycle events over SSE' },
] }));
app.appendChild(SF.createFooter({ version: '0.1.0', links: [{ label: 'SolverForge', url: 'https://www.solverforge.org' }] }));

const analysisModal = SF.createModal({ title: 'Picking score analysis', width: '760px' });
const solver = SF.createSolver({
  backend,
  statusBar,
  onProgress: syncMarkers,
  onPauseRequested: syncMarkers,
  onResumed: syncMarkers,
  onSolution: acceptSnapshot,
  onPaused: acceptSnapshot,
  onCancelled: acceptSnapshot,
  onComplete: acceptSnapshot,
  onFailure(message, meta, snapshot) {
    acceptSnapshot(snapshot, meta);
    SF.showError('Solve failed', message);
  },
  onAnalysis: syncMarkers,
  onError: (message) => SF.showError('Solver lifecycle failed', message),
});

try {
  const catalog = await fetch('/demo-data').then(readJson);
  currentPlan = await backend.getDemoData(catalog.defaultId);
  render(currentPlan);
} catch (error) {
  app.dataset.bootstrapError = 'true';
  panels.overview.appendChild(SF.el('p', { className: 'orders-error' }, `Demo data failed to load: ${error.message}`));
}

function readJson(response) {
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

function showPanel(id) {
  Object.entries(panels).forEach(([key, panel]) => panel.classList.toggle('is-active', key === id));
}

async function startSolve() {
  if (!currentPlan || solver.isRunning() || solver.getLifecycleState() === 'PAUSED') return;
  if (solver.getJobId() && solver.getLifecycleState() !== 'IDLE') await solver.delete();
  await solver.start(clonePlan(currentPlan));
  syncMarkers();
}

function acceptSnapshot(snapshot, meta) {
  if (snapshot?.solution) {
    currentPlan = snapshot.solution;
    render(currentPlan);
  }
  syncMarkers(meta);
}

function syncMarkers(meta) {
  const jobId = solver.getJobId();
  const revision = solver.getSnapshotRevision();
  const state = meta?.lifecycleState || solver.getLifecycleState();
  if (jobId) app.dataset.jobId = String(jobId); else delete app.dataset.jobId;
  if (revision != null) app.dataset.snapshotRevision = String(revision); else delete app.dataset.snapshotRevision;
  if (state && state !== 'IDLE') app.dataset.lifecycleState = state; else delete app.dataset.lifecycleState;
}

async function openAnalysis() {
  if (!solver.getJobId()) return;
  const payload = await solver.analyzeSnapshot();
  const analysis = payload.analysis || payload;
  analysisModal.setBody(SF.createTable({
    columns: ['Constraint', 'Type', 'Weight', 'Matches', 'Score'],
    rows: (analysis.constraints || []).map((row) => [row.name, row.constraintType || row.type || '', row.weight, row.matchCount, row.score]),
  }));
  analysisModal.open();
}

function render(plan) {
  const model = buildPlanModel(plan);
  renderOverview(model);
  renderRoutes(model);
  renderData(model);
}

function renderOverview(model) {
  panels.overview.innerHTML = '';
  panels.overview.appendChild(SF.el('section', { className: 'orders-hero' },
    SF.el('div', null,
      SF.el('p', { className: 'orders-eyebrow' }, 'Closed-route warehouse optimization'),
      SF.el('h1', null, 'Every order item. One trolley. The shortest feasible walk.'),
      SF.el('p', null, 'SolverForge assigns 145 immutable pick steps while respecting four order-dedicated buckets per trolley.'),
    ),
    SF.el('div', { className: 'orders-depot' }, SF.el('span', null, 'DEPOT'), SF.el('strong', null, '(A,1) LEFT · row 0')),
  ));
  panels.overview.appendChild(SF.el('section', { className: 'orders-kpis' },
    kpi('Orders', model.kpis.orders), kpi('Pick items', model.kpis.items),
    kpi('Active trolleys', model.kpis.activeTrolleys), kpi('Closed-route meters', model.kpis.routeMeters),
  ));
  const cards = SF.el('section', { className: 'orders-route-grid', 'aria-label': 'Trolley route cards' });
  model.routes.forEach((route) => cards.appendChild(routeCard(route)));
  panels.overview.appendChild(cards);
  if (model.unassigned.length) panels.overview.appendChild(SF.el('p', { className: 'orders-unassigned' }, `${model.unassigned.length} pick steps are unassigned. Start Solve to construct complete routes.`));
}

function renderRoutes(model) {
  const config = buildRouteTimeline(model);
  if (!timeline) timeline = SF.rail.createTimeline(config); else timeline.setModel(config.model);
  panels.routes.innerHTML = '';
  panels.routes.appendChild(SF.createTable({ columns: ['Items', 'Assigned', 'Unassigned', 'Distance unit'], rows: [[model.kpis.items, model.kpis.items - model.unassigned.length, model.unassigned.length, 'meters']] }));
  panels.routes.appendChild(timeline.el);
}

function renderData(model) {
  panels.data.innerHTML = '';
  panels.data.appendChild(sectionTable('Orders', ['Order', 'Items', 'Volume cm³', 'Trolleys'], model.orders.map((row) => [row.id, row.itemCount, row.volumeCm3, row.trolleyCount])));
  panels.data.appendChild(sectionTable('Products', ['Product', 'Name', 'Volume cm³', 'Location', 'Picks'], model.products.map((row) => [row.id, row.name, row.volumeCm3, row.location, row.picks])));
  panels.data.appendChild(sectionTable('Trolleys', ['Trolley', 'Buckets', 'Capacity / bucket', 'Depot', 'Stops'], model.routes.map((route) => [route.id, route.bucketCount, route.bucketCapacity, formatLocation(route.location), route.routeSteps.length])));
}

function kpi(label, value) {
  return SF.el('div', { className: 'orders-kpi' }, SF.el('span', null, label), SF.el('strong', null, Number(value).toLocaleString()));
}

function routeCard(route) {
  const status = route.excessBuckets ? `${route.excessBuckets} excess` : 'capacity ready';
  return SF.el('article', { className: `orders-route-card${route.excessBuckets ? ' is-over' : ''}` },
    SF.el('header', null, SF.el('h2', null, `Trolley ${route.id}`), SF.el('span', null, status)),
    SF.el('div', { className: 'orders-route-card__metrics' },
      SF.el('strong', null, `${route.routeSteps.length} stops`), SF.el('strong', null, `${route.distanceMeters} m`),
      SF.el('strong', null, `${route.requiredBuckets}/${route.bucketCount} buckets`)),
    SF.el('p', null, route.routeSteps.length ? `${route.orderCount} orders on this closed route` : 'Awaiting construction'));
}

function sectionTable(title, columns, rows) {
  return SF.el('section', { className: 'orders-data-section' }, SF.el('h2', null, title), SF.createTable({ columns, rows }));
}
