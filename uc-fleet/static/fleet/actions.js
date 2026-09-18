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

  // Renders the disruption controls and the latest repair comparison. The
  // baseline session is seeded from the plan currently on screen, so a repair
  // always compares against what the operator is looking at.
  Fleet.renderDisruptions = function (ctx) {
    if (!ctx.sections || !ctx.sections.disruption) return;
    var controls = SF.el('div', {
      className: 'fleet-disruptions',
      style: { display: 'flex', flexWrap: 'wrap', gap: '0.5rem' },
    });
    Fleet.disruptions().forEach(function (disruption) {
      controls.appendChild(SF.createButton({
        text: disruption.label,
        icon: disruption.icon,
        variant: 'default',
        disabled: !!ctx.state.busy,
        onClick: function () { Fleet.applyDisruption(ctx, disruption); },
      }));
    });
    var status = SF.el('p', null, ctx.state.busy
      ? 'Seeding a baseline from the current plan, then solving the repaired plan...'
      : ctx.state.compare
        ? 'Repaired plan applied. The comparison is against the plan shown before the disruption.'
        : 'Solve a scenario, then apply a disruption to re-solve and compare the repaired plan.');
    var section = SF.el('div', { className: 'sf-section' },
      SF.el('h3', null, 'Disruption repair'),
      controls,
      status
    );
    Fleet.replace(ctx.sections.disruption, section, Fleet.comparisonSection(ctx));
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
    if (ctx.state.busy || !ctx.state.plan) {
      if (!ctx.state.plan) {
        SF.showToast({ variant: 'warning', title: 'No plan', message: 'Load a scenario first.' });
      }
      return;
    }
    if (ctx.solver.isRunning() || ctx.solver.getLifecycleState() === 'PAUSED') {
      SF.showToast({ variant: 'warning', title: 'Solve in progress', message: 'Stop the active solve before repairing.' });
      return;
    }

    ctx.state.busy = true;
    ctx.state.compare = null;
    Fleet.renderDisruptions(ctx);
    try {
      var created = await Fleet.requestJson('/plan-sessions', {
        method: 'POST',
        body: {
          domain: 'fleet_readiness',
          scenario: { scenario_id: 'current', data: ctx.state.plan },
        },
      });
      var baseline = await Fleet.waitForSessionSnapshot(created.planSessionId);
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
    } catch (error) {
      SF.showError('Repair failed', error.message || String(error));
    } finally {
      ctx.state.busy = false;
      Fleet.renderDisruptions(ctx);
    }
  };

  Fleet.waitForSessionSnapshot = async function (sessionId) {
    for (var attempt = 0; attempt < 900; attempt += 1) {
      var status = await Fleet.requestJson('/plan-sessions/' + sessionId + '/status');
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
      if (terminal.indexOf(status.lifecycleState) !== -1) {
        if (status.lifecycleState === 'FAILED') throw new Error('Repair solve failed');
        return ctx.backend.getSnapshot(jobId);
      }
      await Fleet.sleep(200);
    }
    throw new Error('Timed out waiting for the repair solve');
  };
})(window);
