// Page skeleton: header, status bar, controls, and one panel per tab.
//
// Built only from stock solverforge-ui factories; app CSS adds the grid.

import { buildApiGuideEndpoints } from './api-guide.mjs';
import { buildVehicleTimelineModel } from './model.mjs';

export function createLayout({ app, config, statusBar, actions, onTabChange }) {
  const header = SF.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: config.title,
    subtitle: config.subtitle,
    tabs: [
      { id: 'overview', label: 'Overview', icon: 'fa-map', active: true },
      { id: 'schedule', label: 'Schedule', icon: 'fa-truck' },
      { id: 'data', label: 'Data', icon: 'fa-table' },
      { id: 'api', label: 'REST API', icon: 'fa-book' },
    ],
    actions,
    onTabChange,
  });
  app.appendChild(header);
  statusBar.bindHeader(header);
  app.appendChild(statusBar.el);

  const bootstrapNotice = SF.el('div', { className: 'sf-content ev-bootstrap-error' });
  bootstrapNotice.hidden = true;
  app.appendChild(bootstrapNotice);

  const shell = SF.el('div', { className: 'sf-content ev-shell' });
  app.appendChild(shell);

  const panels = {};
  const mount = (id) => {
    const panel = SF.el('section', { className: `ev-panel${id === 'overview' ? ' is-active' : ''}` });
    panels[id] = panel;
    shell.appendChild(panel);
    return panel;
  };

  const overview = mount('overview');
  const grid = SF.el('div', { className: 'ev-grid' });
  const stack = SF.el('div', { className: 'ev-stack' });
  const summaryCard = SF.el('section', { className: 'ev-card' });
  const routeCard = SF.el('section', { className: 'ev-card' });
  const routeList = SF.el('div', { className: 'ev-route-list' });
  routeCard.appendChild(SF.el('h3', null, 'Routes'));
  routeCard.appendChild(routeList);
  stack.appendChild(summaryCard);
  stack.appendChild(routeCard);
  const mapCard = SF.el('section', { className: 'ev-card ev-map-card' });
  mapCard.appendChild(SF.el('h3', null, 'Map'));
  mapCard.appendChild(SF.el('div', { className: 'ev-map', id: 'ev-map' }));
  mapCard.appendChild(
    SF.el(
      'p',
      { className: 'ev-legend' },
      SF.el('i', { className: 'fa-solid fa-prescription-bottle-medical' }),
      ' pharmacy  ',
      SF.el('i', { className: 'fa-solid fa-charging-station' }),
      ' charger (coloured when a route stops there)  · dashed lines drive to or from a charger',
    ),
  );
  grid.appendChild(stack);
  grid.appendChild(mapCard);
  overview.appendChild(grid);

  const schedule = mount('schedule');
  const scheduleCard = SF.el('section', { className: 'ev-card' });
  scheduleCard.appendChild(SF.el('h3', null, 'Schedule by van'));
  const timeline = SF.rail.createTimeline({
    label: 'Van',
    labelWidth: 220,
    model: buildVehicleTimelineModel(null),
  });
  scheduleCard.appendChild(timeline.el);
  schedule.appendChild(scheduleCard);

  const dataBody = SF.el('div', { className: 'ev-data' });
  const dataCard = SF.el('section', { className: 'ev-card' });
  dataCard.appendChild(SF.el('h3', null, 'Plan data'));
  dataCard.appendChild(dataBody);
  mount('data').appendChild(dataCard);

  mount('api').appendChild(SF.createApiGuide({ endpoints: buildApiGuideEndpoints() }));

  app.appendChild(
    SF.createFooter({
      links: [
        { label: 'SolverForge', url: 'https://www.solverforge.org' },
        { label: 'Docs', url: 'https://www.solverforge.org/docs' },
        { label: 'Map data © OpenStreetMap contributors', url: 'https://www.openstreetmap.org/copyright' },
      ],
    }),
  );

  return {
    header,
    panels,
    bootstrapNotice,
    summaryCard,
    routeList,
    timeline,
    dataBody,
    analysisModal: SF.createModal({ title: 'Score Analysis', width: '720px' }),
  };
}
