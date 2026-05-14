const AVERAGE_SPEED_KMPH = 50;
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
  let previous = { lat: vehicle.homeLat, lng: vehicle.homeLng };

  for (const [sequence, deliveryId] of (vehicle.deliveryOrder || []).entries()) {
    const delivery = plan.deliveries[deliveryId];
    if (!delivery) continue;

    totalDemand += Number(delivery.demand || 0);
    const travel = estimateTravel(previous, delivery);
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
    previous = delivery;

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
    const depot = { lat: vehicle.homeLat, lng: vehicle.homeLng };
    const returnTravel = estimateTravel(previous, depot);
    totalTravelSeconds += returnTravel;
    endTime = currentTime + returnTravel;
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

function estimateTravel(from, to) {
  const meters = haversineMeters(Number(from.lat), Number(from.lng), Number(to.lat), Number(to.lng));
  const metersPerSecond = (AVERAGE_SPEED_KMPH * 1000) / 3600;
  return Math.round(meters / metersPerSecond);
}

function haversineMeters(lat1, lng1, lat2, lng2) {
  const toRad = (value) => (value * Math.PI) / 180;
  const r = 6371000;
  const dLat = toRad(lat2 - lat1);
  const dLng = toRad(lng2 - lng1);
  const a =
    Math.sin(dLat / 2) * Math.sin(dLat / 2) +
    Math.cos(toRad(lat1)) * Math.cos(toRad(lat2)) *
      Math.sin(dLng / 2) * Math.sin(dLng / 2);
  return 2 * r * Math.asin(Math.sqrt(a));
}
