/* status.mjs — Status bar constraints and score levels for solverforge-lessons */

import { title } from './formatters.mjs';

const SCORE_LEVELS = {
  assign_timeslot: 'medium',
  assign_room: 'medium',
  teacher_availability: 'hard',
  group_availability: 'hard',
  room_kind: 'soft',
  room_capacity: 'hard',
  no_group_conflict: 'hard',
  no_room_conflict: 'hard',
  no_teacher_conflict: 'hard',
  late_lesson: 'soft',
  repeated_subject_day: 'soft',
};

export function buildStatusBarConstraints(constraints) {
  return (constraints || []).map(function (constraint) {
    var name = typeof constraint === 'string' ? constraint : constraint.name;
    return {
      name: title((name || '').replace(/_/g, ' ')),
      type: SCORE_LEVELS[name] || 'hard',
    };
  });
}

export function buildAnalysisHtml(analysis, SF) {
  if (!analysis || !analysis.constraints) return '<p>No analysis available.</p>';
  var html = '<p><strong>Score:</strong> ' + SF.escHtml(analysis.score) + '</p>';
  html += '<table class="sf-table"><thead><tr><th>Constraint</th><th>Type</th><th>Score</th><th>Matches</th></tr></thead><tbody>';
  analysis.constraints.forEach(function (constraint) {
    var matchCount = constraint.matchCount != null ? constraint.matchCount : (constraint.matches ? constraint.matches.length : 0);
    html += '<tr><td>' + SF.escHtml(constraint.name) + '</td><td>' + SF.escHtml(constraint.constraintType || constraint.type || '') + '</td><td>' + SF.escHtml(constraint.score) + '</td><td>' + matchCount + '</td></tr>';
  });
  html += '</tbody></table>';
  return html;
}

export function describeError(err) {
  if (err && err.message) {
    return err.message;
  }
  return String(err || 'unknown error');
}
