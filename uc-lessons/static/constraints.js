/* constraints.js — readable labels and score levels for constraint ids */

// Each constraint is named after its module in src/constraints/, as
// `solverforge check` requires, and score analysis reports that id as `name`.
// The browser shows these labels instead of the raw ids.
var CONSTRAINTS = {
  assign_timeslot: { label: 'Assign Timeslot', level: 'medium' },
  assign_room: { label: 'Assign Room', level: 'medium' },
  teacher_availability: { label: 'Teacher Availability', level: 'hard' },
  group_availability: { label: 'Group Availability', level: 'hard' },
  room_kind: { label: 'Room Kind', level: 'soft' },
  room_capacity: { label: 'Room Capacity', level: 'hard' },
  no_group_conflict: { label: 'No Group Conflict', level: 'hard' },
  no_room_conflict: { label: 'No Room Conflict', level: 'hard' },
  no_teacher_conflict: { label: 'No Teacher Conflict', level: 'hard' },
  late_lesson: { label: 'Avoid Late Lessons', level: 'soft' },
  repeated_subject_day: { label: 'Avoid Repeated Subject Day', level: 'soft' },
};

function known(id) {
  return Object.prototype.hasOwnProperty.call(CONSTRAINTS, id) ? CONSTRAINTS[id] : null;
}

// Returns the label for a constraint id; unknown ids, such as a constraint
// added later with `solverforge generate constraint`, are title-cased.
export function constraintLabel(id) {
  var key = String(id || '');
  var entry = known(key);
  if (entry) return entry.label;
  return key.replace(/_/g, ' ').replace(/\b\w/g, function (match) { return match.toUpperCase(); });
}

// Returns the score level the status bar colors a constraint dot by.
export function constraintLevel(id) {
  var entry = known(String(id || ''));
  return entry ? entry.level : 'hard';
}

// Builds status-bar entries from the generated UI model's constraint ids.
export function buildStatusBarConstraints(constraints) {
  return (constraints || []).map(function (constraint) {
    var id = typeof constraint === 'string' ? constraint : constraint.name;
    return { name: constraintLabel(id), type: constraintLevel(id) };
  });
}

// Makes the stock status bar label the dots it repaints from each analysis
// payload, so hover titles read like the analysis table.
export function withConstraintLabels(statusBar) {
  var paint = statusBar && statusBar.colorDotsFromAnalysis;
  if (typeof paint !== 'function') return statusBar;
  statusBar.colorDotsFromAnalysis = function (constraints) {
    return paint.call(statusBar, (constraints || []).map(function (constraint) {
      return Object.assign({}, constraint, { name: constraintLabel(constraint && constraint.name) });
    }));
  };
  return statusBar;
}
