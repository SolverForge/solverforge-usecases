import test from 'node:test';
import assert from 'node:assert/strict';
import { buildPlanModel, buildRouteTimeline, warehouseDistance } from '../../static/app/orders-model.mjs';

const depot = { shelvingId: '(A,1)', side: 'LEFT', row: 5 };
const plan = {
  pickSteps: [
    { id: 's1', name: 'Tea', productId: '1', orderId: 'A', orderItemId: 'A-1', volumeCm3: 40000, location: { shelvingId: '(A,1)', side: 'LEFT', row: 8 } },
    { id: 's2', name: 'Tissue', productId: '2', orderId: 'A', orderItemId: 'A-2', volumeCm3: 40000, location: { shelvingId: '(B,1)', side: 'LEFT', row: 3 } },
    { id: 's3', name: 'Milk', productId: '3', orderId: 'B', orderItemId: 'B-1', volumeCm3: 1000, location: { shelvingId: '(D,1)', side: 'RIGHT', row: 1 } },
  ],
  trolleys: [{ id: '1', bucketCount: 2, bucketCapacity: 48000, location: depot, stepOrder: [0, 1] }],
};

test('warehouse distance covers source piecewise branches', () => {
  assert.equal(warehouseDistance(depot, { shelvingId: '(A,1)', side: 'LEFT', row: 8 }), 3);
  assert.equal(warehouseDistance({ shelvingId: '(A,1)', side: 'LEFT', row: 2 }, { shelvingId: '(A,1)', side: 'RIGHT', row: 3 }), 7);
  assert.equal(warehouseDistance({ shelvingId: '(A,1)', side: 'RIGHT', row: 2 }, { shelvingId: '(B,1)', side: 'LEFT', row: 8 }), 9);
  assert.equal(warehouseDistance({ shelvingId: '(A,1)', side: 'LEFT', row: 2 }, { shelvingId: '(C,1)', side: 'LEFT', row: 7 }), 19);
});

test('plan model reports routes, bucket ceilings, and unassigned work', () => {
  const model = buildPlanModel(plan);
  assert.deepEqual(model.kpis, { orders: 2, items: 3, activeTrolleys: 1, routeMeters: 30 });
  assert.equal(model.routes[0].requiredBuckets, 2);
  assert.equal(model.routes[0].excessBuckets, 0);
  assert.equal(model.unassigned[0].id, 's3');
  assert.equal(model.products.length, 3);
});

test('route timeline retains an explicit unassigned lane', () => {
  const timeline = buildRouteTimeline(buildPlanModel(plan));
  assert.equal(timeline.model.lanes.at(-1).label, 'Unassigned work');
  assert.equal(timeline.model.lanes.at(-1).items.length, 1);
  assert.ok(Number.isInteger(timeline.model.axis.endMinute));
});
