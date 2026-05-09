import { DAY_MS, MINUTE_MS } from './datetime.mjs';

const DEFAULT_LABEL_WIDTH = 280;
const DEFAULT_VIEWPORT_DAYS = 14;
const SIX_HOUR_MINUTES = 6 * 60;

// Picks a deterministic color family from a row label so the UI feels stable.
export function assignmentTone(label, isAssigned) {
  if (!isAssigned) return 'red';

  const palette = ['blue', 'emerald', 'amber', 'cyan', 'violet', 'slate'];
  const key = String(label || '');
  let hash = 0;
  for (let index = 0; index < key.length; index += 1) {
    hash = ((hash * 31) + key.charCodeAt(index)) >>> 0;
  }
  return palette[hash % palette.length];
}

// Shared metadata rows shown inside detailed timeline blocks.
export function buildBlockMeta(row) {
  return [
    { label: 'Time', value: row.timeLabel },
    { label: 'Skill', value: row.requiredSkill || 'Unspecified' },
  ].filter((entry) => entry.value);
}

// Placeholder shown when a view has no shifts yet.
export function renderEmptyScheduleMessage(sf, container) {
  container.appendChild(sf.el('p', null, 'This schedule view will appear once shifts are available.'));
}

// Adapts the hospital schedule model to the shared `SF.rail.createTimeline()` widget.
export function renderRailSchedule({ sf, container, axis, headerLabel, lanes }) {
  const model = {
    axis: buildTimelineAxis(axis),
    lanes: lanes.map((lane) => buildTimelineLaneModel(lane)),
  };

  const timeline = sf.rail.createTimeline({
    label: headerLabel,
    labelWidth: DEFAULT_LABEL_WIDTH,
    model,
    title: `${headerLabel} schedule`,
    subtitle: 'Drag the day header or lane body to pan horizontally.',
  });

  container.appendChild(timeline.el);
  timeline.setViewport(model.axis.initialViewport);

  return timeline;
}

// Converts the hospital day axis into the timeline widget's minute-based model.
function buildTimelineAxis(axis) {
  const columns = Array.isArray(axis && axis.columns) ? axis.columns : [];
  if (!columns.length) {
    const endMinute = DAY_MS / MINUTE_MS;
    return {
      startMinute: 0,
      endMinute,
      days: [{ label: 'Schedule', startMinute: 0, endMinute }],
      ticks: buildTicks(endMinute),
      initialViewport: { startMinute: 0, endMinute },
    };
  }

  const horizonStartMs = Number(axis.horizonStartMs || columns[0].startMs || 0);
  const endMinute = Math.max(1, Number(axis.horizonMinutes || 1));
  const days = columns.map((column, index) => {
    const startMinute = msToMinuteOffset(column.startMs, horizonStartMs);
    const fallbackEndMinute = index === columns.length - 1
      ? endMinute
      : msToMinuteOffset(columns[index + 1].startMs, horizonStartMs);
    const nextEndMinute = column.endMs == null
      ? fallbackEndMinute
      : msToMinuteOffset(column.endMs, horizonStartMs);

    return {
      label: column.label,
      startMinute,
      endMinute: Math.max(startMinute + 1, nextEndMinute),
      isWeekend: /^(Sat|Sun)\b/.test(String(column.label || '')),
    };
  });

  return {
    startMinute: 0,
    endMinute,
    days,
    ticks: buildTicks(endMinute),
    initialViewport: {
      startMinute: 0,
      endMinute: Math.min(endMinute, DEFAULT_VIEWPORT_DAYS * 24 * 60),
    },
  };
}

// Converts one lane into the shared timeline lane shape.
function buildTimelineLaneModel(lane) {
  return {
    id: lane.id,
    label: lane.name,
    mode: lane.mode === 'overview' ? 'overview' : 'detailed',
    badges: lane.badges || [],
    stats: lane.stats || [],
    items: buildTimelineItems(lane),
    overlays: lane.overlays || [],
  };
}

// Converts all rows in a lane into timeline items.
function buildTimelineItems(lane) {
  return (lane.rows || []).map((row) => toTimelineItem(lane, row));
}

// Converts one presented shift row into a rail item.
function toTimelineItem(lane, row) {
  const display = lane.presentRow(row);
  return {
    id: `${lane.id}-shift-${row.shiftKey}`,
    startMinute: row.startOffsetMinutes,
    endMinute: row.endOffsetMinutes,
    label: display.label,
    meta: display.meta || '',
    summary: display.summary || null,
    tone: display.tone || display.color || 'slate',
    clusterId: display.clusterId || null,
  };
}

// Adds six-hour tick marks across the visible horizon.
function buildTicks(endMinute) {
  const ticks = [];
  for (let minute = 0; minute < endMinute; minute += SIX_HOUR_MINUTES) {
    ticks.push({
      minute,
      label: formatClock(minute),
    });
  }
  return ticks;
}

// Formats a minute offset as a simple hour label.
function formatClock(totalMinutes) {
  const hour = Math.floor(totalMinutes / 60) % 24;
  return `${String(hour).padStart(2, '0')}:00`;
}

// Converts absolute milliseconds into minute offsets within the current horizon.
function msToMinuteOffset(value, horizonStartMs) {
  if (value == null) return 0;
  return Math.max(0, Math.round((Number(value) - horizonStartMs) / MINUTE_MS));
}
