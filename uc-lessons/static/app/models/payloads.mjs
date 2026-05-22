/* payloads.mjs — View payload building for solverforge-lessons */

import { title, entityLabel } from './formatters.mjs';

const SLOT_MINUTES = 60;
const DEFAULT_VIEWPORT_SLOTS = 12;

export function buildSlotAxis(slotCount) {
  var normalizedSlots = Math.max(slotCount, 1);
  var groupSize = normalizedSlots > 24 ? 8 : (normalizedSlots > 12 ? 6 : 4);
  var days = [];
  var ticks = [];

  for (var startSlot = 0; startSlot < normalizedSlots; startSlot += groupSize) {
    var endSlot = Math.min(normalizedSlots, startSlot + groupSize);
    days.push({
      id: 'window-' + startSlot,
      label: 'Window ' + String(days.length + 1),
      subLabel: slotRangeLabel(startSlot, endSlot),
      startMinute: startSlot * SLOT_MINUTES,
      endMinute: endSlot * SLOT_MINUTES,
    });
  }

  for (var slotIndex = 0; slotIndex < normalizedSlots; slotIndex += 1) {
    ticks.push({
      id: 'tick-' + slotIndex,
      minute: slotIndex * SLOT_MINUTES,
      label: 'Slot ' + String(slotIndex + 1),
    });
  }

  return {
    startMinute: 0,
    endMinute: normalizedSlots * SLOT_MINUTES,
    days: days,
    ticks: ticks,
    initialViewport: {
      startMinute: 0,
      endMinute: Math.min(normalizedSlots, DEFAULT_VIEWPORT_SLOTS) * SLOT_MINUTES,
    },
  };
}

export function buildTimelineItem(id, slotIndex, label, meta, toneKey, tones) {
  var TIMELINE_TONES = tones || ['emerald', 'blue', 'amber', 'rose', 'violet', 'slate'];
  var hash = 0;
  var text = String(toneKey || label || '');
  for (var index = 0; index < text.length; index += 1) {
    hash = ((hash * 31) + text.charCodeAt(index)) >>> 0;
  }
  return {
    id: id,
    startMinute: slotIndex * SLOT_MINUTES,
    endMinute: (slotIndex + 1) * SLOT_MINUTES,
    label: String(label),
    meta: meta || '',
    tone: TIMELINE_TONES[hash % TIMELINE_TONES.length],
  };
}

export function slotRangeLabel(startSlot, endSlot) {
  if (endSlot - startSlot <= 1) {
    return 'Slot ' + String(startSlot + 1);
  }
  return 'Slots ' + String(startSlot + 1) + '-' + String(endSlot);
}

export function listLaneBadges(length, longestSequence) {
  if (length === 0) return ['Empty'];
  var badges = [];
  if (length === longestSequence) badges.push('Longest');
  if (length === 1) badges.push('Single');
  return badges;
}

export function buildSummarySection(columns, row, SF) {
  var section = SF.el('div', { className: 'sf-section' });
  section.appendChild(SF.createTable({
    columns: columns,
    rows: [row],
  }));
  return section;
}

