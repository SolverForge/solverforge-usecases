(function (root, factory) {
  var api = factory();
  if (typeof module === 'object' && module.exports) module.exports = api;
  if (root) root.FleetTransforms = api;
})(typeof globalThis !== 'undefined' ? globalThis : this, function () {
  'use strict';

  var HORIZON_DAYS = 56;
  var DAY_MINUTES = 1440;
  var END_MINUTE = HORIZON_DAYS * DAY_MINUTES;

  function array(value) {
    return Array.isArray(value) ? value : [];
  }

  function number(value, fallback) {
    var parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : fallback;
  }

  function integer(value, fallback) {
    return Math.trunc(number(value, fallback));
  }

  function indexBy(items, field) {
    var map = Object.create(null);
    array(items).forEach(function (item) {
      if (item && item[field] != null) map[String(item[field])] = item;
    });
    return map;
  }

  function assignedFact(items, index) {
    if (!Number.isInteger(index) || index < 0) return null;
    return array(items)[index] || null;
  }

  function assignedDay(plan, dayIndex) {
    var day = assignedFact(plan && plan.days, dayIndex);
    var value = day && integer(day.index, null);
    return value != null && value >= 1 && value <= HORIZON_DAYS ? value : null;
  }

  function dayStart(day) {
    return Math.max(0, Math.min(END_MINUTE, (integer(day, 1) - 1) * DAY_MINUTES));
  }

  function dayEnd(startDay, durationDays) {
    return Math.max(dayStart(startDay) + 1, Math.min(
      END_MINUTE,
      dayStart(startDay) + Math.max(1, integer(durationDays, 1)) * DAY_MINUTES
    ));
  }

  function buildAxis() {
    var days = [];
    var ticks = [];
    for (var index = 0; index < HORIZON_DAYS; index += 1) {
      var day = index + 1;
      days.push({
        id: 'day-' + day,
        label: 'D' + day,
        subLabel: 'W' + (Math.floor(index / 7) + 1),
        startMinute: index * DAY_MINUTES,
        endMinute: (index + 1) * DAY_MINUTES,
        isWeekend: index % 7 === 5 || index % 7 === 6,
      });
      ticks.push({ id: 'tick-' + day, minute: index * DAY_MINUTES, label: String(day) });
    }
    return {
      startMinute: 0,
      endMinute: END_MINUTE,
      days: days,
      ticks: ticks,
      initialViewport: { startMinute: 0, endMinute: 14 * DAY_MINUTES },
    };
  }

  function workItem(plan, pkg, dock, startDay, options) {
    var unassigned = options && options.unassigned;
    var displayDay = startDay || Math.max(1, integer(pkg.earliest_start_day, 1));
    return {
      id: (unassigned ? 'unassigned-work-' : 'work-') + pkg.id,
      startMinute: dayStart(displayDay),
      endMinute: dayEnd(displayDay, pkg.duration_days),
      label: pkg.id + ' / ' + title(pkg.package_type),
      meta: unassigned
        ? 'Unassigned work / window D' + pkg.earliest_start_day + '-D' + pkg.latest_finish_day
        : 'Work / ' + dock.id + ' ' + dock.name + ' / priority ' + pkg.priority,
      tone: unassigned ? 'red' : toneForDock(dock.id),
    };
  }

  function requirementItem(kind, assignment, requirement, day, unassigned) {
    var displayDay = day || Math.max(1, integer(requirement && requirement.earliest_day, assignment.baseline_day || 1));
    var duration = requirement && requirement.duration_days || 1;
    var titleText = requirement && requirement.name || assignment.requirement_id || assignment.id;
    return {
      id: (unassigned ? 'unassigned-' : '') + kind + '-' + assignment.id,
      startMinute: dayStart(displayDay),
      endMinute: dayEnd(displayDay, duration),
      label: titleText,
      meta: unassigned
        ? 'Unassigned ' + kind + ' / target D' + displayDay
        : title(kind) + ' / D' + day,
      tone: unassigned ? 'red' : kind === 'inspection' ? 'amber' : 'violet',
    };
  }

  function buildAssignmentMap(plan) {
    plan = plan || {};
    var result = { work: [], inspections: [], training: [], unassigned: [] };
    var inspections = indexBy(plan.inspection_requirements, 'id');
    var training = indexBy(plan.training_requirements, 'id');

    array(plan.work_packages).forEach(function (pkg) {
      var dock = assignedFact(plan.docks, pkg.dock_idx);
      var day = assignedDay(plan, pkg.start_day_idx);
      if (!dock || day == null) {
        result.unassigned.push({ kind: 'work', vesselId: pkg.vessel_id, source: pkg, item: workItem(plan, pkg, dock, day, { unassigned: true }) });
        return;
      }
      result.work.push({ kind: 'work', vesselId: pkg.vessel_id, dockId: dock.id, startDay: day, source: pkg, item: workItem(plan, pkg, dock, day) });
    });

    array(plan.inspection_assignments).forEach(function (assignment) {
      var requirement = inspections[String(assignment.requirement_id)];
      var day = assignedDay(plan, assignment.day_idx);
      var entry = {
        kind: 'inspection',
        vesselId: assignment.vessel_id,
        startDay: day,
        source: assignment,
        requirement: requirement,
        item: requirementItem('inspection', assignment, requirement, day, day == null),
      };
      (day == null ? result.unassigned : result.inspections).push(entry);
    });

    array(plan.training_assignments).forEach(function (assignment) {
      var requirement = training[String(assignment.requirement_id)];
      var day = assignedDay(plan, assignment.day_idx);
      var entry = {
        kind: 'training',
        vesselId: assignment.vessel_id,
        startDay: day,
        source: assignment,
        requirement: requirement,
        item: requirementItem('training', assignment, requirement, day, day == null),
      };
      (day == null ? result.unassigned : result.training).push(entry);
    });
    return result;
  }

  function buildVesselLanes(plan) {
    plan = plan || {};
    var assignments = buildAssignmentMap(plan);
    var byVessel = Object.create(null);
    assignments.work.concat(assignments.inspections, assignments.training).forEach(function (entry) {
      (byVessel[entry.vesselId] || (byVessel[entry.vesselId] = [])).push(entry.item);
    });
    var lanes = array(plan.vessels).map(function (vessel) {
      var items = byVessel[vessel.id] || [];
      return {
        id: 'vessel-' + vessel.id,
        label: vessel.id + ' / ' + vessel.name,
        mode: 'detailed',
        badges: items.length ? [title(vessel.vessel_class)] : [title(vessel.vessel_class), 'No scheduled activity'],
        stats: [
          { label: 'Activity', value: items.length },
          { label: 'Ready from', value: 'D' + vessel.ready_from_day },
        ],
        items: items,
      };
    });
    if (assignments.unassigned.length) {
      lanes.push({
        id: 'unassigned',
        label: 'Unassigned',
        mode: 'detailed',
        badges: ['Needs assignment'],
        stats: [{ label: 'Decisions', value: assignments.unassigned.length }],
        items: assignments.unassigned.map(function (entry) { return entry.item; }),
      });
    }
    return lanes;
  }

  function buildDockLanes(plan) {
    plan = plan || {};
    var assignments = buildAssignmentMap(plan);
    var workByDock = Object.create(null);
    assignments.work.forEach(function (entry) {
      (workByDock[entry.dockId] || (workByDock[entry.dockId] = [])).push({
        id: 'dock-' + entry.source.id,
        startMinute: entry.item.startMinute,
        endMinute: entry.item.endMinute,
        label: entry.source.vessel_id + ' / ' + entry.source.id,
        meta: title(entry.source.package_type) + ' / priority ' + entry.source.priority,
        tone: toneForDock(entry.dockId),
      });
    });
    return array(plan.docks).map(function (dock) {
      var items = workByDock[dock.id] || [];
      var outages = array(plan.dock_outages).filter(function (outage) {
        return outage.dock_id === dock.id;
      }).map(function (outage) {
        var start = Math.max(1, integer(outage.start_day, 1));
        var end = Math.min(HORIZON_DAYS, integer(outage.end_day, start));
        return {
          dayIndex: start - 1,
          dayCount: Math.max(1, end - start + 1),
          label: 'Outage D' + start + '-D' + end,
          tone: 'red',
        };
      });
      return {
        id: 'dock-' + dock.id,
        label: dock.id + ' / ' + dock.name,
        mode: 'detailed',
        badges: items.length ? [title(dock.dock_class)] : [title(dock.dock_class), 'Empty'],
        stats: [
          { label: 'Occupancy', value: items.length },
          { label: 'Capacity', value: dock.capacity },
        ],
        overlays: outages,
        items: items,
      };
    });
  }

  function vesselReadyOnDay(plan, vessel, day) {
    if (day < integer(vessel.ready_from_day, 1)) return false;
    var inspectionByVessel = indexBy(plan.inspection_assignments, 'vessel_id');
    var trainingByVessel = indexBy(plan.training_assignments, 'vessel_id');
    var inspectionRequirements = indexBy(plan.inspection_requirements, 'id');
    var trainingRequirements = indexBy(plan.training_requirements, 'id');
    var packages = array(plan.work_packages).filter(function (pkg) { return pkg.vessel_id === vessel.id; });

    for (var index = 0; index < packages.length; index += 1) {
      var pkg = packages[index];
      var workStart = assignedDay(plan, pkg.start_day_idx);
      if (workStart == null) {
        if (pkg.defer_allowed) continue;
        return false;
      }
      if (day < workStart) continue;

      var readyDay = workStart + integer(pkg.duration_days, 1) + integer(pkg.return_to_service_buffer_days, 0);
      var inspection = inspectionByVessel[vessel.id];
      if (!inspection) return false;
      var inspectionDay = assignedDay(plan, inspection.day_idx);
      var inspectionRequirement = inspectionRequirements[inspection.requirement_id];
      if (inspectionDay == null || !inspectionRequirement) return false;
      var inspectionDuration = integer(inspectionRequirement.duration_days, 1);
      if (day >= inspectionDay && day < inspectionDay + inspectionDuration) return false;
      readyDay = Math.max(readyDay, inspectionDay + inspectionDuration + integer(pkg.return_to_service_buffer_days, 0));

      var training = trainingByVessel[vessel.id];
      if (training) {
        var trainingDay = assignedDay(plan, training.day_idx);
        var trainingRequirement = trainingRequirements[training.requirement_id];
        if (trainingDay == null || !trainingRequirement) return false;
        var trainingDuration = integer(trainingRequirement.duration_days, 1);
        if (day >= trainingDay && day < trainingDay + trainingDuration) return false;
        readyDay = Math.max(readyDay, trainingDay + trainingDuration + integer(pkg.return_to_service_buffer_days, 0));
      }
      if (day < readyDay) return false;
    }
    return true;
  }

  function buildDailyReadiness(plan) {
    plan = plan || {};
    var policy = array(plan.readiness_policies)[0] || {};
    return Array.from({ length: HORIZON_DAYS }, function (_, index) {
      var day = index + 1;
      var byClass = Object.create(null);
      var overall = 0;
      array(plan.vessels).forEach(function (vessel) {
        if (!vesselReadyOnDay(plan, vessel, day)) return;
        overall += 1;
        byClass[vessel.vessel_class] = (byClass[vessel.vessel_class] || 0) + 1;
      });
      var patrol = byClass.patrol_cutter || 0;
      var frigate = byClass.frigate_like || 0;
      var overallFloor = integer(policy.min_ready_overall_per_week, 0);
      var patrolFloor = integer(policy.min_ready_patrol_cutter, 0);
      var frigateFloor = integer(policy.min_ready_frigate_like, 0);
      return {
        day: day,
        week: Math.floor(index / 7) + 1,
        readyOverall: overall,
        readyByClass: byClass,
        floorOverall: overallFloor,
        floorPatrol: patrolFloor,
        floorFrigate: frigateFloor,
        shortfall: Math.max(0, overallFloor - overall) + Math.max(0, patrolFloor - patrol) + Math.max(0, frigateFloor - frigate),
      };
    });
  }

  function demandForPool(pkg, poolId) {
    var field = { PROP: 'prop_demand', ELEC: 'elec_demand', HULL: 'hull_demand', QA: 'qa_demand' }[poolId];
    return field ? integer(pkg[field], 0) : 0;
  }

  function buildResourcePressure(plan) {
    plan = plan || {};
    var assignments = buildAssignmentMap(plan);
    return array(plan.technician_pools).map(function (pool) {
      var daily = Array.from({ length: HORIZON_DAYS }, function (_, index) {
        var day = index + 1;
        var demand = 0;
        assignments.work.forEach(function (entry) {
          if (day >= entry.startDay && day < entry.startDay + integer(entry.source.duration_days, 1)) {
            demand += demandForPool(entry.source, pool.id);
          }
        });
        if (pool.id === 'QA') {
          assignments.inspections.forEach(function (entry) {
            if (day >= entry.startDay && day < entry.startDay + integer(entry.requirement && entry.requirement.duration_days, 1)) {
              demand += integer(entry.requirement && entry.requirement.required_qa_per_day, 0);
            }
          });
        }
        if (pool.id === 'TRNG') {
          assignments.training.forEach(function (entry) {
            if (day >= entry.startDay && day < entry.startDay + integer(entry.requirement && entry.requirement.duration_days, 1)) {
              demand += integer(entry.requirement && entry.requirement.required_capacity_per_day, 0);
            }
          });
        }
        var delta = array(plan.technician_capacity_overrides).filter(function (override) {
          return override.pool_id === pool.id && day >= override.start_day && day <= override.end_day;
        }).reduce(function (sum, override) { return sum + integer(override.delta, 0); }, 0);
        var capacity = Math.max(0, integer(pool.capacity_per_day, 0) + delta);
        return { day: day, demand: demand, capacity: capacity, overtime: integer(pool.overtime_capacity_per_day, 0) };
      });
      var peak = daily.reduce(function (best, row) { return row.demand > best.demand ? row : best; }, daily[0]);
      var overCapacityDays = daily.filter(function (row) { return row.demand > row.capacity; }).length;
      return {
        id: pool.id,
        name: pool.name,
        skill: pool.skill,
        peakDemand: peak.demand,
        peakDay: peak.day,
        capacityAtPeak: peak.capacity,
        overtimeCapacity: peak.overtime,
        utilizationPercent: peak.capacity ? Math.round((peak.demand / peak.capacity) * 100) : (peak.demand ? 100 : 0),
        overCapacityDays: overCapacityDays,
        daily: daily,
      };
    });
  }

  function buildKpis(plan) {
    var assignments = buildAssignmentMap(plan || {});
    var readiness = buildDailyReadiness(plan || {});
    var pressure = buildResourcePressure(plan || {});
    var occupiedMinutes = assignments.work.reduce(function (sum, entry) {
      return sum + integer(entry.source.duration_days, 1) * DAY_MINUTES;
    }, 0);
    var dockMinutes = Math.max(1, array(plan && plan.docks).length * END_MINUTE);
    return {
      readyVesselDays: readiness.reduce(function (sum, day) { return sum + day.readyOverall; }, 0),
      dailyFloorMisses: readiness.filter(function (day) { return day.shortfall > 0; }).length,
      unassignedDecisions: assignments.unassigned.length,
      dockUtilizationPercent: Math.round((occupiedMinutes / dockMinutes) * 100),
      pressuredPools: pressure.filter(function (pool) { return pool.overCapacityDays > 0; }).length,
    };
  }

  function toneForDock(id) {
    var palette = ['emerald', 'blue', 'cyan', 'amber', 'violet', 'rose'];
    var text = String(id || '');
    var hash = 0;
    for (var index = 0; index < text.length; index += 1) hash = ((hash * 31) + text.charCodeAt(index)) >>> 0;
    return palette[hash % palette.length];
  }

  function title(value) {
    return String(value == null ? '' : value).replace(/_/g, ' ').replace(/\b\w/g, function (letter) { return letter.toUpperCase(); });
  }

  return {
    HORIZON_DAYS: HORIZON_DAYS,
    DAY_MINUTES: DAY_MINUTES,
    buildAxis: buildAxis,
    buildAssignmentMap: buildAssignmentMap,
    buildVesselLanes: buildVesselLanes,
    buildDockLanes: buildDockLanes,
    buildDailyReadiness: buildDailyReadiness,
    buildResourcePressure: buildResourcePressure,
    buildKpis: buildKpis,
    vesselReadyOnDay: vesselReadyOnDay,
  };
});
