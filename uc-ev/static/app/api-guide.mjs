// REST endpoints listed in the browser's API tab.

export function buildApiGuideEndpoints() {
  return [
    endpoint('GET', '/health', 'Check service health'),
    endpoint('GET', '/info', 'Fetch app and solver metadata'),
    endpoint('GET', '/demo-data', 'List demo plans'),
    endpoint('GET', '/demo-data/STANDARD', 'Load the 48-pharmacy Bergamo plan'),
    endpoint('POST', '/jobs', 'Prepare road data and start a retained solve', true, '@plan.json'),
    endpoint('GET', '/jobs/{id}', 'Fetch retained job status'),
    endpoint('GET', '/jobs/{id}/status', 'Fetch retained job status (stock UI alias)'),
    endpoint('GET', '/jobs/{id}/snapshot', 'Fetch the latest retained snapshot'),
    endpoint('GET', '/jobs/{id}/analysis?snapshot_revision={n}', 'Score analysis for one snapshot'),
    endpoint('GET', '/jobs/{id}/routes?snapshot_revision={n}', 'Road geometry for one snapshot'),
    endpoint('GET', '/jobs/{id}/telemetry', 'Solver telemetry detail'),
    endpoint('POST', '/jobs/{id}/pause', 'Pause a retained job'),
    endpoint('POST', '/jobs/{id}/resume', 'Resume a paused job'),
    endpoint('POST', '/jobs/{id}/cancel', 'Cancel a retained job'),
    endpoint('DELETE', '/jobs/{id}', 'Delete a terminal retained job'),
    endpoint('GET', '/jobs/{id}/events', 'Stream retained job events (SSE)'),
  ];
}

function endpoint(method, path, description, json = false, data = null) {
  const parts = ['curl'];
  if (method !== 'GET') parts.push('-X', method);
  if (json) parts.push('-H', '"Content-Type: application/json"');
  parts.push(`${window.location.origin}${path}`);
  if (data) parts.push('-d', data);
  return { method, path, description, curl: parts.join(' ') };
}
