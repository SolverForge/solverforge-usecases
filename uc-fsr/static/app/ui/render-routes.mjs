/* render-routes.mjs - route list rendering for the Bergamo FSR demo */

import { assignedVisitSet, formatDuration, routeStats, timeLabel, title } from '../models/utils.mjs';

export function createRouteListRenderer(options) {
  var SF = window.SF;

  return { renderRouteCards: renderRouteCards };

  function renderRouteCards(plan, routeGeometry) {
    var routeGeometryById = geometryByRouteId(routeGeometry);
    options.routeCards.innerHTML = '';
    (plan.technician_routes || []).forEach(function (route, routeIdx) {
      var stats = routeStats(plan, route);
      var routeId = routeKey(route, routeIdx);
      var focusedRouteId = options.getFocusedRouteId ? options.getFocusedRouteId() : null;
      var isFocused = focusedRouteId === routeId;
      var card = SF.el('div', {
        className: 'fsr-route-row' + (isFocused ? ' is-focused' : ''),
        role: 'button',
        tabIndex: 0,
        dataset: { routeId: routeId },
      });
      card.addEventListener('click', function () { focusRoute(routeId); });
      card.addEventListener('keydown', function (event) {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          focusRoute(routeId);
        }
      });

      var top = SF.el('div', { className: 'fsr-route-row__top' });
      top.appendChild(SF.el('strong', null, route.technician_name || route.id || ('Technician ' + (routeIdx + 1))));
      top.appendChild(SF.el('span', { className: 'fsr-route-tag' }, String((route.visits || []).length) + ' stops'));
      card.appendChild(top);

      var meta = SF.el('div', { className: 'fsr-route-row__meta' });
      meta.appendChild(SF.el('span', null, formatDuration(stats.travelMinutes) + ' travel'));
      meta.appendChild(SF.el('span', null, formatDuration(stats.serviceMinutes) + ' service'));
      meta.appendChild(SF.el('span', null, route.territory || 'no territory'));
      if (stats.lateMinutes) meta.appendChild(SF.el('span', null, formatDuration(stats.lateMinutes) + ' late'));
      if (stats.overtimeMinutes) meta.appendChild(SF.el('span', null, formatDuration(stats.overtimeMinutes) + ' overtime'));
      if (stats.unreachable || stats.missingSkills || stats.missingParts) {
        meta.appendChild(SF.el('span', null, String(stats.unreachable + stats.missingSkills + stats.missingParts) + ' hard issues'));
      }
      if (hasGeometryGaps(routeGeometryById[routeId])) meta.appendChild(SF.el('span', null, 'Geometry gaps'));
      card.appendChild(meta);

      var action = SF.createButton({
        text: isFocused ? 'Show All' : 'Highlight',
        variant: isFocused ? 'default' : 'ghost',
      });
      action.addEventListener('click', function (event) {
        event.stopPropagation();
        focusRoute(routeId);
      });
      card.appendChild(SF.el('div', { className: 'fsr-route-row__actions' }, action));
      options.routeCards.appendChild(card);
    });
    renderUnassignedCard(plan);
  }

  function renderUnassignedCard(plan) {
    var assigned = assignedVisitSet(plan.technician_routes || []);
    var rows = (plan.service_visits || []).reduce(function (items, visit, idx) {
      if (assigned[idx]) return items;
      items.push([
        visit.customer || visit.name || visit.id,
        timeLabel(visit.earliest_minute) + '-' + timeLabel(visit.latest_minute),
        formatDuration(visit.duration_minutes || 0),
      ]);
      return items;
    }, []);
    if (!rows.length) return;

    var card = SF.el('div', { className: 'fsr-route-empty' });
    card.appendChild(SF.el('strong', null, 'Unassigned visits'));
    card.appendChild(SF.createTable({ columns: ['Visit', 'Window', 'Duration'], rows: rows }));
    options.routeCards.appendChild(card);
  }

  function geometryByRouteId(routeGeometry) {
    return ((routeGeometry && routeGeometry.routes) || []).reduce(function (index, route) {
      index[String(route.routeId)] = route;
      return index;
    }, {});
  }

  function hasGeometryGaps(routeGeometry) {
    return !!routeGeometry && (routeGeometry.segments || []).some(function (segment) {
      return segment.geometryStatus !== 'ROUTED';
    });
  }

  function focusRoute(routeId) {
    if (options.onFocusRoute) options.onFocusRoute(routeId);
  }

  function routeKey(route, idx) {
    return String(route.id || route.technician_name || ('route-' + idx));
  }
}
