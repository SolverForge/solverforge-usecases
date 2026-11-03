const assert = require('node:assert/strict');
const test = require('node:test');

const { withBrowserEnv } = require('./support/load-browser-modules');

// Frontend tests double as documentation for the intended UI contract.
test('analysis modal body uses DOM content and preserves text escaping', async () => {
  await withBrowserEnv({}, async ({ document, importModule }) => {
    const { buildAnalysisBody } = await importModule('static/app/schedule/analysis-modal.mjs');

    const body = buildAnalysisBody(document, {
      analysis: {
        score: '0hard/0soft',
        constraints: [
          {
            name: '<script>alert(1)</script>',
            score: '0hard/0soft',
            matchCount: 0,
          },
        ],
      },
    }, [
      { id: '<script>alert(1)</script>', name: '<script>alert(1)</script>', type: 'hard' },
    ]);

    const table = body.querySelector('table');
    assert.ok(table, 'expected a rendered table node');
    assert.equal(table.className, 'sf-table');
    assert.equal(body.querySelector('script'), null);
    assert.match(body.textContent, /<script>alert\(1\)<\/script>/);
    assert.match(body.textContent, /ConstraintTypeScoreMatches/);
    assert.match(body.textContent, /hard/);
  });
});

test('analysis modal shows readable labels for constraint ids', async () => {
  await withBrowserEnv({}, async ({ document, importModule }) => {
    const { buildAnalysisBody } = await importModule('static/app/schedule/analysis-modal.mjs');

    const body = buildAnalysisBody(document, {
      score: '-1hard/0soft',
      constraints: [
        { name: 'minimum_rest', score: '-1hard/0soft', matchCount: 1 },
        { name: 'home_hub_match', score: '0hard/0soft', matchCount: 0 },
      ],
    }, [
      { id: 'minimum_rest', name: 'At least 10 hours between 2 shifts', type: 'hard' },
    ]);

    const rows = Array.from(body.querySelector('tbody').querySelectorAll('tr'));
    assert.equal(rows.length, 2);
    assert.equal(rows[0].dataset.constraintId, 'minimum_rest');
    assert.match(rows[0].textContent, /^At least 10 hours between 2 shiftshard/);
    assert.match(rows[1].textContent, /^Home hub match/);
  });
});
