const assert = require('node:assert/strict');
const test = require('node:test');

const {
  DAY_MINUTES,
  HORIZON_DAYS,
  buildAssignmentMap,
  buildAxis,
  buildDockLanes,
  buildVesselLanes,
} = require('../../static/fleet/schedule.js');

function plan(overrides = {}) {
  return {
    vessels: [
      { id: 'PC-01', name: 'Alder', vessel_class: 'patrol_cutter', compatible_dock_classes: ['small'], ready_from_day: 1 },
      { id: 'PC-02', name: 'Brine', vessel_class: 'patrol_cutter', compatible_dock_classes: ['small'], ready_from_day: 1 },
    ],
    docks: [
      { id: 'D1', name: 'North Dock', dock_class: 'large', capacity: 1 },
      { id: 'D2', name: 'South Dock', dock_class: 'small', capacity: 1 },
    ],
    days: Array.from({ length: 56 }, (_, index) => ({
      id: `DAY-${index + 1}`,
      name: `Day ${index + 1}`,
      index: index + 1,
      week: Math.floor(index / 7) + 1,
    })),
    work_packages: [],
    inspection_requirements: [],
    inspection_assignments: [],
    training_requirements: [],
    training_assignments: [],
    dock_outages: [],
    technician_pools: [],
    technician_capacity_overrides: [],
    readiness_policies: [],
    ...overrides,
  };
}

function workPackage(overrides = {}) {
  return {
    id: 'WP-PC-01',
    vessel_id: 'PC-01',
    package_type: 'minor_maintenance',
    duration_days: 4,
    earliest_start_day: 1,
    latest_finish_day: 14,
    priority: 80,
    defer_allowed: false,
    return_to_service_buffer_days: 1,
    dock_idx: 1,
    start_day_idx: 6,
    ...overrides,
  };
}

test('buildAxis emits an exact 56-day integer-minute horizon', () => {
  const axis = buildAxis();

  assert.equal(axis.startMinute, 0);
  assert.equal(axis.endMinute, HORIZON_DAYS * DAY_MINUTES);
  assert.equal(axis.days.length, 56);
  assert.equal(axis.ticks.length, 56);
  assert.deepEqual(axis.initialViewport, { startMinute: 0, endMinute: 14 * DAY_MINUTES });
  assert.equal(axis.days[55].startMinute, 55 * DAY_MINUTES);
  assert.equal(axis.days[55].endMinute, 56 * DAY_MINUTES);
  assert.ok(axis.days.every((day) => Number.isInteger(day.startMinute) && Number.isInteger(day.endMinute)));
  assert.ok(axis.ticks.every((tick) => Number.isInteger(tick.minute)));
});

test('assignment mapping resolves scalar indexes through the actual dock and day facts', () => {
  const data = plan({ work_packages: [workPackage()] });
  const assignments = buildAssignmentMap(data);

  assert.equal(assignments.work.length, 1);
  assert.equal(assignments.unassigned.length, 0);
  assert.equal(assignments.work[0].dockId, 'D2');
  assert.equal(assignments.work[0].startDay, 7);
  assert.equal(assignments.work[0].item.startMinute, 6 * DAY_MINUTES);
  assert.equal(assignments.work[0].item.endMinute, 10 * DAY_MINUTES);
  assert.match(assignments.work[0].item.meta, /South Dock/);
});

test('vessel and dock transforms preserve empty real lanes', () => {
  const data = plan({ work_packages: [workPackage()] });
  const vesselLanes = buildVesselLanes(data);
  const dockLanes = buildDockLanes(data);

  assert.equal(vesselLanes.length, 2);
  assert.equal(vesselLanes[1].id, 'vessel-PC-02');
  assert.equal(vesselLanes[1].items.length, 0);
  assert.ok(vesselLanes[1].badges.includes('No scheduled activity'));
  assert.equal(dockLanes.length, 2);
  assert.equal(dockLanes[0].id, 'dock-D1');
  assert.equal(dockLanes[0].items.length, 0);
  assert.ok(dockLanes[0].badges.includes('Empty'));
});

test('all unassigned work, inspections, and training remain visible in a dedicated lane', () => {
  const data = plan({
    work_packages: [workPackage({ dock_idx: null, start_day_idx: null })],
    inspection_requirements: [{ id: 'INSP-PC-01', name: 'Alder inspection', duration_days: 1, earliest_day: 6, latest_day: 12 }],
    inspection_assignments: [{ id: 'IA-PC-01', requirement_id: 'INSP-PC-01', vessel_id: 'PC-01', work_package_id: 'WP-PC-01', baseline_day: 7, day_idx: null }],
    training_requirements: [{ id: 'TRN-PC-01', name: 'Alder recertification', duration_days: 1, earliest_day: 8, latest_day: 15 }],
    training_assignments: [{ id: 'TA-PC-01', requirement_id: 'TRN-PC-01', vessel_id: 'PC-01', inspection_id: 'IA-PC-01', baseline_day: 8, day_idx: null }],
  });

  const assignments = buildAssignmentMap(data);
  const lanes = buildVesselLanes(data);
  const unassigned = lanes.find((lane) => lane.id === 'unassigned');

  assert.equal(assignments.unassigned.length, 3);
  assert.ok(unassigned);
  assert.equal(unassigned.items.length, 3);
  assert.deepEqual(assignments.unassigned.map((entry) => entry.kind), ['work', 'inspection', 'training']);
  assert.ok(unassigned.items.every((item) => item.tone === 'red' && item.endMinute > item.startMinute));
});
