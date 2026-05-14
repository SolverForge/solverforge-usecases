import { formatClock, kindLabel } from '../models.mjs';

export function renderData({ currentPlan, dataBody, onRecommend }) {
  dataBody.innerHTML = '';
  dataBody.appendChild(buildVehicleTable(currentPlan));
  dataBody.appendChild(buildDeliveryTable(currentPlan, onRecommend));
}

function buildVehicleTable(currentPlan) {
  const wrapper = SF.el('div');
  wrapper.appendChild(SF.el('h4', null, 'Vehicles'));
  const table = SF.el('table', { className: 'deliveries-table' });
  table.appendChild(
    SF.el(
      'thead',
      null,
      SF.el(
        'tr',
        null,
        SF.el('th', null, 'Name'),
        SF.el('th', null, 'Depot'),
        SF.el('th', null, 'Capacity'),
        SF.el('th', null, 'Departure'),
      ),
    ),
  );
  const body = SF.el('tbody');
  currentPlan.vehicles.forEach((vehicle) => {
    body.appendChild(
      SF.el(
        'tr',
        null,
        SF.el('td', null, vehicle.name),
        SF.el('td', null, `${vehicle.homeLat.toFixed(4)}, ${vehicle.homeLng.toFixed(4)}`),
        SF.el('td', null, String(vehicle.capacity)),
        SF.el('td', null, formatClock(vehicle.departureTime)),
      ),
    );
  });
  table.appendChild(body);
  wrapper.appendChild(table);
  return wrapper;
}

function buildDeliveryTable(currentPlan, onRecommend) {
  const wrapper = SF.el('div');
  wrapper.appendChild(SF.el('h4', null, 'Deliveries'));
  const table = SF.el('table', { className: 'deliveries-table' });
  table.appendChild(buildDeliveryHeader());
  const body = SF.el('tbody');
  const previewDeliveries = currentPlan.viewState.preview.deliveries;
  previewDeliveries.forEach((deliveryPreview) => {
    const recommendButton = SF.createButton({ text: 'Recommend', variant: 'ghost', size: 'small' });
    recommendButton.addEventListener('click', () => onRecommend(deliveryPreview.deliveryId));
    body.appendChild(buildDeliveryRow(deliveryPreview, recommendButton));
  });
  table.appendChild(body);
  wrapper.appendChild(table);
  return wrapper;
}

function buildDeliveryHeader() {
  return SF.el(
    'thead',
    null,
    SF.el(
      'tr',
      null,
      SF.el('th', null, 'Label'),
      SF.el('th', null, 'Kind'),
      SF.el('th', null, 'Window'),
      SF.el('th', null, 'Demand'),
      SF.el('th', null, 'Assignment'),
      SF.el('th', null, 'Actions'),
    ),
  );
}

function buildDeliveryRow(deliveryPreview, recommendButton) {
  return SF.el(
    'tr',
    null,
    SF.el('td', null, deliveryPreview.label),
    SF.el('td', null, kindLabel(deliveryPreview.kind)),
    SF.el(
      'td',
      null,
      `${formatClock(deliveryPreview.minStartTime)} to ${formatClock(deliveryPreview.maxEndTime)}`,
    ),
    SF.el('td', null, String(deliveryPreview.demand)),
    SF.el('td', null, deliveryPreview.assignedVehicleName || 'Unassigned'),
    SF.el(
      'td',
      null,
      SF.el('div', { className: 'deliveries-actions-inline' }, recommendButton),
    ),
  );
}
