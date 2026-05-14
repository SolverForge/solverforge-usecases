export function buildRecommendationBody(payload, onApply) {
  const container = SF.el('div', { className: 'deliveries-modal-list' });
  if (!payload.candidates.length) {
    container.appendChild(SF.el('div', { className: 'deliveries-empty' }, 'No valid insertions.'));
    return container;
  }

  payload.candidates.forEach((candidate) => {
    const row = SF.el('div', { className: 'deliveries-list__row' });
    const top = SF.el('div', { className: 'deliveries-list__top' });
    top.appendChild(SF.el('strong', null, `${candidate.vehicleName} · position ${candidate.insertIndex + 1}`));
    top.appendChild(SF.el('span', { className: 'deliveries-tag' }, candidate.score));
    row.appendChild(top);

    const meta = SF.el('div', { className: 'deliveries-list__meta' });
    meta.appendChild(SF.el('span', null, `Δ hard ${candidate.deltaHard}`));
    meta.appendChild(SF.el('span', null, `Δ soft ${candidate.deltaSoft}`));
    row.appendChild(meta);

    const applyButton = SF.createButton({ text: 'Apply', variant: 'success' });
    applyButton.addEventListener('click', () => onApply(candidate.previewPlan));
    row.appendChild(SF.el('div', { className: 'deliveries-list__actions' }, applyButton));
    container.appendChild(row);
  });

  return container;
}

export function buildAnalysisBody(analysis) {
  const container = SF.el('div');
  container.appendChild(SF.el('p', null, `Score: ${analysis.analysis.score}`));
  const table = SF.el('table', { className: 'deliveries-table' });
  table.appendChild(
    SF.el(
      'thead',
      null,
      SF.el(
        'tr',
        null,
        SF.el('th', null, 'Constraint'),
        SF.el('th', null, 'Weight'),
        SF.el('th', null, 'Score'),
        SF.el('th', null, 'Matches'),
      ),
    ),
  );
  const body = SF.el('tbody');
  (analysis.analysis.constraints || []).forEach((constraint) => {
    body.appendChild(
      SF.el(
        'tr',
        null,
        SF.el('td', null, constraint.name),
        SF.el('td', null, constraint.weight),
        SF.el('td', null, constraint.score),
        SF.el('td', null, String(constraint.matchCount)),
      ),
    );
  });
  table.appendChild(body);
  container.appendChild(table);
  return container;
}
