// Builds a stable key that survives duplicate display names in the UI.
export function stableIdentityKey(kind, item, fallback) {
  if (item && item.id != null && String(item.id) !== '') {
    return `${kind}-id:${String(item.id)}`;
  }
  return `${kind}-index:${String(fallback)}`;
}

// Stable key for problem facts such as employees.
export function factKey(fact, fallback) {
  return stableIdentityKey('fact', fact, fallback);
}

// Stable key for planning entities such as shifts.
export function entityKey(entity, fallback) {
  return stableIdentityKey('entity', entity, fallback);
}

// Human-readable label shown to the user for an item.
export function displayLabel(item, fallback) {
  if (!item) return String(fallback);
  return item.name || item.id || fallback;
}

// Counts duplicate display labels so the UI can add badges when names collide.
export function countDisplayLabels(items, fallbackPrefix) {
  const counts = {};
  (items || []).forEach((item, index) => {
    const label = displayLabel(item, `${fallbackPrefix} ${index + 1}`);
    counts[label] = (counts[label] || 0) + 1;
  });
  return counts;
}
