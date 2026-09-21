(function (root, factory) {
  var api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  else root.FlightCrew = api;
}(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  function axis(plan) {
    var end = Math.max.apply(null, plan.flights.map(function (flight) { return flight.arrival_minute; }).concat([1440]));
    var days = [];
    for (var start = 0, index = 0; start < end; start += 1440, index += 1) {
      days.push({ index: index, startMinute: start, endMinute: start + 1440, label: 'Day ' + (index + 1) });
    }
    return { startMinute: 0, endMinute: end, days: days, ticks: days.map(function (day) { return { minute: day.startMinute, label: day.label }; }), initialViewport: { startMinute: 0, endMinute: Math.min(end, 2880) } };
  }

  function buildCrewTimeline(plan) {
    var airports = plan.airports;
    var unassigned = plan.crew_assignments.filter(function (assignment) { return assignment.employee_idx == null; });
    var lanes = plan.employees.map(function (employee, employeeIndex) {
      var assignments = plan.crew_assignments.filter(function (assignment) { return assignment.employee_idx === employeeIndex; });
      return {
        id: employee.id,
        label: employee.name,
        mode: 'detailed',
        badges: [employee.skills.join(', '), airports[employee.home_airport_idx].id],
        stats: [{ label: 'Flights', value: assignments.length }],
        overlays: employee.unavailable_days.map(function (day) { return { dayIndex: day, label: 'Unavailable', tone: 'slate' }; }),
        items: assignments.map(function (assignment) { return assignmentItem(plan, assignment); }),
      };
    });
    if (unassigned.length) {
      lanes.push({ id: 'unassigned', label: 'Unassigned seats', mode: 'detailed', badges: ['Needs crew'], stats: [{ label: 'Seats', value: unassigned.length }], items: unassigned.map(function (assignment) { return assignmentItem(plan, assignment); }) });
    }
    return {
      summary: { labels: ['Flights', 'Crew', 'Required seats', 'Unassigned'], values: [plan.flights.length, plan.employees.length, plan.crew_assignments.length, unassigned.length] },
      timeline: { title: 'Crew rotations', subtitle: 'Duty blocks by employee with unavailable-day overlays', label: 'Crew member', labelWidth: 260, model: { axis: axis(plan), lanes: lanes } },
    };
  }

  function buildFlightTimeline(plan) {
    var lanes = plan.flights.map(function (flight, flightIndex) {
      var seats = plan.crew_assignments.filter(function (assignment) { return assignment.flight_idx === flightIndex; });
      var assigned = seats.filter(function (assignment) { return assignment.employee_idx != null; });
      var route = plan.airports[flight.departure_airport_idx].id + ' → ' + plan.airports[flight.arrival_airport_idx].id;
      var names = assigned.map(function (assignment) { return plan.employees[assignment.employee_idx].name; });
      return {
        id: flight.id,
        label: flight.id + ' · ' + route,
        mode: 'detailed',
        badges: assigned.length === seats.length ? ['Crewed'] : [(seats.length - assigned.length) + ' open'],
        stats: [{ label: 'Seats', value: assigned.length + '/' + seats.length }],
        items: [{ id: flight.id, startMinute: flight.departure_minute, endMinute: flight.arrival_minute, label: route, meta: names.length ? names.join(' · ') : 'No crew assigned', tone: assigned.length === seats.length ? 'emerald' : 'rose' }],
      };
    });
    var open = plan.crew_assignments.filter(function (assignment) { return assignment.employee_idx == null; }).length;
    return {
      summary: { labels: ['Flight legs', 'Fully crewed', 'Open seats', 'Long-haul legs'], values: [plan.flights.length, lanes.filter(function (lane) { return lane.badges[0] === 'Crewed'; }).length, open, plan.flights.filter(function (flight) { return flight.arrival_minute - flight.departure_minute >= 600; }).length] },
      timeline: { title: 'Flight manifests', subtitle: 'Required cockpit and cabin coverage by leg', label: 'Flight', labelWidth: 250, model: { axis: axis(plan), lanes: lanes } },
    };
  }

  function assignmentItem(plan, assignment) {
    var flight = plan.flights[assignment.flight_idx];
    var route = plan.airports[flight.departure_airport_idx].id + ' → ' + plan.airports[flight.arrival_airport_idx].id;
    return { id: assignment.id, startMinute: flight.departure_minute, endMinute: flight.arrival_minute, label: flight.id + ' · ' + route, meta: assignment.required_skill, tone: assignment.required_skill === 'Pilot' ? 'violet' : 'cyan' };
  }

  return { buildCrewTimeline: buildCrewTimeline, buildFlightTimeline: buildFlightTimeline };
}));
