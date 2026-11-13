// Pure plan-to-view transforms for SolverForge EV.
//
// Everything here takes plan JSON (as served by `/demo-data/{id}` or a solver
// snapshot) and returns plain data for the stock solverforge-ui components.
// No DOM access, so `node --test tests/frontend/` covers it directly.

// Constraint ids are the `src/constraints/` module names; the type mirrors
// each module's `hard_weight` (hard) or plain weight (soft).
export const CONSTRAINTS = [
  { id: 'battery_shortfall', type: 'hard', label: 'Battery Shortfall' },
  { id: 'vehicle_capacity', type: 'hard', label: 'Vehicle Capacity' },
  { id: 'customer_time_windows', type: 'hard', label: 'Customer Time Windows' },
  { id: 'unreachable_legs', type: 'hard', label: 'Unreachable Legs' },
  { id: 'total_travel_time', type: 'soft', label: 'Total Travel Time' },
];

const VEHICLE_COLORS = ['#10b981', '#3b82f6', '#8b5cf6', '#f59e0b', '#ec4899', '#06b6d4', '#f43f5e', '#84cc16'];
export const UNUSED_COLOR = '#9ca3af';

export function constraintLabel(id) {
  const known = CONSTRAINTS.find((constraint) => constraint.id === id);
  if (known) return known.label;
  return String(id || '').replace(/_/g, ' ').replace(/\b\w/g, (match) => match.toUpperCase());
}

export function statusBarConstraints() {
  return CONSTRAINTS.map((constraint) => ({ name: constraint.label, type: constraint.type }));
}

export function colorForVehicle(index) {
  return VEHICLE_COLORS[Math.abs(Math.trunc(Number(index) || 0)) % VEHICLE_COLORS.length];
}

export function clonePlan(plan) {
  return JSON.parse(JSON.stringify(plan));
}

export function formatClock(seconds) {
  const safe = Math.max(0, Math.floor(seconds || 0));
  const hours = Math.floor(safe / 3600) % 24;
  const minutes = Math.floor((safe % 3600) / 60);
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
}

export function formatDuration(seconds) {
  const safe = Math.max(0, Math.round(seconds || 0));
  if (safe >= 3600) return `${Math.floor(safe / 3600)}h ${String(Math.round((safe % 3600) / 60)).padStart(2, '0')}m`;
  return `${Math.round(safe / 60)}m`;
}

export function formatKm(meters) {
  return `${Math.round((meters || 0) / 1000)} km`;
}

// Vehicle index of every customer, or `null` when unrouted.
export function customerOwners(plan) {
  const owners = (plan?.customers || []).map(() => null);
  (plan?.vehicles || []).forEach((vehicle, vehicleIndex) => {
    (vehicle.visitOrder || []).forEach((customerIndex) => {
      if (customerIndex >= 0 && customerIndex < owners.length) owners[customerIndex] = vehicleIndex;
    });
  });
  return owners;
}

// Charger indexes any route stops at.
export function usedChargers(plan) {
  const used = new Set();
  (plan?.vehicles || []).forEach((vehicle) => {
    (vehicle.schedule || []).forEach((event) => {
      if (event.kind === 'charge' && event.index != null) used.add(event.index);
    });
  });
  return used;
}

// One summary row per van for the route list.
export function buildRouteCards(plan) {
  return (plan?.vehicles || []).map((vehicle, index) => {
    const schedule = vehicle.schedule || [];
    const charges = schedule.filter((event) => event.kind === 'charge');
    const lowest = schedule.reduce((min, event) => Math.min(min, event.chargeOnArrivalM), Infinity);
    const last = schedule[schedule.length - 1];
    return {
      index,
      id: vehicle.id,
      name: vehicle.name,
      color: colorForVehicle(index),
      stops: (vehicle.visitOrder || []).length,
      load: `${vehicle.routeDemand || 0}/${vehicle.capacity} crates`,
      distance: formatKm(vehicle.routeDistanceM),
      chargeStops: charges.length,
      chargingTime: formatDuration(vehicle.routeChargingSeconds),
      charging: charges.length
        ? `⚡ ${charges.length} × ${formatDuration(vehicle.routeChargingSeconds)}`
        : 'no charging',
      lowestCharge: Number.isFinite(lowest) ? formatKm(lowest) : formatKm(vehicle.initialChargeM),
      returnTime: last ? formatClock(last.arrival) : null,
      hasViolation:
        (vehicle.routeShortfallM || 0) > 0 ||
        (vehicle.routeLateSeconds || 0) > 0 ||
        (vehicle.routeDemand || 0) > vehicle.capacity ||
        (vehicle.routeUnreachableLegs || 0) > 0,
    };
  });
}

