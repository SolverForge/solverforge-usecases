// Converts arbitrary JSON-ish values into displayable table cells.
function stringifyCell(value) {
  if (value == null) return '—';
  if (Array.isArray(value)) return value.join(', ');
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}

// Renders raw entity/fact tables for people who want to inspect the transport data directly.
export function renderDataTables({ sf, container, uiModel, data }) {
  container.innerHTML = '';
  uiModel.entities.concat(uiModel.facts).forEach((entry) => {
    const rows = data[entry.plural] || [];
    if (!rows.length) return;
    const columns = Object.keys(rows[0]).filter((key) => key !== 'score' && key !== 'solverStatus');
    const values = rows.map((row) => columns.map((key) => stringifyCell(row[key])));
    const section = sf.el('div', { className: 'sf-section' });
    section.appendChild(sf.el('h3', null, entry.label));
    section.appendChild(sf.createTable({ columns, rows: values }));
    container.appendChild(section);
  });
}
