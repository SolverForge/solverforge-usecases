import assert from 'node:assert/strict';
import test from 'node:test';

import {
  assignmentMap,
  averageAssignmentLoad,
  clonePlan,
  operatorShifts,
  pad2,
  toneForPriority,
  valueAsString,
} from '../../static/app/utils.js';

// These tests cover the pure browser helpers that shape furnace tables and
// lanes. They run without a DOM so a regression fails in `make test` before it
// reaches the Playwright smoke.

test('toneForPriority maps each priority to its lane color', () => {
  assert.equal(toneForPriority('Express'), 'rose');
  assert.equal(toneForPriority('Urgent'), 'amber');
  assert.equal(toneForPriority('Standard'), 'emerald');
  assert.equal(toneForPriority(undefined), 'emerald');
});

test('averageAssignmentLoad rounds the mean load and handles an empty list', () => {
  assert.equal(averageAssignmentLoad([{ loadWeightKg: 100 }, { loadWeightKg: 201 }]), '151');
  assert.equal(averageAssignmentLoad([]), '0');
});

test('assignmentMap indexes assignments by their work order id', () => {
  const map = assignmentMap([
    { workOrderId: 7, furnaceName: 'Chamber 1' },
    { workOrderId: '8', furnaceName: 'Pit 2' },
  ]);
  assert.equal(map['7'].furnaceName, 'Chamber 1');
  assert.equal(map['8'].furnaceName, 'Pit 2');
});

test('operatorShifts keeps only labeled shifts and sorts by day then label', () => {
  const shifts = operatorShifts(
    [
      { operatorId: 1, rosterDay: 2, shiftLabel: 'Night' },
      { operatorId: 1, rosterDay: 0, shiftLabel: 'Morning' },
      { operatorId: 2, rosterDay: 0, shiftLabel: 'Morning' },
      { operatorId: 1, rosterDay: 0, shiftLabel: null },
      { operatorId: 1, rosterDay: 2, shiftLabel: 'Afternoon' },
    ],
    1,
  );
  assert.deepEqual(shifts, ['Morning', 'Afternoon', 'Night']);
});

test('valueAsString renders nulls, arrays, objects, and scalars', () => {
  assert.equal(valueAsString(null), '—');
  assert.equal(valueAsString(['a', 'b']), 'a, b');
  assert.equal(valueAsString({ a: 1 }), '{"a":1}');
  assert.equal(valueAsString(12), '12');
});

test('pad2 zero-pads single digits', () => {
  assert.equal(pad2(3), '03');
  assert.equal(pad2(12), '12');
});

test('clonePlan produces an independent deep copy', () => {
  const plan = { assignments: [{ workOrderId: 1 }] };
  const copy = clonePlan(plan);
  copy.assignments[0].workOrderId = 99;
  assert.equal(plan.assignments[0].workOrderId, 1);
});
