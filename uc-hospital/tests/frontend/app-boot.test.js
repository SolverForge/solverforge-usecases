const assert = require('node:assert/strict');
const test = require('node:test');

const { createJsonResponse, loadFullApp, withBrowserEnv } = require('./support/load-browser-modules');
const { duplicateNameEmployees, shift } = require('./support/fixtures');

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));
const displayValue = (element) => element.style.display || '';

// This is the broad smoke test for the browser boot path.
test('full app boot renders timeline blocks without runtime render errors', async () => {
  const { document, errors } = await loadFullApp({
    demoData: {
      employees: duplicateNameEmployees(),
      shifts: [
        shift({ id: 'shift-1', employeeIdx: 0 }),
        shift({
          id: 'shift-2',
          start: '2024-01-01T12:00:00',
          end: '2024-01-01T20:00:00',
          requiredSkill: 'Nurse',
          employeeIdx: null,
        }),
      ],
    },
  });

  assert.deepEqual(errors, []);
  assert.ok(document.body.querySelectorAll('.sf-rail-timeline').length >= 1);
  assert.ok(document.body.querySelectorAll('.sf-rail-timeline-row').length >= 1);
  assert.ok(document.body.querySelectorAll('.sf-rail-timeline-item').length >= 1);
  assert.equal(document.body.querySelectorAll('.sf-content').length, 4);
  assert.ok(!document.body.textContent.includes('7860'));
});

test('terminal Solve is exposed by the stock header and restarts through cleanup', async () => {
  const calls = [];
  const eventSources = [];
  let createCount = 0;

  class TestEventSource {
    constructor(url) {
      this.url = url;
      this.readyState = 1;
      eventSources.push(this);
      calls.push(['stream', url]);
    }

    close() {
      this.readyState = 2;
      calls.push(['close', this.url]);
    }
  }
  TestEventSource.CLOSED = 2;

  await withBrowserEnv({
    EventSource: TestEventSource,
    fetch: async (url, options = {}) => {
      const method = options.method || 'GET';
      calls.push([method, url]);

      if (url === '/sf-config.json') {
        return createJsonResponse({
          title: 'SolverForge Hospital',
          subtitle: 'Test shell',
          defaultDemoId: 'LARGE',
        });
      }
      if (url === '/generated/ui-model.json') {
        return createJsonResponse({
          constraints: [],
          entities: [],
          facts: [],
          views: [{ id: 'schedule', label: 'Schedule', kind: 'timeline-by-location' }],
        });
      }
      if (url === '/demo-data') return createJsonResponse(['LARGE']);
      if (url === '/demo-data/LARGE') return createJsonResponse({ employees: [], shifts: [] });
      if (url === '/jobs' && method === 'POST') {
        createCount += 1;
        return createJsonResponse({ id: `job-${createCount}` });
      }
      if (url.startsWith('/jobs/job-1/snapshot') && method === 'GET') {
        return createJsonResponse({
          id: 'job-1',
          jobId: 'job-1',
          snapshotRevision: 3,
          lifecycleState: 'COMPLETED',
          currentScore: '0hard/0soft',
          bestScore: '0hard/0soft',
          solution: { employees: [], shifts: [], score: '0hard/0soft' },
        });
      }
      if (url.startsWith('/jobs/job-1/analysis') && method === 'GET') {
        return createJsonResponse({
          id: 'job-1',
          jobId: 'job-1',
          snapshotRevision: 3,
          lifecycleState: 'COMPLETED',
          analysis: { score: '0hard/0soft', constraints: [] },
        });
      }
      if (url === '/jobs/job-1' && method === 'DELETE') {
        return createJsonResponse('');
      }

      throw new Error(`unexpected ${method} ${url}`);
    },
  }, async ({ importModule, document }) => {
    const { bootApp } = await importModule('static/app/main.mjs');
    const boot = await bootApp(globalThis);
    await flush();

    const solveButton = boot.shell.header.sfControls.solveBtn;
    assert.equal(solveButton.textContent, 'Solve');
    assert.equal(displayValue(solveButton), '');

    solveButton.eventListeners.click[0]({ type: 'click' });
    await flush();
    assert.equal(createCount, 1);
    assert.equal(solveButton.style.display, 'none');

    eventSources[0].onmessage({
      data: JSON.stringify({
        eventType: 'completed',
        jobId: 'job-1',
        lifecycleState: 'COMPLETED',
        snapshotRevision: 3,
        currentScore: '0hard/0soft',
        bestScore: '0hard/0soft',
      }),
    });
    await flush();
    await flush();

    assert.equal(document.getElementById('sf-app').dataset.lifecycleState, 'COMPLETED');
    assert.equal(displayValue(solveButton), '');

    solveButton.eventListeners.click[0]({ type: 'click' });
    await flush();
    await flush();

    assert.equal(createCount, 2);
    assert.deepEqual(calls.filter(([method]) => method === 'DELETE' || method === 'POST'), [
      ['POST', '/jobs'],
      ['DELETE', '/jobs/job-1'],
      ['POST', '/jobs'],
    ]);
  });
});
