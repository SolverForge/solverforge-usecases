const assert = require('node:assert/strict');
const test = require('node:test');

const { withBrowserEnv } = require('./support/load-browser-modules');

const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

test('solver controller keeps the retained job after transport interruption', async () => {
  await withBrowserEnv({}, async ({ importModule, window }) => {
    const { createSolverController } = await importModule('static/app/ui/solver-controller.mjs');
    const calls = [];
    let onStreamError;
    let createCount = 0;
    let planProviderCalls = 0;

    const controller = createSolverController({
      sf: window.SF,
      backend: {
        createJob: async () => {
          createCount += 1;
          return 'job-retained';
        },
        streamJobEvents(id, onMessage, onError) {
          calls.push(['streamJobEvents', id]);
          onStreamError = onError;
          onMessage({
            eventType: 'progress',
            jobId: id,
            lifecycleState: 'SOLVING',
            currentScore: '0hard/-1soft',
            bestScore: '0hard/-1soft',
          });
          return () => calls.push(['closeStream']);
        },
        getSnapshot: async () => null,
        analyzeSnapshot: async () => null,
        pauseJob: async () => {},
        resumeJob: async () => {},
        cancelJob: async (id) => calls.push(['cancelJob', id]),
        deleteJob: async () => {},
      },
      statusBar: null,
      onPlan: () => {},
      onAnalysis: () => {},
      onMeta: () => {},
      onLifecycle: () => {},
      onError: () => {},
    });

    await controller.start(async () => ({}));
    onStreamError(new Error('temporary disconnect'));
    await flush();

    assert.equal(controller.getJobId(), 'job-retained');
    assert.equal(controller.getLifecycleState(), 'SOLVING');

    await controller.start(async () => {
      planProviderCalls += 1;
      return {};
    });
    void controller.cancel();
    await flush();

    assert.equal(createCount, 1);
    assert.equal(planProviderCalls, 0);
    assert.deepEqual(calls, [
      ['streamJobEvents', 'job-retained'],
      ['closeStream'],
      ['streamJobEvents', 'job-retained'],
      ['cancelJob', 'job-retained'],
    ]);
  });
});

test('solver controller pauses and resumes the retained runtime without starting over', async () => {
  await withBrowserEnv({}, async ({ importModule, window }) => {
    const { createSolverController } = await importModule('static/app/ui/solver-controller.mjs');
    const calls = [];
    let onMessage;
    let createCount = 0;
    let planProviderCalls = 0;
    let latestPlan = null;

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
          lifecycleState: 'PAUSED',
          currentScore: '0hard/-4soft',
          bestScore: '0hard/-4soft',
          solution: { id, revision, source: 'paused-snapshot' },
        }),
        analyzeSnapshot: async () => null,
        pauseJob: async (id) => calls.push(['pauseJob', id]),
        resumeJob: async (id) => calls.push(['resumeJob', id]),
        cancelJob: async () => {},
        deleteJob: async () => {},
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

    const pause = controller.pause();
    await flush();
    onMessage({
      eventType: 'paused',
      jobId: 'job-1',
      lifecycleState: 'PAUSED',
      snapshotRevision: 7,
      currentScore: '0hard/-4soft',
      bestScore: '0hard/-4soft',
    });
    await pause;

    assert.equal(controller.getLifecycleState(), 'PAUSED');
    assert.deepEqual(latestPlan, { id: 'job-1', revision: 7, source: 'paused-snapshot' });

    await controller.start(async () => {
      planProviderCalls += 1;
      return {};
    });

    const resume = controller.resume();
    await flush();
    onMessage({
      eventType: 'resumed',
      jobId: 'job-1',
      lifecycleState: 'SOLVING',
      snapshotRevision: 7,
    });
    await resume;

    assert.equal(createCount, 1);
    assert.equal(planProviderCalls, 0);
    assert.equal(controller.getJobId(), 'job-1');
    assert.equal(controller.getLifecycleState(), 'SOLVING');
    assert.deepEqual(calls, [
      ['streamJobEvents', 'job-1'],
      ['pauseJob', 'job-1'],
      ['resumeJob', 'job-1'],
    ]);
  });
});
