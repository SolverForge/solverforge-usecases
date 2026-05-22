/* data-tables.mjs — Entity and fact table rendering for solverforge-lessons */

import { title } from '../models/index.mjs';

export function renderTables(tablesContainer, uiModel, data) {
  tablesContainer.innerHTML = '';
  (uiModel.entities || []).concat(uiModel.facts || []).forEach(function (entry) {
    var rows = data[entry.plural] || [];
    if (!rows.length) return;
    var cols = Object.keys(rows[0]).filter(function (key) { return key !== 'score' && key !== 'solverStatus'; });
    var values = rows.map(function (row) {
      return cols.map(function (key) {
        var value = row[key];
        if (value == null) return '—';
        if (Array.isArray(value)) return value.join(', ');
        if (typeof value === 'object') return JSON.stringify(value);
        return String(value);
      });
    });
    var section = SF.el('div', { className: 'sf-section' });
    section.appendChild(SF.el('h3', null, entry.label));
    section.appendChild(SF.createTable({ columns: cols, rows: values }));
    tablesContainer.appendChild(section);
  });
}
