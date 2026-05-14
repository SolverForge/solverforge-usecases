export function formatClock(seconds) {
  const safe = Math.max(0, Math.floor(seconds || 0));
  const hours = Math.floor(safe / 3600) % 24;
  const minutes = Math.floor((safe % 3600) / 60);
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
}

export function formatDuration(seconds) {
  const safe = Math.max(0, Math.floor(seconds || 0));
  if (safe >= 3600) {
    return `${(safe / 3600).toFixed(1)}h`;
  }
  return `${Math.round(safe / 60)}m`;
}

export function kindLabel(kind) {
  switch (kind) {
    case 'business':
      return 'Business';
    case 'residential':
      return 'Residential';
    case 'restaurant':
      return 'Restaurant';
    default:
      return 'Other';
  }
}

export function iconForKind(kind) {
  switch (kind) {
    case 'business':
      return 'fa-building';
    case 'residential':
      return 'fa-house';
    case 'restaurant':
      return 'fa-utensils';
    default:
      return 'fa-box';
  }
}

export function toneForKind(kind) {
  switch (kind) {
    case 'business':
      return 'blue';
    case 'residential':
      return 'emerald';
    case 'restaurant':
      return 'amber';
    default:
      return 'slate';
  }
}
