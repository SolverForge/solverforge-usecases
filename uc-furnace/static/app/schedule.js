import { DAY_LABELS, DAY_MINUTES, HORIZON_MINUTES, VIEWPORT_DAYS } from './constants.js';
import { averageAssignmentLoad, pad2, toneForPriority } from './utils.js';

var SF = window.SF;

export function buildSchedulePayload(data) {
  var furnaces = data.furnaces || [];
  var assignments = (data.assignments || []).slice().sort(function (left, right) {
    if (left.startMinutes !== right.startMinutes) return left.startMinutes - right.startMinutes;
    return left.furnaceId - right.furnaceId;
  });
  if (!furnaces.length) return null;

  var assignedIds = {};
  assignments.forEach(function (assignment) {
    assignedIds[String(assignment.workOrderId)] = true;
  });

  var lanes = furnaces.map(function (furnace) {
    var furnaceAssignments = assignments.filter(function (assignment) {
      return assignment.furnaceId === furnace.id;
    });
    return {
      id: 'furnace-' + furnace.id,
      label: furnace.name,
      mode: 'detailed',
      badges: [furnace.furnaceType, String(furnace.maxTempCelsius) + ' °C'],
      stats: [
        { label: 'Orders', value: furnaceAssignments.length },
        { label: 'Capacity', value: String(furnace.maxLoadKg) + ' kg' },
      ],
      items: furnaceAssignments.map(function (assignment) {
        return {
          id: 'assignment-' + assignment.workOrderId,
          startMinute: assignment.startMinutes,
          endMinute: assignment.endMinutes,
          label: assignment.orderCode,
          meta: assignment.customer + ' · ' + assignment.process,
          tone: toneForPriority(assignment.priority),
        };
      }),
    };
  });

  var unassignedOrders = (data.workOrders || []).filter(function (order) {
    return !assignedIds[String(order.id)];
  });
  return {
    summary: buildSummarySection(
      ['Furnace lanes', 'Scheduled orders', 'Unassigned', 'Late orders'],
      [[
        String(furnaces.length),
        String(assignments.length),
        String(unassignedOrders.length),
        String(assignments.filter(function (assignment) { return assignment.late; }).length),
      ]]
    ),
    timeline: {
      label: 'Furnace',
      labelWidth: 280,
      zoomPresets: [],
      title: 'Furnace schedule',
      subtitle: 'Seven-day retained-job schedule grouped by furnace',
      model: {
        axis: buildWeekAxis(),
        lanes: lanes,
      },
    },
    unassigned: buildUnassignedSection(unassignedOrders),
  };
}

export function ensureTimeline(viewTimelines, viewId, timelineConfig) {
  var timeline = viewTimelines[viewId];
  if (!timeline) {
    timeline = SF.rail.createTimeline(timelineConfig);
    viewTimelines[viewId] = timeline;
    return timeline;
  }

  timeline.setModel(timelineConfig.model);
  return timeline;
}

export function destroyTimeline(viewTimelines, viewId) {
  var timeline = viewTimelines[viewId];
  if (!timeline) return;
  timeline.destroy();
  delete viewTimelines[viewId];
}

export function destroyAllTimelines(viewTimelines) {
  Object.keys(viewTimelines).forEach(function (viewId) {
    destroyTimeline(viewTimelines, viewId);
  });
}

export function buildWeekAxis() {
  var days = [];
  var ticks = [];

  DAY_LABELS.forEach(function (label, dayIndex) {
    var startMinute = dayIndex * DAY_MINUTES;
    days.push({
      id: 'day-' + dayIndex,
      label: label,
      subLabel: 'Day ' + String(dayIndex + 1),
      startMinute: startMinute,
      endMinute: startMinute + DAY_MINUTES,
    });
    for (var hour = 0; hour < 24; hour += 6) {
      ticks.push({
        id: 'tick-' + dayIndex + '-' + hour,
        minute: startMinute + (hour * 60),
        label: pad2(hour) + ':00',
      });
    }
  });

  return {
    startMinute: 0,
    endMinute: HORIZON_MINUTES,
    days: days,
    ticks: ticks,
    initialViewport: {
      startMinute: 0,
      endMinute: VIEWPORT_DAYS * DAY_MINUTES,
    },
  };
}

function buildSummarySection(columns, rows) {
  var section = SF.el('div', { className: 'sf-section' });
  section.appendChild(SF.createTable({
    columns: columns,
    rows: rows,
  }));
  return section;
}

function buildUnassignedSection(orders) {
  if (!orders.length) return null;
  var section = SF.el('div', { className: 'sf-section' });
  section.appendChild(SF.createTable({
    columns: ['Unassigned order', 'Customer', 'Process', 'Due'],
    rows: orders.map(function (order) {
      return [order.orderCode, order.customer, order.process, order.dueLabel];
    }),
  }));
  return section;
}

export { averageAssignmentLoad };
