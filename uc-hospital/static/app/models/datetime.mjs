export const MINUTE_MS = 60 * 1000;
export const DAY_MS = 24 * 60 * MINUTE_MS;

const WEEKDAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

// Short human-readable label used inside schedule blocks.
export function compactDateTime(value) {
  if (value == null) return '';
  return String(value)
    .replace('T', ' ')
    .replace(/(\d{2}:\d{2}):\d{2}$/, '$1');
}

// Parses backend `NaiveDateTime` strings without letting the browser inject the local timezone.
export function parseDateTimeMs(value) {
  if (value == null || value === '') return null;
  if (typeof value === 'number' && Number.isFinite(value)) return value;

  const normalized = String(value).trim().replace(' ', 'T');
  const match = normalized.match(/^(\d{4})-(\d{2})-(\d{2})(?:T(\d{2}):(\d{2})(?::(\d{2}))?)?/);
  if (match) {
    // Preserve backend NaiveDateTime wall-time components without applying the browser timezone.
    return Date.UTC(
      Number(match[1]),
      Number(match[2]) - 1,
      Number(match[3]),
      Number(match[4] || 0),
      Number(match[5] || 0),
      Number(match[6] || 0),
    );
  }

  const parsed = Date.parse(normalized);
  return Number.isFinite(parsed) ? parsed : null;
}

// Guarantees a usable time span for timeline rendering even when data is partial.
export function normalizeShiftBounds(startMs, endMs) {
  if (startMs == null && endMs == null) {
    return { startMs: null, endMs: null };
  }
  let resolvedStart = startMs;
  let resolvedEnd = endMs;
  if (resolvedStart == null) resolvedStart = resolvedEnd;
  if (resolvedEnd == null || resolvedEnd <= resolvedStart) resolvedEnd = resolvedStart + MINUTE_MS;
  return { startMs: resolvedStart, endMs: resolvedEnd };
}

// Returns the UTC midnight used as the start of the visible wall-time day.
export function wallTimeDayStartMs(ms) {
  const date = new Date(ms);
  return Date.UTC(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate());
}

// Formats the day header shown above the timeline.
export function formatAxisDayLabel(ms) {
  const date = new Date(ms);
  return `${WEEKDAY_NAMES[date.getUTCDay()]} ${date.getUTCDate()} ${MONTH_NAMES[date.getUTCMonth()]}`;
}
