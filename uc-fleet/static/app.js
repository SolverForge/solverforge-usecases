/* SolverForge Fleet browser lifecycle and shell bootstrap. */
(async function () {
  'use strict';

  var Fleet = window.Fleet;
  var app = document.getElementById('sf-app');

  try {
    var assets = await Promise.all([requestJson('/sf-config.json'), requestJson('/generated/ui-model.json')]);
    boot(assets[0], assets[1]);
  } catch (error) {
    app.dataset.bootstrapError = 'true';
    app.appendChild(SF.el('main', { className: 'sf-main' },
      SF.el('div', { className: 'sf-section', role: 'alert' },
        SF.el('h3', null, 'Fleet planner could not start'),
        SF.el('p', null, error.message || String(error))
      )
    ));
  }

  function boot(config, uiModel) {
    var backend = SF.createBackend({ type: 'axum', baseUrl: '' });
    var constraints = (config.constraints || uiModel.constraints || []).map(function (name) {
      return { name: name, type: Fleet.constraintType(name) };
    });
    var statusBar = SF.createStatusBar({
      constraints: constraints,
      onConstraintClick: function () { openAnalysis(); },
    });
    var state = {
      plan: null,
      meta: null,
      analysis: null,
      compare: null,
      catalog: null,
      selectedDemoId: null,
      bootstrapError: null,
      loadingDemo: false,
      busy: false,
    };
    var ctx = {
      app: app,
      backend: backend,
      config: config,
      uiModel: uiModel,
      state: state,
      statusBar: statusBar,
      timelines: { vessel: null, dock: null },
      analysisModal: SF.createModal({ title: 'Score Analysis', width: '760px' }),
    };

    var solver = SF.createSolver({
      backend: backend,
      statusBar: statusBar,
      onProgress: function (meta) { publishMeta(meta); },
      onPauseRequested: function (meta) { publishMeta(meta); },
      onSolution: function (snapshot, meta) { publishSnapshot(snapshot, meta); },
      onPaused: function (snapshot, meta) { publishSnapshot(snapshot, meta); },
      onResumed: function (meta) { publishMeta(meta); },
      onCancelled: function (snapshot, meta) { publishSnapshot(snapshot, meta); },
      onComplete: function (snapshot, meta) { publishSnapshot(snapshot, meta); },
      onFailure: function (message, meta, snapshot, analysis) {
        if (analysis) state.analysis = analysis;
        publishSnapshot(snapshot, meta);
        SF.showError('Solver job failed', message);
      },
      onAnalysis: function (analysis) {
        state.analysis = analysis;
        syncLifecycleMarkers();
      },
      onError: function (message) {
        syncLifecycleMarkers();
        SF.showError('Solver lifecycle error', String(message));
      },
    });
    ctx.solver = solver;
    ctx.renderPlan = renderPlan;
    ctx.setBusy = setBusy;

    var header = SF.createHeader({
      logo: '/sf/img/ouroboros.svg',
      title: config.title || 'SolverForge Fleet',
      subtitle: config.subtitle || 'Synthetic fleet maintenance and readiness planning',
      tabs: [
        { id: 'schedule', label: 'Schedule', icon: 'fa-table-columns', active: true },
        { id: 'readiness', label: 'Readiness', icon: 'fa-gauge-high' },
        { id: 'resources', label: 'Resources', icon: 'fa-chart-simple' },
        { id: 'disruptions', label: 'Disruptions', icon: 'fa-triangle-exclamation' },
        { id: 'data', label: 'Data', icon: 'fa-table' },
        { id: 'api', label: 'REST API', icon: 'fa-book' },
      ],
      actions: {
        onSolve: startSolve,
        onPause: function () { lifecycleAction('Pause', solver.pause); },
        onResume: function () { lifecycleAction('Resume', solver.resume); },
        onCancel: function () { lifecycleAction('Stop', solver.cancel); },
        onAnalyze: openAnalysis,
      },
      onTabChange: function (id) { Fleet.showPanel(ctx, id); },
    });
    ctx.header = header;
    app.appendChild(header);
    statusBar.bindHeader(header);
    app.appendChild(statusBar.el);
    app.appendChild(Fleet.createLayout(ctx));
    app.appendChild(SF.createFooter({
      links: [
        { label: 'SolverForge', url: 'https://www.solverforge.org' },
        { label: 'Docs', url: 'https://www.solverforge.org/docs' },
      ],
    }));
    Fleet.renderApiGuide(ctx);
    syncLifecycleMarkers();
    bootstrapDemoData();
    window.addEventListener('beforeunload', function () { Fleet.destroyTimelines(ctx); });

    async function bootstrapDemoData() {
      state.loadingDemo = true;
      updateSolveAvailability();
      try {
        state.catalog = await backend.listDemoData();
        var choices = normalizeCatalog(state.catalog);
        if (!choices.length) throw new Error('The demo-data catalog did not expose any scenarios.');
        state.selectedDemoId = resolveDefaultId(state.catalog, choices);
        renderScenarioControl(choices);
        await loadDemo(state.selectedDemoId);
      } catch (error) {
        reportBootstrapError(error);
      } finally {
        state.loadingDemo = false;
        updateSolveAvailability();
      }
    }

    async function loadDemo(id) {
      state.loadingDemo = true;
      updateSolveAvailability();
      try {
        var plan = await backend.getDemoData(id);
        state.selectedDemoId = id;
        state.analysis = null;
        state.bootstrapError = null;
        delete app.dataset.bootstrapError;
        ctx.sections.bootstrap.style.display = 'none';
        renderPlan(plan);
        Fleet.renderApiGuide(ctx);
      } catch (error) {
        reportBootstrapError(error);
        throw error;
      } finally {
        state.loadingDemo = false;
        updateSolveAvailability();
      }
    }

    function renderScenarioControl(choices) {
      var select = SF.el('select', { id: 'fleet-scenario', 'aria-label': 'Fleet scenario' });
      choices.forEach(function (choice) {
        var option = SF.el('option', { value: choice.id }, choice.label);
        option.selected = choice.id === state.selectedDemoId;
        select.appendChild(option);
      });
      select.addEventListener('change', function () {
        if (state.busy || solver.isRunning() || solver.getLifecycleState() === 'PAUSED') {
          select.value = state.selectedDemoId;
          SF.showToast({ variant: 'warning', title: 'Scenario locked', message: 'Stop the active solve or repair before switching scenarios.' });
          return;
        }
        loadDemo(select.value).catch(function () { select.value = state.selectedDemoId; });
      });
      Fleet.replace(ctx.sections.scenario,
        SF.el('div', { className: 'sf-section' },
          SF.el('h3', null, 'Scenario'),
          select
        )
      );
      ctx.scenarioSelect = select;
    }

    function startSolve() {
      if (state.busy) {
        SF.showToast({ variant: 'warning', title: 'Repair in progress', message: 'Wait for the disruption repair to finish before starting a solve.' });
        return;
      }
      if (!canSolve() || solver.isRunning() || solver.getLifecycleState() === 'PAUSED') return;
      cleanupTerminalJob()
        .then(function () {
          state.analysis = null;
          return solver.start(Fleet.clonePlan(state.plan));
        })
        .then(syncLifecycleMarkers)
        .catch(function (error) { SF.showError('Solve could not start', error.message || String(error)); });
    }

    function cleanupTerminalJob() {
      var lifecycle = solver.getLifecycleState();
      if (!solver.getJobId() || lifecycle === 'IDLE' || lifecycle === 'PAUSED' || solver.isRunning()) return Promise.resolve();
      return solver.delete().then(function () {
        state.analysis = null;
        state.meta = null;
        syncLifecycleMarkers();
      });
    }

    function lifecycleAction(label, action) {
      action.call(solver)
        .then(syncLifecycleMarkers)
        .catch(function (error) { SF.showError(label + ' failed', error.message || String(error)); });
    }

    function openAnalysis() {
      if (!solver.getJobId()) {
        Fleet.openAnalysis(ctx);
        return;
      }
      solver.analyzeSnapshot()
        .then(function (analysis) {
          state.analysis = analysis;
          Fleet.openAnalysis(ctx);
          syncLifecycleMarkers();
        })
        .catch(function (error) { SF.showError('Analysis failed', error.message || String(error)); });
    }

    function publishSnapshot(snapshot, meta) {
      if (snapshot && snapshot.solution) renderPlan(snapshot.solution);
      publishMeta(meta);
    }

    function publishMeta(meta) {
      state.meta = meta || state.meta;
      if (state.plan) Fleet.renderSummary(ctx);
      Fleet.refreshDisruptionControls(ctx);
      syncLifecycleMarkers(meta);
    }

    function renderPlan(plan) {
      state.plan = Fleet.clonePlan(plan);
      Fleet.renderAll(ctx);
    }

    function syncLifecycleMarkers(meta) {
      var jobId = solver.getJobId();
      var revision = solver.getSnapshotRevision();
      var lifecycle = meta && meta.lifecycleState || solver.getLifecycleState();
      if (jobId != null && jobId !== '') app.dataset.jobId = String(jobId);
      else delete app.dataset.jobId;
      if (revision != null) app.dataset.snapshotRevision = String(revision);
      else delete app.dataset.snapshotRevision;
      app.dataset.lifecycleState = lifecycle || 'IDLE';
      updateSolveAvailability();
    }

    function reportBootstrapError(error) {
      state.bootstrapError = error && error.message || String(error);
      app.dataset.bootstrapError = 'true';
      ctx.sections.bootstrap.textContent = 'Demo data bootstrap failed: ' + state.bootstrapError;
      ctx.sections.bootstrap.style.display = '';
      updateSolveAvailability();
      console.error('Demo data bootstrap failed:', error);
    }

    function canSolve() {
      return !!state.plan && !state.bootstrapError && !state.loadingDemo;
    }

    function updateSolveAvailability() {
      var button = Fleet.findHeaderButton(header, 'Solve');
      if (!button) return;
      var disabled = !canSolve() || state.busy;
      button.disabled = disabled;
      button.setAttribute('aria-disabled', disabled ? 'true' : 'false');
      button.title = disabled
        ? (state.busy ? 'Repair in progress' : (state.bootstrapError || 'Loading scenario...'))
        : '';
      if (ctx.scenarioSelect) {
        ctx.scenarioSelect.disabled = state.busy || state.loadingDemo || solver.isRunning() || solver.getLifecycleState() === 'PAUSED';
      }
    }

    // Busy means a disruption repair owns the pipeline. Starting a solve
    // concurrently would race the repair's final render, so both the header
    // Solve button and the scenario selector are held until it finishes.
    function setBusy(value) {
      state.busy = value;
      updateSolveAvailability();
      Fleet.refreshDisruptionControls(ctx);
    }
  }

  function normalizeCatalog(catalog) {
    catalog = catalog || {};
    var values = catalog.availableIds || catalog.available_ids || catalog.scenarios || [];
    return values.map(function (value) {
      if (typeof value === 'string') return { id: value, label: demoLabel(value) };
      return {
        id: String(value.id || value.name),
        label: value.label || demoLabel(value.id || value.name),
      };
    });
  }

  function resolveDefaultId(catalog, choices) {
    var value = catalog.defaultId || catalog.default_id;
    return choices.some(function (choice) { return choice.id === value; }) ? value : choices[0].id;
  }

  function demoLabel(id) {
    return {
      BASELINE: 'Baseline / compact',
      TECHNICIAN_SHORTAGE: 'Electrical technician shortage',
      BASELINE_FULL: 'Baseline / full fleet',
    }[id] || Fleet.labelize(id);
  }

  async function requestJson(path) {
    var response = await fetch(path);
    if (!response.ok) throw new Error(path + ' returned HTTP ' + response.status);
    return response.json();
  }
})();
