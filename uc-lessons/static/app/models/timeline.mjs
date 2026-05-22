/* timeline.mjs — Timeline model building for solverforge-lessons */

import {
  DAY_MAP,
  WEEKDAYS,
  parseTimeToMinutes,
  formatClock,
  safeId,
  assignedFact,
  factLabel,
  formatTimeslot,
} from './formatters.mjs';

// Construire l'axe à partir des timeslots
export function buildAxisFromTimeslots(timeslots) {
  if (!timeslots || !timeslots.length) {
    return {
      startMinute: 0,
      endMinute: 10 * 60,
      days: [{ id: 'day-0', label: 'Monday', subLabel: '08:00-18:00', startMinute: 0, endMinute: 1440, isWeekend: false }],
      ticks: [],
      initialViewport: { startMinute: 0, endMinute: 600 },
    };
  }

  var presentDays = [];
  timeslots.forEach(function (ts) {
    var day = ts.day_of_week;
    if (day && presentDays.indexOf(day) === -1) {
      presentDays.push(day);
    }
  });
  presentDays.sort(function (a, b) { return DAY_MAP[a] - DAY_MAP[b]; });

  var days = [];
  var ticks = [];
  var maxEndMinute = 0;
  var minStartInDay = 1440;
  var maxEndInDay = 0;

  timeslots.forEach(function (ts) {
    minStartInDay = Math.min(minStartInDay, parseTimeToMinutes(ts.start_time));
    maxEndInDay = Math.max(maxEndInDay, parseTimeToMinutes(ts.end_time));
  });
  if (minStartInDay >= maxEndInDay) {
    minStartInDay = 8 * 60;
    maxEndInDay = 18 * 60;
  }

  presentDays.forEach(function (day) {
    var dayIndex = DAY_MAP[day];
    var dayStart = dayIndex * 1440 + minStartInDay;
    var dayEnd = dayIndex * 1440 + maxEndInDay;
    days.push({
      id: 'day-' + day,
      label: WEEKDAYS[dayIndex],
      subLabel: formatClock(minStartInDay) + '-' + formatClock(maxEndInDay),
      startMinute: dayStart,
      endMinute: dayEnd,
      isWeekend: day === 'Sat' || day === 'Sun',
    });
  });

  presentDays.forEach(function (day) {
    var dayIndex = DAY_MAP[day];
    for (var h = Math.floor(minStartInDay / 60); h <= Math.ceil(maxEndInDay / 60); h += 2) {
      ticks.push({
        id: 'tick-' + day + '-h' + h,
        minute: dayIndex * 1440 + h * 60,
        label: h + 'h',
      });
    }
  });

  timeslots.forEach(function (ts) {
    var end = DAY_MAP[ts.day_of_week] * 1440 + parseTimeToMinutes(ts.end_time);
    maxEndMinute = Math.max(maxEndMinute, end);
  });

  if (presentDays.length === 0) {
    for (var d = 0; d < 5; d++) {
      days.push({
        id: 'day-' + d,
        label: WEEKDAYS[d],
        startMinute: d * 1440,
        endMinute: (d + 1) * 1440,
        isWeekend: false,
      });
      for (var h = 8; h <= 18; h += 2) {
        ticks.push({
          id: 'tick-day' + d + '-h' + h,
          minute: d * 1440 + h * 60,
          label: h + 'h',
        });
      }
    }
    maxEndMinute = 5 * 1440;
  }

  return {
    startMinute: days.length ? days[0].startMinute : 0,
    endMinute: maxEndMinute,
    days: days,
    ticks: ticks,
    initialViewport: {
      startMinute: days.length ? days[0].startMinute : 0,
      endMinute: days.length ? Math.min(maxEndMinute, days[0].endMinute) : Math.min(maxEndMinute, 10 * 60),
    },
  };
}

// Build a timeline item for a lesson
export function buildLessonTimelineItem(prefix, lesson, lessonIndex, lookups, toneForKey, entityLabel) {
  var timeslot = assignedFact(lookups.timeslots, lesson.timeslot_idx);
  if (!timeslot) return null;
  var room = assignedFact(lookups.rooms, lesson.room_idx);
  var teacher = assignedFact(lookups.teachers, lesson.teacher_idx);
  var group = assignedFact(lookups.groups, lesson.group_idx);
  var tsMinutes = { startMinute: 0, endMinute: 60 };
  if (timeslot) {
    var dayIndex = DAY_MAP[timeslot.day_of_week] || 0;
    var startMin = parseTimeToMinutes(timeslot.start_time);
    var endMin = parseTimeToMinutes(timeslot.end_time);
    if (endMin <= startMin) endMin = startMin + 60;
    tsMinutes = {
      startMinute: dayIndex * 1440 + startMin,
      endMinute: dayIndex * 1440 + endMin,
    };
  }
  var subject = lesson.subject || entityLabel(lesson, lessonIndex);
  return {
    id: prefix + '-' + safeId(lesson.id || lessonIndex),
    startMinute: tsMinutes.startMinute,
    endMinute: tsMinutes.endMinute,
    label: subject,
    meta: [
      { label: 'Room', value: factLabel(room, 'Unassigned room') },
      { label: 'Teacher', value: factLabel(teacher, 'Unassigned teacher') },
      { label: 'Cohort', value: factLabel(group, 'Unassigned cohort') },
      { label: 'Period', value: formatTimeslot(timeslot) },
      { label: 'Students', value: String(lesson.student_count || '') },
    ],
    tone: toneForKey(subject),
  };
}
