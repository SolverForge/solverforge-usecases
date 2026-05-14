/* views.js — Custom timeline views for Group, Room, and Teacher */

// biome-ignore-all lint: don't lint

var SLOT_MINUTES = 60;

// Mapping jours de la semaine
var DAY_MAP = {
  Mon: 0, Monday: 0,
  Tue: 1, Tuesday: 1,
  Wed: 2, Wednesday: 2,
  Thu: 3, Thursday: 3,
  Fri: 4, Friday: 4,
  Sat: 5, Saturday: 5,
  Sun: 6, Sunday: 6,
};
var WEEKDAYS = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];
var WEEKDAY_SHORT = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

// Parse une heure au format "HH:MM:SS" ou "HH:MM" en minutes depuis minuit
function parseTimeToMinutes(timeStr) {
  if (!timeStr) return 0;
  var parts = timeStr.split(':');
  var hours = parseInt(parts[0], 10) || 0;
  var minutes = parseInt(parts[1], 10) || 0;
  // Clamp to valid range (0-1439 for a single day)
  return Math.max(0, Math.min(hours * 60 + minutes, 1439));
}

// Convertit un timeslot en minutes absolues (depuis Lundi 00:00)
function timeslotToMinutes(timeslot) {
  if (!timeslot) return { startMinute: 0, endMinute: SLOT_MINUTES };
  var dayIndex = DAY_MAP[timeslot.day_of_week];
  if (dayIndex == null) dayIndex = 0;
  var startMin = parseTimeToMinutes(timeslot.start_time);
  var endMin = parseTimeToMinutes(timeslot.end_time);

  // Garantir que endMinute > startMinute
  if (endMin <= startMin) {
    endMin = startMin + SLOT_MINUTES; // Durée par défaut de 60 minutes
  }

  return {
    startMinute: dayIndex * 1440 + startMin,
    endMinute: dayIndex * 1440 + endMin,
  };
}

function formatClock(totalMinutes) {
  var minutesInDay = ((totalMinutes % 1440) + 1440) % 1440;
  var hours = Math.floor(minutesInDay / 60);
  var minutes = minutesInDay % 60;
  return String(hours).padStart(2, '0') + ':' + String(minutes).padStart(2, '0');
}

function weekdayIndex(day) {
  var index = DAY_MAP[day];
  return index == null ? 0 : index;
}

function weekdayShortLabel(day) {
  return WEEKDAY_SHORT[weekdayIndex(day)] || String(day || 'Day');
}

function safeId(value) {
  return String(value == null ? 'unknown' : value).replace(/[^A-Za-z0-9_-]+/g, '-');
}

function isAssignedIndex(index, collection) {
  return Number.isInteger(index) && index >= 0 && index < collection.length;
}

function assignedFact(collection, index) {
  return isAssignedIndex(index, collection) ? collection[index] : null;
}

function factLabel(fact, fallback) {
  if (!fact) return fallback;
  return fact.name || fact.id || fact.code || fallback;
}

