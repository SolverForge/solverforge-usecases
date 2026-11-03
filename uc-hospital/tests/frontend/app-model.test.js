const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const { ROOT, importModule } = require('./support/load-browser-modules');

const readJson = (relativePath) => JSON.parse(fs.readFileSync(path.resolve(ROOT, relativePath), 'utf8'));

// The CLI owns generated/ui-model.json; the app owns labels, types, and views.
test('app model labels every constraint id the CLI generated', async () => {
  const { buildBrowserModel } = await importModule('static/app/shell/app-model.mjs');
  const uiModel = readJson('static/generated/ui-model.json');
  const appModel = readJson('static/app/app-model.json');

  assert.deepEqual(
    [...uiModel.constraints].sort(),
    appModel.constraints.map((entry) => entry.id).sort(),
  );

  const model = buildBrowserModel(uiModel, appModel);
  assert.deepEqual(model.constraints.map((entry) => entry.id), appModel.constraints.map((entry) => entry.id));
  assert.equal(model.constraints.find((entry) => entry.id === 'minimum_rest').name, 'At least 10 hours between 2 shifts');
  assert.ok(model.constraints.every((entry) => entry.type === 'hard' || entry.type === 'soft'));
  assert.deepEqual(model.views.map((view) => view.kind), ['schedule-by-location', 'schedule-by-employee']);
  assert.deepEqual(model.entities.map((entry) => entry.label), ['Shifts']);
  assert.deepEqual(model.facts.map((entry) => entry.label), ['Employees']);
});

test('constraints added by the CLI appear with a humanized label', async () => {
  const { buildBrowserModel } = await importModule('static/app/shell/app-model.mjs');
  const model = buildBrowserModel(
    { entities: [], facts: [], constraints: ['home_hub_match', 'assigned_shift'] },
    {
      collections: {},
      constraints: [{ id: 'assigned_shift', label: 'Assigned shift', type: 'hard' }],
      views: [{ id: 'by-location', kind: 'schedule-by-location', label: 'By location' }],
    },
  );

  assert.deepEqual(model.constraints, [
    { id: 'assigned_shift', name: 'Assigned shift', type: 'hard' },
    { id: 'home_hub_match', name: 'Home hub match', type: undefined },
  ]);
});

test('status-bar dots repainted from analysis show constraint labels', async () => {
  const { withConstraintLabels } = await importModule('static/app/shell/app-model.mjs');
  const painted = [];
  const statusBar = withConstraintLabels(
    { colorDotsFromAnalysis: (constraints) => painted.push(constraints) },
    [{ id: 'minimum_rest', name: 'At least 10 hours between 2 shifts', type: 'hard' }],
  );

  statusBar.colorDotsFromAnalysis([
    { name: 'minimum_rest', type: 'hard', score: '-1hard/0soft' },
    { name: 'home_hub_match', type: 'soft', score: '0hard/0soft' },
  ]);
  assert.deepEqual(painted, [[
    { name: 'At least 10 hours between 2 shifts', type: 'hard', score: '-1hard/0soft' },
    { name: 'Home hub match', type: 'soft', score: '0hard/0soft' },
  ]]);
});
