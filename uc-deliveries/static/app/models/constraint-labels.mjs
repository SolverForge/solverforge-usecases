// Readable labels for the constraint ids that score analysis reports.
//
// Each constraint is named after its module in `src/constraints/`, which is
// what `solverforge check` requires and what `/jobs/{id}/analysis` returns as
// `name`. The browser shows these labels instead of the raw ids.
const CONSTRAINT_LABELS = {
  all_deliveries_assigned: 'All Deliveries Assigned',
  vehicle_capacity: 'Vehicle Capacity',
  delivery_time_windows: 'Delivery Time Windows',
  total_travel_time: 'Total Travel Time',
};

// Returns the label for a constraint id; unknown ids, such as a constraint
// added later with `solverforge generate constraint`, are title-cased.
export function constraintLabel(id) {
  const key = String(id || '');
  if (Object.prototype.hasOwnProperty.call(CONSTRAINT_LABELS, key)) return CONSTRAINT_LABELS[key];
  return key.replace(/_/g, ' ').replace(/\b\w/g, (match) => match.toUpperCase());
}

// Copies an analysis or status-bar constraint entry with its readable label.
export function labelConstraint(constraint) {
  return { ...constraint, name: constraintLabel(constraint && constraint.name) };
}

// Makes the stock status bar label the constraint dots it repaints from each
// analysis payload, so hover titles read like the analysis table.
export function withConstraintLabels(statusBar) {
  const paint = statusBar && statusBar.colorDotsFromAnalysis;
  if (typeof paint !== 'function') return statusBar;
  statusBar.colorDotsFromAnalysis = (constraints) => paint.call(statusBar, (constraints || []).map(labelConstraint));
  return statusBar;
}
