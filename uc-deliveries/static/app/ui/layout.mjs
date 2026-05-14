import { buildDeliveryTimelineModel, buildVehicleTimelineModel } from '../models.mjs';
import { buildApiGuideEndpoints } from './api-guide.mjs';
import { createSelectField, mountPanel } from './components.mjs';

export function createLayout({ app, config, statusBar, actions, onTabChange }) {
  const panels = {};
  const header = SF.createHeader({
    logo: '/sf/img/ouroboros.svg',
    title: config.title,
    subtitle: config.subtitle,
    tabs: [
      { id: 'overview', label: 'Overview', icon: 'fa-map', active: true },
      { id: 'vehicleTimeline', label: 'By Vehicle', icon: 'fa-truck' },
      { id: 'deliveryTimeline', label: 'By Delivery', icon: 'fa-box' },
      { id: 'data', label: 'Data', icon: 'fa-table' },
      { id: 'api', label: 'REST API', icon: 'fa-book' },
    ],
    actions,
    onTabChange,
  });
  app.appendChild(header);
  statusBar.bindHeader(header);
  app.appendChild(statusBar.el);

  const shell = SF.el('div', { className: 'sf-content deliveries-shell' });
  app.appendChild(shell);
  const controls = createControls(shell);
  const overview = createOverview(shell, panels);
  const timelines = createTimelines(shell, panels);
  const data = createDataPanel(shell, panels);
  createApiPanel(shell, panels);

  return {
    ...controls,
    ...overview,
    ...timelines,
    ...data,
    panels,
    analysisModal: SF.createModal({ title: 'Score Analysis', width: '760px' }),
    recommendationModal: SF.createModal({
      title: 'Delivery Insertion Recommendations',
      width: '820px',
    }),
  };
}

function createControls(shell) {
  const controlsCard = SF.el('section', { className: 'deliveries-card' });
  const controlsRow = SF.el('div', { className: 'deliveries-controls' });
  const demoField = createSelectField('Demo Data', [
    { value: 'PHILADELPHIA', label: 'Philadelphia' },
    { value: 'HARTFORD', label: 'Hartford' },
    { value: 'FIRENZE', label: 'Firenze' },
  ]);
  demoField.select.value = 'PHILADELPHIA';
  const routingField = createSelectField('Routing Mode', [
    { value: 'road_network', label: 'Road Network' },
    { value: 'straight_line', label: 'Straight Line' },
  ]);
  routingField.select.value = 'road_network';
  const reloadButton = SF.createButton({ text: 'Reload Demo', icon: 'fa-rotate-right', variant: 'default' });

  controlsRow.appendChild(demoField.el);
  controlsRow.appendChild(routingField.el);
  controlsRow.appendChild(reloadButton);
  controlsCard.appendChild(controlsRow);
  shell.appendChild(controlsCard);
  return { demoField, routingField, reloadButton };
}

function createOverview(shell, panels) {
  const overviewPanel = mountPanel(shell, panels, 'overview');
  const overviewGrid = SF.el('div', { className: 'deliveries-grid' });
  overviewPanel.appendChild(overviewGrid);
  const leftStack = SF.el('div', { className: 'deliveries-stack' });
  const summaryCard = SF.el('section', { className: 'deliveries-card' });
  const summaryMetrics = SF.el('div', { className: 'deliveries-kpis' });
  const routeListCard = SF.el('section', { className: 'deliveries-card' });
  const routeList = SF.el('div', { className: 'deliveries-list' });
  routeListCard.appendChild(SF.el('h3', null, 'Routes'));
  routeListCard.appendChild(routeList);
  leftStack.appendChild(summaryCard);
  leftStack.appendChild(routeListCard);
  overviewGrid.appendChild(leftStack);

  const mapCard = SF.el('section', { className: 'deliveries-card deliveries-map-card' });
  mapCard.appendChild(SF.el('h3', null, 'Map'));
  const mapEl = SF.el('div', { className: 'deliveries-map', id: 'deliveries-map' });
  mapCard.appendChild(mapEl);
  overviewGrid.appendChild(mapCard);
  return { summaryCard, summaryMetrics, routeList };
}

function createTimelines(shell, panels) {
  const vehicleTimelinePanel = mountPanel(shell, panels, 'vehicleTimeline');
  const vehicleTimelineCard = SF.el('section', { className: 'deliveries-card' });
  vehicleTimelineCard.appendChild(SF.el('h3', null, 'Schedule By Vehicle'));
  vehicleTimelinePanel.appendChild(vehicleTimelineCard);
  const deliveryTimelinePanel = mountPanel(shell, panels, 'deliveryTimeline');
  const deliveryTimelineCard = SF.el('section', { className: 'deliveries-card' });
  deliveryTimelineCard.appendChild(SF.el('h3', null, 'Schedule By Delivery'));
  deliveryTimelinePanel.appendChild(deliveryTimelineCard);
  const vehicleTimeline = SF.rail.createTimeline({ label: 'By vehicle', labelWidth: 260, model: buildVehicleTimelineModel(null) });
  const deliveryTimeline = SF.rail.createTimeline({ label: 'By delivery', labelWidth: 320, model: buildDeliveryTimelineModel(null) });
  vehicleTimelineCard.appendChild(vehicleTimeline.el);
  deliveryTimelineCard.appendChild(deliveryTimeline.el);
  return { vehicleTimeline, deliveryTimeline };
}

function createDataPanel(shell, panels) {
  const dataPanel = mountPanel(shell, panels, 'data');
  const dataCard = SF.el('section', { className: 'deliveries-card' });
  const dataBody = SF.el('div');
  dataCard.appendChild(SF.el('h3', null, 'Draft Data'));
  dataCard.appendChild(dataBody);
  dataPanel.appendChild(dataCard);
  return { dataBody };
}

function createApiPanel(shell, panels) {
  const apiPanel = mountPanel(shell, panels, 'api');
  apiPanel.appendChild(SF.createApiGuide({ endpoints: buildApiGuideEndpoints() }));
}
