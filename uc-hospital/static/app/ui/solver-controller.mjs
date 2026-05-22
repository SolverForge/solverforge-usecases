// Thin adapter around the shared `SF.createSolver()` controller.
export function createSolverController({
  sf,
  backend,
  statusBar,
  onPlan,
  onAnalysis,
  onMeta,
  onLifecycle,
  onError,
}) {
  const solver = sf.createSolver({
    backend,
    statusBar,
    onProgress(meta) {
      publishMeta(meta);
    },
    onSolution(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onPaused(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onResumed(meta) {
      publishMeta(meta);
    },
    onCancelled(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onComplete(snapshot, meta) {
      publishSnapshot(snapshot, meta);
    },
    onFailure(_message, meta, snapshot, analysis) {
      publishSnapshot(snapshot, meta);
      if (analysis) onAnalysis(analysis);
    },
    onAnalysis(analysis) {
      onAnalysis(analysis);
      publishLifecycle();
    },
    onError(message) {
      if (typeof onError === 'function') onError(message);
      publishLifecycle();
    },
  });

  return {
    // Starts a new solve after cleaning up any previous terminal retained job.
    async start(planProvider) {
      if (solver.isRunning() || solver.getLifecycleState() === 'PAUSED') return;
      await cleanupTerminalJob();
      const plan = await planProvider();
      await solver.start(plan);
      publishMeta({
        id: solver.getJobId(),
        jobId: solver.getJobId(),
        lifecycleState: solver.getLifecycleState(),
        currentScore: null,
        bestScore: null,
        telemetry: null,
      });
    },
    pause() {
      return solver.pause().then(() => {
        publishLifecycle();
      });
    },
    resume() {
      return solver.resume().then(() => {
        publishLifecycle();
      });
    },
    cancel() {
      return solver.cancel().then(() => {
        publishLifecycle();
      });
    },
    analyzeSnapshot() {
      return solver.analyzeSnapshot().then((analysis) => {
        onAnalysis(analysis);
        publishLifecycle();
        return analysis;
      });
    },
    getLifecycleState() {
      return solver.getLifecycleState();
    },
    getJobId() {
      return solver.getJobId();
    },
    getSnapshotRevision() {
      return solver.getSnapshotRevision();
    },
  };

  // Publishes the latest solution and lifecycle metadata back to the app shell.
  function publishSnapshot(snapshot, meta) {
    if (snapshot && snapshot.solution) onPlan(snapshot.solution);
    publishMeta(meta);
  }

  // Publishes status metadata and refreshes the shell lifecycle markers.
  function publishMeta(meta) {
    onMeta(meta || null);
    publishLifecycle();
  }

  // Keeps HTML data attributes in sync with the underlying solver lifecycle.
  function publishLifecycle() {
    onLifecycle({
      jobId: solver.getJobId(),
      snapshotRevision: solver.getSnapshotRevision(),
      lifecycleState: solver.getLifecycleState(),
    });
  }

  // Deletes old terminal jobs so "Solve" always starts from a clean retained slot.
  function cleanupTerminalJob() {
    const state = solver.getLifecycleState();
    if (!solver.getJobId() || state === 'IDLE' || state === 'PAUSED' || solver.isRunning()) {
      return Promise.resolve();
    }
    return solver.delete()
      .then(() => {
        onAnalysis(null);
        onMeta(null);
        publishLifecycle();
      })
      .catch((error) => {
        if (typeof onError === 'function') onError(error);
        throw error;
      });
  }
}
