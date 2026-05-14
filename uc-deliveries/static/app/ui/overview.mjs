import {
  buildDeliveryTimelineModel,
  buildVehicleTimelineModel,
  formatClock,
  formatDuration,
  iconForKind,
} from '../models.mjs';
import { kpi } from './components.mjs';

const VEHICLE_COLORS = [
  '#10b981',
  '#3b82f6',
  '#8b5cf6',
  '#f59e0b',
  '#ec4899',
  '#06b6d4',
  '#f43f5e',
  '#84cc16',
  '#14b8a6',
  '#a855f7',
];

export function colorForVehicle(vehicleId) {
  const index = Number(vehicleId);
  const normalized = Number.isFinite(index) ? Math.abs(Math.trunc(index)) : 0;
  return VEHICLE_COLORS[normalized % VEHICLE_COLORS.length];
}

export function routeStyleForVehicle(vehicleId, focusedVehicleId) {
  const color = colorForVehicle(vehicleId);
  if (focusedVehicleId == null) return { color, opacity: 0.8, weight: 3 };
  return Number(vehicleId) === Number(focusedVehicleId)
    ? { color, opacity: 1, weight: 5 }
    : { color, opacity: 0.2, weight: 2 };
}

export function renderSummary({ currentDemo, currentPlan, summaryCard, summaryMetrics }) {
  const preview = currentPlan?.viewState?.preview;
  summaryCard.innerHTML = '';
  summaryCard.appendChild(SF.el('h3', null, currentPlan?.name || 'Delivery Draft'));
  summaryCard.appendChild(
    SF.el(
      'p',
      null,
      `${currentDemo} · ${currentPlan.routingMode.replace('_', ' ')} · ${currentPlan.deliveries.length} deliveries · ${currentPlan.vehicles.length} vehicles`,
    ),
  );

  summaryMetrics.innerHTML = '';
  summaryMetrics.appendChild(kpi('Hard Score', String(preview?.hardScore ?? 0)));
  summaryMetrics.appendChild(kpi('Soft Score', String(preview?.softScore ?? 0)));
  summaryMetrics.appendChild(kpi('Unassigned', String(preview?.unassignedDeliveryIds?.length || 0)));
  summaryMetrics.appendChild(
    kpi(
      'Travel',
      formatDuration(
        (preview?.vehicles || []).reduce((sum, vehicle) => sum + (vehicle.totalTravelSeconds || 0), 0),
      ),
    ),
  );
  summaryCard.appendChild(summaryMetrics);
}

export function renderRouteList({ currentPlan, focusedVehicleId, routeList, onFocusVehicle }) {
  routeList.innerHTML = '';
  const previewVehicles = currentPlan?.viewState?.preview?.vehicles || [];
  if (!previewVehicles.length) {
    routeList.appendChild(SF.el('div', { className: 'deliveries-empty' }, 'No routes yet.'));
    return;
  }

  for (const vehicle of previewVehicles) {
    const isFocused = focusedVehicleId === vehicle.vehicleId;
    const row = SF.el('div', {
      className: `deliveries-list__row${isFocused ? ' is-focused' : ''}`,
      role: 'button',
      tabIndex: 0,
    });
    row.addEventListener('click', () => onFocusVehicle(vehicle.vehicleId));
    row.addEventListener('keydown', (event) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault();
        onFocusVehicle(vehicle.vehicleId);
      }
    });
    const top = SF.el('div', { className: 'deliveries-list__top' });
    top.appendChild(SF.el('strong', null, vehicle.vehicleName));
    top.appendChild(SF.el('span', { className: 'deliveries-tag' }, `${vehicle.stopCount} stops`));
    row.appendChild(top);

    const meta = SF.el('div', { className: 'deliveries-list__meta' });
    meta.appendChild(SF.el('span', null, `${vehicle.totalDemand} units`));
    meta.appendChild(SF.el('span', null, formatDuration(vehicle.totalTravelSeconds)));
    meta.appendChild(SF.el('span', null, `${formatClock(vehicle.startTime)} to ${formatClock(vehicle.endTime)}`));
    if (vehicle.totalLateSeconds > 0) {
      meta.appendChild(SF.el('span', null, `${formatDuration(vehicle.totalLateSeconds)} late`));
    }
    row.appendChild(meta);

    const focusButton = SF.createButton({
      text: isFocused ? 'Show All' : 'Highlight',
      variant: isFocused ? 'default' : 'ghost',
    });
    focusButton.addEventListener('click', (event) => {
      event.stopPropagation();
      onFocusVehicle(vehicle.vehicleId);
    });
    row.appendChild(SF.el('div', { className: 'deliveries-list__actions' }, focusButton));
    routeList.appendChild(row);
  }
}

export function renderMap({ currentPlan, currentRoutes, focusedVehicleId, fitBounds, mapCtrl, setMapCtrl }) {
  if (!mapCtrl) {
    mapCtrl = SF.map.create({
      container: 'deliveries-map',
      center: [currentPlan?.vehicles?.[0]?.homeLat || 39.9526, currentPlan?.vehicles?.[0]?.homeLng || -75.1652],
      zoom: 12,
    });
    setMapCtrl(mapCtrl);
  }
  mapCtrl.clearAll();

  const vehicles = currentPlan?.vehicles || [];
  const deliveries = currentPlan?.deliveries || [];
  const previewDeliveries = currentPlan?.viewState?.preview?.deliveries || [];
  vehicles.forEach((vehicle) => {
    mapCtrl.addVehicleMarker({
      lat: vehicle.homeLat,
      lng: vehicle.homeLng,
      color: colorForVehicle(vehicle.id),
    });
  });

  previewDeliveries.forEach((delivery) => {
    mapCtrl.addVisitMarker({
      lat: deliveries[delivery.deliveryId].lat,
      lng: deliveries[delivery.deliveryId].lng,
      color: delivery.assignedVehicleId == null ? '#9ca3af' : colorForVehicle(delivery.assignedVehicleId),
      icon: iconForKind(delivery.kind),
      assigned: delivery.assignedVehicleId != null,
    });
  });

  const drawableRoutes = currentRoutes?.routingMode === currentPlan?.routingMode ? currentRoutes : null;
  if (drawableRoutes?.vehicles?.length) {
    drawRoutes(mapCtrl, drawableRoutes, focusedVehicleId);
  }
  if (fitBounds) {
    mapCtrl.fitBounds();
  }
}

export function renderTimelines({ currentPlan, vehicleTimeline, deliveryTimeline }) {
  const preview = currentPlan?.viewState?.preview || null;
  vehicleTimeline.setModel(buildVehicleTimelineModel(preview));
  deliveryTimeline.setModel(buildDeliveryTimelineModel(preview));
}

function drawRoutes(mapCtrl, drawableRoutes, focusedVehicleId) {
  drawableRoutes.vehicles.forEach((vehicleRoute) => {
    const style = routeStyleForVehicle(vehicleRoute.vehicleId, focusedVehicleId);
    vehicleRoute.segments.forEach((segment) => {
      mapCtrl.drawEncodedRoute({
        encoded: segment.encodedPolyline,
        color: style.color,
        opacity: style.opacity,
        weight: style.weight,
      });
    });
  });
}
