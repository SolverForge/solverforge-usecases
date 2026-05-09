import { countDisplayLabels, displayLabel, factKey } from './identity.mjs';
import { compareShiftRows } from './presentation.mjs';

// Groups already-presented shift rows by location while preserving lane order.
export function groupShiftRowsByLocation(rows = []) {
  const groupsByKey = {};

  rows.forEach((row) => {
    const groupKey = `location:${String(row.locationLabel)}`;
    if (!groupsByKey[groupKey]) {
      groupsByKey[groupKey] = {
        key: groupKey,
        label: row.locationLabel,
        rows: [],
        sourceIndex: row.shiftIndex,
      };
    } else {
      groupsByKey[groupKey].sourceIndex = Math.min(groupsByKey[groupKey].sourceIndex, row.shiftIndex);
    }
    groupsByKey[groupKey].rows.push(row);
  });

  return Object.keys(groupsByKey)
    .sort((left, right) => groupsByKey[left].sourceIndex - groupsByKey[right].sourceIndex)
    .map((key) => {
      const group = groupsByKey[key];
      group.rows.sort(compareShiftRows);
      return group;
    });
}

// Groups rows by employee and preserves empty employee lanes for visibility.
export function groupShiftRowsByEmployee(rows = [], employees = []) {
  const buckets = {};
  const groups = [];
  const unassignedRows = [];
  const labelCounts = countDisplayLabels(employees, 'Employee');

  rows.forEach((row) => {
    if (!row.isAssigned || !row.employeeKey) {
      unassignedRows.push(row);
      return;
    }
    if (!buckets[row.employeeKey]) {
      buckets[row.employeeKey] = {
        key: row.employeeKey,
        label: row.employeeLabel,
        employee: row.employee || null,
        rows: [],
      };
    }
    buckets[row.employeeKey].rows.push(row);
  });

  employees.forEach((employee, index) => {
    const key = factKey(employee, index);
    const label = displayLabel(employee, index);
    const group = buckets[key] || {
      key,
      label,
      employee: employee || null,
      rows: [],
    };
    group.rows.sort(compareShiftRows);
    group.badges = labelCounts[label] > 1 && employee && employee.id != null
      ? [String(employee.id)]
      : [];
    groups.push(group);
  });

  unassignedRows.sort(compareShiftRows);

  return {
    groups,
    unassignedRows,
  };
}