function scheduledBadges(totalCount, scheduledCount) {
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

function countScheduled(lessons, timeslots) {
  return lessons.reduce(function (count, lesson) {
    return assignedFact(timeslots, lesson.timeslot_idx) ? count + 1 : count;
  }, 0);
}

function teachingWindowLabel(timeslots) {
  if (!timeslots || !timeslots.length) return 'No timetable';
  var minStart = 1440;
  var maxEnd = 0;
  timeslots.forEach(function (timeslot) {
    minStart = Math.min(minStart, parseTimeToMinutes(timeslot.start_time));
    maxEnd = Math.max(maxEnd, parseTimeToMinutes(timeslot.end_time));
  });
  return 'Mon-Fri ' + formatClock(minStart) + '-' + formatClock(maxEnd);
}

function formatTimeslot(timeslot) {
  if (!timeslot) return 'Timeslot unassigned';
  return weekdayShortLabel(timeslot.day_of_week) + ' ' +
    formatClock(parseTimeToMinutes(timeslot.start_time)) + '-' +
    formatClock(parseTimeToMinutes(timeslot.end_time));
}

function buildLessonTimelineItem(prefix, lesson, lessonIndex, lookups, toneForKey, entityLabel) {
  var timeslot = assignedFact(lookups.timeslots, lesson.timeslot_idx);
  if (!timeslot) return null;
  var room = assignedFact(lookups.rooms, lesson.room_idx);
  var teacher = assignedFact(lookups.teachers, lesson.teacher_idx);
  var group = assignedFact(lookups.groups, lesson.group_idx);
  var tsMinutes = timeslotToMinutes(timeslot);
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

// Construire l'axe à partir des timeslots
function buildAxisFromTimeslots(timeslots) {
  if (!timeslots || !timeslots.length) {
    // Fallback : un seul jour de 8h à 18h
    return {
      startMinute: 0,
      endMinute: 10 * 60, // 10 slots de 60 min
      days: [{ id: 'day-0', label: 'Monday', subLabel: '08:00-18:00', startMinute: 0, endMinute: 1440, isWeekend: false }],
      ticks: [],
      initialViewport: { startMinute: 0, endMinute: 600 },
    };
  }

  // Déterminer quels jours sont présents
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

  // Créer les blocs "days" (un par jour)
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

  // Créer les ticks horaires à partir de la fenêtre d'enseignement réelle.
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

  // Calculer la fin maximale à partir des timeslots
  timeslots.forEach(function (ts) {
    var end = DAY_MAP[ts.day_of_week] * 1440 + parseTimeToMinutes(ts.end_time);
    maxEndMinute = Math.max(maxEndMinute, end);
  });

  // Si aucun timeslot n'a de jour valide, utiliser 5 jours par défaut
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

function ensureCustomTimeline(key, customTimelines, SF, timelineConfig) {
  var timeline = customTimelines[key];
  if (!timeline) {
    timeline = SF.rail.createTimeline(timelineConfig);
    customTimelines[key] = timeline;
    return timeline;
  }
  timeline.setModel(timelineConfig.model);
  return timeline;
}

export function renderByGroup(data, container, SF, toneForKey, entityLabel, customTimelines) {
  var lessons = data.lessons || [];
  var groups = data.groups || [];
  var timeslots = data.timeslots || [];
  var rooms = data.rooms || [];
  var teachers = data.teachers || [];

  console.log('renderByGroup: groups=' + groups.length + ', lessons=' + lessons.length);

  if (!lessons.length) {
    container.innerHTML = '<p>No lessons available.</p>';
    return;
  }

  // Créer une lane par groupe existant, même sans lessons
  var byGroup = {};
  groups.forEach(function(group, idx) {
    var groupKey = group.name || 'Group ' + idx;
    byGroup[groupKey] = { group: group, lessons: [] };
  });

  // Assigner les lessons aux groupes
  lessons.forEach(function (lesson) {
    var groupIdx = lesson.group_idx;
    if (groupIdx == null || !groups[groupIdx]) {
      // Lesson sans groupe : créer une lane "Unassigned"
      var unassignedKey = 'Unassigned';
      if (!byGroup[unassignedKey]) {
        byGroup[unassignedKey] = { group: { name: unassignedKey }, lessons: [] };
      }
      byGroup[unassignedKey].lessons.push(lesson);
      return;
    }
    var group = groups[groupIdx];
    var groupKey = group.name || 'Group ' + groupIdx;
    if (!byGroup[groupKey]) {
      byGroup[groupKey] = { group: group, lessons: [] };
    }
    byGroup[groupKey].lessons.push(lesson);
  });

  var axis = buildAxisFromTimeslots(timeslots);

  var lanes = Object.entries(byGroup).map(function (entry) {
    var groupKey = entry[0];
    var groupData = entry[1];
    var scheduledCount = countScheduled(groupData.lessons, timeslots);
    var items = groupData.lessons.map(function (lesson, idx) {
      return buildLessonTimelineItem(
        'group-' + safeId(groupKey),
        lesson,
        idx,
        { timeslots: timeslots, rooms: rooms, teachers: teachers, groups: groups },
        toneForKey,
        entityLabel
      );
    }).filter(Boolean);
    return {
      id: 'group-' + safeId(groupKey),
      label: groupKey + (groupData.group.code ? ' (' + groupData.group.code + ')' : ''),
      mode: 'detailed',
      badges: scheduledBadges(groupData.lessons.length, scheduledCount),
      stats: [],
      items: items,
    };
  });

  var timeline = ensureCustomTimeline('by-group', customTimelines, SF, {
    label: 'Groups',
    labelWidth: 240,
    title: 'Cohort Timetables',
    subtitle: 'Fixed Mon-Fri teaching week',
    zoomPresets: [],
    model: { axis: axis, lanes: lanes },
  });

  container.innerHTML = '';
  var realGroupCount = Object.keys(byGroup).filter(function(key) { return key !== 'Unassigned'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Cohorts', 'Lessons', 'Scheduled', 'Window'],
    rows: [[String(realGroupCount), String(lessons.length), String(countScheduled(lessons, timeslots)), teachingWindowLabel(timeslots)]],
  }));
  container.appendChild(timeline.el);
}

export function renderByRoom(data, container, SF, toneForKey, entityLabel, customTimelines) {
  var lessons = data.lessons || [];
  var rooms = data.rooms || [];
  var timeslots = data.timeslots || [];
  var groups = data.groups || [];
  var teachers = data.teachers || [];

  console.log('renderByRoom: rooms=' + rooms.length + ', lessons=' + lessons.length);

  if (!lessons.length) {
    container.innerHTML = '<p>No lessons available.</p>';
    return;
  }

  // Créer une lane par room existant, même sans lessons
  var byRoom = {};
  rooms.forEach(function(room, idx) {
    var roomKey = room.name || 'Room ' + idx;
    byRoom[roomKey] = { room: room, lessons: [] };
  });

  // Assigner les lessons aux rooms
  lessons.forEach(function (lesson) {
    var roomIdx = lesson.room_idx;
    if (!assignedFact(rooms, roomIdx)) {
      var unassignedKey = 'Unassigned room';
      if (!byRoom[unassignedKey]) {
        byRoom[unassignedKey] = { room: { name: unassignedKey }, lessons: [] };
      }
      byRoom[unassignedKey].lessons.push(lesson);
      return;
    }
    var room = rooms[roomIdx];
    var roomKey = room.name || 'Room ' + roomIdx;
    if (!byRoom[roomKey]) {
      byRoom[roomKey] = { room: room, lessons: [] };
    }
    byRoom[roomKey].lessons.push(lesson);
  });

  var axis = buildAxisFromTimeslots(timeslots);

  var lanes = Object.entries(byRoom).map(function (entry) {
    var roomKey = entry[0];
    var roomData = entry[1];
    var scheduledCount = countScheduled(roomData.lessons, timeslots);
    var items = roomData.lessons.map(function (lesson, idx) {
      return buildLessonTimelineItem(
        'room-' + safeId(roomKey),
        lesson,
        idx,
        { timeslots: timeslots, rooms: rooms, teachers: teachers, groups: groups },
        toneForKey,
        entityLabel
      );
    }).filter(Boolean);
    return {
      id: 'room-' + safeId(roomKey),
      label: roomKey + (roomData.room.code ? ' (' + roomData.room.code + ')' : ''),
      mode: 'detailed',
      badges: roomData.lessons.length === 0 ? ['Empty'] : scheduledBadges(roomData.lessons.length, scheduledCount),
      stats: [],
      items: items,
    };
  });

  var timeline = ensureCustomTimeline('by-room', customTimelines, SF, {
    label: 'Rooms',
    labelWidth: 240,
    title: 'Room Utilization',
    subtitle: 'Fixed Mon-Fri teaching week',
    zoomPresets: [],
    model: { axis: axis, lanes: lanes },
  });

  container.innerHTML = '';
  var realRoomCount = Object.keys(byRoom).filter(function(key) { return key !== 'Unassigned room'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Rooms', 'Lessons', 'Scheduled', 'Window'],
    rows: [[String(realRoomCount), String(lessons.length), String(countScheduled(lessons, timeslots)), teachingWindowLabel(timeslots)]],
  }));
  container.appendChild(timeline.el);
}

export function renderByTeacher(data, container, SF, toneForKey, entityLabel, customTimelines) {
  var lessons = data.lessons || [];
  var teachers = data.teachers || [];
  var timeslots = data.timeslots || [];
  var rooms = data.rooms || [];
  var groups = data.groups || [];

  console.log('renderByTeacher: teachers=' + teachers.length + ', lessons=' + lessons.length);

  if (!lessons.length) {
    container.innerHTML = '<p>No lessons available.</p>';
    return;
  }

  // Créer une lane par teacher existant, même sans lessons
  var byTeacher = {};
  teachers.forEach(function(teacher, idx) {
    var teacherKey = teacher.name || 'Teacher ' + idx;
    byTeacher[teacherKey] = { teacher: teacher, lessons: [] };
  });

  // Assigner les lessons aux teachers
  lessons.forEach(function (lesson) {
    var teacherIdx = lesson.teacher_idx;
    if (teacherIdx == null || !teachers[teacherIdx]) {
      // Lesson sans teacher : créer une lane "Unassigned"
      var unassignedKey = 'Unassigned';
      if (!byTeacher[unassignedKey]) {
        byTeacher[unassignedKey] = { teacher: { name: unassignedKey }, lessons: [] };
      }
      byTeacher[unassignedKey].lessons.push(lesson);
      return;
    }
    var teacher = teachers[teacherIdx];
    var teacherKey = teacher.name || 'Teacher ' + teacherIdx;
    if (!byTeacher[teacherKey]) {
      byTeacher[teacherKey] = { teacher: teacher, lessons: [] };
    }
    byTeacher[teacherKey].lessons.push(lesson);
  });

  var axis = buildAxisFromTimeslots(timeslots);

  var lanes = Object.entries(byTeacher).map(function (entry) {
    var teacherKey = entry[0];
    var teacherData = entry[1];
    var scheduledCount = countScheduled(teacherData.lessons, timeslots);
    var items = teacherData.lessons.map(function (lesson, idx) {
      return buildLessonTimelineItem(
        'teacher-' + safeId(teacherKey),
        lesson,
        idx,
        { timeslots: timeslots, rooms: rooms, teachers: teachers, groups: groups },
        toneForKey,
        entityLabel
      );
    }).filter(Boolean);
    return {
      id: 'teacher-' + safeId(teacherKey),
      label: teacherKey + (teacherData.teacher.code ? ' (' + teacherData.teacher.code + ')' : ''),
      mode: 'detailed',
      badges: teacherData.lessons.length === 0 ? ['Empty'] : scheduledBadges(teacherData.lessons.length, scheduledCount),
      stats: [],
      items: items,
    };
  });

  var timeline = ensureCustomTimeline('by-teacher', customTimelines, SF, {
    label: 'Teachers',
    labelWidth: 240,
    title: 'Teacher Loads',
    subtitle: 'Fixed Mon-Fri teaching week',
    zoomPresets: [],
    model: { axis: axis, lanes: lanes },
  });

  container.innerHTML = '';
  var realTeacherCount = Object.keys(byTeacher).filter(function(key) { return key !== 'Unassigned'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Teachers', 'Lessons', 'Scheduled', 'Window'],
    rows: [[String(realTeacherCount), String(lessons.length), String(countScheduled(lessons, timeslots)), teachingWindowLabel(timeslots)]],
  }));
  container.appendChild(timeline.el);
}
