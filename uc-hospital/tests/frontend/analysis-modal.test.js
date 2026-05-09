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
      { name: '<script>alert(1)</script>', type: 'hard' },
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
