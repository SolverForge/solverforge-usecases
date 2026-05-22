const assert = require('node:assert/strict');
const test = require('node:test');

const { importModule } = require('./support/load-browser-modules');
const { duplicateNameEmployees, employee, shift } = require('./support/fixtures');

// Presentation tests lock down the transport-to-timeline conversion layer.
test('shift presentation builds a day-aligned horizon for overnight schedules', async () => {
  const { buildShiftPresentation } = await importModule('static/app/models/presentation.mjs');

  const presentation = buildShiftPresentation(
    [
      shift({
        start: '2024-01-01T22:00:00',
        end: '2024-01-02T06:00:00',
        location: 'ICU',
      }),
    ],
    [employee('employee-1', 'Jordan')],
    'employeeIdx',
  );

  assert.equal(presentation.axis.columns.length, 2);
  assert.deepEqual(
    Array.from(presentation.axis.columns, (column) => column.label),
    ['Mon 1 Jan', 'Tue 2 Jan'],
  );
  assert.equal(presentation.axis.horizonMinutes, 2880);
  assert.equal(presentation.rows[0].startOffsetMinutes, 1320);
  assert.equal(presentation.rows[0].endOffsetMinutes, 1800);
  assert.equal(presentation.rows[0].durationMinutes, 480);
});

test('shift presentation preserves stable employee identity for duplicate names', async () => {
  const { buildShiftPresentation } = await importModule('static/app/models/presentation.mjs');

  const presentation = buildShiftPresentation(
    [
      shift({ id: 'shift-1', employeeIdx: 0 }),
      shift({
        id: 'shift-2',
        start: '2024-01-01T22:00:00',
        end: '2024-01-02T06:00:00',
        location: 'ICU',
        employeeIdx: 1,
      }),
    ],
    duplicateNameEmployees(),
    'employeeIdx',
  );

  assert.equal(presentation.rows.length, 2);
  assert.equal(presentation.rows[0].employeeLabel, 'Alex');
  assert.equal(presentation.rows[1].employeeLabel, 'Alex');
  assert.notEqual(presentation.rows[0].employeeKey, presentation.rows[1].employeeKey);
});
