/* views.js — Custom timeline views for Group, Room, and Teacher */

// biome-ignore-all lint: don't lint

var SLOT_MINUTES = 60;

// Mapping jours de la semaine
var DAY_MAP = { Mon: 0, Tue: 1, Wed: 2, Thu: 3, Fri: 4, Sat: 5, Sun: 6 };
var WEEKDAYS = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday'];

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
  var dayIndex = DAY_MAP[timeslot.day_of_week] || 0;
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

// Construire l'axe à partir des timeslots
function buildAxisFromTimeslots(timeslots) {
  if (!timeslots || !timeslots.length) {
    // Fallback : un seul jour de 8h à 18h
    return {
      startMinute: 0,
      endMinute: 10 * 60, // 10 slots de 60 min
      days: [{ id: 'day-0', label: 'Lundi', startMinute: 0, endMinute: 1440, isWeekend: false }],
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

  // Créer les blocs "days" (un par jour)
  presentDays.forEach(function (day) {
    var dayIndex = DAY_MAP[day];
    var dayStart = dayIndex * 1440;
    var dayEnd = dayStart + 1440;
    days.push({
      id: 'day-' + day,
      label: WEEKDAYS[dayIndex],
      startMinute: dayStart,
      endMinute: dayEnd,
      isWeekend: day === 'Sat' || day === 'Sun',
    });
  });

  // Créer les ticks horaires (8h-18h, toutes les 4h pour chaque jour)
  presentDays.forEach(function (day) {
    var dayIndex = DAY_MAP[day];
    for (var h = 8; h <= 18; h += 2) {
      ticks.push({
        id: 'tick-' + day + '-h' + h,
        minute: dayIndex * 1440 + h * 60,
        label: h + "h",
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
          label: h + "h",
        });
      }
    }
    maxEndMinute = 5 * 1440;
  }

  return {
    startMinute: 0,
    endMinute: maxEndMinute,
    days: days,
    ticks: ticks,
    initialViewport: {
      startMinute: 0,
      endMinute: Math.min(maxEndMinute, 5 * 1440),
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
    var items = groupData.lessons.map(function (lesson, idx) {
      var timeslotIdx = lesson.timeslot_idx;
      var timeslot = timeslots[timeslotIdx] || {};
      var roomIdx = lesson.room_idx;
      var room = rooms[roomIdx] || {};
      var tsMinutes = timeslotToMinutes(timeslot);
      return {
        id: 'group-' + SF.escHtml(groupKey) + '-lesson-' + idx,
        startMinute: tsMinutes.startMinute,
        endMinute: tsMinutes.endMinute,
        label: entityLabel(lesson, idx),
        meta: (room.name || 'Room ' + roomIdx) + ' | ' + (timeslot.name || timeslot.id || String(timeslotIdx)),
        tone: toneForKey(entityLabel(lesson, idx)),
      };
    });
    return {
      id: 'group-' + SF.escHtml(groupKey),
      label: groupKey + (groupData.group.code ? ' (' + groupData.group.code + ')' : ''),
      mode: 'detailed',
      badges: groupData.lessons.length === 0 ? ['No lessons'] : [],
      stats: [{ label: 'Lessons', value: groupData.lessons.length }],
      items: items,
    };
  });

  var timeline = ensureCustomTimeline('by-group', customTimelines, SF, {
    label: 'Groups',
    labelWidth: 280,
    title: 'Lessons by Group',
    subtitle: 'Group schedule view',
    model: { axis: axis, lanes: lanes },
  });

  container.innerHTML = '';
  var realGroupCount = Object.keys(byGroup).filter(function(key) { return key !== 'Unassigned'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Total Groups', 'Total Lessons'],
    rows: [[String(realGroupCount), String(lessons.length)]],
  }));
  container.appendChild(timeline.el);
}

export function renderByRoom(data, container, SF, toneForKey, entityLabel, customTimelines) {
  var lessons = data.lessons || [];
  var rooms = data.rooms || [];
  var timeslots = data.timeslots || [];
  var groups = data.groups || [];

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
    if (roomIdx == null || !rooms[roomIdx]) {
      // Lesson sans room : créer une lane "Unassigned"
      var unassignedKey = 'Unassigned';
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
    var items = roomData.lessons.map(function (lesson, idx) {
      var timeslotIdx = lesson.timeslot_idx;
      var timeslot = timeslots[timeslotIdx] || {};
      var groupIdx = lesson.group_idx;
      var group = groups[groupIdx] || {};
      var tsMinutes = timeslotToMinutes(timeslot);
      return {
        id: 'room-' + SF.escHtml(roomKey) + '-lesson-' + idx,
        startMinute: tsMinutes.startMinute,
        endMinute: tsMinutes.endMinute,
        label: entityLabel(lesson, idx),
        meta: (group.name || 'Group ' + groupIdx) + ' | ' + (timeslot.name || timeslot.id || String(timeslotIdx)),
        tone: toneForKey(entityLabel(lesson, idx)),
      };
    });
    return {
      id: 'room-' + SF.escHtml(roomKey),
      label: roomKey + (roomData.room.code ? ' (' + roomData.room.code + ')' : ''),
      mode: 'detailed',
      badges: roomData.lessons.length === 0 ? ['Empty'] : [],
      stats: [{ label: 'Lessons', value: roomData.lessons.length }],
      items: items,
    };
  });

  var timeline = ensureCustomTimeline('by-room', customTimelines, SF, {
    label: 'Rooms',
    labelWidth: 280,
    title: 'Lessons by Room',
    subtitle: 'Room schedule view',
    model: { axis: axis, lanes: lanes },
  });

  container.innerHTML = '';
  var realRoomCount = Object.keys(byRoom).filter(function(key) { return key !== 'Unassigned'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Total Rooms', 'Total Lessons'],
    rows: [[String(realRoomCount), String(lessons.length)]],
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
    var items = teacherData.lessons.map(function (lesson, idx) {
      var timeslotIdx = lesson.timeslot_idx;
      var timeslot = timeslots[timeslotIdx] || {};
      var roomIdx = lesson.room_idx;
      var room = rooms[roomIdx] || {};
      var groupIdx = lesson.group_idx;
      var group = groups[groupIdx] || {};
      var tsMinutes = timeslotToMinutes(timeslot);
      return {
        id: 'teacher-' + SF.escHtml(teacherKey) + '-lesson-' + idx,
        startMinute: tsMinutes.startMinute,
        endMinute: tsMinutes.endMinute,
        label: entityLabel(lesson, idx),
        meta: (group.name || 'Group ' + groupIdx) + ' | ' + (room.name || 'Room ' + roomIdx) + ' | ' + (timeslot.name || timeslot.id || String(timeslotIdx)),
        tone: toneForKey(entityLabel(lesson, idx)),
      };
    });
    return {
      id: 'teacher-' + SF.escHtml(teacherKey),
      label: teacherKey + (teacherData.teacher.code ? ' (' + teacherData.teacher.code + ')' : ''),
      mode: 'detailed',
      badges: teacherData.lessons.length === 0 ? ['Empty'] : [],
      stats: [{ label: 'Lessons', value: teacherData.lessons.length }],
      items: items,
    };
  });

  var timeline = ensureCustomTimeline('by-teacher', customTimelines, SF, {
    label: 'Teachers',
    labelWidth: 280,
    title: 'Lessons by Teacher',
    subtitle: 'Teacher schedule view',
    model: { axis: axis, lanes: lanes },
  });

  container.innerHTML = '';
  var realTeacherCount = Object.keys(byTeacher).filter(function(key) { return key !== 'Unassigned'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Total Teachers', 'Total Lessons'],
    rows: [[String(realTeacherCount), String(lessons.length)]],
  }));
  container.appendChild(timeline.el);
}
