export function buildApiGuideEndpoints() {
  return [
    endpoint('GET', '/health', 'Check service health'),
    endpoint('GET', '/info', 'Fetch app and solver metadata'),
    endpoint('GET', '/demo-data', 'List available city demos'),
    endpoint('GET', '/demo-data/PHILADELPHIA', 'Load a canonical delivery plan'),
    endpoint('POST', '/jobs', 'Start a retained solve job', true, '@plan.json'),
    endpoint('GET', '/jobs/{id}', 'Fetch retained job status'),
    endpoint('GET', '/jobs/{id}/status', 'Fetch retained job status through the stock UI alias'),
    endpoint('GET', '/jobs/{id}/snapshot', 'Fetch the latest retained snapshot'),
    endpoint('GET', '/jobs/{id}/analysis?snapshot_revision={n}', 'Analyze an exact snapshot revision'),
    endpoint('GET', '/jobs/{id}/routes?snapshot_revision={n}', 'Fetch encoded route geometry for a retained snapshot'),
    endpoint('POST', '/jobs/{id}/pause', 'Pause a retained job'),
    endpoint('POST', '/jobs/{id}/resume', 'Resume a retained job'),
    endpoint('POST', '/jobs/{id}/cancel', 'Cancel a retained job'),
    endpoint('DELETE', '/jobs/{id}', 'Delete a terminal retained job'),
    endpoint('GET', '/jobs/{id}/events', 'Stream retained job events'),
    endpoint('POST', '/recommendations/delivery-insertions', 'Rank candidate insertions for one delivery', true),
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
