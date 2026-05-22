/* dataset.mjs - demo dataset loader for the Bergamo FSR demo */

import { clonePlan, fetchDemoPlan } from '../models/utils.mjs';

export function createDemoDataController(options) {
  var utils = options.utils;
  var selectedId = 'STANDARD';
  var catalog = { defaultId: selectedId, availableIds: [selectedId] };
  var loading = false;
  var el = document.createDocumentFragment();

  return {
    bootstrap: bootstrap,
    el: el,
    getCatalog: function () { return catalog; },
    getSelectedId: function () { return selectedId; },
    isLoading: function () { return loading; },
    resolvePlan: resolvePlan,
  };

  function bootstrap() {
    setLoading(true);
    notifyCatalog();
    return utils.fetchDemoPlan(selectedId)
      .then(function (plan) { notifyPlan(plan); })
      .catch(function (err) { notifyError(err); })
      .finally(function () { setLoading(false); });
  }

  function resolvePlan() {
    var currentPlan = options.getCurrentPlan();
    if (currentPlan) return Promise.resolve(utils.clonePlan(currentPlan));
    return utils.fetchDemoPlan(selectedId);
  }

  function setLoading(nextLoading) {
    loading = nextLoading;
    if (options.onLoadingChange) options.onLoadingChange(loading);
  }

  function notifyCatalog() {
    if (options.onCatalog) options.onCatalog(catalog, selectedId);
  }

  function notifyPlan(plan) {
    if (options.onPlan) options.onPlan(plan, selectedId);
  }

  function notifyError(err) {
    if (options.onError) options.onError(err);
  }
}
