// Renderers: turn the pure models from `model.mjs` into stock components.

import {
  buildFleetSummary,
  buildMapLayers,
  buildRouteCards,
  buildVehicleTimelineModel,
  constraintLabel,
} from './model.mjs';

export function renderSummary(card, plan) {
  const summary = buildFleetSummary(plan);
  card.replaceChildren(
    SF.el('h3', null, 'Bergamo pharmacy round'),
    SF.el(
      'p',
      { className: 'ev-subtle' },
      `${summary.routed}/${summary.customers} pharmacies routed · ${summary.vans} electric vans · ${summary.chargers} public chargers`,
    ),
  );
  const kpis = SF.el('div', { className: 'ev-kpis' });
  [
    ['Distance', summary.distance],
    ['Driving', summary.driving],
    ['Charging stops', String(summary.chargeStops)],
    ['Charging time', summary.charging],
    ['Range shortfall', summary.shortfall],
    ['Late', summary.late],
  ].forEach(([label, value]) => {
    kpis.appendChild(
      SF.el('div', { className: 'ev-kpi' }, SF.el('span', null, label), SF.el('strong', null, value)),
    );
  });
  card.appendChild(kpis);
}

export function renderRouteList(container, plan, focusedIndex, onFocus) {
  const cards = buildRouteCards(plan);
  container.replaceChildren();
  if (!cards.some((card) => card.stops > 0)) {
    container.appendChild(SF.el('p', { className: 'ev-subtle' }, 'No routes yet. Press Solve to plan the day.'));
  }
  cards.forEach((card) => {
    const row = SF.el('div', {
      className: `ev-route${focusedIndex === card.index ? ' is-focused' : ''}${card.hasViolation ? ' has-violation' : ''}`,
      role: 'button',
      tabIndex: 0,
      dataset: { vehicleId: card.id },
    });
    row.style.setProperty('--ev-route-color', card.color);
    SF.bindActivation(row, () => onFocus(card.index));
    row.appendChild(
      SF.el(
        'div',
        { className: 'ev-route__top' },
        SF.el('strong', null, card.name),
        SF.el('span', { className: 'ev-tag' }, `${card.stops} stops`),
      ),
    );
    row.appendChild(
      SF.el(
        'div',
        { className: 'ev-route__meta' },
        SF.el('span', null, card.load),
        SF.el('span', null, card.distance),
        SF.el('span', null, card.charging),
        SF.el('span', null, `lowest ${card.lowestCharge}`),
        card.returnTime ? SF.el('span', null, `back ${card.returnTime}`) : null,
      ),
    );
    container.appendChild(row);
  });
}

export function renderMap(mapCtrl, plan, routes, focusedIndex, fitBounds) {
  const layers = buildMapLayers(plan, routes, focusedIndex);
  mapCtrl.clearAll();
  layers.hubs.forEach((hub) => mapCtrl.addVehicleMarker({ lat: hub.lat, lng: hub.lng, color: '#0f766e' }));
  layers.chargers.forEach((charger) =>
    mapCtrl.addVisitMarker({
      lat: charger.lat,
      lng: charger.lng,
      color: '#d97706',
      icon: 'fa-charging-station',
      assigned: charger.used,
    }),
  );
  layers.customers.forEach((customer) =>
    mapCtrl.addVisitMarker({
      lat: customer.lat,
      lng: customer.lng,
      color: customer.color,
      icon: 'fa-prescription-bottle-medical',
      assigned: customer.assigned,
    }),
  );
  layers.lines.forEach((line) =>
    mapCtrl.drawRoute({
      points: SF.map.decodePolyline(line.encoded),
      color: line.color,
      weight: line.weight,
      opacity: line.opacity,
      dashArray: line.dashArray,
    }),
  );
  if (fitBounds) mapCtrl.fitBounds();
}

export function renderSchedule(timeline, plan) {
  timeline.setModel(buildVehicleTimelineModel(plan));
}

export function renderData(container, plan) {
  container.replaceChildren();
  const section = (title, columns, rows) => {
    container.appendChild(SF.el('h4', null, title));
    container.appendChild(SF.createTable({ columns, rows }));
  };
  section(
    'Vans',
    ['Name', 'Capacity', 'Range', 'Charge at start', 'Reserve', 'Recharge', 'Stops'],
    (plan?.vehicles || []).map((v) => [
      v.name,
      String(v.capacity),
      `${Math.round(v.batteryRangeM / 1000)} km`,
      `${Math.round(v.initialChargeM / 1000)} km`,
      `${Math.round(v.reserveChargeM / 1000)} km`,
      `${Math.round(v.rechargeSeconds / 60)} min`,
      String((v.visitOrder || []).length),
    ]),
  );
  section(
    'Pharmacies',
    ['Name', 'Crates', 'Window', 'Unload'],
    (plan?.customers || []).map((c) => [
      c.name,
      String(c.demand),
      `${clock(c.minStartTime)}–${clock(c.maxEndTime)}`,
      `${Math.round(c.serviceDuration / 60)} min`,
    ]),
  );
  section(
    'Chargers',
    ['Operator', 'Latitude', 'Longitude'],
    (plan?.chargers || []).map((c) => [c.name, c.lat.toFixed(5), c.lng.toFixed(5)]),
  );
}

export function buildAnalysisBody(analysis) {
  const container = SF.el('div');
  if (!analysis || !analysis.constraints) {
    container.appendChild(SF.el('p', null, 'No analysis available.'));
    return container;
  }
  container.appendChild(SF.el('p', null, SF.el('strong', null, 'Score: '), String(analysis.score)));
  container.appendChild(
    SF.createTable({
      columns: ['Constraint', 'Score', 'Matches'],
      rows: analysis.constraints.map((constraint) => [
        constraintLabel(constraint.name),
        String(constraint.score),
        String(constraint.matchCount ?? (constraint.matches ? constraint.matches.length : 0)),
      ]),
    }),
  );
  return container;
}

function clock(seconds) {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${String(hours).padStart(2, '0')}:${String(minutes).padStart(2, '0')}`;
}
