// Pure view transforms for the order-picking workspace.
//
// These functions turn a SolverForge plan payload into the shapes the
// warehouse canvas, the statistics sidebar, and the picking-plan cards render.
// They are DOM-free so they can be unit tested directly.

// Trolley colours match the source application's palette so a trolley keeps the
// same colour in the canvas, the legend, and its card.
const TROLLEY_COLORS = [
  '#ef4444',
  '#3b82f6',
  '#10b981',
  '#f59e0b',
  '#8b5cf6',
  '#06b6d4',
  '#ec4899',
  '#84cc16',
];

export function trolleyColor(trolleyId) {
  const index = (Number.parseInt(trolleyId, 10) - 1) % TROLLEY_COLORS.length;
  return TROLLEY_COLORS[Number.isFinite(index) && index >= 0 ? index : 0];
}

export function clonePlan(plan) {
  return JSON.parse(JSON.stringify(plan));
}

/**
 * Builds the canvas, sidebar, and card view model from a SolverForge plan.
 *
 * `trolley.stepOrder` holds indexes into `pickSteps`, so resolving a route means
 * substituting those indexes with the immutable step records they point at.
 */
export function buildViewModel(plan) {
  const steps = plan.pickSteps || [];
  const trolleys = plan.trolleys || [];
  const assigned = new Set();

  const routes = trolleys.map((trolley) => {
    const resolved = (trolley.stepOrder || [])
      .map((index) => steps[index])
      .filter((step) => step && step.id);
    resolved.forEach((step) => assigned.add(step.id));

    const orderVolumes = new Map();
    let volumeCm3 = 0;
    for (const step of resolved) {
      orderVolumes.set(step.orderId, (orderVolumes.get(step.orderId) || 0) + step.volumeCm3);
      volumeCm3 += step.volumeCm3;
    }

    const requiredBuckets = [...orderVolumes.values()].reduce(
      (sum, volume) => sum + Math.ceil(volume / trolley.bucketCapacity),
      0,
    );
    const maxCapacity = trolley.bucketCapacity * trolley.bucketCount;
    const capacityPercent = maxCapacity > 0 ? Math.min(100, Math.round((volumeCm3 / maxCapacity) * 100)) : 0;

    return {
      id: trolley.id,
      color: trolleyColor(trolley.id),
      location: trolley.location,
      bucketCount: trolley.bucketCount,
      bucketCapacity: trolley.bucketCapacity,
      steps: resolved,
      itemCount: resolved.length,
      volumeCm3,
      capacityPercent,
      capacityClass: capacityPercent > 90 ? 'high' : capacityPercent > 70 ? 'medium' : 'low',
      requiredBuckets,
      excessBuckets: Math.max(0, requiredBuckets - trolley.bucketCount),
      orderCount: orderVolumes.size,
      distanceMeters: trolley.routeDistanceMeters ?? closedRouteDistance(trolley.location, resolved),
    };
  });

  const unassigned = steps.filter((step) => !assigned.has(step.id));

  return {
    orders: new Set(steps.map((step) => step.orderId)).size,
    items: steps.length,
    activeTrolleys: routes.filter((route) => route.steps.length > 0).length,
    totalMeters: routes.reduce((sum, route) => sum + route.distanceMeters, 0),
    trolleys: routes,
    unassigned,
  };
}

/**
 * Distance in meters between two warehouse locations.
 *
 * This is the browser twin of `calculate_distance` in `src/domain/warehouse.rs`
 * and covers the source's piecewise branches: same shelf/side, opposite sides,
 * contiguous aisle faces, other shelves on one warehouse row, and shelves on
 * different rows.
 */
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

export function formatLocation(location) {
  return `${location.shelvingId} · ${location.side} · row ${location.row}`;
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

function shelfOrigin(id) {
  const match = /^\(([A-E]),([1-3])\)$/.exec(id);
  if (!match) throw new Error(`Unknown shelving ${id}`);
  return { x: (match[1].charCodeAt(0) - 65) * 5, y: (Number(match[2]) - 1) * 13 };
}

function crossAisle(startRow, endRow) {
  return Math.min(startRow + endRow, (10 - startRow) + (10 - endRow));
}
