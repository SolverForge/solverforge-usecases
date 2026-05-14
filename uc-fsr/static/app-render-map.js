/* app-render-map.js - Leaflet map rendering for the Bergamo FSR demo */

(function () {
  'use strict';

  var FSR = window.FSR = window.FSR || {};
  var utils = FSR.utils;

  FSR.createMapRenderer = function (options) {
    return { renderMap: renderMap };

    function renderMap(plan, routeGeometry) {
      if (!options.routeMap) return;
      options.routeMap.clearAll();

      var locations = plan.locations || [];
      var visits = plan.service_visits || [];
      var routes = plan.technician_routes || [];
      var assigned = utils.assignedVisitSet(routes);
      var focusedRouteId = options.getFocusedRouteId ? options.getFocusedRouteId() : null;
      var fitPoints = [];
      var stopNumbers = [];
      var routeGeometryById = geometryByRouteId(routeGeometry);

      routes.forEach(function (route) {
        var start = locations[route.start_location_idx];
        if (!start) return;
        fitPoints.push([utils.locationLat(start), utils.locationLng(start)]);
        options.routeMap.addVehicleMarker({
          lat: utils.locationLat(start),
          lng: utils.locationLng(start),
          color: route.color || '#2563eb',
        });
      });

      visits.forEach(function (visit, idx) {
        var location = locations[visit.location_idx];
        if (!location) return;
        fitPoints.push([utils.locationLat(location), utils.locationLng(location)]);
        options.routeMap.addVisitMarker({
          lat: utils.locationLat(location),
          lng: utils.locationLng(location),
          color: assigned[idx] ? assigned[idx].color : '#64748b',
          icon: utils.iconForVisit(visit),
          assigned: !!assigned[idx],
        });
      });

      routes.forEach(function (route, routeIdx) {
        var routeId = routeKey(route, routeIdx);
        var style = routeStyle(route, routeId, focusedRouteId);
        drawRouteGeometry(routeGeometryById[routeId], route.color, style);
        (route.visits || []).forEach(function (visitIdx, sequenceIdx) {
          var visit = visits[visitIdx];
          if (!visit) return;
          if (!focusedRouteId || focusedRouteId === routeId) {
            stopNumbers.push({
              location: locations[visit.location_idx],
              number: sequenceIdx + 1,
              color: route.color,
            });
          }
        });
      });

      placeStopNumbers(stopNumbers);
      fitMapToPoints(fitPoints);
    }

    function fitMapToPoints(points) {
      if (!points.length || !options.routeMap) return;
      if (options.routeMap.map && options.routeMap.map.invalidateSize) {
        options.routeMap.map.invalidateSize();
      }
      if (window.L && options.routeMap.map && options.routeMap.map.fitBounds) {
        options.routeMap.map.fitBounds(window.L.latLngBounds(points), {
          maxZoom: 12,
          padding: [70, 70],
        });
        return;
      }
      options.routeMap.fitBounds();
    }

    function placeStopNumbers(stops) {
      var groups = {};
      stops.forEach(function (stop) {
        if (!stop.location) return;
        var key = stopLocationKey(stop.location);
        if (!groups[key]) groups[key] = [];
        groups[key].push(stop);
      });

      Object.keys(groups).forEach(function (key) {
        var group = groups[key];
        group.forEach(function (stop, idx) {
          addStopNumber(stop.location, stop.number, stop.color, stopOffset(idx, group.length));
        });
      });
    }

    function stopLocationKey(location) {
      return [
        utils.locationLat(location).toFixed(6),
        utils.locationLng(location).toFixed(6),
      ].join(',');
    }

    function stopOffset(index, count) {
      if (count <= 1) return { x: 0, y: 0 };
      var angle = (-Math.PI / 2) + ((Math.PI * 2 * index) / count);
      var radius = count === 2 ? 14 : 18;
      return { x: Math.cos(angle) * radius, y: Math.sin(angle) * radius };
    }

    function addStopNumber(location, number, color, offset) {
      if (!location) return;
      var marker = options.routeMap.addStopNumber({
        lat: utils.locationLat(location),
        lng: utils.locationLng(location),
        number: number,
        color: color || '#2563eb',
      });
      applyStopOffset(marker, offset || { x: 0, y: 0 });
    }

    function applyStopOffset(marker, offset) {
      if (!marker || (!offset.x && !offset.y)) return;
      var element = marker.getElement ? marker.getElement() : marker._icon;
      var stop = element && element.querySelector ? element.querySelector('.sf-marker-stop') : null;
      if (stop) stop.style.transform = 'translate(' + offset.x.toFixed(1) + 'px, ' + offset.y.toFixed(1) + 'px)';
    }

    function geometryByRouteId(routeGeometry) {
      return ((routeGeometry && routeGeometry.routes) || []).reduce(function (index, route) {
        index[String(route.routeId)] = route;
        return index;
      }, {});
    }

    function drawRouteGeometry(routeGeometry, color, style) {
      if (!routeGeometry || !routeGeometry.segments) return;
      routeGeometry.segments.forEach(function (segment) {
        if (segment.geometryStatus !== 'ROUTED' || !segment.reachable || !segment.encodedPolyline) return;
        options.routeMap.drawEncodedRoute({
          encoded: segment.encodedPolyline,
          color: color || '#2563eb',
          opacity: style && style.opacity,
          weight: style && style.weight,
        });
      });
    }

    function routeKey(route, idx) {
      return String(route.id || route.technician_name || ('route-' + idx));
    }

    function routeStyle(route, routeId, focusedRouteId) {
      if (!focusedRouteId) return { color: route.color || '#2563eb', opacity: 0.82, weight: 3 };
      return focusedRouteId === routeId
        ? { color: route.color || '#2563eb', opacity: 1, weight: 5 }
        : { color: route.color || '#2563eb', opacity: 0.18, weight: 2 };
    }
  };
})();
