import test from 'node:test';
import assert from 'node:assert/strict';
import { buildViewModel, trolleyColor, warehouseDistance } from '../../static/app/orders-model.mjs';

const depot = { shelvingId: '(A,1)', side: 'LEFT', row: 5 };
const plan = {
  pickSteps: [
    { id: 's1', name: 'Tea', productId: '1', orderId: 'A', orderItemId: 'A-1', volumeCm3: 40000, location: { shelvingId: '(A,1)', side: 'LEFT', row: 8 } },
    { id: 's2', name: 'Tissue', productId: '2', orderId: 'A', orderItemId: 'A-2', volumeCm3: 40000, location: { shelvingId: '(B,1)', side: 'LEFT', row: 3 } },
    { id: 's3', name: 'Milk', productId: '3', orderId: 'B', orderItemId: 'B-1', volumeCm3: 1000, location: { shelvingId: '(D,1)', side: 'RIGHT', row: 1 } },
  ],
  trolleys: [
    { id: '1', bucketCount: 2, bucketCapacity: 48000, location: depot, stepOrder: [0, 1], routeDistanceMeters: 30 },
    { id: '2', bucketCount: 2, bucketCapacity: 48000, location: depot, stepOrder: [], routeDistanceMeters: 0 },
  ],
};

test('warehouse distance covers source piecewise branches', () => {
  assert.equal(warehouseDistance(depot, { shelvingId: '(A,1)', side: 'LEFT', row: 8 }), 3);
  assert.equal(warehouseDistance({ shelvingId: '(A,1)', side: 'LEFT', row: 2 }, { shelvingId: '(A,1)', side: 'RIGHT', row: 3 }), 7);
  assert.equal(warehouseDistance({ shelvingId: '(A,1)', side: 'RIGHT', row: 2 }, { shelvingId: '(B,1)', side: 'LEFT', row: 8 }), 9);
  assert.equal(warehouseDistance({ shelvingId: '(A,1)', side: 'LEFT', row: 2 }, { shelvingId: '(C,1)', side: 'LEFT', row: 7 }), 19);
});

test('view model resolves trolley routes into ordered card and canvas data', () => {
  const model = buildViewModel(plan);

  assert.equal(model.orders, 2);
  assert.equal(model.items, 3);
  assert.equal(model.activeTrolleys, 1);
  assert.equal(model.totalMeters, 30);

  const [first, second] = model.trolleys;
  assert.deepEqual(first.steps.map((step) => step.id), ['s1', 's2']);
  assert.equal(first.itemCount, 2);
  assert.equal(first.requiredBuckets, 2);
  assert.equal(first.excessBuckets, 0);
  assert.equal(first.capacityPercent, 83);
  assert.equal(first.capacityClass, 'medium');
  assert.equal(first.distanceMeters, 30);
  assert.equal(second.itemCount, 0);

  assert.deepEqual(model.unassigned.map((step) => step.id), ['s3']);
});

test('trolley colours cycle through the shared palette', () => {
  assert.equal(trolleyColor('1'), '#ef4444');
  assert.equal(trolleyColor('2'), '#3b82f6');
  assert.equal(trolleyColor('9'), trolleyColor('1'));
});
