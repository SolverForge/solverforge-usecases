// Layers the hospital-owned presentation metadata in `app/app-model.json` over
// the CLI-generated `generated/ui-model.json`.
//
// `solverforge generate ...` and `solverforge destroy ...` rewrite
// `generated/ui-model.json` from the Rust sources, so that file only carries
// what the CLI can derive: collection names and constraint ids. Readable
// constraint labels, the hard/soft classification, collection headings, and the
// two schedule views belong to this app and live in `app/app-model.json`, which
// the CLI never touches.

// Turns a constraint id such as `home_hub_match` into `Home hub match`.
export function humanizeConstraintId(id) {
  const text = String(id || '').replace(/_/g, ' ').trim();
  return text ? text.charAt(0).toUpperCase() + text.slice(1) : '';
}

// Builds the constraint entries used by the status bar and the analysis modal.
//
// Known ids keep the catalog order, which matches `create_constraints()` and
// therefore the analysis order. Ids that only the generated model knows about,
// for example a constraint added with `solverforge generate constraint`, follow
// with a humanized label until the catalog gains an entry for them.
function mergeConstraints(generatedIds, catalog) {
  const catalogById = new Map(catalog.map((entry) => [entry.id, entry]));
  const generated = new Set(generatedIds);
  const ids = catalog
    .map((entry) => entry.id)
    .filter((id) => generated.has(id))
    .concat(generatedIds.filter((id) => !catalogById.has(id)));
  return ids.map((id) => {
    const entry = catalogById.get(id);
    return {
      id,
      name: entry ? entry.label : humanizeConstraintId(id),
      type: entry ? entry.type : undefined,
    };
  });
}

// Returns the browser model: generated collections and constraint ids, with the
// app-owned labels, types, and views applied.
export function buildBrowserModel(uiModel, appModel) {
  const withHeading = (entry) => ({
    ...entry,
    label: appModel.collections[entry.plural] || entry.label,
  });
  return {
    entities: uiModel.entities.map(withHeading),
    facts: uiModel.facts.map(withHeading),
    constraints: mergeConstraints(uiModel.constraints, appModel.constraints),
    views: appModel.views,
  };
}

// Makes the stock status bar label the dots it repaints from each analysis
// payload: analysis reports constraint ids, the dots' hover titles show labels.
export function withConstraintLabels(statusBar, constraints) {
  const paint = statusBar && statusBar.colorDotsFromAnalysis;
  if (typeof paint !== 'function') return statusBar;
  const labels = new Map((constraints || []).map((entry) => [entry.id, entry.name]));
  statusBar.colorDotsFromAnalysis = (analyzed) => paint.call(statusBar, (analyzed || []).map((entry) => ({
    ...entry,
    name: labels.get(entry.name) || humanizeConstraintId(entry.name),
  })));
  return statusBar;
}
