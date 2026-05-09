// This module is part of the public documentation surface because the browser
// renders its output directly in the "REST API" guide.

// Chooses the origin used in the curl examples shown in the API guide.
function resolveBaseUrl(root) {
  const location = root.location || (root.window && root.window.location);
  return location && location.origin ? location.origin : 'http://localhost';
}

// Small helper that keeps the curl examples readable.
function curl(method, baseUrl, path, suffix = '') {
  return `curl${method === 'GET' ? '' : ` -X ${method}`}${suffix} ${baseUrl}${path}`;
}

// Builds the endpoint list rendered in the "REST API" tab.
export function buildApiGuideEndpoints(demoId, root = globalThis) {
  const baseUrl = resolveBaseUrl(root);
  return [
    {
      method: 'GET',
      path: '/health',
      description: 'Liveness probe',
      curl: curl('GET', baseUrl, '/health'),
    },
    {
      method: 'GET',
      path: '/info',
      description: 'App identity and version',
      curl: curl('GET', baseUrl, '/info'),
    },
    {
      method: 'GET',
      path: '/demo-data',
      description: 'List available demo dataset ids',
      curl: curl('GET', baseUrl, '/demo-data'),
    },
    {
      method: 'GET',
      path: `/demo-data/${demoId}`,
      description: 'Fetch one demo dataset',
      curl: curl('GET', baseUrl, `/demo-data/${demoId}`),
    },
    {
      method: 'POST',
      path: '/jobs',
      description: 'Create a retained solving job',
      curl: `${curl('POST', baseUrl, '/jobs', ' -H "Content-Type: application/json"')} -d @plan.json`,
    },
    {
      method: 'GET',
      path: '/jobs/{id}',
      description: 'Get current job summary',
      curl: curl('GET', baseUrl, '/jobs/{id}'),
    },
    {
      method: 'GET',
      path: '/jobs/{id}/status',
      description: 'Alias for the current job summary',
      curl: curl('GET', baseUrl, '/jobs/{id}/status'),
    },
    {
      method: 'GET',
      path: '/jobs/{id}/snapshot',
      description: 'Fetch the latest or an exact retained snapshot',
      curl: curl('GET', baseUrl, '/jobs/{id}/snapshot'),
    },
    {
      method: 'GET',
      path: '/jobs/{id}/analysis?snapshot_revision={n}',
      description: 'Analyze an exact snapshot revision',
      curl: `${curl('GET', baseUrl, '/jobs/{id}/analysis')}?snapshot_revision=3`,
    },
    {
      method: 'POST',
      path: '/jobs/{id}/pause',
      description: 'Request an exact runtime pause',
      curl: curl('POST', baseUrl, '/jobs/{id}/pause'),
    },
    {
      method: 'POST',
      path: '/jobs/{id}/resume',
      description: 'Resume a paused retained job',
      curl: curl('POST', baseUrl, '/jobs/{id}/resume'),
    },
    {
      method: 'POST',
      path: '/jobs/{id}/cancel',
      description: 'Stop a live or paused job through runtime cancel',
      curl: curl('POST', baseUrl, '/jobs/{id}/cancel'),
    },
    {
      method: 'DELETE',
      path: '/jobs/{id}',
      description: 'Delete a terminal retained job',
      curl: curl('DELETE', baseUrl, '/jobs/{id}'),
    },
    {
      method: 'GET',
      path: '/jobs/{id}/events',
      description: 'Stream job lifecycle updates (SSE)',
      curl: `curl -N ${baseUrl}/jobs/{id}/events`,
    },
  ];
}
