/* formatters.mjs — Time, date, and label formatting for solverforge-lessons */

var SLOT_MINUTES = 60;

// Mapping jours de la semaine
export var DAY_MAP = {
  Mon: 0, Monday: 0,
  Tue: 1, Tuesday: 1,
  Wed: 2, Wednesday: 2,
  Thu: 3, Thursday: 3,
  Fri: 4, Friday: 4,
  Sat: 5, Saturday: 5,
  Sun: 6, Sunday: 6,
};

export var WEEKDAYS = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];
export var WEEKDAY_SHORT = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

// Parse une heure au format "HH:MM:SS" ou "HH:MM" en minutes depuis minuit
export function parseTimeToMinutes(timeStr) {
  if (!timeStr) return 0;
  var parts = timeStr.split(':');
  var hours = parseInt(parts[0], 10) || 0;
  var minutes = parseInt(parts[1], 10) || 0;
  return Math.max(0, Math.min(hours * 60 + minutes, 1439));
}

// Convertit un timeslot en minutes absolues (depuis Lundi 00:00)
export function timeslotToMinutes(timeslot) {
  if (!timeslot) return { startMinute: 0, endMinute: SLOT_MINUTES };
  var dayIndex = DAY_MAP[timeslot.day_of_week];
  if (dayIndex == null) dayIndex = 0;
  var startMin = parseTimeToMinutes(timeslot.start_time);
  var endMin = parseTimeToMinutes(timeslot.end_time);

  if (endMin <= startMin) {
    endMin = startMin + SLOT_MINUTES;
  }

  return {
    startMinute: dayIndex * 1440 + startMin,
    endMinute: dayIndex * 1440 + endMin,
  };
}

// Format un temps en HH:MM
export function formatClock(totalMinutes) {
  var minutesInDay = ((totalMinutes % 1440) + 1440) % 1440;
  var hours = Math.floor(minutesInDay / 60);
  var minutes = minutesInDay % 60;
  return String(hours).padStart(2, '0') + ':' + String(minutes).padStart(2, '0');
}

export function weekdayIndex(day) {
  var index = DAY_MAP[day];
  return index == null ? 0 : index;
}

export function weekdayShortLabel(day) {
  return WEEKDAY_SHORT[weekdayIndex(day)] || String(day || 'Day');
}

export function safeId(value) {
  return String(value == null ? 'unknown' : value).replace(/[^A-Za-z0-9_-]+/g, '-');
}

export function isAssignedIndex(index, collection) {
  return Number.isInteger(index) && index >= 0 && index < collection.length;
}

export function assignedFact(collection, index) {
  return isAssignedIndex(index, collection) ? collection[index] : null;
}

export function factLabel(fact, fallback) {
  if (!fact) return fallback;
  return fact.name || fact.id || fact.code || fallback;
}

export function entityLabel(entity, fallback) {
  if (!entity) return String(fallback);
  return entity.name || entity.id || fallback;
}

export function scheduledBadges(totalCount, scheduledCount) {
  if (!totalCount) return ['No lessons'];
  var complete = scheduledCount === totalCount;
  return [{
    label: scheduledCount + '/' + totalCount + ' scheduled',
    style: complete ? {
      bg: '#ecfdf5',
      border: '1px solid #a7f3d0',
      color: '#047857',
    } : {
      bg: '#fffbeb',
      border: '1px solid #fde68a',
      color: '#92400e',
    },
  }];
}

export function countScheduled(lessons, timeslots) {
  return lessons.reduce(function (count, lesson) {
    return assignedFact(timeslots, lesson.timeslot_idx) ? count + 1 : count;
  }, 0);
}

export function teachingWindowLabel(timeslots) {
  if (!timeslots || !timeslots.length) return 'No timetable';
  var minStart = 1440;
  var maxEnd = 0;
  timeslots.forEach(function (timeslot) {
    minStart = Math.min(minStart, parseTimeToMinutes(timeslot.start_time));
    maxEnd = Math.max(maxEnd, parseTimeToMinutes(timeslot.end_time));
  });
  return 'Mon-Fri ' + formatClock(minStart) + '-' + formatClock(maxEnd);
}

export function formatTimeslot(timeslot) {
  if (!timeslot) return 'Timeslot unassigned';
  return weekdayShortLabel(timeslot.day_of_week) + ' ' +
    formatClock(parseTimeToMinutes(timeslot.start_time)) + '-' +
    formatClock(parseTimeToMinutes(timeslot.end_time));
}

export function title(text) {
  return String(text || '')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, function (match) { return match.toUpperCase(); });
}

export function toneForKey(key, tones) {
  var text = String(key || '');
  var hash = 0;
  var TIMELINE_TONES = tones || ['emerald', 'blue', 'amber', 'rose', 'violet', 'slate'];
  for (var index = 0; index < text.length; index += 1) {
    hash = ((hash * 31) + text.charCodeAt(index)) >>> 0;
  }
  return TIMELINE_TONES[hash % TIMELINE_TONES.length];
}
