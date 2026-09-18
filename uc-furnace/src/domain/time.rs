use super::constants::{
    RosterDay, ShiftId, CARRY_OUT_END_MINUTE, HORIZON_MINUTES, MINUTES_PER_DAY, NUM_DAYS,
    STAFFING_ROSTER_DAY_END, STAFFING_ROSTER_DAY_START, TIME_STEP,
};
use super::enums::ShiftType;
use super::resources::{Furnace, WorkOrder};
use super::support::ShiftIdentity;

pub fn shift_id_for_roster_day_and_type(
    roster_day: RosterDay,
    shift_type: ShiftType,
) -> Option<ShiftId> {
    match (roster_day, shift_type) {
        (STAFFING_ROSTER_DAY_START, ShiftType::Night) => Some(0),
        (0..=STAFFING_ROSTER_DAY_END, shift_type) => {
            Some(1 + (roster_day as usize * 3) + shift_type.index())
        }
        _ => None,
    }
}

pub fn owning_shift_identity_for_minute(minute: usize) -> Option<ShiftIdentity> {
    let day = (minute / MINUTES_PER_DAY) as RosterDay;
    let minute_of_day = minute % MINUTES_PER_DAY;
    let (roster_day, shift_type) = if minute_of_day < 360 {
        (day - 1, ShiftType::Night)
    } else if minute_of_day < 840 {
        (day, ShiftType::Morning)
    } else if minute_of_day < 1320 {
        (day, ShiftType::Afternoon)
    } else {
        (day, ShiftType::Night)
    };

    shift_id_for_roster_day_and_type(roster_day, shift_type).map(|shift_id| ShiftIdentity {
        shift_id,
        roster_day,
        shift_type,
    })
}

pub fn owning_shift_identity_for_interval(start: usize, end: usize) -> Option<ShiftIdentity> {
    if end <= start {
        return None;
    }

    let start_shift = owning_shift_identity_for_minute(start)?;
    let end_shift = owning_shift_identity_for_minute(end.saturating_sub(1))?;
    (start_shift == end_shift).then_some(start_shift)
}

pub fn shift_identities_for_interval(start: usize, end: usize) -> Vec<ShiftIdentity> {
    if end <= start {
        return Vec::new();
    }

    let mut covered = Vec::new();
    let mut cursor = start;

    while cursor < end {
        let Some(shift) = owning_shift_identity_for_minute(cursor) else {
            break;
        };
        let Some((_, shift_end)) = shift_bounds(shift.roster_day, shift.shift_type) else {
            break;
        };
        covered.push(shift);
        if shift_end <= cursor {
            break;
        }
        cursor = shift_end.min(end);
    }

    covered
}

pub fn shift_bounds(roster_day: RosterDay, shift_type: ShiftType) -> Option<(usize, usize)> {
    match shift_type {
        ShiftType::Morning if (0..=STAFFING_ROSTER_DAY_END).contains(&roster_day) => {
            let day_offset = roster_day as usize * MINUTES_PER_DAY;
            Some((day_offset + 360, day_offset + 840))
        }
        ShiftType::Afternoon if (0..=STAFFING_ROSTER_DAY_END).contains(&roster_day) => {
            let day_offset = roster_day as usize * MINUTES_PER_DAY;
            Some((day_offset + 840, day_offset + 1320))
        }
        ShiftType::Night if roster_day == STAFFING_ROSTER_DAY_START => Some((0, 360)),
        ShiftType::Night if (0..=STAFFING_ROSTER_DAY_END).contains(&roster_day) => {
            let day_offset = roster_day as usize * MINUTES_PER_DAY;
            let end = if roster_day == STAFFING_ROSTER_DAY_END {
                CARRY_OUT_END_MINUTE
            } else {
                day_offset + MINUTES_PER_DAY + 360
            };
            Some((day_offset + 1320, end))
        }
        _ => None,
    }
}

pub fn furnace_accepts_work_order(furnace: &Furnace, work_order: &WorkOrder) -> bool {
    furnace.supports_process(work_order.process)
        && furnace.supports_temperature(work_order.temperature_celsius)
        && furnace.supports_load(work_order.load_weight_kg)
}

pub fn monitoring_bucket_for_minute(minute: usize) -> usize {
    minute / TIME_STEP
}

pub fn heating_ramp_minutes(duration_minutes: usize) -> usize {
    duration_minutes
        .clamp(15, 60)
        .min(duration_minutes.saturating_sub(15))
}

