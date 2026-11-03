import { humanizeConstraintId } from '../shell/app-model.mjs';

// Accept either the raw analysis body or the `/analysis` wrapper returned by the backend.
function normalizeAnalysis(analysis) {
  return analysis && analysis.analysis ? analysis.analysis : analysis;
}

// Maps analysis constraint ids to the readable label and hard/soft type from the browser model.
function constraintLookup(constraints) {
  return new Map((constraints || []).map((constraint) => [constraint.id, constraint]));
}

// Renders one analysis table into the modal container.
function appendAnalysisSection(document, container, title, analysis, constraintsById) {
  const analysisBody = normalizeAnalysis(analysis);
  if (!analysisBody || !Array.isArray(analysisBody.constraints)) return;

  if (container.childNodes.length) {
    container.appendChild(document.createElement('hr'));
  }

  const heading = document.createElement('h3');
  heading.textContent = title;
  container.appendChild(heading);

  const scoreLine = document.createElement('p');
  const scoreLabel = document.createElement('strong');
  scoreLabel.textContent = 'Score:';
  scoreLine.appendChild(scoreLabel);
  scoreLine.appendChild(document.createTextNode(` ${String(analysisBody.score || '—')}`));
  container.appendChild(scoreLine);

  const table = document.createElement('table');
  table.className = 'sf-table';

  const thead = document.createElement('thead');
  const headerRow = document.createElement('tr');
  ['Constraint', 'Type', 'Score', 'Matches'].forEach((label) => {
    const cell = document.createElement('th');
    cell.textContent = label;
    headerRow.appendChild(cell);
  });
  thead.appendChild(headerRow);
  table.appendChild(thead);

  const tbody = document.createElement('tbody');
  analysisBody.constraints.forEach((constraint) => {
    const known = constraintsById.get(constraint.name);
    const row = document.createElement('tr');
    row.dataset.constraintId = String(constraint.name || '');
    [
      known ? known.name : humanizeConstraintId(constraint.name),
      constraint.constraintType || constraint.type || (known && known.type) || '',
      constraint.score || '',
      String(constraint.matchCount != null ? constraint.matchCount : 0),
    ].forEach((value) => {
      const cell = document.createElement('td');
      cell.textContent = String(value);
      row.appendChild(cell);
    });
    tbody.appendChild(row);
  });
  table.appendChild(tbody);
  container.appendChild(table);
}

// Builds DOM content instead of raw HTML so tests can verify escaping and structure.
export function buildAnalysisBody(document, analysis, constraints = []) {
  const analysisBody = normalizeAnalysis(analysis);
  const constraintsById = constraintLookup(constraints);
  const container = document.createElement('div');

  if (!analysisBody || !Array.isArray(analysisBody.constraints)) {
    const empty = document.createElement('p');
    empty.textContent = 'No analysis available.';
    container.appendChild(empty);
    return container;
  }

  appendAnalysisSection(document, container, 'Current retained snapshot', analysisBody, constraintsById);

  return container;
}
