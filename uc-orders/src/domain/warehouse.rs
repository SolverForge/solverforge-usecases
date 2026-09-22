use serde::{Deserialize, Serialize};

const SHELVING_WIDTH: i64 = 2;
const SHELVING_HEIGHT: i64 = 10;
const SHELVING_PADDING: i64 = 3;

/// The aisle-facing side of a shelving unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Left,
    Right,
}

/// One pick face in the fixed five-column, three-row warehouse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseLocation {
    pub shelving_id: String,
    pub side: Side,
    pub row: i64,
}

impl WarehouseLocation {
    pub fn new(shelving_id: impl Into<String>, side: Side, row: i64) -> Self {
        Self {
            shelving_id: shelving_id.into(),
            side,
            row,
        }
    }

    fn shelving_origin(&self) -> Option<(i64, i64)> {
        let bytes = self.shelving_id.as_bytes();
        if bytes.len() != 5 || bytes[0] != b'(' || bytes[2] != b',' || bytes[4] != b')' {
            return None;
        }
        let column = bytes[1];
        let warehouse_row = bytes[3];
        if !(b'A'..=b'E').contains(&column) || !(b'1'..=b'3').contains(&warehouse_row) {
            return None;
        }
        Some((
            i64::from(column - b'A') * (SHELVING_WIDTH + SHELVING_PADDING),
            i64::from(warehouse_row - b'1') * (SHELVING_HEIGHT + SHELVING_PADDING),
        ))
    }
}

/// Reproduces the source warehouse's piecewise aisle distance in meters.
pub fn calculate_distance(start: &WarehouseLocation, end: &WarehouseLocation) -> i64 {
    let (start_shelf_x, start_shelf_y) = start
        .shelving_origin()
        .unwrap_or_else(|| panic!("unknown shelving {}", start.shelving_id));
    let (end_shelf_x, end_shelf_y) = end
        .shelving_origin()
        .unwrap_or_else(|| panic!("unknown shelving {}", end.shelving_id));
    let start_x = start_shelf_x + side_offset(start.side);
    let end_x = end_shelf_x + side_offset(end.side);
    let start_y = start_shelf_y + start.row;
    let end_y = end_shelf_y + end.row;

    if start.shelving_id == end.shelving_id {
        if start.side == end.side {
            (start_y - end_y).abs()
        } else {
            SHELVING_WIDTH + best_cross_aisle_y(start.row, end.row)
        }
    } else if start_shelf_y == end_shelf_y {
        let delta_x = (start_x - end_x).abs();
        if delta_x == SHELVING_PADDING {
            delta_x + (start_y - end_y).abs()
        } else {
            delta_x + best_cross_aisle_y(start.row, end.row)
        }
    } else {
        (start_x - end_x).abs() + (start_y - end_y).abs()
    }
}

fn side_offset(side: Side) -> i64 {
    match side {
        Side::Left => 0,
        Side::Right => SHELVING_WIDTH,
    }
}

fn best_cross_aisle_y(start_row: i64, end_row: i64) -> i64 {
    (start_row + end_row).min((SHELVING_HEIGHT - start_row) + (SHELVING_HEIGHT - end_row))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(shelf: &str, side: Side, row: i64) -> WarehouseLocation {
        WarehouseLocation::new(shelf, side, row)
    }

    #[test]
    fn same_shelving_same_side_uses_vertical_distance() {
        assert_eq!(
            calculate_distance(
                &location("(A,1)", Side::Left, 2),
                &location("(A,1)", Side::Left, 9)
            ),
            7
        );
    }

    #[test]
    fn same_shelving_opposite_sides_walks_around_shortest_end() {
        assert_eq!(
            calculate_distance(
                &location("(A,1)", Side::Left, 2),
                &location("(A,1)", Side::Right, 3)
            ),
            7
        );
    }

    #[test]
    fn neighboring_contiguous_faces_cross_the_three_meter_aisle() {
        assert_eq!(
            calculate_distance(
                &location("(A,1)", Side::Right, 2),
                &location("(B,1)", Side::Left, 8)
            ),
            9
        );
    }

    #[test]
    fn noncontiguous_shelves_on_same_row_walk_around_an_end() {
        assert_eq!(
            calculate_distance(
                &location("(A,1)", Side::Left, 2),
                &location("(C,1)", Side::Left, 7)
            ),
            19
        );
    }

    #[test]
    fn different_warehouse_rows_use_manhattan_distance() {
        assert_eq!(
            calculate_distance(
                &location("(A,1)", Side::Right, 4),
                &location("(C,3)", Side::Left, 6)
            ),
            36
        );
    }
}