pub fn shift_display_start_minute(roster_day: RosterDay, shift_type: ShiftType) -> i32 {
    let roster_day_minutes = roster_day * MINUTES_PER_DAY as i32;
    match shift_type {
        ShiftType::Morning => roster_day_minutes + 360,
        ShiftType::Afternoon => roster_day_minutes + 840,
        ShiftType::Night => roster_day_minutes + 1320,
    }
}

pub fn canonical_shift_label(roster_day: RosterDay, shift_type: ShiftType) -> String {
    format!("{} {}", canonical_day_label(roster_day), shift_type)
}

pub fn canonical_shift_time_label(
    roster_day: RosterDay,
    shift_type: ShiftType,
    end_minute: usize,
) -> String {
    format!(
        "{} -> {}",
        canonical_signed_minute_label(shift_display_start_minute(roster_day, shift_type)),
        canonical_minute_label(end_minute)
    )
}

pub fn canonical_interval_label(start_minute: usize, end_minute: usize) -> String {
    format!(
        "{} -> {}",
        canonical_minute_label(start_minute),
        canonical_minute_label(end_minute)
    )
}

pub fn canonical_minute_label(minutes: usize) -> String {
    canonical_signed_minute_label(minutes as i32)
}

pub fn canonical_signed_minute_label(minutes: i32) -> String {
    const DAY_LABELS: [&str; NUM_DAYS] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    if minutes == HORIZON_MINUTES as i32 {
        return "Sun 24:00".to_string();
    }

    let week = HORIZON_MINUTES as i32;
    let minute_of_week = minutes.rem_euclid(week);
    let week_offset = minutes.div_euclid(week);
    let day_idx = (minute_of_week / MINUTES_PER_DAY as i32) as usize;
    let minute_of_day = minute_of_week % MINUTES_PER_DAY as i32;
    let prefix = match week_offset {
        0 => String::new(),
        offset if offset > 0 => format!("S+{offset} "),
        offset => format!("S{offset} "),
    };

    format!(
        "{}{} {:02}:{:02}",
        prefix,
        DAY_LABELS[day_idx],
        minute_of_day / 60,
        minute_of_day % 60
    )
}

pub fn canonical_day_label(roster_day: RosterDay) -> String {
    const DAY_LABELS: [&str; NUM_DAYS] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let week_offset = roster_day.div_euclid(NUM_DAYS as i32);
    let day_idx = roster_day.rem_euclid(NUM_DAYS as i32) as usize;
    let prefix = match week_offset {
        0 => String::new(),
        offset if offset > 0 => format!("S+{offset} "),
        offset => format!("S{offset} "),
    };

    format!("{}{}", prefix, DAY_LABELS[day_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owning_shift_resolution_is_boundary_safe() {
        let monday_carry_in = owning_shift_identity_for_minute(15).expect("shift for Monday 00:15");
        assert_eq!(monday_carry_in.roster_day, -1);
        assert_eq!(monday_carry_in.shift_type, ShiftType::Night);

        let tuesday_post_midnight =
            owning_shift_identity_for_minute(1440 + 75).expect("shift for Tuesday 01:15");
        assert_eq!(tuesday_post_midnight.roster_day, 0);
        assert_eq!(tuesday_post_midnight.shift_type, ShiftType::Night);

        let sunday_late =
            owning_shift_identity_for_minute(6 * 1440 + 1410).expect("shift for Sunday 23:30");
        assert_eq!(sunday_late.roster_day, 6);
        assert_eq!(sunday_late.shift_type, ShiftType::Night);

        let next_monday = owning_shift_identity_for_minute(HORIZON_MINUTES + 15)
            .expect("shift for next Monday 00:15");
        assert_eq!(next_monday.roster_day, 6);
        assert_eq!(next_monday.shift_type, ShiftType::Night);
    }

    #[test]
    fn canonical_labels_cover_week_boundaries() {
        assert_eq!(canonical_minute_label(0), "Mon 00:00");
        assert_eq!(canonical_minute_label(HORIZON_MINUTES), "Sun 24:00");
        assert_eq!(
            canonical_minute_label(HORIZON_MINUTES + 15),
            "S+1 Mon 00:15"
        );
        assert_eq!(canonical_shift_label(-1, ShiftType::Night), "S-1 Sun Night");
        assert_eq!(
            canonical_shift_time_label(-1, ShiftType::Night, 360),
            "S-1 Sun 22:00 -> Mon 06:00"
        );
    }
}
