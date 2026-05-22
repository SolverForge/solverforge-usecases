/* utils.mjs — shared helpers for the Bergamo FSR demo */

function buildCurl(method, path, json) {
  var parts = ['curl'];
  if (method && method !== 'GET') parts.push('-X', method);
  if (json) parts.push('-H', '"Content-Type: application/json"', '-d', '@plan.json');
  parts.push(window.location.origin + path);
  return parts.join(' ');
}

function fetchDemoCatalog() {
  return requestJson('/demo-data', 'demo data catalog').then(function (catalog) {
    if (!catalog || typeof catalog.defaultId !== 'string' || !Array.isArray(catalog.availableIds)) {
      throw new Error('demo data catalog is missing defaultId or availableIds');
    }
    return { defaultId: catalog.defaultId, availableIds: catalog.availableIds.slice() };
  });
}

function fetchDemoPlan(demoId) {
  return requestJson('/demo-data/' + encodeURIComponent(demoId), 'demo data "' + demoId + '"');
}

function fetchJobRoutes(jobId, snapshotRevision) {
  return requestJson(
    '/jobs/' + encodeURIComponent(jobId) + '/routes?snapshot_revision=' + encodeURIComponent(snapshotRevision),
    'route geometry for job "' + jobId + '"'
  );
}

function requestJson(path, label) {
  return fetch(path).then(function (response) {
    if (!response.ok) throw new Error(label + ' returned HTTP ' + response.status);
    return response.json();
  });
}

function routeSchedule(plan, route) {
  var visits = plan.service_visits || [];
  var entries = [];
  var clock = route.shift_start_minute;
  var previous = route.start_location_idx;

  (route.visits || []).forEach(function (visitIdx) {
    var visit = visits[visitIdx];
    if (!visit) return;
    var leg = legFor(plan, previous, visit.location_idx);
    clock += leg && leg.reachable ? Math.ceil((leg.duration_seconds || 0) / 60) : 0;
    if (clock < visit.earliest_minute) clock = visit.earliest_minute;
    var start = clock;
    var end = start + Math.max(0, visit.duration_minutes || 0);
    entries.push({ visit: visit, start: start, end: end });
    clock = end;
    previous = visit.location_idx;
  });

  return entries;
}

function routeStats(plan, route) {
  var visits = plan.service_visits || [];
  var travelMinutes = 0;
  var serviceMinutes = 0;
  var lateMinutes = 0;
  var overtimeMinutes = 0;
  var missingSkills = 0;
  var missingParts = 0;
  var unreachable = 0;
  var clock = route.shift_start_minute;
  var previous = route.start_location_idx;

  (route.visits || []).forEach(function (visitIdx) {
    var visit = visits[visitIdx];
    if (!visit) {
      unreachable += 1;
      return;
    }
    var leg = legFor(plan, previous, visit.location_idx);
    if (!leg || !leg.reachable) {
      unreachable += 1;
    } else {
      var legMinutes = Math.ceil((leg.duration_seconds || 0) / 60);
      travelMinutes += legMinutes;
      clock += legMinutes;
    }
    if (clock < visit.earliest_minute) clock = visit.earliest_minute;
    if (clock > visit.latest_minute) lateMinutes += clock - visit.latest_minute;
    if (!maskContains(route.skill_mask, visit.required_skill_mask)) missingSkills += 1;
    if (!maskContains(route.inventory_mask, visit.required_parts_mask)) missingParts += 1;
    serviceMinutes += Math.max(0, visit.duration_minutes || 0);
    clock += Math.max(0, visit.duration_minutes || 0);
    previous = visit.location_idx;
  });

  var returnLeg = legFor(plan, previous, route.end_location_idx);
  if (!returnLeg || !returnLeg.reachable) {
    unreachable += 1;
  } else {
    var returnMinutes = Math.ceil((returnLeg.duration_seconds || 0) / 60);
    travelMinutes += returnMinutes;
    clock += returnMinutes;
  }

  var routeMinutes = Math.max(0, clock - route.shift_start_minute);
  overtimeMinutes += Math.max(0, clock - route.shift_end_minute);
  overtimeMinutes += Math.max(0, routeMinutes - route.max_route_minutes);

  return {
    travelMinutes: travelMinutes,
    serviceMinutes: serviceMinutes,
    lateMinutes: lateMinutes,
    overtimeMinutes: overtimeMinutes,
    missingSkills: missingSkills,
    missingParts: missingParts,
    unreachable: unreachable,
  };
}

function legFor(plan, from, to) {
  var width = (plan.locations || []).length;
  var direct = (plan.travel_legs || [])[from * width + to];
  if (direct && direct.from_location_idx === from && direct.to_location_idx === to) {
    return direct;
  }
  return (plan.travel_legs || []).find(function (leg) {
    return leg.from_location_idx === from && leg.to_location_idx === to;
  });
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

function locationLat(location) {
  if (location.lat != null) return Number(location.lat);
  return Number(location.lat_e6 || 0) / 1000000;
}

function locationLng(location) {
  if (location.lng != null) return Number(location.lng);
  return Number(location.lng_e6 || 0) / 1000000;
}

function routeBadges(stats) {
  var badges = [];
  if (stats.unreachable) badges.push('Routing');
  if (stats.missingSkills) badges.push('Skills');
  if (stats.missingParts) badges.push('Parts');
  if (stats.lateMinutes) badges.push('Late');
  if (stats.overtimeMinutes) badges.push('Overtime');
  return badges.length ? badges : ['Feasible'];
}

function iconForVisit(visit) {
  if ((visit.required_skill_mask & 8) === 8) return 'fa-elevator';
  if ((visit.required_skill_mask & 4) === 4) return 'fa-faucet';
  if ((visit.required_skill_mask & 2) === 2) return 'fa-bolt';
  return 'fa-screwdriver-wrench';
}

function maskContains(available, required) {
  return ((available || 0) & (required || 0)) === (required || 0);
}

function toneForRoute(index) {
  return ['blue', 'emerald', 'amber', 'rose', 'violet', 'slate'][index % 6];
}

function formatDuration(minutes) {
  var value = Math.max(0, Math.round(minutes || 0));
  var h = Math.floor(value / 60);
  var m = value % 60;
  if (!h) return String(m) + 'm';
  return String(h) + 'h ' + String(m).padStart(2, '0') + 'm';
}

function timeLabel(minute) {
  var h = Math.floor(minute / 60);
  var m = minute % 60;
  return String(h).padStart(2, '0') + ':' + String(m).padStart(2, '0');
}

function findHeaderButton(header, label) {
  var buttons = header.querySelectorAll('button');
  for (var i = 0; i < buttons.length; i += 1) {
    if ((buttons[i].textContent || '').trim() === label) return buttons[i];
  }
  return null;
}

function clonePlan(data) {
  return JSON.parse(JSON.stringify(data));
}

function title(text) {
  return String(text || '')
    .replace(/_/g, ' ')
    .replace(/\b\w/g, function (match) { return match.toUpperCase(); });
}

export {
  assignedVisitSet,
  buildCurl,
  clonePlan,
  fetchDemoCatalog,
  fetchDemoPlan,
  fetchJobRoutes,
  findHeaderButton,
  formatDuration,
  iconForVisit,
  legFor,
  locationLat,
  locationLng,
  maskContains,
  requestJson,
  routeBadges,
  routeSchedule,
  routeStats,
  timeLabel,
  title,
  toneForRoute,
};
