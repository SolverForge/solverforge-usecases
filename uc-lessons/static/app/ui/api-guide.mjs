/* api-guide.mjs — REST API guide rendering for solverforge-lessons */

export function renderApiGuide(apiGuideContainer, demoCatalog, SF) {
  apiGuideContainer.innerHTML = '';
  apiGuideContainer.appendChild(SF.createApiGuide({
    endpoints: buildApiGuideEndpoints(demoCatalog),
  }));
}

export function buildApiGuideEndpoints(demoCatalog) {
  var defaultDemoPath = demoCatalog.defaultId
    ? '/demo-data/' + demoCatalog.defaultId
    : '/demo-data/{defaultId}';
  return [
    { method: 'GET', path: '/demo-data', description: 'Discover the default and available demo data IDs', curl: buildCurlCommand('GET', '/demo-data') },
    { method: 'GET', path: defaultDemoPath, description: 'Fetch the discovered default demo data', curl: buildCurlCommand('GET', defaultDemoPath) },
    { method: 'POST', path: '/jobs', description: 'Create a retained solving job', curl: buildCurlCommand('POST', '/jobs', { json: true, data: '@plan.json' }) },
    { method: 'GET', path: '/jobs/{id}', description: 'Get current job summary', curl: buildCurlCommand('GET', '/jobs/{id}') },
    { method: 'GET', path: '/jobs/{id}/snapshot', description: 'Fetch the latest retained snapshot', curl: buildCurlCommand('GET', '/jobs/{id}/snapshot') },
    { method: 'GET', path: '/jobs/{id}/analysis?snapshot_revision={n}', description: 'Analyze an exact snapshot revision', curl: buildCurlCommand('GET', '/jobs/{id}/analysis?snapshot_revision=3', { quoteUrl: true }) },
    { method: 'POST', path: '/jobs/{id}/pause', description: 'Request an exact runtime pause', curl: buildCurlCommand('POST', '/jobs/{id}/pause') },
    { method: 'POST', path: '/jobs/{id}/resume', description: 'Resume a paused retained job', curl: buildCurlCommand('POST', '/jobs/{id}/resume') },
    { method: 'POST', path: '/jobs/{id}/cancel', description: 'Cancel a live or paused job', curl: buildCurlCommand('POST', '/jobs/{id}/cancel') },
    { method: 'DELETE', path: '/jobs/{id}', description: 'Delete a terminal retained job', curl: buildCurlCommand('DELETE', '/jobs/{id}') },
    { method: 'GET', path: '/jobs/{id}/events', description: 'Stream job lifecycle updates (SSE)', curl: buildCurlCommand('GET', '/jobs/{id}/events', { stream: true }) },
  ];
}

export function buildCurlCommand(method, path, options) {
  var parts = ['curl'];
  if (options && options.stream) {
    parts.push('-N');
  }
  if (method && method !== 'GET') {
    parts.push('-X', method);
  }
  if (options && options.json) {
    parts.push('-H', '"Content-Type: application/json"');
  }

  var url = buildApiUrl(path);
  parts.push(options && options.quoteUrl ? '"' + url + '"' : url);

  if (options && options.data) {
    parts.push('-d', options.data);
  }

  return parts.join(' ');
}

export function buildApiUrl(path) {
  return currentOrigin() + path;
}

export function currentOrigin() {
  return window.location.origin || (window.location.protocol + '//' + window.location.host);
}
