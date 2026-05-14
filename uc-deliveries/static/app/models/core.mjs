import { buildPreview } from './preview.mjs';

export function clonePlan(plan) {
  return JSON.parse(JSON.stringify(plan));
}

export function refreshPlan(plan) {
  const normalized = normalizePlan(clonePlan(plan));
  normalized.viewState = normalized.viewState || {};
  normalized.viewState.preview = buildPreview(normalized);
  return normalized;
}

export function refreshServerPlan(plan) {
  const normalized = normalizePlan(clonePlan(plan));
  normalized.viewState = normalized.viewState || {};
  if (!normalized.viewState.preview) {
    normalized.viewState.preview = buildPreview(normalized);
  }
  return normalized;
}

export function normalizePlan(plan) {
  const normalized = clonePlan(plan);
  normalized.deliveries = normalized.deliveries || [];
  normalized.vehicles = normalized.vehicles || [];
  normalized.viewState = normalized.viewState || {};

  const oldToNew = new Map();
  normalized.deliveries = normalized.deliveries.map((delivery, index) => {
    oldToNew.set(delivery.id, index);
    return {
      kind: 'other',
      demand: 1,
      minStartTime: 0,
      maxEndTime: 24 * 3600,
      serviceDuration: 0,
      ...delivery,
      id: index,
    };
  });

  normalized.vehicles = normalized.vehicles.map((vehicle, index) => ({
    ...vehicle,
    name: vehicle.name || `Vehicle ${index + 1}`,
    capacity: Number(vehicle.capacity || 0),
    homeLat: Number(vehicle.homeLat || 0),
    homeLng: Number(vehicle.homeLng || 0),
    departureTime: Number(vehicle.departureTime || 0),
    deliveryOrder: (vehicle.deliveryOrder || [])
      .map((oldId) => oldToNew.get(oldId))
      .filter((value) => value !== undefined),
    id: index,
  }));

  return normalized;
}
