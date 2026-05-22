import { groupShiftRowsByEmployee } from './grouping.mjs';
import { buildShiftPresentation } from './presentation.mjs';
import { assignmentTone, buildBlockMeta, renderEmptyScheduleMessage, renderRailSchedule } from './rail-renderer.mjs';
import { parseDateTimeMs, wallTimeDayStartMs } from './datetime.mjs';

// Renders the "By employee" schedule tab on top of the shared rail timeline.
export function renderEmployeeView({ sf, container, data, view }) {
  const shifts = data[view.entityPlural] || [];
  const employees = data[view.sourcePlural] || [];
  if (!shifts.length) {
    renderEmptyScheduleMessage(sf, container);
    return;
  }

  const presentation = buildShiftPresentation(shifts, employees, view.variableField);
  const grouped = groupShiftRowsByEmployee(presentation.rows, employees);
  const lanes = grouped.groups.map((group) => ({
    id: `${view.id}-employee-${group.key}`,
    name: group.label,
    mode: 'detailed',
    badges: group.badges || [],
    stats: [{ label: 'Shifts', value: group.rows.length }],
    overlays: buildEmployeeOverlays(group.employee, presentation.axis),
    rows: group.rows,
    presentRow(row) {
      return {
        label: row.locationLabel,
        meta: buildBlockMeta(row),
        tone: assignmentTone(row.locationLabel, true),
      };
    },
  }));

  if (grouped.unassignedRows.length) {
    lanes.push({
      id: `${view.id}-employee-unassigned`,
      name: 'Unassigned shifts',
      mode: 'detailed',
      badges: ['Needs assignment'],
      stats: [{ label: 'Shifts', value: grouped.unassignedRows.length }],
      rows: grouped.unassignedRows,
      presentRow(row) {
        return {
          label: row.locationLabel,
          meta: buildBlockMeta(row),
          tone: assignmentTone(row.locationLabel, false),
        };
      },
    });
  }

  renderRailSchedule({
    sf,
    container,
    axis: presentation.axis,
    headerLabel: 'Employee',
    lanes,
    unassignedCount: presentation.unassignedCount,
  });
}

// Builds colored day overlays from employee unavailability and preferences.
function buildEmployeeOverlays(employee, axis) {
  if (!employee || !axis || !Array.isArray(axis.columns) || !axis.columns.length) return [];

  const dayIndexByStartMs = new Map(
    axis.columns.map((column, index) => [column.startMs, index]),
  );

  return []
    .concat(buildDayOverlays(employee.unavailableDates, 'Unavailable', 'red', dayIndexByStartMs))
    .concat(buildDayOverlays(employee.undesiredDates, 'Undesired', 'amber', dayIndexByStartMs))
    .concat(buildDayOverlays(employee.desiredDates, 'Desired', 'emerald', dayIndexByStartMs));
}

// Converts a list of dates into one-day overlay blocks on the timeline axis.
function buildDayOverlays(values, label, tone, dayIndexByStartMs) {
  if (!Array.isArray(values) || !values.length) return [];

  return values.map((value, index) => {
    const parsed = parseDateTimeMs(value);
    if (parsed == null) return null;

    const dayIndex = dayIndexByStartMs.get(wallTimeDayStartMs(parsed));
    if (dayIndex == null) return null;

    return {
      id: `${label.toLowerCase()}-${dayIndex}-${index}`,
      dayIndex,
      dayCount: 1,
      label,
      tone,
    };
  }).filter(Boolean);
}
