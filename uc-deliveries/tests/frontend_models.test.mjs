import test from 'node:test';
import assert from 'node:assert/strict';

import {
  buildDeliveryTimelineModel,
  buildVehicleTimelineModel,
  refreshPlan,
  refreshServerPlan,
} from '../static/app/models.mjs';
import {
  colorForVehicle,
  routeStyleForVehicle,
} from '../static/app/ui/overview.mjs';

function basePlan() {
  return {
    name: 'Test City',
    routingMode: 'road_network',
    deliveries: [
      {
        id: 0,
        label: 'A',
        kind: 'business',
        lat: 43.7696,
        lng: 11.2558,
        demand: 3,
        minStartTime: 8 * 3600,
        maxEndTime: 12 * 3600,
        serviceDuration: 10 * 60,
      },
      {
        id: 1,
        label: 'B',
        kind: 'residential',
        lat: 43.7710,
        lng: 11.2620,
        demand: 2,
        minStartTime: 9 * 3600,
        maxEndTime: 18 * 3600,
        serviceDuration: 5 * 60,
      },
    ],
    vehicles: [
      {
        id: 0,
        name: 'Alpha',
        capacity: 10,
        homeLat: 43.7696,
        homeLng: 11.2558,
        departureTime: 6 * 3600,
        deliveryOrder: [0, 1],
      },
    ],
    viewState: {},
  };
}

test('refreshPlan builds preview assignments and scores', () => {
  const plan = refreshPlan(basePlan());
  assert.equal(plan.viewState.preview.vehicles.length, 1);
  assert.equal(plan.viewState.preview.deliveries[0].assignedVehicleName, 'Alpha');
  assert.equal(plan.viewState.preview.unassignedDeliveryIds.length, 0);
});

test('refreshServerPlan preserves authoritative backend preview', () => {
  const plan = basePlan();
  plan.viewState.preview = {
    hardScore: -99,
    softScore: -12345,
    unassignedDeliveryIds: [1],
    vehicles: [],
    deliveries: [],
  };

  const refreshed = refreshServerPlan(plan);

  assert.equal(refreshed.viewState.preview.hardScore, -99);
  assert.equal(refreshed.viewState.preview.softScore, -12345);
  assert.deepEqual(refreshed.viewState.preview.unassignedDeliveryIds, [1]);
});

test('refreshPlan recomputes draft preview without client-side routing estimates', () => {
  const plan = basePlan();
  plan.viewState.preview = {
    hardScore: -99,
    softScore: -12345,
    unassignedDeliveryIds: [1],
    vehicles: [],
    deliveries: [],
  };

  const refreshed = refreshPlan(plan);

  assert.equal(Math.abs(refreshed.viewState.preview.hardScore), 0);
  assert.notEqual(refreshed.viewState.preview.softScore, -12345);
  assert.equal(refreshed.viewState.preview.vehicles.length, 1);
  assert.equal(refreshed.viewState.preview.vehicles[0].totalTravelSeconds, 0);
});

test('refreshPlan uses dominant unassigned-delivery hard penalty', () => {
  const plan = basePlan();
  plan.vehicles[0].deliveryOrder = [0];

  const refreshed = refreshPlan(plan);

  assert.equal(refreshed.viewState.preview.unassignedDeliveryIds.length, 1);
  assert.ok(
    refreshed.viewState.preview.hardScore <= -1_000_000,
    `expected unassigned hard penalty to dominate, got ${refreshed.viewState.preview.hardScore}`,
  );
});

test('timeline model builders emit lane collections', () => {
  const preview = refreshPlan(basePlan()).viewState.preview;
  const vehicleModel = buildVehicleTimelineModel(preview);
  const deliveryModel = buildDeliveryTimelineModel(preview);

  assert.equal(vehicleModel.lanes.length, 1);
  assert.equal(vehicleModel.lanes[0].items.length, 2);
  assert.equal(deliveryModel.lanes.length, 2);
  assert.ok(deliveryModel.lanes[0].items.length >= 1);
});

test('vehicle map colors are unique across the ten-vehicle tutorial dataset', () => {
  const colors = Array.from({ length: 10 }, (_, vehicleId) => colorForVehicle(vehicleId));

  assert.equal(new Set(colors).size, 10);
  assert.notEqual(colorForVehicle(1), colorForVehicle(9));
});

test('route highlight style is keyed by vehicle id, not color reuse', () => {
  assert.deepEqual(routeStyleForVehicle(1, 1), {
    color: colorForVehicle(1),
    opacity: 1,
    weight: 5,
  });
  assert.deepEqual(routeStyleForVehicle(9, 1), {
    color: colorForVehicle(9),
    opacity: 0.2,
    weight: 2,
  });
});
