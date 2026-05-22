/* models/index.mjs — Consolidated model exports for solverforge-lessons */

export { clonePlan } from './core.mjs';
export {
  DAY_MAP,
  WEEKDAYS,
  WEEKDAY_SHORT,
  parseTimeToMinutes,
  timeslotToMinutes,
  formatClock,
  weekdayIndex,
  weekdayShortLabel,
  safeId,
  isAssignedIndex,
  assignedFact,
  factLabel,
  entityLabel,
  scheduledBadges,
  countScheduled,
  teachingWindowLabel,
  formatTimeslot,
  title,
  toneForKey,
} from './formatters.mjs';
export { buildAxisFromTimeslots, buildLessonTimelineItem } from './timeline.mjs';
export {
  buildSlotAxis,
  buildTimelineItem,
  slotRangeLabel,
  listLaneBadges,
  buildSummarySection,
  buildScalarViewPayload,
  buildListViewPayload,
} from './payloads.mjs';
export {
  buildStatusBarConstraints,
  buildAnalysisHtml,
  describeError,
} from './status.mjs';
export {
  requestJson,
  fetchDemoCatalog,
  fetchDemoPlan,
} from './demo.mjs';
