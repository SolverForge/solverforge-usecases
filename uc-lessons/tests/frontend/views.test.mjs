import assert from 'node:assert/strict';
import test from 'node:test';

import { formatClock, parseTimeToMinutes, timeslotToMinutes } from '../../static/views.js';

// Pure time transforms behind the group, room, and teacher timelines. They run
// browserless so a scheduling regression is caught in `make test`.

test('parseTimeToMinutes reads HH:MM and HH:MM:SS and clamps to one day', () => {
  assert.equal(parseTimeToMinutes('08:30'), 510);
  assert.equal(parseTimeToMinutes('08:30:00'), 510);
  assert.equal(parseTimeToMinutes(''), 0);
  assert.equal(parseTimeToMinutes('25:00'), 1439);
});

test('timeslotToMinutes anchors a slot to its weekday offset', () => {
  assert.deepEqual(
    timeslotToMinutes({ day_of_week: 'Tue', start_time: '09:00', end_time: '10:00' }),
    { startMinute: 1980, endMinute: 2040 },
  );
});

test('timeslotToMinutes defaults a missing slot to one hour', () => {
  assert.deepEqual(timeslotToMinutes(null), { startMinute: 0, endMinute: 60 });
  assert.deepEqual(timeslotToMinutes(undefined), { startMinute: 0, endMinute: 60 });
});

test('timeslotToMinutes forces a positive duration for a degenerate slot', () => {
  assert.deepEqual(
    timeslotToMinutes({ day_of_week: 'Mon', start_time: '10:00', end_time: '10:00' }),
    { startMinute: 600, endMinute: 660 },
  );
});

test('formatClock wraps absolute minutes back into a clock label', () => {
  assert.equal(formatClock(1980), '09:00');
  assert.equal(formatClock(1410), '23:30');
  assert.equal(formatClock(-30), '23:30');
});
