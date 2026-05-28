/* app-render.js - top-level rendering coordinator for the Bergamo FSR demo */

(function () {
  'use strict';

  var FSR = window.FSR = window.FSR || {};
  var utils = FSR.utils;

  FSR.createRenderer = function (options) {
    var SF = options.SF;
    var routeTimeline = null;
    var mapRenderer = FSR.createMapRenderer(options);
    var routeListRenderer = FSR.createRouteListRenderer(options);

    return {
      buildAnalysisBody: buildAnalysisBody,
      destroy: destroy,
      renderAll: renderAll,
      renderApiGuide: renderApiGuide,
    };

    function renderAll(plan, routeGeometry) {
      renderSummary(plan);
      mapRenderer.renderMap(plan, routeGeometry);
      routeListRenderer.renderRouteCards(plan, routeGeometry);
      renderTimeline(plan);
      renderTables(plan);
    }

    function renderSummary(plan) {
      var routes = plan.technician_routes || [];
      var visits = plan.service_visits || [];
      var assigned = utils.assignedVisitSet(routes);
      var assignedCount = Object.keys(assigned).length;
      var routeMetrics = routes.map(function (route) { return utils.routeStats(plan, route); });
      var travelMinutes = routeMetrics.reduce(function (sum, stats) { return sum + stats.travelMinutes; }, 0);
      var serviceMinutes = routeMetrics.reduce(function (sum, stats) { return sum + stats.serviceMinutes; }, 0);
      var routeIssues = routeMetrics.reduce(function (sum, stats) {
        return sum + stats.unreachable + stats.missingSkills + stats.missingParts + stats.lateMinutes + stats.overtimeMinutes;
      }, 0);

      options.summaryContainer.innerHTML = '';
      options.summaryContainer.appendChild(SF.createTable({
        columns: ['Dataset', 'Visits', 'Assigned', 'Technicians', 'Travel', 'Service', 'Hard issues', 'Score'],
        rows: [[
          options.getSelectedDemoId() || options.getDemoCatalog().defaultId || 'STANDARD',
          String(visits.length),
          String(assignedCount),
          String(routes.length),
          utils.formatDuration(travelMinutes),
          utils.formatDuration(serviceMinutes),
          String(routeIssues + Math.max(0, visits.length - assignedCount)),
          String(plan.score || 'unsolved'),
        ]],
      }));
    }

    function renderTimeline(plan) {
      var timelineConfig = buildTimelineConfig(plan);
      options.timelineContainer.innerHTML = '';
      options.timelineContainer.appendChild(SF.el('div', { className: 'sf-section' }, SF.createTable({
        columns: ['Route lanes', 'Window', 'Source'],
        rows: [[String((plan.technician_routes || []).length), '08:00-18:00', 'Latest SolverForge solution payload']],
      })));

      if (!routeTimeline) routeTimeline = SF.rail.createTimeline(timelineConfig);
      else routeTimeline.setModel(timelineConfig.model);
      options.timelineContainer.appendChild(routeTimeline.el);
    }

    function buildTimelineConfig(plan) {
      return {
        label: 'Technician',
        labelWidth: 280,
        title: 'Bergamo Field Service Routes',
        subtitle: 'Ordered service visits per technician',
        model: {
          axis: buildDayAxis(),
          lanes: (plan.technician_routes || []).map(routeLane(plan)),
        },
      };
    }

    function routeLane(plan) {
      return function (route, routeIdx) {
        var items = utils.routeSchedule(plan, route).map(function (entry, entryIdx) {
          return {
            id: 'route-' + routeIdx + '-visit-' + entryIdx,
            startMinute: entry.start,
            endMinute: entry.end,
            label: entry.visit.customer || entry.visit.name || entry.visit.id,
            meta: utils.timeLabel(entry.start) + '-' + utils.timeLabel(entry.end),
            tone: utils.toneForRoute(routeIdx),
          };
        });
        var stats = utils.routeStats(plan, route);
        return {
          id: route.id || ('route-' + routeIdx),
          label: route.technician_name || route.id || ('Technician ' + (routeIdx + 1)),
          mode: 'detailed',
          badges: utils.routeBadges(stats),
          stats: [
            { label: 'Stops', value: (route.visits || []).length },
            { label: 'Travel', value: utils.formatDuration(stats.travelMinutes) },
            { label: 'Service', value: utils.formatDuration(stats.serviceMinutes) },
          ],
          items: items,
        };
      };
    }

    function buildDayAxis() {
      var ticks = [];
      for (var minute = options.dayStart; minute <= options.dayEnd; minute += 60) {
        ticks.push({ id: 'tick-' + minute, minute: minute, label: utils.timeLabel(minute) });
      }
      return {
        startMinute: options.dayStart,
        endMinute: options.dayEnd,
        days: [{ id: 'bergamo-day', label: 'Service day', subLabel: '08:00-18:00', startMinute: options.dayStart, endMinute: options.dayEnd }],
        ticks: ticks,
        initialViewport: { startMinute: options.dayStart, endMinute: options.dayEnd },
      };
    }

    function renderTables(plan) {
      options.tablesContainer.innerHTML = '';
      ['technician_routes', 'service_visits', 'locations'].forEach(function (key) {
        var rows = plan[key] || [];
        if (!rows.length) return;
        var columns = Object.keys(rows[0]).filter(function (column) { return column !== 'score'; });
        var values = rows.map(function (row) {
          return columns.map(function (column) {
            var value = row[column];
            if (value == null) return '-';
            if (Array.isArray(value)) return value.join(', ');
            return String(value);
          });
        });
        var section = SF.el('div', { className: 'sf-section' });
        section.appendChild(SF.el('h3', null, utils.title(key)));
        section.appendChild(SF.createTable({ columns: columns, rows: values }));
        options.tablesContainer.appendChild(section);
      });
    }

    function renderApiGuide() {
      var catalog = options.getDemoCatalog();
      var demoId = options.getSelectedDemoId() || catalog.defaultId;
      options.apiGuideContainer.innerHTML = '';
      options.apiGuideContainer.appendChild(SF.createApiGuide({
        endpoints: [
          { method: 'GET', path: '/health', description: 'Check service health', curl: utils.buildCurl('GET', '/health') },
          { method: 'GET', path: '/info', description: 'Fetch app and solver metadata', curl: utils.buildCurl('GET', '/info') },
          { method: 'GET', path: '/demo-data', description: 'Discover demo datasets', curl: utils.buildCurl('GET', '/demo-data') },
          { method: 'GET', path: '/demo-data/' + (demoId || '{id}'), description: 'Fetch Bergamo seed data', curl: utils.buildCurl('GET', '/demo-data/' + (demoId || 'STANDARD')) },
          { method: 'POST', path: '/jobs', description: 'Create a retained solve job', curl: utils.buildCurl('POST', '/jobs', true) },
          { method: 'GET', path: '/jobs/{id}', description: 'Fetch current job summary', curl: utils.buildCurl('GET', '/jobs/{id}') },
          { method: 'GET', path: '/jobs/{id}/status', description: 'Fetch current job summary through the stock UI alias', curl: utils.buildCurl('GET', '/jobs/{id}/status') },
          { method: 'GET', path: '/jobs/{id}/events', description: 'Stream typed SolverForge lifecycle events', curl: utils.buildCurl('GET', '/jobs/{id}/events') },
          { method: 'GET', path: '/jobs/{id}/snapshot', description: 'Fetch latest route snapshot', curl: utils.buildCurl('GET', '/jobs/{id}/snapshot') },
          { method: 'GET', path: '/jobs/{id}/routes?snapshot_revision={n}', description: 'Fetch encoded route geometry for a retained snapshot', curl: utils.buildCurl('GET', '/jobs/{id}/routes?snapshot_revision={n}') },
          { method: 'GET', path: '/jobs/{id}/analysis', description: 'Analyze the latest retained score', curl: utils.buildCurl('GET', '/jobs/{id}/analysis') },
          { method: 'POST', path: '/jobs/{id}/pause', description: 'Request a retained-job pause', curl: utils.buildCurl('POST', '/jobs/{id}/pause') },
          { method: 'POST', path: '/jobs/{id}/resume', description: 'Resume a paused retained job', curl: utils.buildCurl('POST', '/jobs/{id}/resume') },
          { method: 'POST', path: '/jobs/{id}/cancel', description: 'Cancel a live or paused job', curl: utils.buildCurl('POST', '/jobs/{id}/cancel') },
          { method: 'DELETE', path: '/jobs/{id}', description: 'Delete a terminal retained job', curl: utils.buildCurl('DELETE', '/jobs/{id}') },
        ],
      }));
    }

    function buildAnalysisBody(analysis) {
      var container = SF.el('div', { className: 'fsr-analysis-body' });
      if (!analysis || !analysis.constraints) {
        container.appendChild(SF.el('p', null, 'No analysis available.'));
        return container;
      }

      container.appendChild(SF.el('p', null, SF.el('strong', null, 'Score: '), String(analysis.score)));
      container.appendChild(SF.createTable({
        columns: ['Constraint', 'Weight', 'Score', 'Matches'],
        rows: analysis.constraints.map(function (constraint) {
          return [
            constraint.name,
            constraint.weight,
            constraint.score,
            String(constraint.matchCount || 0),
          ];
        }),
      }));
      return container;
    }

    function destroy() {
      if (routeTimeline) routeTimeline.destroy();
    }
  };
})();
