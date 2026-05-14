/* app-route-state.js - snapshot-scoped route geometry loading */

(function () {
  'use strict';

  var FSR = window.FSR = window.FSR || {};

  FSR.createRouteGeometryController = function (options) {
    var currentPlanIdentity = null;
    var currentRouteIdentity = null;
    var routeInFlightIdentity = null;
    var latestRouteIdentity = null;
    var routeRequestToken = 0;

    return {
      identityFrom: identityFrom,
      invalidate: invalidate,
      load: load,
      setPlanIdentity: setPlanIdentity,
    };

    function invalidate() {
      routeRequestToken += 1;
      currentPlanIdentity = null;
      currentRouteIdentity = null;
      routeInFlightIdentity = null;
      latestRouteIdentity = null;
      if (options.onClearFocus) options.onClearFocus();
      options.onRoutesChange(null);
    }

    function setPlanIdentity(identity) {
      currentPlanIdentity = identity;
      if (!identityEquals(currentRouteIdentity, identity)) {
        currentRouteIdentity = null;
        options.onRoutesChange(null);
      }
    }

    function load(identity) {
      if (!identity) return Promise.resolve();
      latestRouteIdentity = identity;
      if (routeInFlightIdentity || identityEquals(currentRouteIdentity, identity)) {
        return Promise.resolve();
      }
      return fetchLatest();
    }

    function fetchLatest() {
      var identity = latestRouteIdentity;
      if (!identity || routeInFlightIdentity || identityEquals(currentRouteIdentity, identity)) {
        return Promise.resolve();
      }
      var token = routeRequestToken;
      routeInFlightIdentity = identity;
      return options.utils.fetchJobRoutes(identity.jobId, identity.snapshotRevision)
        .then(function (routes) {
          if (!responseStillCurrent(token, identity)) return;
          currentRouteIdentity = identity;
          options.onRoutesChange(routes);
        })
        .catch(function (err) {
          if (!responseStillCurrent(token, identity)) return;
          currentRouteIdentity = null;
          options.onRoutesChange(null);
          console.error('Route geometry failed:', err);
        })
        .then(function () {
          if (identityEquals(routeInFlightIdentity, identity)) routeInFlightIdentity = null;
          if (
            latestRouteIdentity
            && !identityEquals(latestRouteIdentity, identity)
            && responseStillCurrent(token, latestRouteIdentity)
            && !identityEquals(currentRouteIdentity, latestRouteIdentity)
          ) {
            return fetchLatest();
          }
        });
    }

    function identityFrom(meta) {
      var jobId = meta && meta.jobId != null ? meta.jobId : options.solver.getJobId();
      var snapshotRevision = meta && meta.snapshotRevision != null
        ? meta.snapshotRevision
        : options.solver.getSnapshotRevision();
      if (jobId == null || jobId === '' || snapshotRevision == null) return null;
      return { jobId: String(jobId), snapshotRevision: String(snapshotRevision) };
    }

    function responseStillCurrent(token, identity) {
      return token === routeRequestToken && identityEquals(currentPlanIdentity, identity);
    }

    function identityEquals(left, right) {
      return !!left
        && !!right
        && left.jobId === right.jobId
        && left.snapshotRevision === right.snapshotRevision;
    }
  };
})();
