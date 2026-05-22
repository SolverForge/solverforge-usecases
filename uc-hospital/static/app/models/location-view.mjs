import { groupShiftRowsByLocation } from './grouping.mjs';
import { buildShiftPresentation } from './presentation.mjs';
import { assignmentTone, buildBlockMeta, renderEmptyScheduleMessage, renderRailSchedule } from './rail-renderer.mjs';

// Renders the "By location" schedule tab on top of the shared rail timeline.
export function renderLocationView({ sf, container, data, view }) {
  const shifts = data[view.entityPlural] || [];
  const employees = data[view.sourcePlural] || [];
  if (!shifts.length) {
    renderEmptyScheduleMessage(sf, container);
    return;
  }

  const presentation = buildShiftPresentation(shifts, employees, view.variableField);
  const groups = groupShiftRowsByLocation(presentation.rows);
  const lanes = groups.map((group) => ({
    id: `${view.id}-location-${group.key}`,
    name: group.label,
    mode: 'detailed',
    stats: [
      { label: 'Shifts', value: group.rows.length },
      { label: 'Open', value: group.rows.reduce((count, row) => count + (row.isAssigned ? 0 : 1), 0) },
    ],
    rows: group.rows,
    presentRow(row) {
        return {
          label: row.employeeLabel,
          meta: buildBlockMeta(row),
          tone: assignmentTone(row.employeeLabel, row.isAssigned),
        };
      },
    }));

  renderRailSchedule({
    sf,
    container,
    axis: presentation.axis,
    headerLabel: 'Location',
    lanes,
    unassignedCount: presentation.unassignedCount,
  });
}
