const assert = require('node:assert/strict');
const test = require('node:test');

const { importModule } = require('./support/load-browser-modules');
const { duplicateNameEmployees, employee, shift } = require('./support/fixtures');

// Grouping tests document how the schedule rows are organized before rendering.
test('location grouping keeps unassigned shifts visible and sorts by start time', async () => {
  const { buildShiftPresentation } = await importModule('static/app/schedule/presentation.mjs');
  const { groupShiftRowsByLocation } = await importModule('static/app/schedule/grouping.mjs');

  const presentation = buildShiftPresentation(
    [
      shift({
        id: 'shift-2',
        start: '2024-01-01T12:00:00',
        end: '2024-01-01T20:00:00',
        requiredSkill: 'Nurse',
        employeeIdx: null,
      }),
      shift({ id: 'shift-1' }),
      shift({
        id: 'shift-3',
        start: '2024-01-01T07:00:00',
        end: '2024-01-01T15:00:00',
        location: 'ICU',
      }),
    ],
    [employee('employee-1', 'Jordan')],
  );

  const groups = groupShiftRowsByLocation(presentation.rows);

  assert.equal(presentation.unassignedCount, 1);
  assert.equal(groups.length, 2);
  assert.equal(groups[0].label, 'ER');
  assert.equal(groups[0].rows[0].shift.id, 'shift-1');
  assert.equal(groups[0].rows[1].shift.id, 'shift-2');
  assert.equal(groups[0].rows[1].employeeLabel, 'Unassigned');
});

test('location grouping preserves first-seen lane order', async () => {
  const { buildShiftPresentation } = await importModule('static/app/schedule/presentation.mjs');
  const { groupShiftRowsByLocation } = await importModule('static/app/schedule/grouping.mjs');

  const presentation = buildShiftPresentation(
    [
      shift({ id: 'shift-a', location: 'Radiology' }),
      shift({ id: 'shift-b', location: 'ER' }),
      shift({ id: 'shift-c', location: 'Radiology', start: '2024-01-01T18:00:00', end: '2024-01-01T23:00:00' }),
    ],
    [employee('employee-1', 'Jordan')],
  );

  const groups = groupShiftRowsByLocation(presentation.rows);

  assert.deepEqual(Array.from(groups, (group) => group.label), ['Radiology', 'ER']);
});

test('employee grouping keeps duplicate employee names separate and preserves empty lanes', async () => {
  const { buildShiftPresentation } = await importModule('static/app/schedule/presentation.mjs');
  const { groupShiftRowsByEmployee } = await importModule('static/app/schedule/grouping.mjs');

  const presentation = buildShiftPresentation(
    [shift({ employeeIdx: null })],
    duplicateNameEmployees(),
  );
  const grouped = groupShiftRowsByEmployee(presentation.rows, duplicateNameEmployees());

  assert.equal(grouped.groups.length, 2);
  assert.equal(grouped.groups[0].rows.length, 0);
  assert.equal(grouped.groups[1].rows.length, 0);
  assert.deepEqual(Array.from(grouped.groups[0].badges), ['employee-1']);
  assert.deepEqual(Array.from(grouped.groups[1].badges), ['employee-2']);
  assert.equal(grouped.unassignedRows.length, 1);
});
