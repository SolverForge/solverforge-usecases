import { buildApiGuideEndpoints } from './api-guide.mjs';

// Builds the header tabs from the generated views plus the local utility panels.
function buildTabs(uiModel) {
  const tabs = uiModel.views.map((view, index) => ({
    id: view.id,
    label: view.label,
    icon: 'fa-table-cells-large',
    active: index === 0,
  }));
  tabs.push({ id: 'data', label: 'Data', icon: 'fa-table' });
  tabs.push({ id: 'api', label: 'REST API', icon: 'fa-book' });
  return tabs;
}

// Creates the shared SolverForge UI shell and the hospital-specific panels.
export function createAppShell({
  root = globalThis,
  sf,
  appElement,
  config,
  uiModel,
  demoId,
  statusBar,
  activeTab,
  actions,
}) {
  const viewRoots = {};
  const header = sf.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: config.title,
    subtitle: config.subtitle,
    tabs: buildTabs(uiModel),
    actions,
    onTabChange(tabId) {
      shell.setActiveTab(tabId);
      if (typeof actions.onTabChange === 'function') actions.onTabChange(tabId);
    },
  });
  appElement.appendChild(header);
  statusBar.bindHeader(header);
  appElement.appendChild(statusBar.el);

  uiModel.views.forEach((view) => {
    const panel = sf.el('div', { className: 'sf-content', style: { display: 'none' } });
    const rootEl = sf.el('div', { id: `view-${view.id}` });
    panel.appendChild(rootEl);
    viewRoots[view.id] = rootEl;
    appElement.appendChild(panel);
    viewRoots[view.id].panel = panel;
  });

  const dataPanel = sf.el('div', { className: 'sf-content', style: { display: 'none' } });
  const dataRoot = sf.el('div', { id: 'sf-tables' });
  dataPanel.appendChild(dataRoot);
  appElement.appendChild(dataPanel);

  const apiPanel = sf.el('div', { className: 'sf-content', style: { display: 'none' } });
  apiPanel.appendChild(sf.createApiGuide({ endpoints: buildApiGuideEndpoints(demoId, root) }));
  appElement.appendChild(apiPanel);

  appElement.appendChild(sf.createFooter({
    links: [
      { label: 'SolverForge', url: 'https://www.solverforge.org' },
      { label: 'Docs', url: 'https://www.solverforge.org/docs' },
    ],
  }));

  const analysisModal = sf.createModal({ title: 'Score Analysis', width: '700px' });

  const shell = {
    header,
    viewRoots,
    dataRoot,
    dataPanel,
    apiPanel,
    analysisModal,
    setActiveTab(tabId) {
      Object.entries(viewRoots).forEach(([viewId, rootEl]) => {
        rootEl.panel.style.display = viewId === tabId ? '' : 'none';
      });
      dataPanel.style.display = tabId === 'data' ? '' : 'none';
      apiPanel.style.display = tabId === 'api' ? '' : 'none';
    },
    syncLifecycleMarkers({ jobId, snapshotRevision, lifecycleState }) {
      if (jobId) appElement.dataset.jobId = String(jobId);
      else delete appElement.dataset.jobId;

      if (snapshotRevision != null) appElement.dataset.snapshotRevision = String(snapshotRevision);
      else delete appElement.dataset.snapshotRevision;

      if (lifecycleState && lifecycleState !== 'IDLE') appElement.dataset.lifecycleState = lifecycleState;
      else delete appElement.dataset.lifecycleState;
    },
    openAnalysis(body) {
      analysisModal.setBody(body);
      analysisModal.open();
    },
  };

  shell.setActiveTab(activeTab);
  return shell;
}
