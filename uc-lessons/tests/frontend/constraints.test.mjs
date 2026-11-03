import assert from 'node:assert/strict';
import test from 'node:test';

import {
  buildStatusBarConstraints,
  constraintLabel,
  withConstraintLabels,
} from '../../static/constraints.js';

// Score analysis names each constraint by its module id; the browser shows a
// readable label and colors status-bar dots by the constraint's score level.

test('constraintLabel turns analysis constraint ids into readable labels', () => {
  assert.equal(constraintLabel('late_lesson'), 'Avoid Late Lessons');
  assert.equal(constraintLabel('room_kind'), 'Room Kind');
  assert.equal(constraintLabel('home_hub_match'), 'Home Hub Match');
});

test('buildStatusBarConstraints labels generated ids with their score level', () => {
  assert.deepEqual(buildStatusBarConstraints(['assign_room', 'room_capacity', 'repeated_subject_day']), [
    { name: 'Assign Room', type: 'medium' },
    { name: 'Room Capacity', type: 'hard' },
    { name: 'Avoid Repeated Subject Day', type: 'soft' },
  ]);
});

test('withConstraintLabels relabels the dots repainted from analysis', () => {
  const painted = [];
  const statusBar = withConstraintLabels({ colorDotsFromAnalysis: (constraints) => painted.push(constraints) });
  statusBar.colorDotsFromAnalysis([{ name: 'no_room_conflict', type: 'hard', score: '0hard/0medium/0soft' }]);
  assert.deepEqual(painted, [[{ name: 'No Room Conflict', type: 'hard', score: '0hard/0medium/0soft' }]]);
});
