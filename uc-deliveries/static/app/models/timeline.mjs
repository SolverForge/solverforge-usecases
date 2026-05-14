import { formatClock, kindLabel, toneForKind } from './formatters.mjs';

export function buildVehicleTimelineModel(preview) {
  const axis = buildAxis(preview);
  return {
    axis,
    lanes: (preview?.vehicles || []).map((vehicle) => ({
      id: `vehicle-${vehicle.vehicleId}`,
      label: vehicle.vehicleName,
      mode: 'detailed',
      badges: [
        `${vehicle.stopCount} stops`,
        `${vehicle.totalDemand}/${vehicle.totalDemand - vehicle.capacityOverage + vehicle.capacityOverage || vehicle.totalDemand}`,
      ],
      items: vehicle.stops.map((stop) => ({
        id: `vehicle-${vehicle.vehicleId}-stop-${stop.deliveryId}`,
        startMinute: Math.floor(stop.serviceStartTime / 60),
        endMinute: Math.ceil(stop.departureTime / 60),
        label: stop.label,
        meta: `${kindLabel(stop.kind)} · ${formatClock(stop.arrivalTime)} arrival`,
        tone: toneForKind(stop.kind),
      })),
    })),
  };
}

export function buildDeliveryTimelineModel(preview) {
  const axis = buildAxis(preview);
  return {
    axis,
    lanes: (preview?.deliveries || []).map((delivery) => {
      const items = [
        {
          id: `delivery-window-${delivery.deliveryId}`,
          startMinute: Math.floor(delivery.minStartTime / 60),
          endMinute: Math.ceil(delivery.maxEndTime / 60),
          label: 'Window',
          meta: `${formatClock(delivery.minStartTime)} to ${formatClock(delivery.maxEndTime)}`,
          tone: 'slate',
        },
      ];

      if (delivery.serviceStartTime != null && delivery.departureTime != null) {
        items.push({
          id: `delivery-service-${delivery.deliveryId}`,
          startMinute: Math.floor(delivery.serviceStartTime / 60),
          endMinute: Math.ceil(delivery.departureTime / 60),
          label: delivery.assignedVehicleName || 'Assigned',
          meta: `${delivery.label} · ${kindLabel(delivery.kind)}`,
          tone: toneForKind(delivery.kind),
        });
      }

      return {
        id: `delivery-${delivery.deliveryId}`,
        label: delivery.label,
        mode: 'detailed',
        badges: [kindLabel(delivery.kind), delivery.assignedVehicleName || 'Unassigned'],
        items,
      };
    }),
  };
}

function buildAxis(preview) {
  let minMinute = 6 * 60;
  let maxMinute = 21 * 60;
  const previewData = preview || { deliveries: [], vehicles: [] };

  for (const delivery of previewData.deliveries || []) {
    minMinute = Math.min(minMinute, Math.floor((delivery.minStartTime || 0) / 60) - 30);
    maxMinute = Math.max(maxMinute, Math.ceil((delivery.maxEndTime || 0) / 60) + 30);
  }
  for (const vehicle of previewData.vehicles || []) {
    minMinute = Math.min(minMinute, Math.floor((vehicle.startTime || 0) / 60) - 15);
    maxMinute = Math.max(maxMinute, Math.ceil((vehicle.endTime || 0) / 60) + 15);
  }

  minMinute = Math.max(0, minMinute);
  maxMinute = Math.min(24 * 60, Math.max(minMinute + 120, maxMinute));

  const ticks = [];
  for (let minute = minMinute; minute <= maxMinute; minute += 60) {
    ticks.push({ minute, label: formatClock(minute * 60) });
  }

  return {
    startMinute: minMinute,
    endMinute: maxMinute,
    days: [{ dayIndex: 0, label: 'Day 1' }],
    ticks,
    initialViewport: { startMinute: minMinute, endMinute: maxMinute },
  };
}
