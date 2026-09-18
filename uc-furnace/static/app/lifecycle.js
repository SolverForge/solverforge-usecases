import { syncConstraintStatus } from './analysis.js';

export function cleanupTerminalJob(ctx) {
  var state = ctx.solver.getLifecycleState();
  if (!ctx.solver.getJobId() || state === 'IDLE' || state === 'PAUSED' || ctx.solver.isRunning()) {
    return Promise.resolve(null);
  }
  return ctx.solver.delete()
    .then(function () {
      ctx.lastAnalysis = null;
      syncLifecycleMarkers(ctx);
      return null;
    })
    .catch(function (err) {
      console.error('Delete failed:', err);
      return null;
    });
}

export function syncLifecycleMarkers(ctx, meta) {
  var jobId = ctx.solver.getJobId();
  var snapshotRevision = ctx.solver.getSnapshotRevision();
  var lifecycleState = meta && meta.lifecycleState ? meta.lifecycleState : ctx.solver.getLifecycleState();
  var score = meta && (meta.currentScore || meta.bestScore);
  var moves = meta && meta.telemetry ? meta.telemetry.movesPerSecond : null;

  if (jobId) {
    ctx.app.dataset.jobId = String(jobId);
  } else {
    delete ctx.app.dataset.jobId;
  }
  if (snapshotRevision != null) {
    ctx.app.dataset.snapshotRevision = String(snapshotRevision);
  } else {
    delete ctx.app.dataset.snapshotRevision;
  }
  if (lifecycleState && lifecycleState !== 'IDLE') {
    ctx.app.dataset.lifecycleState = lifecycleState;
  } else {
    delete ctx.app.dataset.lifecycleState;
  }

  ctx.statusBar.setLifecycleState(lifecycleState);
  ctx.statusBar.updateMoves(moves);
  if (score) {
    ctx.statusBar.updateScore(score);
    ctx.statusBar.colorDotsByScore(score);
  } else {
    ctx.statusBar.updateScore(null);
  }
  syncConstraintStatus(ctx);
}
