/* core.mjs — Core plan model utilities for solverforge-lessons */

export function clonePlan(data) {
  return JSON.parse(JSON.stringify(data));
}
