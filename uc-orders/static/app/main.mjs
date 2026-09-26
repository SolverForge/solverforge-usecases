// Order Picking workspace.
//
// This reproduces the source application's single-page surface: a dataset
// panel, the isometric warehouse canvas with its trolley legend, a statistics
// sidebar, and the picking-plan cards. The solve lifecycle is delegated to the
// shared SolverForge UI shell (`SF.createSolver`) so retained jobs, SSE, pause,
// resume, cancel, and snapshot analysis stay on the stock contract.

import { buildViewModel, clonePlan, formatLocation } from './orders-model.mjs';
import { createWarehouse } from './warehouse.mjs';

const app = document.getElementById('sf-app');
const [config, uiModel] = await Promise.all([
  fetch('/sf-config.json').then(readJson),
  fetch('/generated/ui-model.json').then(readJson),
]);

const backend = SF.createBackend({ baseUrl: '' });
const statusBar = SF.createStatusBar({
  constraints: uiModel.constraints.map((name) => ({
    name: name.replaceAll('_', ' '),
    type: name === 'required_buckets' ? 'hard' : 'soft',
  })),
});

let currentPlan = null;
let datasetLabel = 'STANDARD';

const header = SF.createHeader({
  logo: '/sf/img/ouroboros.svg',
  title: config.title,
  subtitle: config.subtitle,
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

const datasetSummary = SF.el('span', { className: 'orders-toolbar__summary' }, datasetLabel);
const settings = buildSettingsPanel();
const settingsToggle = SF.createButton({
  text: 'Data',
  icon: 'fa-sliders-h',
  variant: 'default',
  size: 'small',
  onClick: toggleSettings,
});
const toolbar = SF.el('div', { className: 'orders-toolbar' }, settingsToggle, datasetSummary);

const statsGrid = SF.el('div', { className: 'orders-stats' });
const warehouse = createWarehouse();
const content = SF.el(
  'div',
  { className: 'orders-content' },
  statsGrid,
  SF.el('div', { className: 'orders-canvas' }, warehouse.el),
);

const cardsGrid = SF.el('div', { className: 'orders-cards' });
const planSection = SF.el(
  'section',
  { className: 'orders-plan' },
  SF.el('h2', null, SF.el('i', { className: 'fa-solid fa-tasks' }), 'Picking plan'),
  cardsGrid,
);

const main = SF.el('div', { className: 'orders-main' }, toolbar, settings.panel, content, planSection);
app.appendChild(main);
app.appendChild(
  SF.createFooter({
    version: '0.1.0',
    links: [{ label: 'SolverForge', url: 'https://www.solverforge.org' }],
  }),
);

const analysisModal = SF.createModal({ title: 'Score analysis', width: '720px' });

const solver = SF.createSolver({
  backend,
  statusBar,
  onProgress: (meta) => {
    syncMarkers(meta);
    applySolving(meta?.lifecycleState);
  },
  onPauseRequested: syncMarkers,
  onResumed: (meta) => {
    syncMarkers(meta);
    applySolving('SOLVING');
  },
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

await bootstrap();

async function bootstrap() {
  try {
    const catalog = await fetch('/demo-data').then(readJson);
    datasetLabel = catalog.defaultId;
    datasetSummary.textContent = datasetLabel;
    currentPlan = await backend.getDemoData(catalog.defaultId);
    render(currentPlan);
  } catch (error) {
    app.dataset.bootstrapError = 'true';
    main.prepend(SF.el('p', { className: 'orders-error' }, `Demo data failed to load: ${error.message}`));
  }
}

function readJson(response) {
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  return response.json();
}

async function startSolve() {
  if (!currentPlan || solver.isRunning() || solver.getLifecycleState() === 'PAUSED') return;
  if (solver.getJobId() && solver.getLifecycleState() !== 'IDLE') await solver.delete();
  applySolving('SOLVING');
  await solver.start(clonePlan(currentPlan));
  syncMarkers();
  syncGenerateState();
}

function acceptSnapshot(snapshot, meta) {
  if (snapshot?.solution) {
    currentPlan = snapshot.solution;
    render(currentPlan);
  }
  syncMarkers(meta);
  applySolving(meta?.lifecycleState || solver.getLifecycleState());
  syncGenerateState();
}

function syncMarkers(meta) {
  const jobId = solver.getJobId();
  const revision = solver.getSnapshotRevision();
  const state = meta?.lifecycleState || solver.getLifecycleState();
  if (jobId) app.dataset.jobId = String(jobId);
  else delete app.dataset.jobId;
  if (revision != null) app.dataset.snapshotRevision = String(revision);
  else delete app.dataset.snapshotRevision;
  if (state && state !== 'IDLE') app.dataset.lifecycleState = state;
  else delete app.dataset.lifecycleState;
}

function applySolving(state) {
  warehouse.setSolving(state === 'SOLVING' || state === 'STARTING' || state === 'PAUSE_REQUESTED');
}

function syncGenerateState() {
  const busy = solver.isRunning();
  settings.generate.disabled = busy;
}

async function openAnalysis() {
  if (!solver.getJobId()) return;
  const payload = await solver.analyzeSnapshot();
  const analysis = payload.analysis || payload;
  const rows = (analysis.constraints || []).map((row) =>
    SF.el(
      'div',
      { className: 'orders-constraint' },
      SF.el('span', { className: 'orders-constraint__name' }, row.name),
      SF.el(
        'span',
        { className: `orders-constraint__score is-${row.constraintType || row.type || 'hard'}` },
        row.score,
      ),
    ),
  );
  analysisModal.setBody(SF.el('div', { className: 'orders-analysis' }, ...rows));
  analysisModal.open();
}

function render(plan) {
  const model = buildViewModel(plan);
  renderStats(model);
  warehouse.render(model);
  renderCards(model);
}

function renderStats(model) {
  statsGrid.textContent = '';
  statsGrid.append(
    statCard('fa-clipboard-list', model.orders, 'Orders'),
    statCard('fa-cube', model.items, 'Items'),
    statCard('fa-dolly', model.activeTrolleys, 'Active'),
    statCard('fa-route', model.totalMeters, 'meters'),
  );
}

function statCard(iconName, value, label) {
  return SF.el(
    'div',
    { className: 'orders-stat' },
    SF.el('div', { className: 'orders-stat__icon' }, SF.el('i', { className: `fa-solid ${iconName}` })),
    SF.el(
      'div',
      { className: 'orders-stat__info' },
      SF.el('span', { className: 'orders-stat__value' }, Number(value).toLocaleString()),
      SF.el('span', { className: 'orders-stat__label' }, label),
    ),
  );
}

function renderCards(model) {
  cardsGrid.textContent = '';
  for (const trolley of model.trolleys) {
    const body = trolley.itemCount
      ? SF.el(
          'div',
          { className: 'orders-card__items' },
          ...trolley.steps.map((step, index) =>
            SF.el(
              'div',
              { className: 'orders-card__item', title: `${step.name} · ${formatLocation(step.location)}` },
              SF.el('span', { className: 'orders-card__item-number' }, String(index + 1)),
              step.name.length > 15 ? `${step.name.slice(0, 15)}…` : step.name,
            ),
          ),
        )
      : SF.el('div', { className: 'orders-card__empty' }, 'No items assigned');

    cardsGrid.appendChild(
      SF.el(
        'article',
        { className: 'orders-card', dataset: { trolleyId: trolley.id } },
        SF.el(
          'div',
          { className: 'orders-card__header' },
          SF.el('span', { className: 'orders-card__badge', style: { background: trolley.color } }),
          SF.el(
            'div',
            { className: 'orders-card__info' },
            SF.el('div', { className: 'orders-card__title' }, `Trolley ${trolley.id}`),
            SF.el('div', { className: 'orders-card__stats' }, `${trolley.itemCount} items`),
          ),
          SF.el(
            'div',
            { className: 'orders-card__capacity', title: `${trolley.capacityPercent}% of bucket volume` },
            SF.el('div', {
              className: `orders-card__capacity-fill is-${trolley.capacityClass}`,
              style: { width: `${trolley.capacityPercent}%` },
            }),
          ),
        ),
        body,
      ),
    );
  }
}

function buildSettingsPanel() {
  const orders = slider({ label: 'Orders', icon: 'fa-shopping-cart', min: 10, max: 100, value: 40 });
  const trolleys = slider({ label: 'Trolleys', icon: 'fa-dolly', min: 3, max: 15, value: 8 });
  const buckets = slider({ label: 'Buckets', icon: 'fa-box', min: 2, max: 10, value: 6 });

  const generate = SF.createButton({
    text: 'Generate New',
    icon: 'fa-sync-alt',
    variant: 'primary',
    onClick: () => generateNewData(orders.input, trolleys.input, buckets.input, generate),
  });

  const panel = SF.el(
    'div',
    { className: 'orders-settings', hidden: 'hidden' },
    SF.el('div', { className: 'orders-settings__grid' },
      orders.row,
      trolleys.row,
      buckets.row,
      SF.el('div', { className: 'orders-settings__action' }, generate)),
  );

  return { panel, generate };
}

function slider({ label, icon, min, max, value }) {
  const badge = SF.el('span', { className: 'orders-setting__badge' }, String(value));
  const input = SF.el('input', {
    className: 'orders-setting__range',
    type: 'range',
    min: String(min),
    max: String(max),
    value: String(value),
    onInput: (event) => {
      badge.textContent = event.target.value;
    },
  });
  const row = SF.el(
    'div',
    { className: 'orders-setting' },
    SF.el('label', null, SF.el('i', { className: `fa-solid ${icon}` }), label, badge),
    input,
  );
  return { row, input };
}

function toggleSettings() {
  settings.panel.toggleAttribute('hidden');
}

async function generateNewData(ordersInput, trolleysInput, bucketsInput, button) {
  const body = {
    ordersCount: Number(ordersInput.value),
    trolleysCount: Number(trolleysInput.value),
    bucketCount: Number(bucketsInput.value),
  };

  button.disabled = true;
  button.classList.add('is-loading');
  try {
    const response = await fetch('/demo-data/generate', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const plan = await response.json();
    datasetLabel = `${body.ordersCount} orders · ${body.trolleysCount} trolleys · ${body.bucketCount} buckets`;
    datasetSummary.textContent = datasetLabel;
    render(plan);
    settings.panel.setAttribute('hidden', 'hidden');
    SF.showToast({ variant: 'success', message: 'Generated new picking data.' });
  } catch (error) {
    SF.showError('Could not generate data', error.message);
  } finally {
    button.disabled = false;
    button.classList.remove('is-loading');
  }
}
