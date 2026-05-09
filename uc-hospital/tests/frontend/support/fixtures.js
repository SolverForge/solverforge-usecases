// Minimal transport-shaped employees used by frontend tests.
function employee(id, name = id) {
  return { id, name };
}

// Minimal transport-shaped shift used by frontend tests.
function shift(overrides = {}) {
  return {
    id: 'shift-1',
    start: '2024-01-01T08:00:00',
    end: '2024-01-01T16:00:00',
    location: 'ER',
    requiredSkill: 'Doctor',
    employeeIdx: 0,
    ...overrides,
  };
}

// Two employees with the same display name so identity-collision handling is testable.
function duplicateNameEmployees() {
  return [
    employee('employee-1', 'Alex'),
    employee('employee-2', 'Alex'),
  ];
}

module.exports = {
  duplicateNameEmployees,
  employee,
  shift,
};
