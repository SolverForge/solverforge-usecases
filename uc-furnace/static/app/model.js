// Furnace's UI model on top of the CLI-generated `ui-model.json`.
//
// The CLI owns `static/generated/ui-model.json` and rewrites it on every model
// change: it lists constraint ids and the snake_case fact and entity
// collections. The tabs and the hard/soft split below belong to this app.

// Tabs drawn by `render.js`, keyed by view id.
export var VIEWS = [
  { id: 'schedule', label: 'Schedule', kind: 'timeline', icon: 'fa-fire' },
  { id: 'crew', label: 'Crew', kind: 'table', icon: 'fa-users' },
  { id: 'orders', label: 'Orders', kind: 'table', icon: 'fa-industry' },
];

// Soft constraint ids; every other constraint is hard. Mirrors
// `constraint_type` in `src/api/routes/analysis.rs`.
var SOFT_CONSTRAINTS = [
  'express_lateness',
  'urgent_lateness',
  'standard_lateness',
  'thermal_changeover',
  'early_start_pressure',
  'night_start_preference',
  'overtime',
  'shift_load_balance',
];

// Status-bar entries for the CLI's constraint ids.
export function constraintEntries(uiModel) {
  return (uiModel.constraints || []).map(function (name) {
    return { name: name, type: SOFT_CONSTRAINTS.indexOf(name) >= 0 ? 'soft' : 'hard' };
  });
}

// Data-tab tables for the CLI's collections: `key` is the plan JSON field,
// `label` the plural heading.
export function collectionEntries(uiModel) {
  return (uiModel.entities || []).concat(uiModel.facts || []).map(function (entry) {
    var words = String(entry.plural).split('_');
    return {
      key: words[0] + words.slice(1).map(capitalize).join(''),
      label: words.map(capitalize).join(' '),
    };
  });
}

function capitalize(word) {
  return word.charAt(0).toUpperCase() + word.slice(1);
}
