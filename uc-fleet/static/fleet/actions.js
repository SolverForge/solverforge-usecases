(function (global) {
  'use strict';

  var Fleet = global.Fleet = global.Fleet || {};

  // The four disruptions the app knows how to apply to a plan. Each one maps to
  // a RepairEvent understood by POST /plan-sessions/{id}/repair.
  Fleet.disruptions = function () {
    return [
      {
        id: 'technician_drop',
        label: 'Technician drop',
        icon: 'fa-user-gear',
        event: {
          type: 'technician_capacity_drop',
          payload: { pool_id: 'ELEC', start_day: 15, end_day: 21, delta: -1 },
        },
      },
      {
        id: 'dock_outage',
        label: 'Dock outage',
        icon: 'fa-warehouse',
        event: {
          type: 'dock_outage',
          payload: { dock_id: 'D2', start_day: 22, end_day: 31 },
        },
      },
      {
        id: 'delayed_parts',
        label: 'Delayed parts',
        icon: 'fa-boxes-stacked',
        event: {
          type: 'delayed_delivery',
          payload: { delivery_id: 'DELIV-01', new_arrival_day: 29 },
        },
      },
      {
        id: 'raised_floor',
        label: 'Raised readiness floor',
        icon: 'fa-chart-line',
        event: {
          type: 'policy_override',
          payload: { value: 20 },
        },
      },
    ];
  };

  // Explains whether a repair can start right now, so the buttons and the copy
  // agree with the solver lifecycle instead of failing silently on click.
  Fleet.disruptionAvailability = function (ctx) {
    if (!ctx.state.plan) return { enabled: false, reason: 'Loading the scenario plan...' };
    if (ctx.state.busy) return { enabled: false, reason: 'A disruption repair is already running.' };
    if (ctx.solver.isRunning()) {
      return { enabled: false, reason: 'A solve is running. Stop it before applying a disruption.' };
    }
    if (ctx.solver.getLifecycleState() === 'PAUSED') {
      return { enabled: false, reason: 'The solve is paused. Resume or stop it before applying a disruption.' };
    }
    return {
      enabled: true,
      reason: 'A repair seeds a baseline from the plan on screen, solves the disrupted plan, and compares the two revisions.',
    };
  };

  Fleet.renderDisruptions = function (ctx) {
    if (!ctx.sections || !ctx.sections.disruption) return;
    var availability = Fleet.disruptionAvailability(ctx);
    var buttons = [];
    var controls = SF.el('div', { style: { display: 'flex', flexWrap: 'wrap', gap: '0.5rem' } });
    Fleet.disruptions().forEach(function (disruption) {
      var button = SF.createButton({
        text: disruption.label,
        icon: disruption.icon,
        variant: 'default',
        disabled: !availability.enabled,
        onClick: function () { Fleet.applyDisruption(ctx, disruption); },
      });
      buttons.push(button);
      controls.appendChild(button);
    });

    var reason = SF.el('p', null, availability.reason);
    var progress = SF.el('div');
    var section = SF.el('div', { className: 'sf-section' },
      SF.el('h3', null, 'Disruption repair'),
      controls,
      reason,
      progress
    );
    ctx.disruption = { reason: reason, progress: progress, buttons: buttons };
    Fleet.replace(ctx.sections.disruption, section, Fleet.comparisonSection(ctx));
    Fleet.renderRepairProgress(ctx);
  };

  // Lightweight refresh used by solver events: only updates availability text and
  // button state so it is safe to call on every progress tick.
  Fleet.refreshDisruptionControls = function (ctx) {
    if (!ctx.disruption) return;
    var availability = Fleet.disruptionAvailability(ctx);
    ctx.disruption.buttons.forEach(function (button) {
      button.disabled = !availability.enabled;
    });
    ctx.disruption.reason.textContent = availability.reason;
  };

  Fleet.renderRepairProgress = function (ctx) {
    if (!ctx.disruption) return;
    var repair = ctx.state.repair;
    if (!repair || !repair.rows || !repair.rows.length) {
      ctx.disruption.progress.textContent = '';
      return;
    }
    Fleet.replace(ctx.disruption.progress, SF.createTable({
      columns: ['Repair step', 'State'],
      rows: repair.rows,
    }));
  };

  Fleet.setRepair = function (ctx, label, rows) {
    ctx.state.repair = { label: label, rows: rows };
    Fleet.renderRepairProgress(ctx);
  };

  Fleet.comparisonSection = function (ctx) {
    if (!ctx.state.compare) return null;
    var diff = ctx.state.compare.diff || {};
    return SF.el('div', { className: 'sf-section' },
      SF.el('h3', null, 'Repair comparison'),
      SF.createTable({
        columns: ['Measure', 'Value'],
        rows: [
          ['Moved assignments', String(diff.movedAssignments || 0)],
          ['Deferred assignments', String(diff.deferredAssignments || 0)],
          ['Unchanged assignments', String(diff.unchangedAssignments || 0)],
          ['Ready vessel-day delta', Fleet.signed(diff.readinessDelta)],
          ['Approval required', ctx.state.compare.requiresApproval ? 'yes' : 'no'],
          ['Reason', ctx.state.compare.approvalReason || 'Within approval thresholds'],
        ],
      })
    );
  };

  Fleet.applyDisruption = async function (ctx, disruption) {
    if (!Fleet.disruptionAvailability(ctx).enabled) return;

    ctx.setBusy(true);
    ctx.state.compare = null;
    ctx.state.repairStartedAt = Date.now();
    Fleet.setRepair(ctx, disruption.label, [
      ['Baseline session', 'seeding from the current plan'],
    ]);
    Fleet.renderDisruptions(ctx);

    try {
      var created = await Fleet.requestJson('/plan-sessions', {
        method: 'POST',
        body: {
          domain: 'fleet_readiness',
          scenario: { scenario_id: 'current', data: ctx.state.plan },
        },
      });
      var baseline = await Fleet.waitForSessionSnapshot(ctx, created.planSessionId);
      Fleet.setRepair(ctx, disruption.label, [
        ['Baseline session', 'ready at revision ' + baseline.snapshotRevision],
        ['Disruption', disruption.label + ' applied'],
        ['Repair solve', 'starting'],
      ]);
      var repair = await Fleet.requestJson('/plan-sessions/' + created.planSessionId + '/repair', {
        method: 'POST',
        body: {
          baseline_revision: baseline.snapshotRevision,
          event: disruption.event,
        },
      });
      var snapshot = await Fleet.waitForRepairSnapshot(ctx, repair.repairJobId);
      ctx.renderPlan(snapshot.solution);
      ctx.state.compare = await Fleet.requestJson(repair.compareUrl);
      Fleet.setRepair(ctx, disruption.label, [
        ['Disruption', disruption.label + ' applied'],
        ['Repair solve', 'completed at revision ' + repair.repairJobId],
        ['Comparison', 'ready below'],
      ]);
    } catch (error) {
      Fleet.setRepair(ctx, disruption.label, [
        ['Disruption', disruption.label],
        ['Result', 'failed: ' + (error.message || String(error))],
      ]);
      SF.showError('Repair failed', error.message || String(error));
    } finally {
      ctx.setBusy(false);
      Fleet.renderDisruptions(ctx);
    }
  };

  Fleet.waitForSessionSnapshot = async function (ctx, sessionId) {
    for (var attempt = 0; attempt < 900; attempt += 1) {
      var status = await Fleet.requestJson('/plan-sessions/' + sessionId + '/status');
      Fleet.setRepair(ctx, 'Seeding baseline', [
        ['Baseline session', status.lifecycleState + ' / revision ' + (status.latestRevision != null ? status.latestRevision : 'pending')],
        ['Baseline score', status.bestScore || status.currentScore || 'unscored'],
      ]);
      if (status.latestRevision != null) {
        return Fleet.requestJson('/plan-sessions/' + sessionId + '/snapshots/latest');
      }
      await Fleet.sleep(200);
    }
    throw new Error('Timed out waiting for the seeded baseline snapshot');
  };

  Fleet.waitForRepairSnapshot = async function (ctx, jobId) {
    var terminal = ['COMPLETED', 'CANCELLED', 'FAILED', 'TERMINATED_BY_CONFIG'];
    for (var attempt = 0; attempt < 900; attempt += 1) {
      var status = await ctx.backend.getJobStatus(jobId);
      Fleet.setRepair(ctx, 'Solving repaired plan', [
        ['Repair job', jobId],
        ['Lifecycle', status.lifecycleState],
        ['Repair score', status.bestScore || status.currentScore || 'unscored'],
        ['Elapsed', Fleet.elapsedSeconds(ctx) + 's'],
      ]);
      if (terminal.indexOf(status.lifecycleState) !== -1) {
        if (status.lifecycleState === 'FAILED') throw new Error('Repair solve failed');
        return ctx.backend.getSnapshot(jobId);
      }
      await Fleet.sleep(200);
    }
    throw new Error('Timed out waiting for the repair solve');
  };

  Fleet.elapsedSeconds = function (ctx) {
    var started = ctx.state.repairStartedAt || Date.now();
    return Math.round((Date.now() - started) / 1000);
  };
})(window);
