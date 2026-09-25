import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

// app-utils.js is a classic-script IIFE that hangs pure route helpers on
// window.FSR.utils. Load it under a minimal window so the helpers can be
// exercised without a DOM or a running server.
globalThis.window = { location: { origin: 'http://localhost:7860' } };
const source = readFileSync(new URL('../../static/app-utils.js', import.meta.url), 'utf8');
new Function(source)();
const U = globalThis.window.FSR.utils;

const plan = {
  locations: [{}, {}, {}],
  service_visits: [
    { location_idx: 1, earliest_minute: 0, latest_minute: 600, duration_minutes: 30, required_skill_mask: 1, required_parts_mask: 1 },
    { location_idx: 2, earliest_minute: 200, latest_minute: 300, duration_minutes: 60, required_skill_mask: 2, required_parts_mask: 4 },
  ],
  travel_legs: [
    { from_location_idx: 0, to_location_idx: 1, duration_seconds: 600, reachable: true },
    { from_location_idx: 1, to_location_idx: 2, duration_seconds: 300, reachable: true },
    { from_location_idx: 2, to_location_idx: 0, duration_seconds: 900, reachable: true },
  ],
};

const route = {
  shift_start_minute: 480,
  shift_end_minute: 1020,
  max_route_minutes: 480,
  start_location_idx: 0,
  end_location_idx: 0,
  visits: [0, 1],
  skill_mask: 3,
  inventory_mask: 5,
};

test('formatDuration and timeLabel format minutes for operators', () => {
  assert.equal(U.formatDuration(0), '0m');
  assert.equal(U.formatDuration(95), '1h 35m');
  assert.equal(U.timeLabel(485), '08:05');
});

test('title humanizes snake_case labels', () => {
  assert.equal(U.title('minimize_travel'), 'Minimize Travel');
});

test('maskContains requires every required bit', () => {
  assert.equal(U.maskContains(5, 4), true);
  assert.equal(U.maskContains(4, 5), false);
});

test('routeSchedule chains travel, earliest start, and service time', () => {
  const entries = U.routeSchedule(plan, route);
  assert.equal(entries.length, 2);
  assert.equal(entries[0].start, 490);
  assert.equal(entries[0].end, 520);
  assert.equal(entries[1].start, 525);
  assert.equal(entries[1].end, 585);
});

test('routeStats reports travel, service, and lateness per route', () => {
  const stats = U.routeStats(plan, route);
  assert.equal(stats.travelMinutes, 30);
  assert.equal(stats.serviceMinutes, 90);
  assert.equal(stats.lateMinutes, 225);
  assert.equal(stats.missingSkills, 0);
  assert.equal(stats.missingParts, 0);
  assert.equal(stats.unreachable, 0);
  assert.deepEqual(U.routeBadges(stats), ['Late']);
});

test('assignedVisitSet maps each visit to its route', () => {
  const assigned = U.assignedVisitSet([{ id: 'r1', visits: [0, 2] }, { id: 'r2', visits: [1] }]);
  assert.equal(assigned[0].id, 'r1');
  assert.equal(assigned[1].id, 'r2');
  assert.equal(assigned[2].id, 'r1');
});

test('iconForVisit and toneForRoute are stable', () => {
  assert.equal(U.iconForVisit({ required_skill_mask: 8 }), 'fa-elevator');
  assert.equal(U.iconForVisit({ required_skill_mask: 1 }), 'fa-screwdriver-wrench');
  assert.equal(U.toneForRoute(0), 'blue');
  assert.equal(U.toneForRoute(6), 'blue');
});