// Fleet KPIs for the summary card.
export function buildFleetSummary(plan) {
  const vehicles = plan?.vehicles || [];
  const sum = (field) => vehicles.reduce((total, vehicle) => total + (vehicle[field] || 0), 0);
  const routed = customerOwners(plan).filter((owner) => owner != null).length;
  return {
    customers: (plan?.customers || []).length,
    routed,
    vans: vehicles.length,
    chargers: (plan?.chargers || []).length,
    distance: formatKm(sum('routeDistanceM')),
    driving: formatDuration(sum('routeDrivingSeconds')),
    charging: formatDuration(sum('routeChargingSeconds')),
    chargeStops: vehicles.reduce(
      (total, vehicle) => total + (vehicle.schedule || []).filter((event) => event.kind === 'charge').length,
      0,
    ),
    shortfall: formatKm(sum('routeShortfallM')),
    late: formatDuration(sum('routeLateSeconds')),
  };
}

// Rail timeline: one lane per van, one block per delivery or charging stop.
export function buildVehicleTimelineModel(plan) {
  const customers = plan?.customers || [];
  const chargers = plan?.chargers || [];
  const lanes = (plan?.vehicles || []).map((vehicle, index) => {
    const items = [];
    (vehicle.schedule || []).forEach((event, eventIndex) => {
      if (event.kind === 'visit') {
        const customer = customers[event.index] || {};
        items.push({
          id: `${vehicle.id}-visit-${eventIndex}`,
          startMinute: Math.floor(event.start / 60),
          endMinute: Math.max(Math.floor(event.start / 60) + 1, Math.ceil(event.end / 60)),
          label: customer.name || `Customer ${event.index}`,
          meta: `${formatClock(event.start)}–${formatClock(event.end)} · ${formatKm(event.chargeOnArrivalM)} left${event.lateS > 0 ? ` · ${formatDuration(event.lateS)} late` : ''}`,
          tone: event.lateS > 0 ? 'rose' : 'emerald',
        });
      } else if (event.kind === 'charge') {
        const charger = chargers[event.index] || {};
        items.push({
          id: `${vehicle.id}-charge-${eventIndex}`,
          startMinute: Math.floor(event.start / 60),
          endMinute: Math.max(Math.floor(event.start / 60) + 1, Math.ceil(event.end / 60)),
          label: `Charge · ${charger.name || `Charger ${event.index}`}`,
          meta: `Arrive with ${formatKm(event.chargeOnArrivalM)}, leave full`,
          tone: event.chargeOnArrivalM < 0 ? 'red' : 'amber',
        });
      }
    });
    const charges = items.filter((item) => item.tone === 'amber' || item.tone === 'red').length;
    return {
      id: `lane-${vehicle.id}`,
      label: vehicle.name,
      mode: 'detailed',
      badges: [`${(vehicle.visitOrder || []).length} stops`, `${charges} charge${charges === 1 ? '' : 's'}`],
      items,
    };
  });
  return { axis: buildAxis(plan), lanes };
}

function buildAxis(plan) {
  let start = 7 * 60;
  let end = 19 * 60;
  (plan?.vehicles || []).forEach((vehicle) => {
    start = Math.min(start, Math.floor((vehicle.departureTime || 0) / 60));
    (vehicle.schedule || []).forEach((event) => {
      end = Math.max(end, Math.ceil(event.end / 60) + 15);
    });
  });
  start = Math.max(0, start - (start % 60));
  end = Math.min(24 * 60, end + ((60 - (end % 60)) % 60));
  const ticks = [];
  for (let minute = start; minute <= end; minute += 60) ticks.push({ minute, label: formatClock(minute * 60) });
  return {
    startMinute: start,
    endMinute: end,
    days: [{ dayIndex: 0, label: 'Delivery day' }],
    ticks,
    initialViewport: { startMinute: start, endMinute: end },
  };
}

// Map layers for one plan and (optionally) its snapshot route geometry.
export function buildMapLayers(plan, routes, focusedIndex = null) {
  const owners = customerOwners(plan);
  const used = usedChargers(plan);
  const hubs = new Map();
  (plan?.vehicles || []).forEach((vehicle) => hubs.set(`${vehicle.homeLat},${vehicle.homeLng}`, vehicle));
  const vehicleIndex = new Map((plan?.vehicles || []).map((vehicle, index) => [vehicle.id, index]));
  const lines = [];
  (routes?.vehicles || []).forEach((route) => {
    const index = vehicleIndex.get(route.vehicleId);
    if (index == null) return;
    const focused = focusedIndex == null || focusedIndex === index;
    (route.legs || []).forEach((leg) => {
      lines.push({
        encoded: leg.encodedPolyline,
        color: colorForVehicle(index),
        weight: focused ? (focusedIndex == null ? 3 : 5) : 2,
        opacity: focused ? 0.85 : 0.18,
        dashArray: leg.chargingDetour ? '6 6' : undefined,
      });
    });
  });
  return {
    hubs: [...hubs.values()].map((vehicle) => ({ lat: vehicle.homeLat, lng: vehicle.homeLng })),
    customers: (plan?.customers || []).map((customer, index) => ({
      lat: customer.lat,
      lng: customer.lng,
      color: owners[index] == null ? UNUSED_COLOR : colorForVehicle(owners[index]),
      assigned: owners[index] != null,
    })),
    chargers: (plan?.chargers || []).map((charger, index) => ({
      lat: charger.lat,
      lng: charger.lng,
      used: used.has(index),
    })),
    lines,
  };
}
