/* ui/index.mjs — Consolidated UI exports for solverforge-lessons */

export { createLayout, setActiveTab } from './layout.mjs';
export { renderOverview, renderViews, renderTimelinePanel, ensureTimeline, destroyTimeline, destroyAllTimelines, ensureCustomTimeline } from './overview.mjs';
export { renderByGroup, renderByTeacher, renderByRoom } from './lessons.mjs';
export { renderTables } from './data-tables.mjs';
export { renderApiGuide, buildApiGuideEndpoints, buildCurlCommand, buildApiUrl, currentOrigin } from './api-guide.mjs';
