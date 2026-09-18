import { syncConstraintStatus } from './analysis.js';
import { buildSchedulePayload, destroyTimeline, ensureTimeline, averageAssignmentLoad } from './schedule.js';
import { assignmentMap, clonePlan, operatorShifts, valueAsString } from './utils.js';

var SF = window.SF;

export function renderAll(ctx, data) {
  ctx.currentPlan = clonePlan(data);
  renderOverview(ctx, data);
  renderViews(ctx, data);
  renderTables(ctx, data);
  syncConstraintStatus(ctx);
}

export function renderOverview(ctx, data) {
  var assignedCount = (data.assignments || []).length;
  var workOrderCount = (data.workOrders || []).length;
  var lateCount = (data.assignments || []).filter(function (assignment) { return assignment.late; }).length;
  var staffedCount = (data.operatorShiftAssignments || []).filter(function (assignment) {
    return assignment.shiftId != null && assignment.rosterDay >= 0;
  }).length;
  var kpiSection = SF.el('div', { className: 'sf-section' });
  kpiSection.appendChild(SF.createTable({
    columns: ['Furnaces', 'Orders', 'Assigned', 'Late', 'Operators', 'Planned shifts'],
    rows: [[
      String((data.furnaces || []).length),
      String(workOrderCount),
      String(assignedCount),
      String(lateCount),
      String((data.operators || []).length),
      String(staffedCount),
    ]],
  }));

  var headline = SF.el('div', { className: 'sf-section' });
  headline.appendChild(SF.el(
    'p',
    null,
    'This app now uses the stock SolverForge shell: one app entrypoint, one status bar, one retained-job lifecycle, and one timeline surface driven by the current furnace DTO.'
  ));

  var sections = [kpiSection, headline];
  var schedulePayload = buildSchedulePayload(data);
  if (schedulePayload) {
    var preview = SF.el('div', { className: 'sf-section' });
    preview.appendChild(SF.createTable({
      columns: ['Scheduled orders', 'Unassigned', 'Late', 'Avg load kg'],
      rows: [[
        String(assignedCount),
        String(Math.max(workOrderCount - assignedCount, 0)),
        String(lateCount),
        averageAssignmentLoad(data.assignments || []),
      ]],
    }));
    preview.appendChild(ensureTimeline(ctx.viewTimelines, 'overview-schedule', schedulePayload.timeline).el);
    if (schedulePayload.unassigned) {
      preview.appendChild(schedulePayload.unassigned.cloneNode(true));
    }
    sections.push(preview);
  } else {
    destroyTimeline(ctx.viewTimelines, 'overview-schedule');
  }

  ctx.overviewContainer.replaceChildren.apply(ctx.overviewContainer, sections);
}

export function renderViews(ctx, data) {
  (ctx.uiModel.views || []).forEach(function (view) {
    var container = document.getElementById('view-' + view.id);
    if (!container) return;

    if (view.id === 'schedule') {
      renderScheduleView(ctx, container, data);
      return;
    }

    container.innerHTML = '';
    if (view.id === 'crew') {
      renderCrewView(container, data);
      return;
    }
    if (view.id === 'orders') {
      renderOrdersView(container, data);
      return;
    }

    container.appendChild(SF.el('p', null, 'No renderer is registered for this view yet.'));
  });
}

function renderScheduleView(ctx, container, data) {
  var payload = buildSchedulePayload(data);
  if (!payload) {
    destroyTimeline(ctx.viewTimelines, 'schedule');
    container.replaceChildren(SF.el('p', null, 'No furnace assignments are available yet.'));
    return;
  }

  var children = [
    payload.summary,
    ensureTimeline(ctx.viewTimelines, 'schedule', payload.timeline).el,
  ];
  if (payload.unassigned) {
    children.push(payload.unassigned);
  }
  container.replaceChildren.apply(container, children);
}

function renderCrewView(container, data) {
  var section = SF.el('div', { className: 'sf-section' });
  var rows = (data.operators || []).map(function (operator) {
    var shifts = operatorShifts(data.operatorShiftAssignments || [], operator.id);
    return [
      operator.name,
      operator.role,
      operator.dayOnly ? 'Day only' : 'All shifts',
      shifts.length ? shifts.join(', ') : 'Rest',
      operator.skills && operator.skills.length ? operator.skills.join(', ') : '—',
    ];
  });

  section.appendChild(SF.createTable({
    columns: ['Operator', 'Role', 'Availability', 'Planned shifts', 'Skills'],
    rows: rows,
  }));
  container.appendChild(section);
}

function renderOrdersView(container, data) {
  var assigned = assignmentMap(data.assignments || []);
  var rows = (data.workOrders || []).map(function (order) {
    var assignment = assigned[String(order.id)];
    return [
      order.orderCode,
      order.customer,
      order.process,
      String(order.temperatureCelsius) + ' °C',
      assignment ? assignment.furnaceName : 'Unassigned',
      assignment ? assignment.windowLabel : 'Not scheduled',
      order.dueLabel,
      assignment && assignment.late ? String(assignment.lateMinutes) + ' min' : 'On time',
    ];
  });

  var section = SF.el('div', { className: 'sf-section' });
  section.appendChild(SF.createTable({
    columns: ['Order', 'Customer', 'Process', 'Temperature', 'Furnace', 'Window', 'Due', 'Delay'],
    rows: rows,
  }));
  container.appendChild(section);
}

function renderTables(ctx, data) {
  ctx.tablesContainer.innerHTML = '';
  (ctx.uiModel.entities || []).concat(ctx.uiModel.facts || []).forEach(function (entry) {
    var rows = data[entry.plural] || [];
    if (!rows.length) return;
    ctx.tablesContainer.appendChild(buildCollectionSection(entry.label, rows));
  });
}

function buildCollectionSection(label, rows) {
  var cols = Object.keys(rows[0]);
  var values = rows.map(function (row) {
    return cols.map(function (key) {
      return valueAsString(row[key]);
    });
  });
  var section = SF.el('div', { className: 'sf-section' });
  section.appendChild(SF.el('h3', null, label));
  section.appendChild(SF.createTable({ columns: cols, rows: values }));
  return section;
}
