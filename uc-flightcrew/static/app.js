/* SolverForge Flight Crew browser shell. */
(async function () {
  'use strict';

  var app = document.getElementById('sf-app');
  var config = await fetch('/sf-config.json').then(json);
  var uiModel = await fetch('/generated/ui-model.json').then(json);
  var backend = SF.createBackend({ baseUrl: '' });
  var statusBar = SF.createStatusBar({
    constraints: uiModel.constraints.map(function (name) {
      return { name: name.replaceAll('_', ' '), type: constraintType(name) };
    }),
  });
  var currentPlan = null;
  var timelines = {};

  var header = SF.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: config.title,
    subtitle: config.subtitle,
    tabs: [
      { id: 'crew', label: 'By crew', icon: 'fa-users', active: true },
      { id: 'flights', label: 'By flight', icon: 'fa-plane' },
      { id: 'data', label: 'Data', icon: 'fa-table' },
      { id: 'api', label: 'REST API', icon: 'fa-book' },
    ],
    onTabChange: showTab,
    actions: {
      onSolve: startSolve,
      onPause: function () { solver.pause(); },
      onResume: function () { solver.resume(); },
      onCancel: function () { solver.cancel(); },
      onAnalyze: openAnalysis,
    },
  });
  app.appendChild(header);
  statusBar.bindHeader(header);
  app.appendChild(statusBar.el);

  var panels = {};
  ['crew', 'flights', 'data', 'api'].forEach(function (id) {
    panels[id] = SF.el('main', { className: 'sf-content flightcrew-panel', id: 'panel-' + id });
    if (id !== 'crew') panels[id].style.display = 'none';
    app.appendChild(panels[id]);
  });
  panels.api.appendChild(SF.createApiGuide({ endpoints: [
    { method: 'GET', path: '/demo-data', description: 'Discover demo schedules' },
    { method: 'GET', path: '/demo-data/STANDARD', description: 'Load the flight network' },
    { method: 'POST', path: '/jobs', description: 'Start a retained solve' },
    { method: 'GET', path: '/jobs/{id}/snapshot', description: 'Read the latest crew plan' },
    { method: 'GET', path: '/jobs/{id}/analysis?snapshot_revision={n}', description: 'Analyze one scored snapshot' },
    { method: 'POST', path: '/jobs/{id}/pause', description: 'Pause search exactly' },
    { method: 'POST', path: '/jobs/{id}/resume', description: 'Resume search' },
    { method: 'POST', path: '/jobs/{id}/cancel', description: 'Stop and retain the best plan' },
  ] }));
  app.appendChild(SF.createFooter({ links: [
    { label: 'SolverForge', url: 'https://www.solverforge.org' },
    { label: 'Source problem', url: 'https://huggingface.co/spaces/blackopsrepl/flight-crew-scheduling-java' },
  ] }));

  var analysisModal = SF.createModal({ title: 'Crew schedule score analysis', width: '760px' });
  var solver = SF.createSolver({
    backend: backend,
    statusBar: statusBar,
    onProgress: syncMarkers,
    onPauseRequested: syncMarkers,
    onResumed: syncMarkers,
    onSolution: acceptSnapshot,
    onPaused: acceptSnapshot,
    onCancelled: acceptSnapshot,
    onComplete: acceptSnapshot,
    onFailure: function (message, meta, snapshot) {
      acceptSnapshot(snapshot, meta);
      SF.showError('Solve failed', message);
    },
    onAnalysis: syncMarkers,
    onError: function (message) { SF.showError('Solver lifecycle failed', message); },
  });

  try {
    var catalog = await fetch('/demo-data').then(json);
    currentPlan = await backend.getDemoData(catalog.defaultId);
    render(currentPlan);
  } catch (error) {
    app.dataset.bootstrapError = 'true';
    panels.crew.appendChild(SF.el('p', { className: 'flightcrew-error' }, 'Demo data failed to load: ' + error.message));
  }

  function json(response) {
    if (!response.ok) throw new Error('HTTP ' + response.status);
    return response.json();
  }

  function constraintType(name) {
    return name === 'first_assignment_home' || name === 'last_assignment_home' ? 'soft' : 'hard';
  }

  function showTab(id) {
    Object.keys(panels).forEach(function (key) { panels[key].style.display = key === id ? '' : 'none'; });
  }

  async function startSolve() {
    if (!currentPlan || solver.isRunning()) return;
    var state = solver.getLifecycleState();
    if (solver.getJobId() && state !== 'IDLE' && state !== 'PAUSED') await solver.delete();
    await solver.start(JSON.parse(JSON.stringify(currentPlan)));
    syncMarkers();
  }

  function acceptSnapshot(snapshot, meta) {
    if (snapshot && snapshot.solution) {
      currentPlan = snapshot.solution;
      render(currentPlan);
    }
    syncMarkers(meta);
  }

  function syncMarkers(meta) {
    var jobId = solver.getJobId();
    var revision = solver.getSnapshotRevision();
    var state = meta && meta.lifecycleState ? meta.lifecycleState : solver.getLifecycleState();
    if (jobId) app.dataset.jobId = String(jobId); else delete app.dataset.jobId;
    if (revision != null) app.dataset.snapshotRevision = String(revision); else delete app.dataset.snapshotRevision;
    if (state && state !== 'IDLE') app.dataset.lifecycleState = state; else delete app.dataset.lifecycleState;
  }

  async function openAnalysis() {
    if (!solver.getJobId()) return;
    var payload = await solver.analyzeSnapshot();
    var analysis = payload.analysis || payload;
    analysisModal.setBody(SF.createTable({
      columns: ['Constraint', 'Type', 'Weight', 'Matches', 'Score'],
      rows: (analysis.constraints || []).map(function (row) {
        return [row.name, row.constraintType || constraintType(row.name), row.weight, row.matchCount, row.score];
      }),
    }));
    analysisModal.open();
  }

  function render(plan) {
    renderTimeline('crew', FlightCrew.buildCrewTimeline(plan));
    renderTimeline('flights', FlightCrew.buildFlightTimeline(plan));
    renderData(plan);
  }

  function renderTimeline(id, payload) {
    panels[id].innerHTML = '';
    panels[id].appendChild(SF.createTable({ columns: payload.summary.labels, rows: [payload.summary.values] }));
    if (!timelines[id]) timelines[id] = SF.rail.createTimeline(payload.timeline);
    else timelines[id].setModel(payload.timeline.model);
    panels[id].appendChild(timelines[id].el);
  }

  function renderData(plan) {
    panels.data.innerHTML = '';
    panels.data.appendChild(SF.createTable({
      columns: ['Flight', 'Route', 'Departure minute', 'Arrival minute', 'Required seats'],
      rows: plan.flights.map(function (flight, index) {
        var route = plan.airports[flight.departure_airport_idx].id + ' → ' + plan.airports[flight.arrival_airport_idx].id;
        var seats = plan.crew_assignments.filter(function (assignment) { return assignment.flight_idx === index; }).length;
        return [flight.id, route, flight.departure_minute, flight.arrival_minute, seats];
      }),
    }));
  }
}());
