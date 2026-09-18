(function (global) {
  'use strict';

  var Fleet = global.Fleet = global.Fleet || {};
  var T = global.FleetTransforms;

  Fleet.createLayout = function (ctx) {
    var panels = {};
    var main = SF.el('main', { className: 'sf-main' });
    [
      { id: 'schedule', active: true },
      { id: 'readiness' },
      { id: 'resources' },
      { id: 'disruptions' },
      { id: 'data' },
      { id: 'api' },
    ].forEach(function (entry) {
      var panel = SF.el('div', {
        className: 'sf-content',
        style: { display: entry.active ? '' : 'none' },
        dataset: { tabId: entry.id },
      });
      panels[entry.id] = panel;
      main.appendChild(panel);
    });

    ctx.panels = panels;
    ctx.main = main;
    ctx.sections = {
      scenario: SF.el('div'),
      bootstrap: SF.el('div', { role: 'alert', style: { display: 'none' } }),
      summary: SF.el('div'),
      vesselTimeline: SF.el('div'),
      dockTimeline: SF.el('div'),
      unassigned: SF.el('div'),
      readiness: SF.el('div'),
      resources: SF.el('div'),
      disruption: SF.el('div'),
      data: SF.el('div'),
      api: SF.el('div'),
    };

    [
      ctx.sections.scenario,
      ctx.sections.bootstrap,
      ctx.sections.summary,
      ctx.sections.vesselTimeline,
      ctx.sections.dockTimeline,
      ctx.sections.unassigned,
    ].forEach(function (node) {
      panels.schedule.appendChild(node);
    });
    panels.readiness.appendChild(ctx.sections.readiness);
    panels.resources.appendChild(ctx.sections.resources);
    panels.disruptions.appendChild(ctx.sections.disruption);
    panels.data.appendChild(ctx.sections.data);
    panels.api.appendChild(ctx.sections.api);
    return main;
  };

  Fleet.renderAll = function (ctx) {
    if (!ctx.state.plan) return;
    Fleet.renderSummary(ctx);
    Fleet.renderSchedules(ctx);
    Fleet.renderReadiness(ctx);
    Fleet.renderResources(ctx);
    Fleet.renderDisruptions(ctx);
    Fleet.renderData(ctx);
  };

  Fleet.renderSummary = function (ctx) {
    if (!ctx.state.plan) return;
    var plan = ctx.state.plan;
    var assignments = T.buildAssignmentMap(plan);
    var kpis = T.buildKpis(plan);
    var section = SF.el('div', { className: 'sf-section' });
    section.appendChild(SF.createTable({
      columns: [
        'Vessels',
        'Work packages',
        'Assigned',
        'Unassigned',
        'Ready vessel-days',
        'Dock utilization',
      ],
      rows: [[
        String((plan.vessels || []).length),
        String((plan.work_packages || []).length),
        String(assignments.work.length),
        String(assignments.unassigned.length),
        Fleet.formatNumber(kpis.readyVesselDays),
        kpis.dockUtilizationPercent + '%',
      ]],
    }));
    Fleet.replace(ctx.sections.summary, section);
  };

  Fleet.renderSchedules = function (ctx) {
    var plan = ctx.state.plan;
    renderTimeline(ctx, 'vessel', ctx.sections.vesselTimeline, {
      label: 'Vessel',
      labelWidth: 250,
      zoomPresets: ['1w', '2w', '4w', 'reset'],
      title: 'Vessel schedule',
      subtitle: '56-day readiness sequence grouped by vessel',
      model: { axis: T.buildAxis(), lanes: T.buildVesselLanes(plan) },
    });
    renderTimeline(ctx, 'dock', ctx.sections.dockTimeline, {
      label: 'Dock',
      labelWidth: 250,
      zoomPresets: ['1w', '2w', '4w', 'reset'],
      title: 'Dock occupancy',
      subtitle: 'Every dock stays visible; closure windows render as outage overlays',
      model: { axis: T.buildAxis(), lanes: T.buildDockLanes(plan) },
    });
    Fleet.renderUnassigned(ctx);
  };

  Fleet.renderUnassigned = function (ctx) {
    var entries = T.buildAssignmentMap(ctx.state.plan).unassigned;
    if (!entries.length) {
      Fleet.replace(ctx.sections.unassigned);
      return;
    }
    var section = SF.el('div', { className: 'sf-section' });
    section.appendChild(SF.createTable({
      columns: ['Type', 'Reference', 'Vessel', 'Window'],
      rows: entries.map(function (entry) {
        var source = entry.source || {};
        var requirement = entry.requirement || {};
        return [
          Fleet.labelize(entry.kind),
          requirement.name || source.id || source.requirement_id || '-',
          entry.vesselId || source.vessel_id || '-',
          source.earliest_start_day != null
            ? 'D' + source.earliest_start_day + '-D' + source.latest_finish_day
            : '-',
        ];
      }),
    }));
    Fleet.replace(ctx.sections.unassigned, section);
  };

  Fleet.renderReadiness = function (ctx) {
    var days = T.buildDailyReadiness(ctx.state.plan);
    var section = SF.el('div', { className: 'sf-section' });
    section.appendChild(SF.createTable({
      columns: ['Day', 'Week', 'Ready', 'Patrol cutter', 'Frigate like', 'Support', 'Floor'],
      rows: days.map(function (day) {
        var byClass = day.readyByClass || {};
        return [
          'D' + day.day,
          'W' + day.week,
          day.readyOverall + '/' + day.floorOverall,
          (byClass.patrol_cutter || 0) + '/' + day.floorPatrol,
          (byClass.frigate_like || 0) + '/' + day.floorFrigate,
          String(byClass.support || 0),
          day.shortfall ? 'Short by ' + day.shortfall : 'Met',
        ];
      }),
    }));
    Fleet.replace(ctx.sections.readiness, section);
  };

  Fleet.renderResources = function (ctx) {
    var plan = ctx.state.plan;
    var pressure = T.buildResourcePressure(plan);
    var deliveries = plan.deliveries || [];

    var pools = SF.el('div', { className: 'sf-section' });
    pools.appendChild(SF.el('h3', null, 'Technician pools'));
    pools.appendChild(SF.createTable({
      columns: ['Pool', 'Peak day', 'Peak / capacity', 'Utilization', 'Overtime headroom', 'Days over capacity'],
      rows: pressure.map(function (pool) {
        return [
          pool.id + ' / ' + pool.name,
          'D' + pool.peakDay,
          pool.peakDemand + ' / ' + pool.capacityAtPeak,
          pool.utilizationPercent + '%',
          String(pool.overtimeCapacity),
          String(pool.overCapacityDays),
        ];
      }),
    }));

    var parts = SF.el('div', { className: 'sf-section' });
    parts.appendChild(SF.el('h3', null, 'Parts posture'));
    parts.appendChild(SF.createTable({
      columns: ['Part', 'Description', 'On hand', 'Inbound'],
      rows: (plan.parts || []).map(function (part) {
        var inbound = deliveries.filter(function (delivery) {
          return delivery.part_id === part.id;
        });
        return [
          part.id,
          part.name,
          String(part.initial_on_hand),
          inbound.length
            ? inbound.map(function (delivery) {
              return '+' + delivery.quantity + ' on D' + delivery.arrival_day;
            }).join(', ')
            : 'No inbound delivery',
        ];
      }),
    }));

    Fleet.replace(ctx.sections.resources, pools, parts);
  };

  Fleet.renderData = function (ctx) {
    var plan = ctx.state.plan || {};
    var fragment = document.createDocumentFragment();
    Object.keys(plan).filter(function (key) {
      return Array.isArray(plan[key]);
    }).forEach(function (name) {
      var records = plan[name];
      var section = SF.el('div', { className: 'sf-section' });
      section.appendChild(SF.el('h3', null, Fleet.labelize(name) + ' / ' + records.length));
      if (!records.length) {
        section.appendChild(SF.el('p', null, 'No records in this collection.'));
      } else {
        var keys = records.reduce(function (all, record) {
          Object.keys(record || {}).forEach(function (key) {
            if (all.indexOf(key) === -1) all.push(key);
          });
          return all;
        }, []);
        section.appendChild(SF.createTable({
          columns: keys.map(Fleet.labelize),
          rows: records.map(function (record) {
            return keys.map(function (key) { return cellValue(record[key]); });
          }),
        }));
      }
      fragment.appendChild(section);
    });
    Fleet.replace(ctx.sections.data, fragment);
  };

  Fleet.renderApiGuide = function (ctx) {
    var selected = ctx.state.selectedDemoId || '{id}';
    Fleet.replace(ctx.sections.api, SF.createApiGuide({ endpoints: [
      endpoint('GET', '/demo-data', 'List selectable SolverForge demo scenarios.'),
      endpoint('GET', '/demo-data/' + selected, 'Load the selected planning problem.'),
      endpoint('POST', '/jobs', 'Start a retained solve. The browser drives this route only through SF.createSolver.'),
      endpoint('GET', '/jobs/{id}/events', 'Observe authoritative lifecycle events over SSE.'),
      endpoint('GET', '/jobs/{id}/snapshot?snapshot_revision={revision}', 'Read an exact retained solution snapshot.'),
      endpoint('GET', '/jobs/{id}/analysis?snapshot_revision={revision}', 'Analyze the exact retained score revision.'),
      endpoint('POST', '/jobs/{id}/pause', 'Request a managed pause.'),
      endpoint('POST', '/jobs/{id}/resume', 'Resume from the retained checkpoint.'),
      endpoint('POST', '/jobs/{id}/cancel', 'Stop an active or paused job.'),
      endpoint('DELETE', '/jobs/{id}', 'Delete a terminal retained job before another solve.'),
      endpoint('POST', '/plan-sessions', 'Create an integration-oriented plan session.'),
      endpoint('GET', '/plan-sessions/{id}/status', 'Read session lifecycle, revision, and telemetry.'),
      endpoint('GET', '/plan-sessions/{id}/result', 'Read the projected fleet result for an external client.'),
      endpoint('POST', '/plan-sessions/{id}/repair', 'Apply a disruption and create a lineage-preserving repair solve.'),
      endpoint('GET', '/plan-sessions/{id}/revisions/{from}/compare/next', 'Compare baseline and repaired revisions.'),
    ] }));
  };

  Fleet.openAnalysis = function (ctx) {
    if (!ctx.state.analysis) {
      SF.showToast({ variant: 'warning', title: 'No analysis available', message: 'Run the solver before opening score analysis.' });
      return;
    }
    var analysis = ctx.state.analysis.analysis || ctx.state.analysis;
    var constraints = analysis.constraints || [];
    ctx.analysisModal.setBody(SF.createTable({
      columns: ['Constraint', 'Weight', 'Score', 'Matches'],
      rows: constraints.map(function (constraint) {
        return [
          constraint.name,
          constraint.weight || Fleet.constraintType(constraint.name),
          constraint.score,
          constraint.matchCount != null ? constraint.matchCount : '-',
        ];
      }),
    }));
    ctx.analysisModal.open();
  };

  Fleet.showPanel = function (ctx, id) {
    Object.keys(ctx.panels).forEach(function (key) {
      ctx.panels[key].style.display = key === id ? '' : 'none';
    });
    if (id === 'schedule' && ctx.state.plan) Fleet.renderSchedules(ctx);
    if (id === 'disruptions') Fleet.renderDisruptions(ctx);
  };

  Fleet.destroyTimelines = function (ctx) {
    Object.keys(ctx.timelines).forEach(function (key) {
      if (ctx.timelines[key]) ctx.timelines[key].destroy();
    });
  };

  function renderTimeline(ctx, key, section, config) {
    if (!ctx.timelines[key]) ctx.timelines[key] = SF.rail.createTimeline(config);
    else ctx.timelines[key].setModel(config.model);
    Fleet.replace(section, SF.el('div', { className: 'sf-section' }, ctx.timelines[key].el));
  }

  function cellValue(value) {
    if (value == null) return 'Unassigned';
    if (Array.isArray(value)) return value.join(', ');
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  }

  function endpoint(method, path, description) {
    return {
      method: method,
      path: path,
      description: description,
      curl: 'curl -s -X ' + method + ' http://127.0.0.1:7860' + path,
    };
  }
})(window);
