use std::str::FromStr;

use crate::domain::{PickStep, Plan, Side, Trolley, WarehouseLocation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoData {
    Standard,
}

const AVAILABLE_DEMO_DATA: &[DemoData] = &[DemoData::Standard];

pub fn default_demo_data() -> DemoData {
    DemoData::Standard
}

pub fn available_demo_data() -> &'static [DemoData] {
    AVAILABLE_DEMO_DATA
}

impl DemoData {
    pub fn id(self) -> &'static str {
        "STANDARD"
    }

    pub fn default_demo_data() -> Self {
        Self::Standard
    }

    pub fn available_demo_data() -> &'static [Self] {
        AVAILABLE_DEMO_DATA
    }
}

impl FromStr for DemoData {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .eq_ignore_ascii_case("STANDARD")
            .then_some(Self::Standard)
            .ok_or(())
    }
}

pub fn generate(_: DemoData) -> Plan {
    let products = product_catalog();
    let mut pick_steps = Vec::with_capacity(145);

    for (order_idx, sequence) in ORDER_PRODUCT_SEQUENCES.iter().enumerate() {
        let order_number = order_idx + 1;
        for (item_idx, &product_idx) in sequence.iter().enumerate() {
            let item_number = item_idx + 1;
            let product = &products[product_idx];
            pick_steps.push(PickStep::new(
                format!("step-order-{order_number}-item-{item_number}"),
                product.name,
                product_idx.to_string(),
                order_number.to_string(),
                format!("order-{order_number}-item-{item_number}"),
                product.volume_cm3,
                WarehouseLocation::new(product.shelving_id, product.side, product.row),
            ));
        }
    }

    let depot = WarehouseLocation::new("(A,1)", Side::Left, 0);
    let trolleys = (1..=5)
        .map(|number| Trolley::new(number.to_string(), 4, 48_000, depot.clone()))
        .collect();

    // The source demo preassigns steps round-robin. Empty lists let SolverForge's
    // list construction heuristic create the first complete assignment itself.
    Plan::new(pick_steps, trolleys)
}

#[derive(Clone, Copy)]
struct ProductSpec {
    name: &'static str,
    volume_cm3: i64,
    shelving_id: &'static str,
    side: Side,
    row: i64,
}

fn product_catalog() -> [ProductSpec; 37] {
    use Side::{Left as L, Right as R};
    [
        product("Kelloggs Cornflakes", 12600, "(B,2)", L, 6),
        product("Cream Crackers", 322, "(D,3)", L, 8),
        product("Tea Bags 240 packet", 180, "(D,2)", R, 7),
        product("Tomato Soup Can", 1000, "(D,3)", L, 1),
        product("Baked Beans in Tomato Sauce", 1000, "(B,2)", R, 2),
        product("Classic Mint Sauce", 640, "(D,2)", R, 7),
        product("Raspberry Conserve", 640, "(D,2)", L, 9),
        product("Orange Fine Shred Marmalade", 392, "(C,3)", R, 2),
        product("Free Range Eggs 6 Pack", 1200, "(A,3)", L, 9),
        product("Mature Cheddar 400G", 450, "(A,3)", L, 10),
        product("Butter Packet", 300, "(A,3)", L, 6),
        product("Iceberg Lettuce Each", 2500, "(A,2)", L, 1),
        product("Carrots 1Kg", 1000, "(A,1)", L, 5),
        product("Organic Fair Trade Bananas 5 Pack", 1800, "(A,1)", R, 10),
        product("Gala Apple Minimum 5 Pack", 5000, "(A,1)", R, 5),
        product("Orange Bag 3kg", 8700, "(A,1)", R, 3),
        product(
            "Fairy Non Biological Laundry Liquid 4.55L",
            5000,
            "(E,2)",
            R,
            10,
        ),
        product("Toilet Tissue 8 Roll White", 20000, "(E,2)", R, 8),
        product("Kitchen Roll 200 Sheets x 2", 13500, "(E,2)", L, 5),
        product("Stainless Steel Cleaner 500Ml", 500, "(E,2)", L, 8),
        product("Antibacterial Surface Spray", 1200, "(E,2)", R, 6),
        product("Beef Lean Steak Mince 500g", 500, "(B,2)", L, 3),
        product("Smoked Salmon 120G", 150, "(B,2)", L, 9),
        product("Steak Burgers 454G", 450, "(B,2)", R, 8),
        product("Pork Cooked Ham 125G", 125, "(B,2)", L, 9),
        product("Chicken Breast Fillets 300G", 300, "(B,3)", L, 4),
        product("6 Milk Bricks Pack", 7392, "(D,1)", L, 7),
        product("Milk Brick", 1232, "(D,1)", R, 5),
        product("Skimmed Milk 2.5L", 2500, "(D,1)", L, 4),
        product("3L Orange Juice", 3000, "(D,1)", R, 8),
        product("Alcohol Free Beer 4 Pack", 13500, "(D,1)", L, 4),
        product("Pepsi Regular Bottle", 1000, "(D,1)", R, 1),
        product("Pepsi Diet 6 x 330ml", 5040, "(D,1)", L, 8),
        product("Schweppes Lemonade 2L", 2000, "(D,1)", R, 9),
        product("Coke Zero 8 x 330ml", 5760, "(D,1)", L, 9),
        product(
            "Natural Mineral Water Still 6 X 1.5Ltr",
            9000,
            "(D,1)",
            R,
            1,
        ),
        product("Cocktail Crisps 6 Pack", 2000, "(D,2)", L, 5),
    ]
}

