export function mountPanel(root, panels, id) {
  const panel = SF.el('section', {
    className: `deliveries-panel${id === 'overview' ? ' is-active' : ''}`,
  });
  panels[id] = panel;
  root.appendChild(panel);
  return panel;
}

export function createSelectField(label, options) {
  const select = SF.el('select');
  options.forEach((option) => {
    select.appendChild(SF.el('option', { value: option.value }, option.label));
  });
  const el = SF.el(
    'div',
    { className: 'deliveries-field' },
    SF.el('label', null, label),
    select,
  );
  return { el, select };
}

export function kpi(label, value) {
  return SF.el(
    'div',
    { className: 'deliveries-kpi' },
    SF.el('div', { className: 'deliveries-kpi__label' }, label),
    SF.el('div', { className: 'deliveries-kpi__value' }, value),
  );
}

export function showError(error) {
  const detail = error?.message || String(error);
  SF.showError('Deliveries UI', detail);
}

export async function fetchJson(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Request failed: ${url}`);
  }
  return response.json();
}
