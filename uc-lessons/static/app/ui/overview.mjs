/* overview.mjs — Overview panel rendering for solverforge-lessons */

import { title } from '../models/index.mjs';

export function renderOverview(overviewContainer, uiModel, data) {
  overviewContainer.innerHTML = '';
  if ((uiModel.views || []).length) {
    overviewContainer.appendChild(SF.el(
      'p',
      null,
      'The generated views now mount the standard solverforge-ui timeline surface for every planning variable declared in your project.'
    ));
    overviewContainer.appendChild(SF.createTable({
      columns: ['Active views', 'Constraints', 'Current score'],
      rows: [[
        String(uiModel.views.length),
        String((uiModel.constraints || []).length),
        String(data.score || '—'),
      ]],
    }));
    return;
  }
  overviewContainer.appendChild(SF.el('p', null, 'No planning variables are declared yet. Use `solverforge generate entity`, `generate fact`, and `generate variable` to shape the app.'));
}

export function renderViews(data, uiModel, viewPanels, renderTimelinePanel, buildListViewPayload, buildScalarViewPayload) {
  (uiModel.views || []).forEach(function (view) {
    var container = document.getElementById('view-' + view.id);
    if (!container) return;
    if (view.kind === 'list') {
      renderTimelinePanel(
        container,
        view.id,
        buildListViewPayload(data, view),
        'This list-variable timeline will appear once the referenced facts and entities contain data.'
      );
    } else {
      renderTimelinePanel(
        container,
        view.id,
        buildScalarViewPayload(data, view),
        'This scalar-variable timeline will appear once the referenced facts and entities contain data.'
      );
    }
  });
}

export function renderTimelinePanel(container, viewId, payload, emptyMessage, viewTimelines, SF) {
  container.innerHTML = '';
  if (!payload) {
    destroyTimeline(viewId, viewTimelines);
    container.appendChild(SF.el('p', null, emptyMessage));
    return;
  }

  container.appendChild(payload.summary);
  container.appendChild(ensureTimeline(viewId, viewTimelines, SF, payload.timeline).el);
}

export function ensureTimeline(viewId, viewTimelines, SF, timelineConfig) {
  var timeline = viewTimelines[viewId];
  if (!timeline) {
    timeline = SF.rail.createTimeline(timelineConfig);
    viewTimelines[viewId] = timeline;
    return timeline;
  }

  timeline.setModel(timelineConfig.model);
  return timeline;
}

export function destroyTimeline(viewId, viewTimelines) {
  var timeline = viewTimelines[viewId];
  if (!timeline) return;
  timeline.destroy();
  delete viewTimelines[viewId];
}

export function destroyAllTimelines(viewTimelines, customTimelines) {
  Object.keys(viewTimelines).forEach(function (viewId) {
    destroyTimeline(viewId, viewTimelines);
  });
  Object.keys(customTimelines || {}).forEach(function (key) {
    if (customTimelines[key]) {
      customTimelines[key].destroy();
      delete customTimelines[key];
    }
  });
}

export function ensureCustomTimeline(key, customTimelines, SF, timelineConfig) {
  var timeline = customTimelines[key];
  if (!timeline) {
    timeline = SF.rail.createTimeline(timelineConfig);
    customTimelines[key] = timeline;
    return timeline;
  }
  timeline.setModel(timelineConfig.model);
  return timeline;
}