const fn product(
    name: &'static str,
    volume_cm3: i64,
    shelving_id: &'static str,
    side: Side,
    row: i64,
) -> ProductSpec {
    ProductSpec {
        name,
        volume_cm3,
        shelving_id,
        side,
        row,
    }
}

const ORDER_PRODUCT_SEQUENCES: [&[usize]; 8] = [
    &[
        14, 21, 22, 13, 23, 7, 31, 4, 28, 35, 10, 36, 5, 18, 20, 2, 9, 29, 11, 12,
    ],
    &[10, 14, 20, 4, 6, 17, 30, 1],
    &[23, 35, 0, 10, 21, 30, 34, 4],
    &[
        4, 18, 14, 25, 15, 36, 3, 26, 31, 17, 10, 33, 1, 29, 34, 13, 9, 28, 23, 7, 0, 16, 30, 21, 8,
    ],
    &[
        2, 5, 1, 33, 8, 15, 16, 35, 14, 31, 18, 22, 25, 32, 4, 36, 20, 7,
    ],
    &[
        27, 4, 10, 31, 25, 23, 18, 9, 29, 15, 14, 28, 6, 24, 32, 3, 7, 5, 19, 2, 17, 16, 22,
    ],
    &[
        9, 5, 25, 10, 20, 29, 12, 22, 33, 16, 34, 30, 26, 4, 31, 7, 0, 11, 32, 14, 13,
    ],
    &[
        4, 18, 9, 29, 31, 11, 0, 10, 33, 20, 28, 19, 7, 8, 35, 24, 36, 15, 3, 21, 14, 6,
    ],
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn standard_demo_matches_source_shape_and_stable_ids() {
        let plan = generate(DemoData::Standard);
        assert_eq!(plan.trolleys.len(), 5);
        assert_eq!(plan.pick_steps.len(), 145);
        assert_eq!(
            plan.pick_steps
                .iter()
                .map(|step| &step.product_id)
                .collect::<HashSet<_>>()
                .len(),
            37
        );
        assert_eq!(
            plan.pick_steps
                .iter()
                .map(|step| &step.order_id)
                .collect::<HashSet<_>>()
                .len(),
            8
        );
        assert_eq!(
            plan.pick_steps
                .iter()
                .map(|step| &step.id)
                .collect::<HashSet<_>>()
                .len(),
            145
        );
        assert_eq!(
            plan.pick_steps
                .iter()
                .map(|step| &step.order_item_id)
                .collect::<HashSet<_>>()
                .len(),
            145
        );
        assert!(plan
            .trolleys
            .iter()
            .all(|trolley| trolley.step_order.is_empty()));
        assert!(plan
            .trolleys
            .iter()
            .all(|trolley| trolley.bucket_count == 4 && trolley.bucket_capacity == 48_000));
    }
}
