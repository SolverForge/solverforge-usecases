import { compactDateTime, normalizeShiftBounds, parseDateTimeMs, MINUTE_MS, DAY_MS, wallTimeDayStartMs, formatAxisDayLabel } from './datetime.mjs';
import { entityKey, factKey, displayLabel } from './identity.mjs';

// Derives the visible day-aligned timeline axis from the current shift rows.
function buildScheduleAxis(rows) {
  const timedRows = (rows || []).filter((row) => row.startMs != null && row.endMs != null);
  if (!timedRows.length) {
    return {
      horizonStartMs: 0,
      horizonEndMs: DAY_MS,
      horizonMinutes: DAY_MS / MINUTE_MS,
      columns: [{ label: 'Schedule', startMs: 0, endMs: DAY_MS }],
    };
  }

  const minStartMs = timedRows.reduce((min, row) => Math.min(min, row.startMs), timedRows[0].startMs);
  const maxEndMs = timedRows.reduce((max, row) => Math.max(max, row.endMs), timedRows[0].endMs);
  const displayEndMs = maxEndMs > minStartMs ? maxEndMs - 1 : maxEndMs;
  const horizonStartMs = wallTimeDayStartMs(minStartMs);
  const horizonEndMs = wallTimeDayStartMs(displayEndMs) + DAY_MS;
  const dayCount = Math.max(1, Math.round((horizonEndMs - horizonStartMs) / DAY_MS));
  const columns = [];

  for (let dayIndex = 0; dayIndex < dayCount; dayIndex += 1) {
    const columnStartMs = horizonStartMs + (dayIndex * DAY_MS);
    columns.push({
      label: formatAxisDayLabel(columnStartMs),
      startMs: columnStartMs,
      endMs: columnStartMs + DAY_MS,
    });
  }

  return {
    horizonStartMs,
    horizonEndMs,
    horizonMinutes: Math.max(1, Math.round((horizonEndMs - horizonStartMs) / MINUTE_MS)),
    columns,
  };
}

// Consistent ordering shared by grouping and rendering.
export function compareShiftRows(a, b) {
  if (a.startSortKey !== b.startSortKey) {
    return String(a.startSortKey).localeCompare(String(b.startSortKey));
  }
  if (a.endSortKey !== b.endSortKey) {
    return String(a.endSortKey).localeCompare(String(b.endSortKey));
  }
  if (a.locationLabel !== b.locationLabel) {
    return String(a.locationLabel).localeCompare(String(b.locationLabel));
  }
  return String(a.shiftKey).localeCompare(String(b.shiftKey));
}

// Converts raw transport data into render-friendly rows plus a shared axis model.
export function buildShiftPresentation(shifts = [], employees = [], variableField = 'employeeIdx') {
  const employeeByIndex = {};
  employees.forEach((employee, index) => {
    employeeByIndex[index] = employee;
  });

  let rows = shifts.map((shift, shiftIndex) => {
    const employeeIndex = shift[variableField];
    const employee = employeeIndex == null ? null : employeeByIndex[employeeIndex] || null;
    const bounds = normalizeShiftBounds(parseDateTimeMs(shift.start), parseDateTimeMs(shift.end));
    const startLabel = compactDateTime(shift.start);
    const endLabel = compactDateTime(shift.end);
    const timeLabel = startLabel && endLabel
      ? `${startLabel} → ${endLabel}`
      : startLabel || endLabel || 'Unscheduled';

    return {
      shift,
      shiftIndex,
      shiftKey: entityKey(shift, shiftIndex),
      employee,
      employeeIndex: employee == null ? null : employeeIndex,
      employeeKey: employee == null ? null : factKey(employee, employeeIndex),
      employeeLabel: employee == null ? 'Unassigned' : displayLabel(employee, employeeIndex),
      isAssigned: employee != null,
      locationLabel: shift.location || 'Unspecified location',
      requiredSkill: shift.requiredSkill || '',
      startLabel,
      endLabel,
      startMs: bounds.startMs,
      endMs: bounds.endMs,
      startSortKey: shift.start || '',
      endSortKey: shift.end || '',
      timeLabel,
    };
  });

  const axis = buildScheduleAxis(rows);
  rows = rows.map((row) => {
    const startOffsetMinutes = row.startMs == null
      ? 0
      : Math.max(0, Math.round((row.startMs - axis.horizonStartMs) / MINUTE_MS));
    const endOffsetMinutes = row.endMs == null
      ? startOffsetMinutes + 1
      : Math.max(startOffsetMinutes + 1, Math.round((row.endMs - axis.horizonStartMs) / MINUTE_MS));
    const clampedEndOffsetMinutes = Math.min(axis.horizonMinutes, endOffsetMinutes);

    return {
      ...row,
      startOffsetMinutes,
      endOffsetMinutes: Math.max(startOffsetMinutes + 1, clampedEndOffsetMinutes),
      durationMinutes: Math.max(1, clampedEndOffsetMinutes - startOffsetMinutes),
    };
  }).sort(compareShiftRows);

  return {
    rows,
    unassignedCount: rows.reduce((count, row) => count + (row.isAssigned ? 0 : 1), 0),
    axis,
  };
}
