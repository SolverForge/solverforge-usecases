/* layout.mjs - application shell construction for the Bergamo FSR demo */

export function createAppLayout(options) {
  var SF = window.SF;
  var app = options.app;

  var panels = {
    map: SF.el('div', { className: 'sf-content' }),
    routes: SF.el('div', { className: 'sf-content', style: { display: 'none' } }),
    data: SF.el('div', { className: 'sf-content', style: { display: 'none' } }),
    api: SF.el('div', { className: 'sf-content', style: { display: 'none' } }),
  };

  var header = SF.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: options.config.title,
    subtitle: options.config.subtitle,
    tabs: [
      { id: 'map', label: 'Map', icon: 'fa-map-location-dot', active: true },
      { id: 'routes', label: 'Routes', icon: 'fa-list-ol' },
      { id: 'data', label: 'Data', icon: 'fa-table' },
      { id: 'api', label: 'REST API', icon: 'fa-book' },
    ],
    actions: {
      onSolve: options.onSolve,
      onPause: options.onPause,
      onResume: options.onResume,
      onCancel: options.onCancel,
      onAnalyze: options.onAnalyze,
    },
    onTabChange: function (tab) {
      Object.keys(panels).forEach(function (key) {
        panels[key].style.display = key === tab ? '' : 'none';
      });
      if (tab === 'map' && options.onMapTabShown) options.onMapTabShown();
    },
  });

  app.appendChild(header);
  options.statusBar.bindHeader(header);
  app.appendChild(options.statusBar.el);

  var bootstrapNotice = SF.el('div', {
    className: 'sf-content',
    style: {
      display: 'none',
      padding: '12px 16px',
      marginBottom: '12px',
      border: '1px solid #dc2626',
      borderRadius: '8px',
      background: '#fef2f2',
      color: '#991b1b',
    },
  });
  app.appendChild(bootstrapNotice);

  var summaryContainer = SF.el('div', { className: 'fsr-summary' });
  var mapShell = SF.el('div', { className: 'fsr-map-shell' });
  var routeListCard = SF.el('section', { className: 'sf-section fsr-routes-card' });
  var routeCards = SF.el('div', { className: 'fsr-route-list' });
  var mapCard = SF.el('section', { className: 'sf-section fsr-map-card' });
  var mapContainer = SF.el('div', { id: 'fsr-map', className: 'sf-map-container fsr-map' });

  routeListCard.appendChild(SF.el('h3', null, 'Routes'));
  routeListCard.appendChild(routeCards);
  mapCard.appendChild(SF.el('h3', null, 'Map'));
  mapCard.appendChild(mapContainer);
  mapShell.appendChild(routeListCard);
  mapShell.appendChild(mapCard);
  panels.map.appendChild(summaryContainer);
  panels.map.appendChild(mapShell);

  var timelineContainer = SF.el('div');
  var tablesContainer = SF.el('div');
  var apiGuideContainer = SF.el('div');
  panels.routes.appendChild(timelineContainer);
  panels.data.appendChild(tablesContainer);
  panels.api.appendChild(apiGuideContainer);

  app.appendChild(panels.map);
  app.appendChild(panels.routes);
  app.appendChild(panels.data);
  app.appendChild(panels.api);
  app.appendChild(SF.createFooter({
    links: [
      { label: 'SolverForge', url: 'https://www.solverforge.org' },
      { label: 'Docs', url: 'https://www.solverforge.org/docs' },
    ],
  }));

  return {
    apiGuideContainer: apiGuideContainer,
    bootstrapNotice: bootstrapNotice,
    header: header,
    routeCards: routeCards,
    summaryContainer: summaryContainer,
    tablesContainer: tablesContainer,
    timelineContainer: timelineContainer,
  };
}
