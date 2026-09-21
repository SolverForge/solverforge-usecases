const test = require('node:test');
const assert = require('node:assert/strict');
const model = require('../../static/flightcrew-model.js');

function plan() {
  return {
    airports: [{ id: 'LHR' }, { id: 'JFK' }],
    employees: [{ id: 'E1', name: 'Avery', home_airport_idx: 0, skills: ['Pilot'], unavailable_days: [1] }],
    flights: [{ id: 'SF101', departure_airport_idx: 0, arrival_airport_idx: 1, departure_minute: 60, arrival_minute: 540 }],
    crew_assignments: [{ id: 'A1', flight_idx: 0, required_skill: 'Pilot', employee_idx: 0 }],
  };
}

test('crew timeline preserves exact flight geometry and availability', () => {
  const output = model.buildCrewTimeline(plan());
  assert.equal(output.timeline.model.lanes[0].items[0].startMinute, 60);
  assert.equal(output.timeline.model.lanes[0].items[0].endMinute, 540);
  assert.equal(output.timeline.model.lanes[0].overlays[0].dayIndex, 1);
});

test('flight timeline reports missing seats', () => {
  const input = plan();
  input.crew_assignments.push({ id: 'A2', flight_idx: 0, required_skill: 'Flight attendant', employee_idx: null });
  const output = model.buildFlightTimeline(input);
  assert.equal(output.summary.values[2], 1);
  assert.equal(output.timeline.model.lanes[0].badges[0], '1 open');
});
