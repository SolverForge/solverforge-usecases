const TONES = ['emerald', 'blue', 'amber', 'rose', 'violet'];
const STOP_MINUTES = 10;

export function clonePlan(plan) {
  return JSON.parse(JSON.stringify(plan));
}

export function warehouseDistance(start, end) {
  const a = shelfOrigin(start.shelvingId);
  const b = shelfOrigin(end.shelvingId);
  const ax = a.x + (start.side === 'RIGHT' ? 2 : 0);
  const bx = b.x + (end.side === 'RIGHT' ? 2 : 0);
  const ay = a.y + start.row;
  const by = b.y + end.row;
  if (start.shelvingId === end.shelvingId) {
    return start.side === end.side ? Math.abs(ay - by) : 2 + crossAisle(start.row, end.row);
  }
  if (a.y === b.y) {
    const dx = Math.abs(ax - bx);
    return dx + (dx === 3 ? Math.abs(ay - by) : crossAisle(start.row, end.row));
  }
  return Math.abs(ax - bx) + Math.abs(ay - by);
}

export function buildPlanModel(plan) {
  const steps = plan.pickSteps || [];
  const trolleys = plan.trolleys || [];
  const assigned = new Set();
  const routes = trolleys.map((trolley, trolleyIndex) => {
    const routeSteps = (trolley.stepOrder || []).map((index) => {
      assigned.add(index);
      return { ...steps[index], index };
    }).filter((step) => step.id);
    const orderVolumes = new Map();
    routeSteps.forEach((step) => orderVolumes.set(step.orderId, (orderVolumes.get(step.orderId) || 0) + step.volumeCm3));
    const requiredBuckets = [...orderVolumes.values()].reduce((sum, volume) => sum + Math.ceil(volume / trolley.bucketCapacity), 0);
    return {
      ...trolley,
      trolleyIndex,
      routeSteps,
      requiredBuckets,
      excessBuckets: Math.max(0, requiredBuckets - trolley.bucketCount),
      orderCount: orderVolumes.size,
      distanceMeters: closedRouteDistance(trolley.location, routeSteps),
    };
  });
  const unassigned = steps.map((step, index) => ({ ...step, index })).filter((step) => !assigned.has(step.index));
  const orders = groupRows(steps, 'orderId').map(([id, rows]) => ({
    id,
    itemCount: rows.length,
    volumeCm3: rows.reduce((sum, row) => sum + row.volumeCm3, 0),
    trolleyCount: routes.filter((route) => route.routeSteps.some((step) => step.orderId === id)).length,
  }));
  const products = groupRows(steps, 'productId').map(([id, rows]) => ({
    id,
    name: rows[0].name,
    volumeCm3: rows[0].volumeCm3,
    location: formatLocation(rows[0].location),
    picks: rows.length,
  }));
  return {
    routes,
    unassigned,
    orders,
    products,
    kpis: {
      orders: orders.length,
      items: steps.length,
      activeTrolleys: routes.filter((route) => route.routeSteps.length).length,
      routeMeters: routes.reduce((sum, route) => sum + route.distanceMeters, 0),
    },
  };
}

export function buildRouteTimeline(model) {
  const lanes = model.routes.map((route, index) => ({
    id: `trolley-${route.id}`,
    label: `Trolley ${route.id}`,
    mode: 'detailed',
    badges: route.excessBuckets ? [`${route.excessBuckets} excess buckets`] : [],
    stats: [{ label: 'Stops', value: route.routeSteps.length }, { label: 'Meters', value: route.distanceMeters }],
    items: route.routeSteps.map((step, position) => timelineItem(step, position, TONES[index % TONES.length])),
  }));
  if (model.unassigned.length) {
    lanes.push({
      id: 'unassigned',
      label: 'Unassigned work',
      mode: 'detailed',
      badges: [`${model.unassigned.length} items`],
      stats: [{ label: 'Needs route', value: model.unassigned.length }],
      items: model.unassigned.map((step, position) => timelineItem(step, position, 'slate')),
    });
  }
  const stopCount = Math.max(1, ...lanes.map((lane) => lane.items.length));
  return {
    title: 'Ordered picking routes',
    subtitle: 'Each block is one immutable order item; sequence runs left to right from the depot.',
    label: 'Trolley',
    labelWidth: 250,
    zoomPresets: [],
    model: { axis: buildAxis(stopCount), lanes },
  };
}

export function formatLocation(location) {
  return `${location.shelvingId} · ${location.side} · row ${location.row}`;
}

function timelineItem(step, position, tone) {
  return {
    id: step.id,
    startMinute: position * STOP_MINUTES,
    endMinute: (position + 1) * STOP_MINUTES,
    label: step.name,
    meta: `Order ${step.orderId} · ${formatLocation(step.location)}`,
    tone,
  };
}

function buildAxis(stopCount) {
  const endMinute = stopCount * STOP_MINUTES;
  return {
    startMinute: 0,
    endMinute,
    days: [{ id: 'route', label: 'Pick sequence', subLabel: `${stopCount} positions`, startMinute: 0, endMinute }],
    ticks: Array.from({ length: stopCount }, (_, index) => ({ id: `stop-${index}`, minute: index * STOP_MINUTES, label: String(index + 1) })),
    initialViewport: { startMinute: 0, endMinute: Math.min(endMinute, 400) },
  };
}

function closedRouteDistance(depot, steps) {
  if (!steps.length) return 0;
  let distance = 0;
  let previous = depot;
  steps.forEach((step) => {
    distance += warehouseDistance(previous, step.location);
    previous = step.location;
  });
  return distance + warehouseDistance(previous, depot);
}

function groupRows(rows, key) {
  const groups = new Map();
  rows.forEach((row) => groups.set(row[key], [...(groups.get(row[key]) || []), row]));
  return [...groups.entries()].sort((a, b) => Number(a[0]) - Number(b[0]));
}

function shelfOrigin(id) {
  const match = /^\(([A-E]),([1-3])\)$/.exec(id);
  if (!match) throw new Error(`Unknown shelving ${id}`);
  return { x: (match[1].charCodeAt(0) - 65) * 5, y: (Number(match[2]) - 1) * 13 };
}

function crossAisle(startRow, endRow) {
  return Math.min(startRow + endRow, (10 - startRow) + (10 - endRow));
}
