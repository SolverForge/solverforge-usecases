export function syncLifecycleDataset(app, solver, meta) {
  const jobId = solver.getJobId();
  const snapshotRevision =
    meta?.snapshotRevision != null ? meta.snapshotRevision : solver.getSnapshotRevision();
  if (jobId) {
    app.dataset.jobId = String(jobId);
  } else {
    delete app.dataset.jobId;
  }
  if (snapshotRevision != null) {
    app.dataset.snapshotRevision = String(snapshotRevision);
  } else {
    delete app.dataset.snapshotRevision;
  }
}

export function sameRouteIdentity(left, right) {
  return (
    left?.jobId === right?.jobId &&
    left?.snapshotRevision === right?.snapshotRevision &&
    left?.routingMode === right?.routingMode
  );
}
