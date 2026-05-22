/* demo.mjs — Demo data fetching for solverforge-lessons */

export function requestJson(path, label) {
  return fetch(path)
    .then(function (response) {
      if (!response.ok) {
        throw new Error(label + ' returned HTTP ' + response.status);
      }
      return response.json();
    });
}

export function fetchDemoCatalog() {
  return requestJson('/demo-data', 'demo data catalog')
    .then(function (catalog) {
      if (!catalog || typeof catalog.defaultId !== 'string' || !Array.isArray(catalog.availableIds)) {
        throw new Error('demo data catalog is missing defaultId or availableIds');
      }
      if (catalog.availableIds.indexOf(catalog.defaultId) === -1) {
        throw new Error('demo data catalog defaultId is not present in availableIds');
      }
      return {
        defaultId: catalog.defaultId,
        availableIds: catalog.availableIds.slice(),
      };
    });
}

export function fetchDemoPlan(demoId) {
  return requestJson('/demo-data/' + encodeURIComponent(demoId), 'demo data "' + demoId + '"');
}