export function buildScalarViewPayload(data, view, entityLabelFunc, titleFunc, SF) {
  var entityLabel = entityLabelFunc || entityLabel;
  var titleFn = titleFunc || title;
  var entities = data[view.entityPlural] || [];
  var facts = data[view.sourcePlural] || [];
  if (!entities.length || !facts.length) return null;

  var byIndex = {};
  facts.forEach(function (fact, index) {
    byIndex[index] = fact;
  });

  var assignments = facts.map(function () { return []; });
  var detached = [];
  entities.forEach(function (entity) {
    var idx = entity[view.variableField];
    if (idx == null || byIndex[idx] == null) {
      detached.push(entity);
      return;
    }
    assignments[idx].push(entity);
  });

  var peakLoad = assignments.reduce(function (maxCount, items) {
    return Math.max(maxCount, items.length);
  }, 0);
  var horizon = Math.max(peakLoad, detached.length, 1);
  var axis = buildSlotAxis(horizon);
  var lanes = facts.map(function (fact, factIndex) {
    var items = assignments[factIndex] || [];
    return {
      id: view.id + '-lane-' + factIndex,
      label: String(entityLabel(fact, factIndex)),
      mode: 'detailed',
      badges: items.length ? [] : ['Empty'],
      stats: [{ label: titleFn(view.entityPlural), value: items.length }],
      items: items.map(function (entity, itemIndex) {
        return buildTimelineItem(
          view.id + '-fact-' + factIndex + '-entity-' + itemIndex,
          itemIndex,
          entityLabel(entity, itemIndex),
          'Assignment ' + String(itemIndex + 1),
          entityLabel(entity, itemIndex)
        );
      }),
    };
  });

  if (detached.length) {
    lanes.push({
      id: view.id + '-detached',
      label: view.allowsUnassigned ? 'Unassigned' : 'Unmapped',
      mode: 'detailed',
      badges: [view.allowsUnassigned ? 'Needs assignment' : 'Out of range'],
      stats: [{ label: titleFn(view.entityPlural), value: detached.length }],
      items: detached.map(function (entity, itemIndex) {
        return buildTimelineItem(
          view.id + '-detached-' + itemIndex,
          itemIndex,
          entityLabel(entity, itemIndex),
          view.allowsUnassigned ? 'Awaiting assignment' : 'Invalid source index',
          entityLabel(entity, itemIndex)
        );
      }),
    });
  }

  return {
    summary: buildSummarySection(
      ['Source lanes', titleFn(view.entityPlural), 'Peak load', 'Unassigned'],
      [
        String(facts.length),
        String(entities.length),
        String(peakLoad),
        String(detached.length),
      ],
      SF
    ),
    timeline: {
      label: titleFn(view.sourcePlural),
      labelWidth: 280,
      title: view.label,
      subtitle: titleFn(view.entityPlural) + ' grouped by ' + titleFn(view.sourcePlural),
      model: {
        axis: axis,
        lanes: lanes,
      },
    },
  };
}

export function buildListViewPayload(data, view, entityLabelFunc, titleFunc, SF) {
  var entityLabel = entityLabelFunc || entityLabel;
  var titleFn = titleFunc || title;
  var entities = data[view.entityPlural] || [];
  var facts = data[view.sourcePlural] || [];
  if (!entities.length || !facts.length) return null;

  var byIndex = {};
  facts.forEach(function (fact, index) {
    byIndex[index] = fact;
  });

  var rows = entities.map(function (entity, entityIndex) {
    var sequence = Array.isArray(entity[view.variableField]) ? entity[view.variableField] : [];
    return {
      entity: entity,
      entityIndex: entityIndex,
      sequence: sequence,
    };
  });

  rows.sort(function (left, right) {
    if (right.sequence.length !== left.sequence.length) {
      return right.sequence.length - left.sequence.length;
    }
    return String(entityLabel(left.entity, left.entityIndex)).localeCompare(
      String(entityLabel(right.entity, right.entityIndex))
    );
  });

  var totalItems = rows.reduce(function (sum, row) {
    return sum + row.sequence.length;
  }, 0);
  var longestSequence = rows.reduce(function (maxCount, row) {
    return Math.max(maxCount, row.sequence.length);
  }, 0);
  var emptyEntities = rows.filter(function (row) { return row.sequence.length === 0; }).length;
  var horizon = Math.max(longestSequence, 1);
  var axis = buildSlotAxis(horizon);

  var lanes = rows.map(function (row) {
    return {
      id: view.id + '-entity-' + row.entityIndex,
      label: entityLabel(row.entity, row.entityIndex),
      mode: 'detailed',
      badges: listLaneBadges(row.sequence.length, longestSequence),
      stats: [{ label: titleFn(view.sourcePlural), value: row.sequence.length }],
      items: row.sequence.map(function (factIndex, sequenceIndex) {
        var fact = byIndex[factIndex];
        return buildTimelineItem(
          view.id + '-entity-' + row.entityIndex + '-item-' + sequenceIndex,
          sequenceIndex,
          entityLabel(fact, factIndex),
          'Position ' + String(sequenceIndex + 1),
          entityLabel(fact, factIndex)
        );
      }),
    };
  });

  return {
    summary: buildSummarySection(
      [titleFn(view.entityPlural), titleFn(view.sourcePlural), 'Longest sequence', 'Empty lanes', 'Average items / lane'],
      [
        String(rows.length),
        String(totalItems),
        String(longestSequence),
        String(emptyEntities),
        rows.length ? (totalItems / rows.length).toFixed(1) : '0.0',
      ],
      SF
    ),
    timeline: {
      label: titleFn(view.entityPlural),
      labelWidth: 280,
      title: view.label,
      subtitle: titleFn(view.sourcePlural) + ' ordered inside each ' + titleFn(view.entityPlural),
      model: {
        axis: axis,
        lanes: lanes,
      },
    },
  };
}
