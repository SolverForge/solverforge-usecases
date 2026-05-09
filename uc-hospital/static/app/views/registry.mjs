import { renderEmployeeView } from '../schedule/employee-view.mjs';
import { renderLocationView } from '../schedule/location-view.mjs';

// Maps the generated UI-model view kinds to the local renderer functions.
export function createViewRegistry() {
  return {
    'schedule-by-location': renderLocationView,
    'schedule-by-employee': renderEmployeeView,
  };
}
