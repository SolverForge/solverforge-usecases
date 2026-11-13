import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  buildFleetSummary,
  buildMapLayers,
  buildRouteCards,
  buildVehicleTimelineModel,
  constraintLabel,
  customerOwners,
  formatClock,
  formatDuration,
  usedChargers,
} from '../../static/app/model.mjs';

function plan() {
  return {
    customers: [
      { id: 'a', name: 'Farmacia A', lat: 45.7, lng: 9.6, demand: 3 },
      { id: 'b', name: 'Farmacia B', lat: 45.8, lng: 9.7, demand: 4 },
      { id: 'c', name: 'Farmacia C', lat: 45.9, lng: 9.8, demand: 2 },
    ],
    chargers: [
      { id: 'k0', name: 'Enel X', lat: 45.75, lng: 9.65 },
      { id: 'k1', name: 'Be Charge', lat: 45.95, lng: 9.85 },
    ],
    vehicles: [
      {
        id: 'van-0',
        name: 'Van 1',
        capacity: 10,
        homeLat: 45.67,
        homeLng: 9.66,
        departureTime: 27_000,
        initialChargeM: 40_000,
        visitOrder: [1, 0],
        routeDemand: 7,
        routeDistanceM: 61_400,
        routeDrivingSeconds: 4_200,
        routeChargingSeconds: 2_700,
        routeLateSeconds: 0,
        routeShortfallM: 0,
        routeUnreachableLegs: 0,
        schedule: [
          { kind: 'depart', index: null, arrival: 27_000, start: 27_000, end: 27_000, chargeOnArrivalM: 40_000, lateS: 0 },
          { kind: 'visit', index: 1, arrival: 28_800, start: 28_800, end: 29_400, chargeOnArrivalM: 21_000, lateS: 0 },
          { kind: 'charge', index: 0, arrival: 30_000, start: 30_000, end: 32_700, chargeOnArrivalM: 12_000, lateS: 0 },
          { kind: 'visit', index: 0, arrival: 33_300, start: 33_300, end: 33_900, chargeOnArrivalM: 150_000, lateS: 120 },
          { kind: 'return', index: null, arrival: 35_000, start: 35_000, end: 35_000, chargeOnArrivalM: 141_000, lateS: 0 },
        ],
      },
      { id: 'van-1', name: 'Van 2', capacity: 8, homeLat: 45.67, homeLng: 9.66, departureTime: 27_000, initialChargeM: 90_000, visitOrder: [], schedule: [] },
    ],
  };
}

test('clock and duration formatting', () => {
  assert.equal(formatClock(27_000), '07:30');
  assert.equal(formatDuration(2_700), '45m');
  assert.equal(formatDuration(5_400), '1h 30m');
});

test('constraint labels cover every module and title-case unknown ids', () => {
  assert.equal(constraintLabel('battery_shortfall'), 'Battery Shortfall');
  assert.equal(constraintLabel('some_new_rule'), 'Some New Rule');
});

test('ownership and charger use come from the routes', () => {
  assert.deepEqual(customerOwners(plan()), [0, 0, null]);
  assert.deepEqual([...usedChargers(plan())], [0]);
});

test('route cards summarise load, charging, and the lowest charge', () => {
  const [busy, idle] = buildRouteCards(plan());
  assert.equal(busy.stops, 2);
  assert.equal(busy.load, '7/10 crates');
  assert.equal(busy.chargeStops, 1);
  assert.equal(busy.chargingTime, '45m');
  assert.equal(busy.charging, '⚡ 1 × 45m');
  assert.equal(busy.lowestCharge, '12 km');
  assert.equal(busy.returnTime, '09:43');
  assert.equal(busy.hasViolation, false);
  assert.equal(idle.stops, 0);
  assert.equal(idle.lowestCharge, '90 km');
  assert.equal(idle.charging, 'no charging');
});

test('fleet summary counts routed customers and charging stops', () => {
  const summary = buildFleetSummary(plan());
  assert.equal(summary.routed, 2);
  assert.equal(summary.customers, 3);
  assert.equal(summary.chargeStops, 1);
  assert.equal(summary.distance, '61 km');
});

test('timeline keeps every van lane and marks charges and late visits', () => {
  const model = buildVehicleTimelineModel(plan());
  assert.equal(model.lanes.length, 2);
  const items = model.lanes[0].items;
  assert.deepEqual(items.map((item) => item.tone), ['emerald', 'amber', 'rose']);
  assert.equal(items[1].label, 'Charge · Enel X');
  assert.equal(items[1].startMinute, 500);
  assert.equal(items[1].endMinute, 545);
  assert.equal(model.lanes[1].items.length, 0);
  assert.equal(model.axis.startMinute, 420);
  assert.ok(Number.isInteger(model.axis.endMinute));
});

test('map layers colour by owner, mark used chargers, and dash detours', () => {
  const routes = {
    vehicles: [
      {
        vehicleId: 'van-0',
        legs: [
          { chargingDetour: false, encodedPolyline: 'a' },
          { chargingDetour: true, encodedPolyline: 'b' },
        ],
      },
    ],
  };
  const layers = buildMapLayers(plan(), routes, null);
  assert.equal(layers.hubs.length, 1);
  assert.deepEqual(layers.customers.map((c) => c.assigned), [true, true, false]);
  assert.deepEqual(layers.chargers.map((c) => c.used), [true, false]);
  assert.equal(layers.lines[0].dashArray, undefined);
  assert.equal(layers.lines[1].dashArray, '6 6');

  const focused = buildMapLayers(plan(), routes, 1);
  assert.ok(focused.lines.every((line) => line.opacity < 0.5));
});
