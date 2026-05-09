const assert = require('node:assert/strict');
const test = require('node:test');

const { withBrowserEnv } = require('./support/load-browser-modules');

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

test('solver controller stop leaves only a terminal job for the next solve to replace', async () => {
  await withBrowserEnv({}, async ({ importModule, window }) => {
    const { createSolverController } = await importModule('static/app/shell/solver-controller.mjs');
    const calls = [];
    let onMessage;
    let createCount = 0;
    let latestPlan = null;
    let restartProviderCalls = 0;

    const controller = createSolverController({
      sf: window.SF,
      backend: {
        createJob: async () => {
          createCount += 1;
          return `job-${createCount}`;
        },
        streamJobEvents(id, callback) {
          calls.push(['streamJobEvents', id]);
          onMessage = callback;
          return () => calls.push(['closeStream', id]);
        },
        getSnapshot: async (id, revision) => ({
          id,
          jobId: id,
          snapshotRevision: revision,
          lifecycleState: 'CANCELLED',
          currentScore: '0hard/-2soft',
          bestScore: '0hard/-2soft',
          solution: { id, revision, source: 'stopped-snapshot' },
        }),
        analyzeSnapshot: async () => null,
        pauseJob: async () => {},
        resumeJob: async (id) => calls.push(['resumeJob', id]),
        cancelJob: async (id) => calls.push(['cancelJob', id]),
        deleteJob: async (id) => calls.push(['deleteJob', id]),
      },
      statusBar: null,
      onPlan: (plan) => {
        latestPlan = plan;
      },
      onAnalysis: () => {},
      onMeta: () => {},
      onLifecycle: () => {},
      onError: () => {},
    });

    await controller.start(async () => ({ source: 'initial-plan' }));

    const stop = controller.cancel();
    await flush();
    onMessage({
      eventType: 'cancelled',
      jobId: 'job-1',
      lifecycleState: 'CANCELLED',
      snapshotRevision: 5,
      currentScore: '0hard/-2soft',
      bestScore: '0hard/-2soft',
    });
    await stop;

    assert.equal(controller.getLifecycleState(), 'CANCELLED');
    assert.deepEqual(latestPlan, { id: 'job-1', revision: 5, source: 'stopped-snapshot' });

    await controller.start(async () => {
      restartProviderCalls += 1;
      return latestPlan;
    });

    assert.equal(createCount, 2);
    assert.equal(restartProviderCalls, 1);
    assert.equal(controller.getJobId(), 'job-2');
    assert.deepEqual(calls, [
      ['streamJobEvents', 'job-1'],
      ['cancelJob', 'job-1'],
      ['closeStream', 'job-1'],
      ['deleteJob', 'job-1'],
      ['streamJobEvents', 'job-2'],
    ]);
  });
});

test('solver controller aborts a retry when terminal cleanup fails', async () => {
  await withBrowserEnv({}, async ({ importModule, window }) => {
    const { createSolverController } = await importModule('static/app/shell/solver-controller.mjs');
    const calls = [];
    let onMessage;
    let createCount = 0;
    let planProviderCalls = 0;

    const controller = createSolverController({
      sf: window.SF,
      backend: {
        createJob: async () => {
          createCount += 1;
          return `job-${createCount}`;
        },
        streamJobEvents(id, callback) {
          calls.push(['streamJobEvents', id]);
          onMessage = callback;
          return () => calls.push(['closeStream', id]);
        },
        getSnapshot: async (id, revision) => ({
          id,
          jobId: id,
          snapshotRevision: revision,
          lifecycleState: 'COMPLETED',
          currentScore: '0hard/0soft',
          bestScore: '0hard/0soft',
          solution: { id, revision },
        }),
        analyzeSnapshot: async () => null,
        pauseJob: async () => {},
        resumeJob: async () => {},
        cancelJob: async () => {},
        deleteJob: async (id) => {
          calls.push(['deleteJob', id]);
          throw new Error('delete failed');
        },
      },
      statusBar: null,
      onPlan: () => {},
      onAnalysis: () => {},
      onMeta: () => {},
      onLifecycle: () => {},
      onError: () => {},
    });

    await controller.start(async () => ({}));
    onMessage({
      eventType: 'completed',
      jobId: 'job-1',
      lifecycleState: 'COMPLETED',
      snapshotRevision: 1,
      currentScore: '0hard/0soft',
      bestScore: '0hard/0soft',
    });
    await flush();

    await assert.rejects(
      () => controller.start(async () => {
        planProviderCalls += 1;
        return {};
      }),
      /delete failed/,
    );

    assert.equal(createCount, 1);
    assert.equal(planProviderCalls, 0);
    assert.deepEqual(calls, [
      ['streamJobEvents', 'job-1'],
      ['closeStream', 'job-1'],
      ['deleteJob', 'job-1'],
    ]);
  });
});
