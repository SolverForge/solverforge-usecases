var SF = window.SF;

export function openAnalysis(ctx) {
  if (!ctx.solver.getJobId()) return;
  ctx.solver.analyzeSnapshot()
    .then(function (analysis) {
      ctx.lastAnalysis = analysis;
      syncConstraintStatus(ctx);
      ctx.analysisModal.setBody({
        unsafeHtml: buildAnalysisHtml(analysis && analysis.analysis ? analysis.analysis : analysis),
      });
      ctx.analysisModal.open();
    })
    .catch(function (err) {
      console.error('Analysis failed:', err);
    });
}

export function openConstraintAnalysis(ctx, constraint) {
  if (!constraint || !constraint.name || !ctx.solver.getJobId()) return;
  var path = '/jobs/' + encodeURIComponent(String(ctx.solver.getJobId()))
    + '/analysis/' + encodeURIComponent(constraint.name);
  var revision = ctx.solver.getSnapshotRevision();
  if (revision != null) {
    path += '?snapshot_revision=' + encodeURIComponent(String(revision));
  }
  fetch(path)
    .then(function (response) {
      if (!response.ok) throw new Error('Constraint analysis request failed');
      return response.json();
    })
    .then(function (detail) {
      ctx.analysisModal.setBody({
        unsafeHtml: buildConstraintDetailHtml(detail),
      });
      ctx.analysisModal.open();
    })
    .catch(function (err) {
      console.error('Constraint analysis failed:', err);
    });
}

export function syncConstraintStatus(ctx) {
  if (!ctx.lastAnalysis || !ctx.lastAnalysis.analysis || !ctx.lastAnalysis.analysis.constraints) {
    ctx.statusBar.updateConstraintDots(ctx.uiModel.constraints || []);
    return;
  }
  ctx.statusBar.colorDotsFromAnalysis(normalizeConstraintSummaries(ctx.lastAnalysis.analysis.constraints));
}

function normalizeConstraintSummaries(constraints) {
  return (constraints || []).map(function (constraint) {
    return {
      name: constraint.name,
      type: constraint.type || constraint.constraintType || 'hard',
      score: constraint.score,
    };
  });
}

function buildAnalysisHtml(analysis) {
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

function buildConstraintDetailHtml(detail) {
  if (!detail || !detail.constraint) return '<p>No constraint detail available.</p>';
  var constraint = detail.constraint;
  var html = '<p><strong>Score:</strong> ' + SF.escHtml(detail.score) + '</p>';
  html += '<p><strong>Constraint:</strong> ' + SF.escHtml(constraint.name) + ' (' + SF.escHtml(constraint.type || constraint.constraintType || '') + ')</p>';
  html += '<p><strong>Weight:</strong> ' + SF.escHtml(constraint.weight) + ' · <strong>Matches:</strong> ' + String(constraint.matchCount) + '</p>';
  if (!constraint.matches.length) {
    html += '<p>No matches recorded for this snapshot.</p>';
    return html;
  }
  html += '<table class="sf-table"><thead><tr><th>Score</th><th>Justification</th></tr></thead><tbody>';
  constraint.matches.forEach(function (match) {
    html += '<tr><td>' + SF.escHtml(match.score) + '</td><td>' + SF.escHtml(match.justification) + '</td></tr>';
  });
  html += '</tbody></table>';
  return html;
}
