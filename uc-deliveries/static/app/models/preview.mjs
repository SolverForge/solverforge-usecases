const UNASSIGNED_DELIVERY_HARD_PENALTY = 1_000_000;

export function buildPreview(plan) {
  const assigned = new Set();
  const vehicles = [];
  const deliveries = (plan.deliveries || []).map((delivery) => ({
    deliveryId: delivery.id,
    label: delivery.label,
    kind: delivery.kind,
    demand: delivery.demand,
    minStartTime: delivery.minStartTime,
    maxEndTime: delivery.maxEndTime,
    serviceDuration: delivery.serviceDuration,
    assignedVehicleId: null,
    assignedVehicleName: null,
    sequence: null,
    arrivalTime: null,
    serviceStartTime: null,
    departureTime: null,
    lateSeconds: null,
  }));

  let capacityOverage = 0;
  let lateSeconds = 0;
  let travelSeconds = 0;

  for (const vehicle of plan.vehicles || []) {
    const metrics = computeVehicleMetrics(plan, vehicle);
    capacityOverage += metrics.capacityOverage;
    lateSeconds += metrics.totalLateSeconds;
    travelSeconds += metrics.totalTravelSeconds;
    vehicles.push(metrics);

    for (const stop of metrics.stops) {
      assigned.add(stop.deliveryId);
      const delivery = deliveries[stop.deliveryId];
      delivery.assignedVehicleId = vehicle.id;
      delivery.assignedVehicleName = vehicle.name;
      delivery.sequence = stop.sequence;
      delivery.arrivalTime = stop.arrivalTime;
      delivery.serviceStartTime = stop.serviceStartTime;
      delivery.departureTime = stop.departureTime;
      delivery.lateSeconds = stop.lateSeconds;
    }
  }

  const unassignedDeliveryIds = deliveries
    .filter((delivery) => !assigned.has(delivery.deliveryId))
    .map((delivery) => delivery.deliveryId);

  return {
    hardScore: -(
      unassignedDeliveryIds.length * UNASSIGNED_DELIVERY_HARD_PENALTY +
      capacityOverage +
      lateSeconds
    ),
    softScore: -travelSeconds,
    unassignedDeliveryIds,
    vehicles,
    deliveries,
  };
}

function computeVehicleMetrics(plan, vehicle) {
  const stops = [];
  let totalDemand = 0;
  let totalTravelSeconds = 0;
  let totalWaitSeconds = 0;
  let totalServiceSeconds = 0;
  let totalLateSeconds = 0;
  let endTime = vehicle.departureTime || 0;
  let currentTime = vehicle.departureTime || 0;

  for (const [sequence, deliveryId] of (vehicle.deliveryOrder || []).entries()) {
    const delivery = plan.deliveries[deliveryId];
    if (!delivery) continue;

    totalDemand += Number(delivery.demand || 0);
    const travel = 0;
    totalTravelSeconds += travel;
    const arrivalTime = currentTime + travel;
    const serviceStartTime = Math.max(arrivalTime, delivery.minStartTime || 0);
    const waitSeconds = Math.max(0, serviceStartTime - arrivalTime);
    const departureTime = serviceStartTime + Number(delivery.serviceDuration || 0);
    const lateSeconds = Math.max(0, departureTime - Number(delivery.maxEndTime || 0));

    totalWaitSeconds += waitSeconds;
    totalServiceSeconds += Number(delivery.serviceDuration || 0);
    totalLateSeconds += lateSeconds;
    currentTime = departureTime;
    endTime = departureTime;

    stops.push({
      deliveryId,
      label: delivery.label,
      kind: delivery.kind,
      sequence,
      demand: delivery.demand,
      minStartTime: delivery.minStartTime,
      maxEndTime: delivery.maxEndTime,
      arrivalTime,
      serviceStartTime,
      departureTime,
      travelSecondsFromPrevious: travel,
      waitSeconds,
      lateSeconds,
    });
  }

  if (stops.length) {
    endTime = currentTime;
  }

  return {
    vehicleId: vehicle.id,
    vehicleName: vehicle.name,
    totalDemand,
    capacityOverage: Math.max(0, totalDemand - Number(vehicle.capacity || 0)),
    stopCount: stops.length,
    totalTravelSeconds,
    totalWaitSeconds,
    totalServiceSeconds,
    totalLateSeconds,
    startTime: vehicle.departureTime || 0,
    endTime,
    stops,
  };
}
