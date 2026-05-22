/* render.mjs - top-level rendering coordinator for the Bergamo FSR demo */

import { createMapRenderer } from './render-map.mjs';
import { createRouteListRenderer } from './render-routes.mjs';
import {
  buildCurl,
  formatDuration,
  routeBadges,
  routeSchedule,
  routeStats,
  timeLabel,
  title,
  toneForRoute,
} from '../models/utils.mjs';

export function createRenderer(options) {
  var SF = window.SF;
  var routeTimeline = null;
  var mapRenderer = createMapRenderer(options);
  var routeListRenderer = createRouteListRenderer(options);

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
    var assigned = assignedVisitSet(routes);
    var assignedCount = Object.keys(assigned).length;
    var routeMetrics = routes.map(function (route) { return routeStats(plan, route); });
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
        formatDuration(travelMinutes),
        formatDuration(serviceMinutes),
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
      var items = routeSchedule(plan, route).map(function (entry, entryIdx) {
        return {
          id: 'route-' + routeIdx + '-visit-' + entryIdx,
          startMinute: entry.start,
          endMinute: entry.end,
          label: entry.visit.customer || entry.visit.name || entry.visit.id,
          meta: timeLabel(entry.start) + '-' + timeLabel(entry.end),
          tone: toneForRoute(routeIdx),
        };
      });
      var stats = routeStats(plan, route);
      return {
        id: route.id || ('route-' + routeIdx),
        label: route.technician_name || route.id || ('Technician ' + (routeIdx + 1)),
        mode: 'detailed',
        badges: routeBadges(stats),
        stats: [
          { label: 'Stops', value: (route.visits || []).length },
          { label: 'Travel', value: formatDuration(stats.travelMinutes) },
          { label: 'Service', value: formatDuration(stats.serviceMinutes) },
        ],
        items: items,
      };
    };
  }

  function buildDayAxis() {
    var ticks = [];
    for (var minute = options.dayStart; minute <= options.dayEnd; minute += 60) {
      ticks.push({ id: 'tick-' + minute, minute: minute, label: timeLabel(minute) });
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
      section.appendChild(SF.el('h3', null, title(key)));
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
        { method: 'GET', path: '/demo-data', description: 'Discover demo datasets', curl: buildCurl('GET', '/demo-data') },
        { method: 'GET', path: '/demo-data/' + (demoId || '{id}'), description: 'Fetch Bergamo seed data', curl: buildCurl('GET', '/demo-data/' + (demoId || 'STANDARD')) },
        { method: 'POST', path: '/jobs', description: 'Create a retained solve job', curl: buildCurl('POST', '/jobs', true) },
        { method: 'GET', path: '/jobs/{id}/events', description: 'Stream typed SolverForge lifecycle events', curl: buildCurl('GET', '/jobs/{id}/events') },
        { method: 'GET', path: '/jobs/{id}/snapshot', description: 'Fetch latest route snapshot', curl: buildCurl('GET', '/jobs/{id}/snapshot') },
        { method: 'GET', path: '/jobs/{id}/routes?snapshot_revision={n}', description: 'Fetch encoded route geometry for a retained snapshot', curl: buildCurl('GET', '/jobs/{id}/routes?snapshot_revision={n}') },
        { method: 'GET', path: '/jobs/{id}/analysis', description: 'Analyze the latest retained score', curl: buildCurl('GET', '/jobs/{id}/analysis') },
      ],
    }));
  }

  function buildAnalysisBody(analysis) {
    var SF = window.SF;
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

  function assignedVisitSet(routes) {
    var assigned = {};
    (routes || []).forEach(function (route) {
      (route.visits || []).forEach(function (visitIdx) {
        assigned[visitIdx] = route;
      });
    });
    return assigned;
  }
}
