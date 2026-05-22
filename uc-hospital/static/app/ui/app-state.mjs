// Minimal local state kept outside the shared SolverForge UI widgets.
export function createAppState(initialActiveTab) {
  return {
    currentPlan: null,
    activeTab: initialActiveTab,
  };
}
