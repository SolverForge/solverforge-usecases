/* layout.mjs — Shell and panel layout for solverforge-lessons */

import { title } from '../models/index.mjs';

export function createLayout({ app, config, statusBar, actions, onTabChange, SF }) {
  var activeTab = 'by-group';
  var viewPanels = {};
  var panels = {};

  var tabs = [];
  tabs.push({ id: 'by-group', label: 'By Group', icon: 'fa-users', active: true });
  tabs.push({ id: 'by-room', label: 'By Room', icon: 'fa-door-open' });
  tabs.push({ id: 'by-teacher', label: 'By Teacher', icon: 'fa-chalkboard-user' });
  tabs.push({ id: 'data', label: 'Data', icon: 'fa-table' });
  tabs.push({ id: 'api', label: 'REST API', icon: 'fa-book' });

  var header = SF.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: config.title,
    subtitle: config.subtitle,
    tabs: tabs,
    actions: actions,
    onTabChange: function (tab) {
      activeTab = tab;
      onTabChange(tab);
    },
  });

  app.className = 'sf-app solverforge-lessons-app';
  app.appendChild(header);
  statusBar.bindHeader(header);
  app.appendChild(statusBar.el);

  // Bootstrap notice
  var bootstrapNotice = SF.el('div', {
    className: 'sf-content',
    style: {
      display: 'none',
      padding: '16px',
      marginBottom: '16px',
      borderRadius: '12px',
      border: '1px solid #dc2626',
      background: '#fef2f2',
      color: '#991b1b',
    },
  });
  app.appendChild(bootstrapNotice);

  // Overview panel
  var overviewPanel = SF.el('div', { className: 'sf-content', style: { display: activeTab === 'overview' ? '' : 'none' } });
  var overviewContainer = SF.el('div', { id: 'sf-overview' });
  overviewPanel.appendChild(overviewContainer);
  app.appendChild(overviewPanel);
  panels.overview = overviewPanel;

  // Data panel
  var dataPanel = SF.el('div', { className: 'sf-content', style: { display: activeTab === 'data' ? '' : 'none' } });
  var tablesContainer = SF.el('div', { id: 'sf-tables' });
  dataPanel.appendChild(tablesContainer);
  app.appendChild(dataPanel);
  panels.data = dataPanel;

  // API panel
  var apiPanel = SF.el('div', { className: 'sf-content', style: { display: activeTab === 'api' ? '' : 'none' } });
  var apiGuideContainer = SF.el('div');
  apiPanel.appendChild(apiGuideContainer);
  app.appendChild(apiPanel);
  panels.api = apiPanel;

  // Custom view panels
  var byGroupPanel = SF.el('div', { className: 'sf-content', style: { display: activeTab === 'by-group' ? '' : 'none' } });
  var byGroupContainer = SF.el('div', { id: 'sf-by-group' });
  byGroupPanel.appendChild(byGroupContainer);
  app.appendChild(byGroupPanel);
  viewPanels['by-group'] = byGroupPanel;
  panels.byGroup = byGroupPanel;

  var byRoomPanel = SF.el('div', { className: 'sf-content', style: { display: activeTab === 'by-room' ? '' : 'none' } });
  var byRoomContainer = SF.el('div', { id: 'sf-by-room' });
  byRoomPanel.appendChild(byRoomContainer);
  app.appendChild(byRoomPanel);
  viewPanels['by-room'] = byRoomPanel;
  panels.byRoom = byRoomPanel;

  var byTeacherPanel = SF.el('div', { className: 'sf-content', style: { display: activeTab === 'by-teacher' ? '' : 'none' } });
  var byTeacherContainer = SF.el('div', { id: 'sf-by-teacher' });
  byTeacherPanel.appendChild(byTeacherContainer);
  app.appendChild(byTeacherPanel);
  viewPanels['by-teacher'] = byTeacherPanel;
  panels.byTeacher = byTeacherPanel;

  app.appendChild(SF.createFooter({
    links: [
      { label: 'SolverForge', url: 'https://www.solverforge.org' },
      { label: 'Docs', url: 'https://www.solverforge.org/docs' },
    ],
  }));

  var analysisModal = SF.createModal({ title: 'Score Analysis', width: '700px' });

  return {
    app,
    SF,
    header,
    statusBar,
    bootstrapNotice,
    panels,
    viewPanels,
    byGroupContainer,
    byRoomContainer,
    byTeacherContainer,
    overviewContainer,
    tablesContainer,
    apiGuideContainer,
    analysisModal,
    activeTab,
  };
}

export function setActiveTab(layout, tabId) {
  layout.activeTab = tabId;
  Object.keys(layout.panels).forEach(function (key) {
    if (layout.panels[key]) {
      layout.panels[key].style.display = key === tabId ? '' : 'none';
    }
  });
  Object.keys(layout.viewPanels).forEach(function (key) {
    if (layout.viewPanels[key]) {
      layout.viewPanels[key].style.display = key === tabId ? '' : 'none';
    }
  });
}
