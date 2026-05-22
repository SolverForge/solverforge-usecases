/* lessons.mjs — Group, Room, and Teacher timeline views for solverforge-lessons */

import {
  safeId,
  assignedFact,
  factLabel,
  countScheduled,
  teachingWindowLabel,
  buildAxisFromTimeslots,
} from '../models/index.mjs';

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

function buildLessonTimelineItem(prefix, lesson, lessonIndex, lookups, toneForKey, entityLabel, SF) {
  var timeslot = assignedFact(lookups.timeslots, lesson.timeslot_idx);
  if (!timeslot) return null;
  var room = assignedFact(lookups.rooms, lesson.room_idx);
  var teacher = assignedFact(lookups.teachers, lesson.teacher_idx);
  var group = assignedFact(lookups.groups, lesson.group_idx);
  var tsMinutes = { startMinute: 0, endMinute: 60 };
  var dayIndex = 0;
  var startMin = 0;
  var endMin = 60;

  if (timeslot) {
    var DAY_MAP = { Mon: 0, Monday: 0, Tue: 1, Tuesday: 1, Wed: 2, Wednesday: 2, Thu: 3, Thursday: 3, Fri: 4, Friday: 4, Sat: 5, Saturday: 5, Sun: 6, Sunday: 6 };
    dayIndex = DAY_MAP[timeslot.day_of_week] || 0;
    var parts = (timeslot.start_time || '').split(':');
    startMin = (parseInt(parts[0], 10) || 0) * 60 + (parseInt(parts[1], 10) || 0);
    parts = (timeslot.end_time || '').split(':');
    endMin = (parseInt(parts[0], 10) || 0) * 60 + (parseInt(parts[1], 10) || 0);
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
      { label: 'Period', value: timeslot ? (dayIndex + ' ' + startMin + '-' + endMin) : 'Unassigned' },
      { label: 'Students', value: String(lesson.student_count || '') },
    ],
    tone: toneForKey(subject),
  };
}

export function renderByGroup(data, container, SF, toneForKey, entityLabel, customTimelines) {
  var lessons = data.lessons || [];
  var groups = data.groups || [];
  var timeslots = data.timeslots || [];
  var rooms = data.rooms || [];
  var teachers = data.teachers || [];

  if (!lessons.length) {
    container.innerHTML = '<p>No lessons available.</p>';
    return;
  }

  var byGroup = {};
  groups.forEach(function (group, idx) {
    var groupKey = group.name || 'Group ' + idx;
    byGroup[groupKey] = { group: group, lessons: [] };
  });

  lessons.forEach(function (lesson) {
    var groupIdx = lesson.group_idx;
    if (groupIdx == null || !groups[groupIdx]) {
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
        entityLabel,
        SF
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
  var realGroupCount = Object.keys(byGroup).filter(function (key) { return key !== 'Unassigned'; }).length;
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

  if (!lessons.length) {
    container.innerHTML = '<p>No lessons available.</p>';
    return;
  }

  var byRoom = {};
  rooms.forEach(function (room, idx) {
    var roomKey = room.name || 'Room ' + idx;
    byRoom[roomKey] = { room: room, lessons: [] };
  });

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
        entityLabel,
        SF
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
  var realRoomCount = Object.keys(byRoom).filter(function (key) { return key !== 'Unassigned room'; }).length;
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

  if (!lessons.length) {
    container.innerHTML = '<p>No lessons available.</p>';
    return;
  }

  var byTeacher = {};
  teachers.forEach(function (teacher, idx) {
    var teacherKey = teacher.name || 'Teacher ' + idx;
    byTeacher[teacherKey] = { teacher: teacher, lessons: [] };
  });

  lessons.forEach(function (lesson) {
    var teacherIdx = lesson.teacher_idx;
    if (teacherIdx == null || !teachers[teacherIdx]) {
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
        entityLabel,
        SF
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
  var realTeacherCount = Object.keys(byTeacher).filter(function (key) { return key !== 'Unassigned'; }).length;
  container.appendChild(SF.createTable({
    columns: ['Teachers', 'Lessons', 'Scheduled', 'Window'],
    rows: [[String(realTeacherCount), String(lessons.length), String(countScheduled(lessons, timeslots)), teachingWindowLabel(timeslots)]],
  }));
  container.appendChild(timeline.el);
}
