export function assignmentMap(assignments) {
  var byId = {};
  (assignments || []).forEach(function (assignment) {
    byId[String(assignment.workOrderId)] = assignment;
  });
  return byId;
}

export function operatorShifts(assignments, operatorId) {
  return (assignments || [])
    .filter(function (assignment) {
      return assignment.operatorId === operatorId && assignment.shiftLabel;
    })
    .sort(function (left, right) {
      if (left.rosterDay !== right.rosterDay) return left.rosterDay - right.rosterDay;
      return String(left.shiftLabel).localeCompare(String(right.shiftLabel));
    })
    .map(function (assignment) { return assignment.shiftLabel; });
}

export function averageAssignmentLoad(assignments) {
  if (!assignments.length) return '0';
  var total = assignments.reduce(function (sum, assignment) {
    return sum + assignment.loadWeightKg;
  }, 0);
  return String(Math.round(total / assignments.length));
}

export function toneForPriority(priority) {
  if (priority === 'Express') return 'rose';
  if (priority === 'Urgent') return 'amber';
  return 'emerald';
}

export function clonePlan(data) {
  return JSON.parse(JSON.stringify(data));
}

export function valueAsString(value) {
  if (value == null) return '—';
  if (Array.isArray(value)) {
    return value.map(function (entry) { return valueAsString(entry); }).join(', ');
  }
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}

export function pad2(value) {
  return value < 10 ? '0' + String(value) : String(value);
}
